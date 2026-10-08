# HyperTeleScalp

Private Rust project for deterministic signals from two paid Telegram channels
and eventual execution on Hyperliquid perpetuals. **Phase 0 is offline only.
LIVE TRADING IS DISABLED and cannot be enabled in this build.**

Read [AGENTS.md](AGENTS.md) and [current project state](docs/PROJECT_STATE.md)
before development. The existing homelab Telegram bot is unrelated.

## Getting started

```bash
cd /opt/homelab/hypertele-scalp
source /home/server/.cargo/env
cargo run --locked -- check-config config/config.example.toml
cargo run --locked -- replay-fixtures tests/fixtures/synthetic.ndjson
bash scripts/check.sh
cargo bench --locked --bench pipeline
cargo bench --locked --bench latency_samples
```

The Rust toolchain is pinned. No Telegram account, wallet, network service,
dotenv file or real credentials are required. `replay-fixtures` classifies the
clearly artificial `HTS_FIXTURE_V1` grammar; it never executes. Real channel
parsers currently ignore all messages. Raw Telegram export import is future work.

## Boundaries

One crate with `model`, `telegram`, `parser`, `market`, `risk`, `execution`,
`recovery`, `replay`, `telemetry` and `config`. The no-op backend records
simulation receipts only, without claiming exchange acceptance or fills.
Simulation risk checks cover representative gates; account-risk enforcement,
durable recovery, precision certification and real transports are not implemented.

Start with [product specification](docs/PRODUCT_SPEC.md),
[architecture](docs/ARCHITECTURE.md), [protocol audit](docs/HYPERLIQUID_PROTOCOL.md),
[reference audit](docs/REFERENCE_AUDIT.md), [testing](docs/TESTING.md) and
[latency evidence](docs/LATENCY.md). Production service is an uninstalled example.
No third-party implementation source was copied. No open-source license has
been assigned to this private project's original code.
