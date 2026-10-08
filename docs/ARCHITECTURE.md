# Architecture

## Intended runtime

```text
persistent MTProto user connection
  -> live/source-channel filter -> caption/text channel parser
  -> classification + identity + bounded admission
  -> fresh cached metadata/BBO/account snapshots + full risk gates
  -> ONE execution actor -> durable intent -> ONE signer/nonce allocator
  -> already-open action WebSocket -> capped IOC -> ack/fill
  -> exchange-side protection for actual fills -> reconciliation
```

Telemetry uses fixed events through bounded nonblocking admission to a background
HDR collector. Logs are separate (eventually tracing/journald). Offline corpus
import/replay/labeling and optional AI/OCR workers are separate from execution.

## Current implementation versus future work

| Boundary | Phase 0 | Later |
|---|---|---|
| Domain | Exact Decimal/Price/Quantity; identity, entry and update enums | Certified instrument IDs/precision; attributed fills/fees |
| Telegram | Borrowed MessageView and typed metadata | Persistent MTProto user session and reconnect provenance |
| Parser | A/B fail-closed channel stubs; explicitly synthetic fixtures | Proven channel-specific grammar versions |
| Market | Coherent immutable snapshot value | Background catalog + BBO/l2Book subscriptions, connection generations |
| Risk | Simulation subset; typed rejections; capped prepared intent | All account/exposure/strategy/compromise gates |
| Execution | Bounded single-owner queue; no-op receipts; expiry recheck | Async actor, exclusive signer/process lock and action socket |
| Recovery | Bounded in-memory source dedupe and delivery-state vocabulary | Durable journal, deterministic CLOIDs, authoritative reconciliation |
| Replay | Bounded streaming normalized NDJSON classification | Raw export normalization, labels, edit graph, strategy/backtest engine |
| Telemetry | Monotonic stages, fixed events, bounded sink, HDR summaries | Background runtime wiring and fixed counters/logging integration |

No async runtime or networking library is needed for bootstrap. The crate is
not a daemon. A `PreparedOrder` is a simulation result, not a protocol-ready or
live-authorized exchange order. No fill is invented for a no-op receipt.

## Concurrency and snapshot publication decision

Use bounded Tokio admission and one execution task when network adapters arrive.
Exactly one process must own the wallet; an OS-level lifetime lock and API-wallet
separation are needed in addition to Rust ownership. Reject a full queue or old
head item, never run a backlog to catch up. Own the nonce allocator in that actor.
Background jobs may reconnect/refresh with bounded backoff; submission has no retries.

Market snapshots must publish price, size, timestamps, catalog/connection generation
and readiness coherently. ArcSwap of immutable per-asset snapshots is a candidate;
small safe locks are a valid baseline. Independent bid/ask atomics can tear a
snapshot; a seqlock or unsafe lock-free structure is not justified. Benchmark
contention, allocation and stale-state handling before selecting a mechanism.
Freshness and connection health are separate. A live socket does not prove a
new quote, and a quiet unchanged book does not alone prove disconnect.

## Durability designs under investigation

A: establish stable intent/nonce/context, durably journal before send, then
sign/post and durably record outcomes. Pending/ambiguous actions gate restart.
This remains the conservative planned baseline; no journal exists in bootstrap.

B: derive deterministic recoverable CLOID/context, post then asynchronously
persist. B is unapproved. A crash before persistence can erase intended size,
price, protection policy and knowledge of whether sending started. Exchange
lookup visibility/retention, a crash-discoverable source corpus and fully
recoverable original context would need proof; deterministic IDs alone are insufficient.
Measure append/sync and directory-entry durability and fault-inject power-loss
boundaries before considering equivalence. Never remove durability for a speed claim.
