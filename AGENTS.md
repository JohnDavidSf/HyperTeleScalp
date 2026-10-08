# HyperTeleScalp agent contract

Read this file first, then `docs/PROJECT_STATE.md`. Inspect the working tree and
existing work before edits. This is a PRIVATE, money-moving project currently
restricted to Phase 0/offline development. Live trading is DISABLED.

## Scope and host

- Project location is `/opt/homelab/hypertele-scalp/`, `775 server:server`.
- Read `/home/server/CLAUDE.md` before any HOST changes. In
  `/home/server/HOMELAB_CODEX_HANDOFF.md`, the top 2026-10-07 decommission
  addendum supersedes conflicting old body content. Verify live state.
- Never touch unrelated homelab services, Docker/containers, the old Telegram
  bot/Hyperliquid monitor, UFW, Tailscale, DNS, routing, sysctl, mounts, media,
  Home Assistant, systemd services, CPU governor, IRQs or reboot state as part
  of project work. No restarts/reboots/performance tuning during bootstrap.
- User-level Rust belongs to `server`, never root; prefer rustls when network
  dependencies are eventually added. No global Python/Node/system upgrades.
- Explicit session instructions outrank local guidelines. Full machine
  permissions do not imply a request to change unrelated services.

## Trading and secrets

- NEVER trade real money without explicit owner authorization for that phase,
  account, release and exposure. Use replay, shadow mode and testnet first.
- No Live backend exists. Do not add one during bootstrap. Configurations
  requesting live/testnet execution or `trading_enabled=true` must be rejected.
- Never expose or commit private keys, seeds, API hashes, Telegram sessions,
  credentials, cookies, tokens, private corpus data or execution journals.
- Do not read the old bot's credentials or reuse its code, token, polling or
  state. Telegram ingestion will use a separate MTProto USER session.
- Use the dedicated repository SSH deploy key, never credential-helper store
  or credentials in remote URLs. Private key must never be printed.
- Future runtime: dedicated minimally privileged `hypertele` user and dedicated
  API wallet; owner authorization and hardening validation required before deployment.

## Architecture invariants

- Normal text processing is deterministic and channel-specific. Unknown,
  incomplete, ambiguous or changed templates cannot become trades. Ignore them.
- Real channel parsers remain stubs until history supports a reviewed grammar.
  `HTS_FIXTURE_V1` is artificial and must never be described as a provider template.
- AI and OCR stay outside the text hot path. Caption first; image workers are
  separate, asynchronous and disabled for automatic trading until validated.
- Exact integer/decimal comparisons, checked arithmetic, typed errors. No
  authoritative floating point, hidden retries, fallback guesses or unsafe code.
- Exactly one execution owner/signer per wallet. Bounded execution queue;
  recheck source age, queue age, market freshness and risk before signing.
- Ambiguous delivery must halt affected execution and reconcile against
  authoritative exchange state. NEVER blindly retry or treat `unknownOid` as
  proof that an action was not accepted. CLOIDs are reconciliation identities,
  not a blanket exactly-once execution guarantee.
- IOC fills may be partial. Track actual quantity/average price/fees and protect
  only attributed filled exposure. Native exchange protection is mandatory;
  protection failure is an explicit degraded state with a tested policy.
- Hot path has no DNS/connect/TLS/login/subscription setup, REST/SQL/filesystem
  lookup, OCR/model inference, config loading, sleeps, retries or formatted logs.
  Durability is the one separately measured synchronous exception.
- Use `Instant` for local durations; wall/source time only for correlation and
  conservatively established source age. Reconnect history is not live execution.
- Check CURRENT official Hyperliquid docs before protocol changes. Record date,
  reference commits and testnet evidence. A reference implementation is not the contract.

## Work and verification

- Use development branches; no direct main pushes, force pushes or overwriting
  existing work. Preferred bootstrap branch: `codex/bootstrap`.
- Update `docs/PROJECT_STATE.md` after meaningful work. Decisions belong in
  `docs/DECISIONS.md`; preserve concise current state and explicit limitations.
- Performance claims need reproducible measurements. Optimization claims need
  before/after benchmarks, including tail latency. No host tuning before baseline.
- Required gate: `cargo fmt --all -- --check`, `cargo check --locked --all-targets`,
  `cargo clippy --locked --all-targets -- -D warnings`,
  `cargo test --locked --all-targets`, `cargo build --locked --release`,
  `git diff --check`. Keep Cargo.lock committed. CI uses no exchange secrets or connectivity.
- Add meaningful invariant/regression tests. Tests do not prove exchange acceptance
  or profitability. Keep code APIs small and auditable; no speculative workspace/framework.
- Stop at the approved phase boundary. Phase 1 requires explicit owner approval.
