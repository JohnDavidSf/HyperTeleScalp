use std::collections::BTreeSet;

use thiserror::Error;

use crate::model::{SignalId, SourceId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum IntentError {
    #[error("source message already has an intent")]
    Duplicate,
    #[error("intent ledger full; no silent eviction is allowed")]
    Capacity,
}

/// Single-owner, bounded, IN-MEMORY simulation ledger. This provides no crash durability.
pub struct IntentLedger {
    sources: BTreeSet<SourceId>,
    capacity: usize,
}

impl IntentLedger {
    pub fn new(capacity: usize) -> Self {
        Self {
            sources: BTreeSet::new(),
            capacity,
        }
    }

    pub fn claim(&mut self, id: SignalId) -> Result<(), IntentError> {
        // Parser upgrades/edits cannot create a second entry for the same source message.
        if self.sources.contains(&id.source) {
            return Err(IntentError::Duplicate);
        }
        if self.sources.len() >= self.capacity {
            return Err(IntentError::Capacity);
        }
        self.sources.insert(id.source);
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeliveryState {
    Prepared,
    Journaled,
    Posting,
    Acknowledged,
    Ambiguous,
    Reconciling,
    Resolved,
    DegradedProtection,
}
