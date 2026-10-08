# Latency baseline and telemetry

Measured on beelink, Intel N150 (4 cores), Ubuntu 24.04.3, x86_64, Rust 1.99.0.
Project storage is ext4 on `/dev/mapper/ubuntu--vg-ubuntu--lv`. No affinity, governor,
IRQ, routing, service, linker or compiler CPU tuning was applied. This is a shared
homelab with 14 running containers; results are a short baseline under normal
development load, not controlled or guaranteed production tails.

## Clock and stages

T0 is optional Telegram source wall-clock timestamp, for correlation/source-age
estimation only. T1..T8 use `Instant`: receipt, parse, risk, sign, socket handoff,
ack, fill, protection acknowledgement. Most important eventual local metric is
T5 - T1. Measure adjacent stages separately. Fill updates can precede action acks;
T6 and T7 are independent observations, both after transport. T8 follows a fill.
Never subtract wall-clock values to measure local processing latency.

Implemented: monotonic stage recording, bounded structured event channel with
nonblocking `try_send`, dropped-event counter and HDR summaries. The collector
must run outside execution; it is not wired into a daemon yet. Stage durations
are capped at 60 seconds for histogram recording. Overflow/disconnection drops
telemetry visibly. Standard-library channels can take internal locks: no claim
of a lock-free hot path is made. No signing, transport, ack or fill is simulated
as a real latency event by the CLI.

Report p50/p90/p95/p99/p99.9/max, sample count, units, dropped events, workload,
toolchain/commit/profile and competing load. Logs and latency metrics are separate.
Future normal logs use tracing to journald. Do not synchronously format logs,
serialize metrics or introduce high-cardinality signal/order labels in execution.

Required future fixed counter vocabulary: `messages_received`, `messages_ignored`,
`signals_parsed`, `signals_rejected`, `signals_stale`, `signals_outside_entry`,
`signals_duplicate`, `orders_attempted`, `orders_acked`, `orders_rejected`,
`orders_ambiguous`, `ioc_zero_fill`, `ioc_partial_fill`, `ioc_full_fill`,
`protection_success`, `protection_failure`, `ws_reconnects`,
`telegram_reconnects`, `rate_limits`. These lifecycle counters are not wired yet.

## Synthetic release microbenchmarks

`cargo bench --locked --bench pipeline`: Criterion, 100 samples, 1 second warmup
and 2 second measurement per function. Inputs are artificial `HTS_FIXTURE_V1`
messages and static synthetic quotes/policies, NOT actual channel templates.
Numbers below are central time estimates, NOT request latency percentiles.

| Operation | Estimate | 95% confidence interval |
|---|---:|---:|
| Channel stub classification (always Ignore) | 8.1781 ns | 8.1572–8.2027 ns |
| Synthetic text parse | 204.46 ns | 204.04–204.94 ns |
| Directional TP/SL validation only | 3.3324 ns | 3.2926–3.3753 ns |
| Exact range containment only | 1.4058 ns | 1.4045–1.4071 ns |
| Simulation risk decision | 95.290 ns | 94.820–95.920 ns |
| Synthetic parse → prepared simulation order | 293.37 ns | 292.87–293.96 ns |

Raw Criterion estimates are preserved in `baselines/criterion.json`. Samples
contain outliers. Static inputs and an Ignore stub make several operations very
cheap; these results do not predict the future real parser/executor.

`cargo bench --locked --bench latency_samples`: release, 2,000 warmup iterations,
20,000 measured synthetic parse + simulation-risk calls with fresh local times.
The measurement includes timestamp/setup overhead and HDR quantization; it
excludes dedupe, queue transport, signing, journaling, socket writes and exchange.

| p50 | p90 | p95 | p99 | p99.9 | max |
|---:|---:|---:|---:|---:|---:|
| 407 ns | 500 ns | 555 ns | 604 ns | 768 ns | 15,375 ns |

This is not T5-T1. The sampler also runs in debug under `cargo test --all-targets`;
debug output is a smoke check, not this release baseline. The subsequent safety
change to multi-range rejection does not affect these single-range inputs; no
before/after optimization claim is made.

## Public API network baseline

At 2026-10-07 23:54 UTC, `python3 scripts/baseline.py network` made 10 independent
IPv4 HTTPS connections to `api.hyperliquid.xyz/info`, GET only (HTTP 405 expected),
with bounded timeouts, and 20 ICMP probes. No account or exchange action request.
DNS is curl name lookup; TCP/TLS times are differences of cumulative curl times.

| Measurement | p50 | p90 | observed max |
|---|---:|---:|---:|
| DNS lookup | 2.562 ms | 5.217 ms | 9.560 ms |
| TCP connection, excluding DNS | 8.995 ms | 9.249 ms | 9.922 ms |
| TLS establishment, excluding DNS/TCP | 45.622 ms | 66.725 ms | 70.100 ms |
| Connection through TLS, including DNS/TCP | 59.160 ms | 79.668 ms | 80.262 ms |

Ping destination was 18.245.104.113: min/mean/max 8.547/9.104/9.898 ms,
reported mdev 0.333 ms, 0/20 packet loss. This supersedes the stale audit's
~2.5 ms observation for this run; the cause of the difference is not established.
Ten connections cannot establish reliable p99/p99.9; raw samples and arithmetic
quantiles are in `baselines/network.json`. ICMP/CDN edges are not order latency.
HTTP cold TLS costs strengthen the persistent-connection requirement, but do
not measure action-WS send/ack. WS keepalive/reconnect and Telegram/DC latency
remain unmeasured until their separately approved implementations.

## File durability baseline

At 2026-10-07 23:54 UTC, `python3 scripts/baseline.py durability` created temporary
files ONLY under project `target/`, appended 4,096 bytes, timed the sync call
with `perf_counter_ns`, and removed its own scratch files. Each sync variant
has 20 warmup + 200 measured samples. Timing excludes write/serialization and
directory fsync; files already existed. Host load was not isolated.

| Sync call | p50 | p90 | p95 | p99 | observed max |
|---|---:|---:|---:|---:|---:|
| fsync | 2.956 ms | 3.394 ms | 4.301 ms | 6.178 ms | 6.345 ms |
| fdatasync | 2.885 ms | 3.686 ms | 4.262 ms | 5.994 ms | 6.572 ms |

Raw summaries/method are in `baselines/durability.json`. With 200 samples, p99.9
is effectively the maximum and is not a stable tail estimate. This is a storage
baseline, not a complete journal or power-loss test. Neither the hardware's
power-loss guarantees nor file/directory crash ordering has been certified.

Candidate A: durable recoverable intent before post, terminal records afterward.
Candidate B: deterministic CLOID + recoverable context before post, asynchronous
durable event afterward. B is NOT approved: a deterministic ID alone does not
recover lost source/size/protection context or establish authoritative absence.
Prove crash recovery equivalence before considering it. These sync timings can
consume the desired local millisecond budget; retain correctness and measure
the actual journal before selecting any optimization.

No network execution, Tokyo/VPS/provider comparison or performance improvement
claim was performed. Compare eventual end-to-end signal→order p99 before spending
on infrastructure or changing the host.
