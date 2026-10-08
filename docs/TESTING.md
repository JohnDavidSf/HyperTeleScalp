# Testing and acceptance

Run `bash scripts/check.sh` (or the equivalent CI commands). Lockfile is committed;
fmt/check/clippy/test/release/diff-check require no exchange/Telegram access or
credentials. Dependency/toolchain installation requires registry access, but
tests themselves are offline. CI does not assert profitability or live readiness.

Bootstrap tests prove hype/ticker mentions/incomplete/malformed messages are
ignored, real channel stubs never accept the synthetic grammar, caption-first
classification, exact decimal/tick behavior, duplicates across parser versions,
bounded no-eviction ledger, stale signal/queue/book, unknown source age, missing
book/thin depth/excess spread, outside-range entry, invalid TP/SL in either
direction, caps/budgets at the worst allowed price, short/CMP directionality,
no live backend, expired/duplicate no-op intents, unsafe config rejection,
bounded nonblocking telemetry, monotonic stage discipline including fill-before-ack,
and malformed/overlapping multi-zone rejection. Replay classifies
bounded records and cannot call execution. Property tests exercise arbitrary
UTF-8 and exact decimal round trips; they are not exhaustive formal proofs.

Fixtures are clearly artificial placeholders. Phase 1 requires real approved
history and labeled non-signals, negatives that resemble templates, old/new
template splits and held-out time periods. Replay every message; measure false
positives first. Export importer must preserve identities, edits/replies,
captions/media, timestamps and Unicode, including loss/reporting semantics.

Future executor fault tests must cover zero/partial/full fill; lost entry/fill/
protection ack; disconnect before/during/after write; durable pending/terminal
write failure; crash at each journal/send/ack boundary; stale/old snapshot;
duplicate daemon/key; manual positions; insufficient margin; revoked wallet;
rate limit and malformed replies. Assert exact quantities/CLOIDs/price caps,
halt/reconcile behavior and no blind retry. Protection failure must prove
degraded state, safe retry and optional emergency reduce-only flattening.

Grouping/IOC partial-fill native protection must be tested separately against
the current protocol; unit mocks cannot prove its behavior. Approved testnet
validation needs small bounded exposure, independent before/after order/fill/
position reconciliation, actual native protection and process-offline survival.
No testnet or live acceptance was performed in Phase 0.

Criterion benchmarks and HDR samples are local synthetic baselines; rerun with
representative actual grammars when supplied. Record toolchain, commit, hardware,
optimization profile, input, sample size, competing load and tail distributions.
Do not widen tests merely to inflate count or claim one timing as production p99.
