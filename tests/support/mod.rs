use std::time::{Duration, Instant};

use hypertele_scalp::{
    market::MarketSnapshot,
    model::{MessageClassification, MessageMeta, MessageOrigin, Signal, SourceId},
    parser::{SignalParser, SyntheticFixtureParser},
    risk::RiskLimits,
    telegram::MessageView,
};

pub const VALID: &str = "HTS_FIXTURE_V1|BTC|LONG|100|101|110,120|90";

pub fn meta(now: Instant) -> MessageMeta {
    MessageMeta {
        source: SourceId {
            channel_id: -1001,
            message_id: 1,
        },
        reference_message_id: None,
        received_at: now,
        source_timestamp: None,
        source_age_at_receive: Some(Duration::ZERO),
        origin: MessageOrigin::Replay,
        edited: false,
        forwarded: false,
    }
}

pub fn signal(now: Instant) -> Result<Signal, Box<dyn std::error::Error>> {
    let meta = meta(now);
    match SyntheticFixtureParser.classify(&MessageView {
        meta: &meta,
        text: VALID,
        caption: None,
        has_media: false,
    }) {
        MessageClassification::NewSignal(signal) => Ok(*signal),
        _ => Err("synthetic fixture did not parse".into()),
    }
}

pub fn market(now: Instant) -> Result<MarketSnapshot, Box<dyn std::error::Error>> {
    Ok(MarketSnapshot {
        symbol: "BTC".into(),
        bid: Some("100".parse()?),
        ask: Some("100.01".parse()?),
        bid_size: Some("10".parse()?),
        ask_size: Some("10".parse()?),
        received_at: now,
        exchange_timestamp_ms: None,
        supported_native_perp: true,
        ready: true,
    })
}

pub fn limits() -> Result<RiskLimits, Box<dyn std::error::Error>> {
    Ok(RiskLimits {
        symbol_allowlist: vec!["BTC".into()],
        max_signal_age: Duration::from_secs(3),
        max_market_age: Duration::from_millis(250),
        max_queue_age: Duration::from_millis(50),
        max_spread_bps: 20,
        max_slippage_bps: 10,
        max_notional: "1000".parse()?,
        max_loss: "100".parse()?,
    })
}
