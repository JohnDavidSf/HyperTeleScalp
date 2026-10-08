# Roadmap and approval boundaries

## Phase 0 — current authorized scope

Secure clone/development branch, user Rust, project contract/docs, typed offline
models, parser/risk interfaces, no-op backend, fixtures/property tests, bounded
replay, telemetry structures, benchmark/network/durability baselines and CI.
Commit/push branch; stop and report. No Telegram, wallet, orders or service.

## Phase 1 — proposed; NOT approved

Import owner-supplied histories from both channels. Define normalization and
edit/reply/media provenance, private storage/retention and labeling. Analyze
formats/frequency/symbols; implement reviewed deterministic channel grammars;
replay every historical message and document false positives/unknowns. Add
representative held-out fixtures and benchmarks. Remain offline.

## Later separately approved phases

1. MTProto user ingestion in read-only/shadow mode; connection/session review,
   reconnect/live-age controls and receive-time telemetry. No execution.
2. Read-only Hyperliquid catalog/BBO/account adapters, precision certification,
   coherent snapshot publishing and readiness/market replay.
3. Durable actor/signing/action transport against mocks, full risk gates,
   deterministic CLOIDs and crash/ambiguous/protection regression tests.
4. Explicitly authorized small testnet acceptance, native partial-fill protection,
   reconnect/restart reconciliation and latency soak.
5. Strategy calibration with fees, delays, liquidity and fill assumptions;
   compare infrastructure by end-to-end p99 only if measured benefit justifies it.
6. Explicit live authorization, dedicated account/API wallet and approved
   budgets, service/user/backup/hardening validation, release certification.

No phase implies approval for the next, and no export or testnet success authorizes live trading.
