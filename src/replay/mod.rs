use std::{
    io::{BufRead, Read},
    time::{Duration, Instant, SystemTime},
};

use serde::Deserialize;
use thiserror::Error;

use crate::{
    model::{MessageClassification, MessageMeta, MessageOrigin, SourceId},
    parser::SignalParser,
    telegram::MessageView,
};

pub const MAX_RECORD_BYTES: u64 = 65_536;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReplayRecord {
    pub channel_id: i64,
    pub message_id: i32,
    pub source_timestamp_ms: Option<u64>,
    pub reference_message_id: Option<i32>,
    pub text: String,
    pub caption: Option<String>,
    pub media: Vec<String>,
    pub edited: bool,
    pub forwarded: bool,
    pub label: Option<String>,
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct ReplaySummary {
    pub messages: u64,
    pub ignored: u64,
    pub signals: u64,
    pub updates: u64,
}

#[derive(Debug, Error)]
pub enum ReplayError {
    #[error("replay I/O: {0}")]
    Io(#[from] std::io::Error),
    #[error("invalid normalized JSON record on line {line}")]
    Invalid {
        line: u64,
        #[source]
        source: serde_json::Error,
    },
    #[error("normalized replay record exceeds 64 KiB")]
    Oversized,
    #[error("invalid source identity or timestamp in replay record")]
    Metadata,
}

/// Streams bounded normalized NDJSON; raw Telegram export conversion is future work.
/// Historical messages are classified only; this function has no execution backend.
pub fn replay(
    reader: impl BufRead,
    parser: &impl SignalParser,
) -> Result<ReplaySummary, ReplayError> {
    let mut reader = reader;
    let mut summary = ReplaySummary::default();
    let mut line = Vec::new();
    let mut line_number = 0;
    loop {
        line.clear();
        let read = reader
            .by_ref()
            .take(MAX_RECORD_BYTES + 1)
            .read_until(b'\n', &mut line)?;
        if read == 0 {
            break;
        }
        if line.len() as u64 > MAX_RECORD_BYTES {
            return Err(ReplayError::Oversized);
        }
        line_number += 1;
        if line.iter().all(u8::is_ascii_whitespace) {
            continue;
        }
        let record: ReplayRecord =
            serde_json::from_slice(&line).map_err(|source| ReplayError::Invalid {
                line: line_number,
                source,
            })?;
        if record.channel_id == 0 || record.message_id <= 0 {
            return Err(ReplayError::Metadata);
        }
        let timestamp = match record.source_timestamp_ms {
            Some(ms) => Some(
                SystemTime::UNIX_EPOCH
                    .checked_add(Duration::from_millis(ms))
                    .ok_or(ReplayError::Metadata)?,
            ),
            None => None,
        };
        let meta = MessageMeta {
            source: SourceId {
                channel_id: record.channel_id,
                message_id: record.message_id,
            },
            reference_message_id: record.reference_message_id,
            received_at: Instant::now(),
            source_timestamp: timestamp,
            source_age_at_receive: None,
            origin: MessageOrigin::Replay,
            edited: record.edited,
            forwarded: record.forwarded,
        };
        let view = MessageView {
            meta: &meta,
            text: &record.text,
            caption: record.caption.as_deref(),
            has_media: !record.media.is_empty(),
        };
        summary.messages += 1;
        match parser.classify(&view) {
            MessageClassification::Ignore(_) => summary.ignored += 1,
            MessageClassification::NewSignal(_) => summary.signals += 1,
            _ => summary.updates += 1,
        }
    }
    Ok(summary)
}
