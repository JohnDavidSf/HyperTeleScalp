# HyperTeleScalp current state

Updated 2026-10-07 (America/Montreal). **LIVE TRADING: DISABLED.** Phase 0 only;
Phase 1 is not approved. Repository is private, at `/opt/homelab/hypertele-scalp/`
with directory `775 server:server`. Branch: `codex/bootstrap`.

Implementation revision: pending bootstrap commit; initial upstream main was
`952424494139034e4b4f55a88f5dc861d0a1995f`. Current revision is available with
`git rev-parse HEAD`; this record will identify the verified implementation
commit before handoff. No direct main changes or force pushes.

## Goal and architecture

Listen to two paid Telegram channels via persistent MTProto USER connection;
deterministic channel-specific parsing → typed intent/dedupe → cached metadata
and fresh coherent book → bounded queue → one risk/execution actor and signer →
persistent Hyperliquid action WS → capped IOC → protect actual fills natively →
reconcile. Target competitive window is 3–5 seconds; local T5−T1 target is
sub-ms to low single-digit ms, subject to measurement and durability correctness.
AI/OCR are separate asynchronous/offline paths. Unknown text always skips.

## Implemented and verified scope

- One Rust crate; pinned toolchain, lockfile, CI and host-safe check/baseline scripts.
- Typed identity, direction, classifications/updates, entry range/CMP/multiple
  zones; checked exact u128 decimal (12 fractional digits), positive price/size.
- Parser interface, real channel stubs that ALWAYS Ignore, explicitly artificial
  fixture grammar and normalized bounded NDJSON replay classification.
- Strict bounded config: only DryRun/Replay, `trading_enabled=true` rejected,
  empty allowlist/zero budgets by example, strategy/margin/leverage unset.
- Simulation-only risk subset: identity, duplicate/edited/forwarded safeguards,
  known source age, queue/book age, metadata/readiness, spread/depth, entry cap,
  directional protection, exact notional/loss at worst allowed cap. Dedupe is
  bounded in-memory with no eviction; it is not restart-safe exactly-once intent.
- No-op backend emits simulation receipts and cannot submit live trades. No
  signer, transport, credential input or Live/Testnet backend exists.
- Monotonic stage/event/HDR interfaces, bounded telemetry with visible drops;
  synthetic benchmarks and invariant/property tests. Network stages not fabricated.
- Architecture, current protocol/references, risk/security/operations/edge-case
  backlog and future documentary service example, which was NOT installed.

Not implemented: real history-export importer or channel grammars; MTProto
session/login/reconnect; live feeds/catalog or atomic publication; full account
risk; signer/nonces/action WS; durable journal/CLOID/exactly-once recovery;
fills/native protection; daemon/runtime singleton; counters/task wiring; OCR/AI;
strategy/backtest PnL; testnet/live backend. No trade or wallet connection occurred.

## Checks and measurements

Final local gate PASS: fmt/check/clippy/test/release/diff-check. All 18 integration
tests passed, including two 256-case property tests, overlapping/malformed
multi-zone rejection and fill-before-ack telemetry. Benchmark smoke checks and
CLI config/replay commands passed. No remote CI result is claimed before it can
be observed. Local command output remains in ignored `target/bootstrap-checks.log`.

Synthetic release Criterion estimates: Ignore stub 8.1781 ns; fixture parse
204.46 ns; protection validation 3.3324 ns; range check 1.4058 ns; simulation
risk 95.290 ns; parse→prepared simulation order 293.37 ns. Separate 20,000-sample
HDR parse+risk: p50 407 ns, p99 604 ns, p99.9 768 ns, max 15,375 ns. These exclude
signing, journal, socket, exchange and real channel parsing. No optimization claim.

Public API baseline: ping mean 9.104 ms, mdev 0.333 ms, 20 probes/no loss;
10 cold connections DNS/TCP/TLS median 2.562/8.995/45.622 ms. File sync 200 samples:
fsync p50/p99 2.956/6.178 ms; fdatasync 2.885/5.994 ms. Small samples, shared load,
not production tail or power-loss certification. See `LATENCY.md` and raw JSON.

