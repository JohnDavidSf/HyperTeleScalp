mod decimal;
pub use decimal::{Decimal, DecimalError, Price, Quantity};

use std::time::{Duration, Instant, SystemTime};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SourceId {
    pub channel_id: i64,
    pub message_id: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SignalId {
    pub source: SourceId,
    pub template_version: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Long,
    Short,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageOrigin {
    Live,
    Historical,
    Replay,
}

#[derive(Debug, Clone)]
pub struct MessageMeta {
    pub source: SourceId,
    pub reference_message_id: Option<i32>,
    pub received_at: Instant,
    pub source_timestamp: Option<SystemTime>,
    /// Estimate at receipt, supplied by ingestion. Never fabricated from local replay timing.
    pub source_age_at_receive: Option<Duration>,
    pub origin: MessageOrigin,
    pub edited: bool,
    pub forwarded: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PriceRange {
    pub lower: Price,
    pub upper: Price,
}

impl PriceRange {
    pub fn contains(self, price: Price) -> bool {
        self.lower <= price && price <= self.upper
    }
    pub fn is_valid(self) -> bool {
        self.lower <= self.upper
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EntryPolicy {
    Range(PriceRange),
    /// Reference and directional bound must be explicit; no unbounded CMP trade.
    CurrentPrice {
        reference: Price,
        lower: Option<Price>,
        upper: Option<Price>,
    },
    MultipleRanges(Vec<PriceRange>),
}

impl EntryPolicy {
    pub fn allowed_range(&self, price: Price, direction: Direction) -> Option<PriceRange> {
        match self {
            Self::Range(range) if range.is_valid() && range.contains(price) => Some(*range),
            Self::MultipleRanges(ranges)
                if !ranges.is_empty()
                    && ranges.len() <= 8
                    && ranges.iter().all(|range| range.is_valid()) =>
            {
                let mut matching = ranges.iter().copied().filter(|range| range.contains(price));
                let selected = matching.next()?;
                // Multiple matching zones need an explicit source policy; never guess a cap.
                matching.next().is_none().then_some(selected)
            }
            Self::CurrentPrice {
                reference,
                lower,
                upper,
            } => {
                // Missing directional price cap is never inferred from the current quote.
                let range = match direction {
                    Direction::Long => PriceRange {
                        lower: lower.unwrap_or(*reference),
                        upper: (*upper)?,
                    },
                    Direction::Short => PriceRange {
                        lower: (*lower)?,
                        upper: upper.unwrap_or(*reference),
                    },
                };
                (range.is_valid() && range.contains(price)).then_some(range)
            }
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseEvidence {
    ExactChannelTemplate,
    SyntheticFixture,
}

#[derive(Debug, Clone)]
pub struct Signal {
    pub id: SignalId,
    pub meta: MessageMeta,
    pub symbol: String,
    pub direction: Direction,
    pub entry: EntryPolicy,
    pub take_profits: Vec<Price>,
    pub stop_loss: Option<Price>,
    pub leverage_suggestion: Option<u32>,
    pub evidence: ParseEvidence,
}

#[derive(Debug, Clone)]
pub enum SignalUpdate {
    TakeProfits(Vec<Price>),
    StopLoss(Price),
    MoveStopToEntry,
    TakePartials,
    SecureProfits,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IgnoreReason {
    UnknownChannel,
    UnknownTemplate,
    Malformed,
    Incomplete,
    Oversized,
}

#[derive(Debug, Clone)]
pub enum MessageClassification {
    Ignore(IgnoreReason),
    NewSignal(Box<Signal>),
    Update {
        target: SourceId,
        update: SignalUpdate,
    },
    Close {
        target: SourceId,
    },
    Cancel {
        target: SourceId,
    },
}
