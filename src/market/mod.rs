//! Coherent snapshot contract. Publication, subscriptions and exchange precision checks are future work.
use std::time::Instant;

use crate::model::{Price, Quantity};

#[derive(Debug, Clone)]
pub struct MarketSnapshot {
    pub symbol: String,
    pub bid: Option<Price>,
    pub ask: Option<Price>,
    pub bid_size: Option<Quantity>,
    pub ask_size: Option<Quantity>,
    pub received_at: Instant,
    pub exchange_timestamp_ms: Option<u64>,
    pub supported_native_perp: bool,
    pub ready: bool,
}
