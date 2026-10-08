# Phase 0 exact file inventory

Repository: `/opt/homelab/hypertele-scalp/`, branch `codex/bootstrap`.
Only the pre-existing `README.md` was modified; all other listed source/docs files
were created. Cargo.lock is generated and committed. Build outputs/logs under
ignored `target/` are not source deliverables.

| Action | Repository-relative file |
|---|---|
| Created | `.env.example` |
| Created | `.github/workflows/ci.yml` |
| Created | `.gitignore` |
| Created | `AGENTS.md` |
| Created | `Cargo.lock` |
| Created | `Cargo.toml` |
| Modified | `README.md` |
| Created | `benches/latency_samples.rs` |
| Created | `benches/pipeline.rs` |
| Created | `config/config.example.toml` |
| Created | `docs/ARCHITECTURE.md` |
| Created | `docs/DECISIONS.md` |
| Created | `docs/EDGE_CASES.md` |
| Created | `docs/FILE_MANIFEST.md` |
| Created | `docs/HOT_PATH.md` |
| Created | `docs/HYPERLIQUID_PROTOCOL.md` |
| Created | `docs/LATENCY.md` |
| Created | `docs/OPERATIONS.md` |
| Created | `docs/PRODUCT_SPEC.md` |
| Created | `docs/PROJECT_STATE.md` |
| Created | `docs/REFERENCE_AUDIT.md` |
| Created | `docs/RISK.md` |
| Created | `docs/ROADMAP.md` |
| Created | `docs/SECURITY.md` |
| Created | `docs/SIGNAL_FORMAT.md` |
| Created | `docs/TELEGRAM.md` |
| Created | `docs/TESTING.md` |
| Created | `docs/baselines/criterion.json` |
| Created | `docs/baselines/durability.json` |
| Created | `docs/baselines/latency_samples.json` |
| Created | `docs/baselines/network.json` |
| Created | `rust-toolchain.toml` |
| Created | `rustfmt.toml` |
| Created | `scripts/baseline.py` |
| Created | `scripts/check.sh` |
| Created | `src/config.rs` |
| Created | `src/execution/mod.rs` |
| Created | `src/lib.rs` |
| Created | `src/main.rs` |
| Created | `src/market/mod.rs` |
| Created | `src/model/decimal.rs` |
| Created | `src/model/mod.rs` |
| Created | `src/parser/mod.rs` |
| Created | `src/recovery/mod.rs` |
| Created | `src/replay/mod.rs` |
| Created | `src/risk/mod.rs` |
| Created | `src/telegram/mod.rs` |
| Created | `src/telemetry/mod.rs` |
| Created | `systemd/hypertele-scalp.service.example` |
| Created | `tests/bootstrap.rs` |
| Created | `tests/fixtures/README.md` |
| Created | `tests/fixtures/arbitrary-utf8.txt` |
| Created | `tests/fixtures/synthetic.ndjson` |
| Created | `tests/support/mod.rs` |

Total: 54 repository files (53 created, one modified).

## Repository/access/tooling metadata outside the source inventory

- Project `.git/` was created by clone; local config sets the deploy-key origin,
  branch upstream and owner-supplied identity. Development directory mode is 775.
- `/home/server/.ssh/config`: added repository-specific alias, preserving existing
  configuration; mode 600.
- `/home/server/.ssh/hypertele_scalp_known_hosts`: created dedicated pinned GitHub
  public host-key file; mode 600. No existing known_hosts was replaced.
- Existing dedicated `/home/server/.ssh/hypertele_scalp_ed25519` and `.pub`
  were used without overwrite; private key was never printed or committed.
- User-level rustup installed under `/home/server/.rustup/` and Cargo/tool shims
  and caches under `/home/server/.cargo/`. Minimal stable/pinned 1.99.0 toolchains
  have rustfmt/clippy. Shell profile files were not changed.
- Temporary official installer `/tmp/hypertele-rustup-init.sh` and read-only
  reference checkouts `/tmp/hypertele-reference-hlcli/`,
  `/tmp/hypertele-reference-sdk/` were used for inspection. No reference source
  was copied into the project.

No real .env, session, wallet, token, journal, installed systemd unit, runtime
user, host service/configuration change or external trading action was created.
