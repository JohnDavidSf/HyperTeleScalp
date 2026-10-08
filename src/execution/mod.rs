use std::{collections::VecDeque, time::Instant};

use serde::Deserialize;
use thiserror::Error;

use crate::{
    model::{Direction, Price, Quantity, SignalId},
    recovery::{IntentError, IntentLedger},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BackendMode {
    DryRun,
    Replay,
}

#[derive(Debug, Clone)]
pub struct PreparedOrder {
    id: SignalId,
    direction: Direction,
    limit_price: Price,
    quantity: Quantity,
    valid_until: Instant,
}

impl PreparedOrder {
    pub(crate) fn new(
        id: SignalId,
        direction: Direction,
        limit_price: Price,
        quantity: Quantity,
        valid_until: Instant,
    ) -> Self {
        Self {
            id,
            direction,
            limit_price,
            quantity,
            valid_until,
        }
    }
    pub fn id(&self) -> SignalId {
        self.id
    }
    pub fn direction(&self) -> Direction {
        self.direction
    }
    pub fn limit_price(&self) -> Price {
        self.limit_price
    }
    pub fn quantity(&self) -> Quantity {
        self.quantity
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum ExecutionError {
    #[error("prepared simulation intent expired")]
    Expired,
    #[error(transparent)]
    Intent(#[from] IntentError),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SimulationReceipt {
    pub id: SignalId,
    pub mode: BackendMode,
}

pub trait ExecutionBackend {
    fn submit(
        &mut self,
        order: &PreparedOrder,
        now: Instant,
    ) -> Result<SimulationReceipt, ExecutionError>;
}

/// Does not own a signer, network client, socket or credentials. Cannot submit an exchange order.
pub struct NoopBackend {
    mode: BackendMode,
    ledger: IntentLedger,
}

impl NoopBackend {
    pub fn new(mode: BackendMode, dedupe_capacity: usize) -> Self {
        Self {
            mode,
            ledger: IntentLedger::new(dedupe_capacity),
        }
    }

    pub const fn can_submit_live(&self) -> bool {
        false
    }
}

impl ExecutionBackend for NoopBackend {
    fn submit(
        &mut self,
        order: &PreparedOrder,
        now: Instant,
    ) -> Result<SimulationReceipt, ExecutionError> {
        if now > order.valid_until {
            return Err(ExecutionError::Expired);
        }
        self.ledger.claim(order.id)?;
        Ok(SimulationReceipt {
            id: order.id,
            mode: self.mode,
        })
    }
}

/// Single-owner bounded queue; future actor must revalidate ages immediately before signing.
pub struct ExecutionQueue<T> {
    entries: VecDeque<T>,
    capacity: usize,
}

impl<T> ExecutionQueue<T> {
    pub fn new(capacity: usize) -> Self {
        Self {
            entries: VecDeque::new(),
            capacity,
        }
    }
    pub fn try_push(&mut self, item: T) -> Result<(), T> {
        if self.entries.len() >= self.capacity {
            Err(item)
        } else {
            self.entries.push_back(item);
            Ok(())
        }
    }
    pub fn pop(&mut self) -> Option<T> {
        self.entries.pop_front()
    }
}
