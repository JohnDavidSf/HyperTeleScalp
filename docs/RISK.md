# Risk contract

LIVE TRADING DISABLED. Bootstrap exposes simulation decisions only; there is
no signer or live authorization. Config accepts only DryRun/Replay and rejects
`trading_enabled=true`. Empty symbol allowlist and zero notional/loss budgets
fail closed. Sample nonzero timing/bps settings are synthetic development
examples, not live policies. Margin/leverage are optional and unset.

## Implemented simulation subset

Reject inconsistent/invalid source identity, edited/forwarded unreviewed
messages, unallowlisted/unsupported/mismatched symbols, not-ready exchange
snapshot, unknown source age, old signal/queue/BBO, missing/crossed book,
excess spread, insufficient quoted size, price outside entry, invalid/missing
TP/SL, checked arithmetic failure, and modeled notional/stop-distance limits.
Prepared simulation price is capped by both slippage and source bounds; budgets
and TP/SL are checked at the worst allowed price. No-op submission rechecks
expiry and claims a bounded source identity. No IOC or protection is actually sent.

The modeled loss uses stop distance; it is not a realized-loss guarantee.
Fees, funding, gaps and stop slippage remain unmodeled. Account exposure and
margin are not represented by an apparently complete dummy account.

## Mandatory future pre-sign gate inventory

| Area | Gates still required before any live authorization |
|---|---|
| Master/source | trading_enabled, exact known template, proven channel, valid direction, source/live age and reconnect provenance |
| Instrument | account-supported perp, explicit alias mapping, listing/delisting/catalog generation, no HIP-3 ambiguity |
| Market/entry | fresh coherent book and connection generation, spread, depth, slippage, configurable entry deviation, short-term velocity |
| Per trade | notional, actual quantity precision, maximum potential loss including costs and protection slippage |
| Account | maximum gross/net exposure, total margin usage, combined potential stop loss, simultaneous positions, per-symbol exposure |
| Circuit breakers | daily realized loss, daily drawdown, consecutive losses, signals/channel/minute, compromised-channel burst/format change |
| Readiness | exchange health, account mode, configured leverage/margin verified against authoritative state, available collateral |
| Recovery | no duplicate open intent, no unresolved ambiguous action, startup reconciliation, no duplicate signer/daemon |
| Manual interaction | same-symbol manual position/order changes, account changes, revoked API wallet |

Cross margin shares collateral and therefore requires global risk budgets; it
never grants unlimited risk. Keep cross versus isolated and leverage configurable.
Signal suggestions cannot alter those settings without a separately allowed policy.
Concurrent same/opposite-direction messages from both channels must be governed
by an explicit per-symbol exposure/conflict policy before allowing execution.

TP alpha/stop beta can be configured for offline experiments but no calibration
or strategy transformation is implemented. Compare fees, delays, fills, stop
gaps and realized outcomes empirically. New-entry deviation defaults to strict
source bounds in bootstrap; widened deviation/velocity policy is unresolved.
