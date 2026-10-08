use std::time::{Duration, Instant};

use thiserror::Error;

use crate::{
    execution::PreparedOrder,
    market::MarketSnapshot,
    model::{Decimal, Direction, Quantity, Signal},
};

#[derive(Debug, Clone)]
pub struct RiskLimits {
    pub symbol_allowlist: Vec<String>,
    pub max_signal_age: Duration,
    pub max_market_age: Duration,
    pub max_queue_age: Duration,
    pub max_spread_bps: u32,
    pub max_slippage_bps: u32,
    pub max_notional: Decimal,
    pub max_loss: Decimal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum Rejection {
    #[error("symbol is not explicitly allowed")]
    SymbolNotAllowed,
    #[error("exchange state is not ready or instrument is unsupported")]
    ExchangeNotReady,
    #[error("signal source identity is invalid or inconsistent")]
    InvalidIdentity,
    #[error("forwarded or edited messages require a proven update policy")]
    UnreviewedMessage,
    #[error("source age is unknown")]
    UnknownSourceAge,
    #[error("signal is stale")]
    StaleSignal,
    #[error("execution queue age exceeded")]
    StaleQueue,
    #[error("market state is stale")]
    StaleMarket,
    #[error("missing, crossed, or empty executable book")]
    InvalidBook,
    #[error("spread exceeds policy")]
    Spread,
    #[error("current executable price is outside entry policy")]
    OutsideEntry,
    #[error("TP/SL is missing or invalid for direction and entry")]
    InvalidProtection,
    #[error("liquidity cannot cover requested quantity")]
    ThinBook,
    #[error("notional budget exceeded")]
    Notional,
    #[error("potential stop-loss budget exceeded")]
    Loss,
    #[error("checked arithmetic or monotonic time validation failed")]
    Arithmetic,
}

pub fn validate_signal(signal: &Signal, entry: crate::model::Price) -> Result<(), Rejection> {
    let stop = signal.stop_loss.ok_or(Rejection::InvalidProtection)?;
    if signal.take_profits.is_empty() || signal.take_profits.len() > 16 {
        return Err(Rejection::InvalidProtection);
    }
    let valid = match signal.direction {
        Direction::Long => stop < entry && signal.take_profits.iter().all(|tp| *tp > entry),
        Direction::Short => stop > entry && signal.take_profits.iter().all(|tp| *tp < entry),
    };
    if valid {
        Ok(())
    } else {
        Err(Rejection::InvalidProtection)
    }
}

/// A SUBSET of risk checks, for simulation only. No account-risk authorization is produced.
pub fn evaluate_dry_run(
    signal: &Signal,
    market: &MarketSnapshot,
    quantity: Quantity,
    limits: &RiskLimits,
    queued_at: Instant,
    now: Instant,
) -> Result<PreparedOrder, Rejection> {
    if signal.id.source != signal.meta.source
        || signal.id.template_version == 0
        || signal.id.source.channel_id == 0
        || signal.id.source.message_id <= 0
    {
        return Err(Rejection::InvalidIdentity);
    }
    if signal.meta.forwarded || signal.meta.edited {
        return Err(Rejection::UnreviewedMessage);
    }
    if !limits.symbol_allowlist.contains(&signal.symbol) {
        return Err(Rejection::SymbolNotAllowed);
    }
    if market.symbol != signal.symbol || !market.ready || !market.supported_native_perp {
        return Err(Rejection::ExchangeNotReady);
    }
    let source_age = signal
        .meta
        .source_age_at_receive
        .ok_or(Rejection::UnknownSourceAge)?;
    let local_age = now
        .checked_duration_since(signal.meta.received_at)
        .ok_or(Rejection::Arithmetic)?;
    let age = source_age
        .checked_add(local_age)
        .ok_or(Rejection::Arithmetic)?;
    if age > limits.max_signal_age {
        return Err(Rejection::StaleSignal);
    }
    if now
        .checked_duration_since(queued_at)
        .ok_or(Rejection::Arithmetic)?
        > limits.max_queue_age
    {
        return Err(Rejection::StaleQueue);
    }
    if now
        .checked_duration_since(market.received_at)
        .ok_or(Rejection::Arithmetic)?
        > limits.max_market_age
    {
        return Err(Rejection::StaleMarket);
    }
    let bid = market.bid.ok_or(Rejection::InvalidBook)?;
    let ask = market.ask.ok_or(Rejection::InvalidBook)?;
    if bid > ask {
        return Err(Rejection::InvalidBook);
    }
    let spread = ask.decimal().atoms() - bid.decimal().atoms();
    if spread.checked_mul(10_000).ok_or(Rejection::Arithmetic)?
        > bid
            .decimal()
            .atoms()
            .checked_mul(u128::from(limits.max_spread_bps))
            .ok_or(Rejection::Arithmetic)?
    {
        return Err(Rejection::Spread);
    }
    let (entry, available) = match signal.direction {
        Direction::Long => (ask, market.ask_size),
        Direction::Short => (bid, market.bid_size),
    };
    if available.ok_or(Rejection::InvalidBook)? < quantity {
        return Err(Rejection::ThinBook);
    }
    let range = signal
        .entry
        .allowed_range(entry, signal.direction)
        .ok_or(Rejection::OutsideEntry)?;
    validate_signal(signal, entry)?;
    let notional = entry
        .decimal()
        .checked_product_ceil(quantity.decimal())
        .map_err(|_| Rejection::Arithmetic)?;
    if notional > limits.max_notional {
        return Err(Rejection::Notional);
    }
    let stop = signal.stop_loss.ok_or(Rejection::InvalidProtection)?;
    let loss = entry
        .decimal()
        .abs_diff(stop.decimal())
        .checked_product_ceil(quantity.decimal())
        .map_err(|_| Rejection::Arithmetic)?;
    if loss > limits.max_loss {
        return Err(Rejection::Loss);
    }

    // Cap the allowable IOC price by both entry range and slippage. Conservative integer rounding.
    let atoms = entry.decimal().atoms();
    let slip = atoms
        .checked_mul(u128::from(limits.max_slippage_bps))
        .ok_or(Rejection::Arithmetic)?
        / 10_000;
    let cap_atoms = match signal.direction {
        Direction::Long => atoms
            .checked_add(slip)
            .ok_or(Rejection::Arithmetic)?
            .min(range.upper.decimal().atoms()),
        Direction::Short => atoms
            .checked_sub(slip)
            .ok_or(Rejection::Arithmetic)?
            .max(range.lower.decimal().atoms()),
    };
    // SL/TP and budgets must remain safe at the worst permitted entry, not only the observed BBO.
    let cap = crate::model::Price::new(Decimal::from_atoms(cap_atoms))
        .map_err(|_| Rejection::Arithmetic)?;
    validate_signal(signal, cap)?;
    if cap
        .decimal()
        .checked_product_ceil(quantity.decimal())
        .map_err(|_| Rejection::Arithmetic)?
        > limits.max_notional
    {
        return Err(Rejection::Notional);
    }
    if cap
        .decimal()
        .abs_diff(stop.decimal())
        .checked_product_ceil(quantity.decimal())
        .map_err(|_| Rejection::Arithmetic)?
        > limits.max_loss
    {
        return Err(Rejection::Loss);
    }
    let signal_deadline = signal
        .meta
        .received_at
        .checked_add(
            limits
                .max_signal_age
                .checked_sub(source_age)
                .ok_or(Rejection::StaleSignal)?,
        )
        .ok_or(Rejection::Arithmetic)?;
    let market_deadline = market
        .received_at
        .checked_add(limits.max_market_age)
        .ok_or(Rejection::Arithmetic)?;
    let queue_deadline = queued_at
        .checked_add(limits.max_queue_age)
        .ok_or(Rejection::Arithmetic)?;
    Ok(PreparedOrder::new(
        signal.id,
        signal.direction,
        cap,
        quantity,
        signal_deadline.min(market_deadline).min(queue_deadline),
    ))
}
