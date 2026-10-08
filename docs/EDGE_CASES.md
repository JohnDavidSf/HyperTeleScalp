# Edge cases and acceptance contract

This is a future executor acceptance backlog, not a claim of implemented handling.
Bootstrap covers deterministic negative parsing, exact arithmetic, duplicate
intents, source/queue/book age, range/spread/depth/TP/SL/budget gates, no-op-only
execution, config bounds and telemetry ordering. See `TESTING.md` for current
evidence. ALL network, account, durable recovery, update and protection behavior
below still needs implementation plus independent fault/testnet validation.

## Sources, identity and updates

| Case | Required behavior |
|---|---|
| Both channels signal same coin simultaneously | Serialize intents; account/per-symbol limits and explicit aggregation policy |
| Both channels signal opposite directions | Explicit conflict/hedge policy; never silently flip an existing position |
| Same signal reposted | Canonical identity/content/reference policy; no duplicate exposure |
| Forwarded copy | Reject executable forward until provenance policy is validated |
| Deleted message | Preserve audit/state; deletion cannot silently erase an existing position |
| Edited message | Treat as versioned update, not a new entry |
| Edit changes entry | Revalidate unexecuted intent; never chase or duplicate filled entry |
| Edit changes TP | Explicit owned-position policy with safe protection replacement |
| Edit changes SL | Explicit policy; maintain protection during replacement |
| Reply with updated TP | Resolve source identity; reject unmatched/ambiguous reply |
| Reply with “move SL to entry” | Exact supported grammar and owned fill price; no guessed linkage |
| “secure profits” | Unknown semantics skip; defined policy required before partial/stop action |
| “take partials” | Explicit fraction/quantity and ownership; ambiguity skips |
| “close now” | Proven reference and owned reduce-only size; reconcile result |
| “cancel setup” | Cancel only unexecuted/owned intent; filled position needs separate policy |
| “do not enter” | Suppress matching intent; maintain already-required protection |
| Signal followed by correction | Version/order/age rules; no second entry by accident |
| Ordinary hype mentioning ticker | Ignore |
| Analysis mentioning entry-like numbers but not signal | Ignore unless exact channel grammar proves a signal |
| Provider removes a signal | Retain evidence and apply explicit cancellation/update policy |
| Parse template changes | Skip; offline investigation and new labeled fixtures |

## Numbers, symbols and semantics

| Case | Required behavior |
|---|---|
| Malformed numbers | Reject; checked arithmetic cannot silently truncate/overflow |
| Comma decimal separator | Only explicit channel grammar may normalize; otherwise reject |
| Thousands separators | Grammar-specific unambiguous normalization; otherwise reject |
| Unicode dash | Explicit grammar support, no broad punctuation guessing |
| Emoji around symbols | Exact permitted wrappers and symbol mapping |
| `$BTC`, `BTCUSDT`, `BTC`, aliases | Validated channel-specific alias map against cached metadata |
| Symbol not on Hyperliquid | Reject |
| Delisted symbol | Invalidate metadata/readiness; reject and reconcile existing exposure |
| Newly listed symbol | Background catalog update; remain disabled until allowlisted/validated |
| Symbol alias collision | Reject ambiguous mapping |
| HIP-3/non-native symbol ambiguity | Require dex-qualified identity and separately verified asset/account rules |
| Long/short missing | Ignore or manual review |
| Several entry zones | Preserve zones; reject invalid or overlapping ambiguous executable policy |
| Several TP targets | Preserve all; explicit backtestable selection/scale-out policy |
| No SL | No executable entry until protection policy proves safe |
| No TP | Bootstrap rejects incomplete protection; future policy must be explicit |
| CMP semantics | Preserve reference and directional bounds; never infer unlimited entry |
| Provider TP/SL impossible relative to direction | Reject against observed and worst allowed entry |
| Provider typo | Reject, retain evidence, never infer a trade |

## Market, entry and protection

