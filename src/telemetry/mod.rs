use std::{
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
        mpsc::{Receiver, SyncSender, TrySendError, sync_channel},
    },
    time::{Instant, SystemTime},
};

use hdrhistogram::Histogram;
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(usize)]
pub enum Stage {
    Receive = 0,
    Parse,
    Risk,
    Sign,
    Transport,
    Ack,
    Fill,
    Protection,
}

pub struct LatencyStages {
    pub source_time: Option<SystemTime>,
    local: [Option<Instant>; 8],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum TelemetryError {
    #[error("latency stages must be present, ordered, and assigned once")]
    StageOrder,
    #[error("histogram setup or recording failed")]
    Histogram,
    #[error("telemetry capacity must be in 1..=65536")]
    Capacity,
}

impl LatencyStages {
    pub fn new(received: Instant, source_time: Option<SystemTime>) -> Self {
        let mut local = [None; 8];
        local[0] = Some(received);
        Self { source_time, local }
    }
    pub fn mark(&mut self, stage: Stage, time: Instant) -> Result<(), TelemetryError> {
        let index = stage as usize;
        if index == 0 || self.local[index].is_some() {
            return Err(TelemetryError::StageOrder);
        }
        let prerequisite = match stage {
            Stage::Ack | Stage::Fill => Stage::Transport as usize,
            Stage::Protection => Stage::Fill as usize,
            _ => index - 1,
        };
        let previous = self.local[prerequisite].ok_or(TelemetryError::StageOrder)?;
        if time < previous {
            return Err(TelemetryError::StageOrder);
        }
        self.local[index] = Some(time);
        Ok(())
    }
    pub fn event(&self, stage: Stage) -> Option<LatencyEvent> {
        let end = self.local[stage as usize]?;
        let start = self.local[0]?;
        let nanos = u64::try_from(end.checked_duration_since(start)?.as_nanos()).ok()?;
        Some(LatencyEvent { stage, nanos })
    }

    pub fn between(&self, start: Stage, end: Stage) -> Option<std::time::Duration> {
        self.local[end as usize]?.checked_duration_since(self.local[start as usize]?)
    }
}

/// Fixed structured cumulative T(stage)-T1 duration. No formatted strings or signal labels.
#[derive(Debug, Clone, Copy)]
pub struct LatencyEvent {
    pub stage: Stage,
    pub nanos: u64,
}

pub struct TelemetrySink {
    sender: SyncSender<LatencyEvent>,
    dropped: Arc<AtomicU64>,
}
pub struct TelemetryCollector {
    receiver: Receiver<LatencyEvent>,
    histograms: Vec<Histogram<u64>>,
    dropped: Arc<AtomicU64>,
}

pub fn bounded_telemetry(
    capacity: usize,
) -> Result<(TelemetrySink, TelemetryCollector), TelemetryError> {
    if !(1..=65_536).contains(&capacity) {
        return Err(TelemetryError::Capacity);
    }
    let (sender, receiver) = sync_channel(capacity);
    let dropped = Arc::new(AtomicU64::new(0));
    let histograms = (0..8)
        .map(|_| {
            Histogram::<u64>::new_with_bounds(1, 60_000_000_000, 3)
                .map_err(|_| TelemetryError::Histogram)
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok((
        TelemetrySink {
            sender,
            dropped: dropped.clone(),
        },
        TelemetryCollector {
            receiver,
            histograms,
            dropped,
        },
    ))
}

impl TelemetrySink {
    pub fn record(&self, event: LatencyEvent) {
        if let Err(TrySendError::Full(_) | TrySendError::Disconnected(_)) =
            self.sender.try_send(event)
        {
            self.dropped.fetch_add(1, Ordering::Relaxed);
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct LatencySummary {
    pub count: u64,
    pub p50_ns: u64,
    pub p90_ns: u64,
    pub p95_ns: u64,
    pub p99_ns: u64,
    pub p999_ns: u64,
    pub max_ns: u64,
    pub dropped: u64,
}

impl TelemetryCollector {
    /// Run on a background collector, never on the future execution owner.
    pub fn drain(&mut self) -> Result<(), TelemetryError> {
        for event in self.receiver.try_iter() {
            if event.nanos > 60_000_000_000 {
                self.dropped.fetch_add(1, Ordering::Relaxed);
                continue;
            }
            self.histograms[event.stage as usize]
                .record(event.nanos)
                .map_err(|_| TelemetryError::Histogram)?;
        }
        Ok(())
    }
    pub fn summary(&self, stage: Stage) -> LatencySummary {
        let h = &self.histograms[stage as usize];
        LatencySummary {
            count: h.len(),
            p50_ns: h.value_at_quantile(0.5),
            p90_ns: h.value_at_quantile(0.9),
            p95_ns: h.value_at_quantile(0.95),
            p99_ns: h.value_at_quantile(0.99),
            p999_ns: h.value_at_quantile(0.999),
            max_ns: h.max(),
            dropped: self.dropped.load(Ordering::Relaxed),
        }
    }
}