## Host/access facts and allowed changes

beelink: Ubuntu 24.04.3, kernel 7.0.0-30-generic, Intel N150 (4 cores), ~15 GiB
RAM; ~68 GiB root before build caches (~65 GiB afterward), ~551 GiB media free. Fourteen containers,
one old Telegram bot, no failed systemd units at inspection. Dual-NIC policy
routes/timer verified read-only; removed infrastructure absent. Reboot marker
persists and Docker daemon log rotation defaults are not applied to existing
containers; report only, no host fix. Actual public exposure was not externally
audited. No unrelated services, host settings or credentials changed/read for reuse.

Git auth: dedicated repository SSH deploy key via `github-hypertele-scalp` alias,
strict official GitHub Ed25519 host pin in a dedicated known_hosts file. Owner
installed the public deploy key with write permission. No PAT, key overwrite or
credential-store helper. Repo-local identity: John-David <johndsfeir@gmail.com>.
User-level Rust installed as server with minimal rustfmt/clippy, no profile file
edits, apt, native OpenSSL or Python/Node/library upgrades. Exact file inventory
is in `FILE_MANIFEST.md`.

## Exact toolchain output

`rustc -Vv`:

```text
rustc 1.99.0 (b940084d7 2026-09-28)
binary: rustc
commit-hash: b940084d7eb6a299eb4bfeb8e34901bc051e7ac4
commit-date: 2026-09-28
host: x86_64-unknown-linux-gnu
release: 1.99.0
LLVM version: 23.1.1
```

`cargo --version`: `cargo 1.99.0 (5f94df478 2026-08-27)`.
`rustup --version`: `rustup 1.29.1 (d95a37b6a 2026-08-13)`.
`rustfmt --version`: `rustfmt 1.10.0-stable (b940084d7e 2026-09-28)`.
`cargo clippy --version`: `clippy 0.1.99 (b940084d7e 2026-09-28)`.
`rustup show` in the project:

```text
Default host: x86_64-unknown-linux-gnu
rustup home:  /home/server/.rustup

installed toolchains
--------------------
stable-x86_64-unknown-linux-gnu (default)
1.99.0-x86_64-unknown-linux-gnu (active)

active toolchain
----------------
name: 1.99.0-x86_64-unknown-linux-gnu
active because: overridden by '/opt/homelab/hypertele-scalp/rust-toolchain.toml'
installed targets:
  x86_64-unknown-linux-gnu
```

Direct dependencies: serde 1.0.229 (strict config/corpus types), serde_json
1.0.151 (NDJSON), toml 1.1.6+spec-1.1.0 (config), thiserror 2.0.21 (typed errors),
hdrhistogram 7.6.0 (latency summaries). Dev only: Criterion 0.8.2 (benchmarks),
proptest 1.11.0 (UTF-8/arithmetic properties). No Telegram/exchange/TLS dependency.

## Unresolved decisions and next approved work

Current official Hyperliquid docs checked 2026-10-07; reference commits and
engineering selection are in `REFERENCE_AUDIT.md`. Grammers migrated from archived
GitHub to Codeberg, current candidate 0.10.0; investigate integration before choosing.
IOC grouped protection on partial fills MUST be independently validated. CLOID
alone does not prove safe resend or async journal equivalence. Full account risk,
margin/leverage/TP/SL strategy, symbol alias/HIP-3 policy, source clock-age policy,
single-owner persistence and runtime sandbox need explicit future decisions.

Recommended Phase 1 (owner approval required): privacy-safe import of both actual
channel histories, labels/template discovery, deterministic grammars and full-corpus
replay/negative benchmarks. No credentials or live trading needed. Keep unknown
templates/images skipped; review evidence before moving to MTProto shadow/testnet.
