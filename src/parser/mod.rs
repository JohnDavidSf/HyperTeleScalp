use crate::{
    model::{
        Direction, EntryPolicy, IgnoreReason, MessageClassification, ParseEvidence, PriceRange,
        Signal, SignalId,
    },
    telegram::MessageView,
};

pub const MAX_MESSAGE_BYTES: usize = 4096;
pub const FIXTURE_PREFIX: &str = "HTS_FIXTURE_V1|";

pub trait SignalParser {
    fn classify(&self, message: &MessageView<'_>) -> MessageClassification;
}

/// Both channel A and B use this fail-closed stub until owner-supplied history proves a grammar.
pub struct ChannelParser {
    pub channel_id: i64,
}

impl SignalParser for ChannelParser {
    fn classify(&self, message: &MessageView<'_>) -> MessageClassification {
        let reason = if message.meta.source.channel_id != self.channel_id {
            IgnoreReason::UnknownChannel
        } else if message.effective_text().len() > MAX_MESSAGE_BYTES {
            IgnoreReason::Oversized
        } else {
            IgnoreReason::UnknownTemplate
        };
        MessageClassification::Ignore(reason)
    }
}

/// Explicitly artificial grammar used exclusively for offline fixtures and microbenchmarks.
/// HTS_FIXTURE_V1|BTC|LONG|100|101|110,120|90
pub struct SyntheticFixtureParser;

impl SignalParser for SyntheticFixtureParser {
    fn classify(&self, message: &MessageView<'_>) -> MessageClassification {
        let text = message.effective_text();
        if text.len() > MAX_MESSAGE_BYTES {
            return MessageClassification::Ignore(IgnoreReason::Oversized);
        }
        let Some(payload) = text.strip_prefix(FIXTURE_PREFIX) else {
            return MessageClassification::Ignore(IgnoreReason::UnknownTemplate);
        };
        match parse_fixture(payload, message) {
            Ok(signal) => MessageClassification::NewSignal(Box::new(signal)),
            Err(reason) => MessageClassification::Ignore(reason),
        }
    }
}

fn parse_fixture(payload: &str, message: &MessageView<'_>) -> Result<Signal, IgnoreReason> {
    let mut fields = payload.split('|');
    let mut next = || {
        fields
            .next()
            .filter(|value| !value.is_empty())
            .ok_or(IgnoreReason::Incomplete)
    };
    let symbol = next()?;
    if symbol.len() > 32 || !symbol.bytes().all(|byte| byte.is_ascii_alphanumeric()) {
        return Err(IgnoreReason::Malformed);
    }
    let direction = match next()? {
        "LONG" => Direction::Long,
        "SHORT" => Direction::Short,
        _ => return Err(IgnoreReason::Malformed),
    };
    let lower = next()?.parse().map_err(|_| IgnoreReason::Malformed)?;
    let upper = next()?.parse().map_err(|_| IgnoreReason::Malformed)?;
    let mut take_profits = Vec::new();
    for value in next()?.split(',') {
        if take_profits.len() == 16 {
            return Err(IgnoreReason::Oversized);
        }
        take_profits.push(value.parse().map_err(|_| IgnoreReason::Malformed)?);
    }
    let stop_loss = next()?.parse().map_err(|_| IgnoreReason::Malformed)?;
    if fields.next().is_some() || lower > upper {
        return Err(IgnoreReason::Malformed);
    }
    Ok(Signal {
        id: SignalId {
            source: message.meta.source,
            template_version: 1,
        },
        meta: message.meta.clone(),
        symbol: symbol.to_owned(),
        direction,
        entry: EntryPolicy::Range(PriceRange { lower, upper }),
        take_profits,
        stop_loss: Some(stop_loss),
        leverage_suggestion: None,
        evidence: ParseEvidence::SyntheticFixture,
    })
}