| Case | Required behavior |
|---|---|
| Current price outside range | Skip |
| Price crosses allowed range while parsing | Re-read fresh snapshot/revalidate before signing/posting |
| Price jumps several % between receive and execution | Age/velocity/range/slippage caps; skip runaway quote |
| Stale BBO | Reject |
| Missing ask/bid | Reject |
| Giant spread | Reject |
| Thin book | Reject insufficient executable liquidity |
| Sudden velocity spike | Configured measured velocity gate/circuit breaker |
| IOC zero fill | Record zero; no protection for requested unfilled size, no implicit retry |
| IOC partial fill | Track actual filled quantity/average/fees; protect only actual exposure |
| IOC full fill | Reconcile exact fill and mandatory native protection |
| Entry accepted but ack lost | Ambiguous; halt action path and reconcile identity/state |
| Entry filled but ack lost | Reconcile fills/position; protect actual fill, never blindly resend |
| Protection accepted but ack lost | Ambiguous protection; reconcile before retrying or flattening |
| Entry filled but protection placement fails | Degraded protection; safe retry/cancel and configurable emergency reduce-only flatten |
| Native grouped TP/SL with partial IOC | Independently validate activation/cancellation semantics; assume no guarantee yet |

## Connections, recovery and time

| Case | Required behavior |
|---|---|
| Connection loss before send | Only prove “not sent” from transport state; revalidate age before any retry |
| Connection loss during send | Ambiguous; halt/reconcile |
| Connection loss immediately after send | Ambiguous; halt/reconcile |
| Telegram disconnect | Backoff outside execution; no stale catch-up trading |
| Telegram replay after reconnect | Dedupe, origin and maximum age checks |
| Telegram backlog | Historical/replayed messages are research/state; never catch up old signals |
| Hyperliquid WS disconnect | Mark readiness/feed state invalid; bounded reconnect and reconcile |
| Exchange rate limit | Reject/defer only outside hot submit; do not accumulate executable backlog |
| Exchange 5xx | Delivery classification determines ambiguous vs definite rejection; no blind retry |
| Stale exchange state | Halt affected path until authoritative reconciliation/freshness |
| Process panic | Native protection persists; recover durable intents before readiness |
| Process restart | Single owner, journal replay, orders/fills/positions reconciliation before entry |
| Machine reboot | Same recovery; no implicit restart authorization or stale catch-up |
| Power loss | Crash-safe journal/file/directory ordering and exchange-side protection |
| Internet loss | Reject new execution; protection remains exchange-side |
| DNS failure | Reconnect unavailable; no DNS on normal submit path |
| Clock adjustment | Local age/latency use Instant; nonce/source-age wall-clock policy fail-safe |
| Two concurrent signals | Bounded queue and single signer/actor; each revalidated after waiting |
| Execution queue buildup | Reject/drop stale intents, no catch-up |
| Duplicate signer | Exclusive key owner across tasks/processes; halt unexpected nonce activity |
| Duplicate daemon | Proven account/key-scoped singleton guard before readiness |
| Unknown CLOID/order status | Unknown is not proof of non-delivery; reconcile all authoritative evidence |
| Journal sync/write fails | No post without required durable intent; conservative halt after uncertain terminal persistence |

## Account and source risk

| Case | Required behavior |
|---|---|
| API wallet revoked | Halt; owner intervention and key lifecycle recovery |
| Insufficient margin | Reject/reconcile fills/canceled parent and protection state |
| Leverage changed externally | Invalidate verification; never blindly reset from source suggestion |
| Position modified manually | Reconcile ownership and remaining protected quantity; configurable halt |
| User opens same symbol manually | Explicit ownership/mixing policy; cannot silently close manual exposure |
| Source channel compromised | Channel circuit breaker and master disable |
| Source suddenly sends 50 fake signals | Per-channel rate cap, bounded queues, compromised-source breaker |
| Bot starts while existing positions/orders exist | Full authoritative reconciliation and ownership policy before readiness |
| Gross/net/daily-loss/drawdown limits breached | Block new risk; protection/managed reductions remain safely available |
| Cross margin several simultaneous positions | Cap gross notional, margin use, summed modeled loss and count; cross is not unlimited risk |

## Images

| Case | Required behavior |
|---|---|
| Image has text + caption conflict | Valid deterministic caption has its own path; image shadow worker flags conflict, never overrides a trade |
| Image signal arrives with no caption | Separate bounded OCR worker; image execution disabled until separately validated |
| OCR misreads decimal point | Same deterministic grammar plus strict confidence/sanity; ambiguity skips |
| OCR misreads ticker | Exact catalog mapping/confidence; ambiguity skips |
| Image arrives before caption/edit | Versioned source identity; later caption cannot double-enter |

No unknown text, OCR result, ambiguous reference or uncertainty above may be
promoted automatically into live exposure. Explicit source/update/risk policies
and tests are prerequisites; current channel parsers ignore every message.
