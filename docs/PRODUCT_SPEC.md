# Product specification

HyperTeleScalp will ingest TWO paid Telegram channels through a persistent
MTProto USER connection, classify their messages deterministically and eventually
execute on Hyperliquid perpetual markets available to the dedicated account.
Each channel has roughly 6,000 followers; crowd responses may move price 1–3%
in seconds. The competitive window is approximately 3–5 seconds, not a promised
delivery deadline or profitability claim. Roughly 95% of signals are expected as
text; chatter, screenshots and later edits/updates must be handled separately.

The owner authorizes Phase 0/bootstrap only. No Telegram login, wallet,
credentials, real order, Live backend, service installation or host tuning.
Future testnet execution also requires a separately approved phase. Current CLI
validates configuration and streams artificial replay classifications only.

## Intended behavior

Known channel + exact known template produces a typed NewSignal, Update, Close
or Cancel. Everything else is Ignore/manual review. Never interpret ticker
mentions, hype, general analysis, news, memes, promotion, reposts or PnL images
as entries without a proven channel grammar. Do not guess missing direction,
entry semantics, stop, symbol identity or references.

Fresh metadata and executable BBO/depth are cached before a message arrives.
Signal/queue age, entry, slippage, account risk and readiness gates precede one
execution owner. Entry is a capped aggressive IOC, with skipping preferable to
chasing. Observe fills, attach native reduce-only protection to actual filled
exposure, reconcile authoritative state and halt on ambiguous delivery.

TP/SL are provider inputs, not automatically our policy. Offline experiments
may test `entry + alpha * (provider_tp - entry)` and a stop distance scaled by
beta, with each fraction in (0,1]. Direction is mirrored for shorts. Neither
formula is assumed optimal; provider targets, fees, latency, book depth and
execution outcomes must be modeled. Cross/isolated margin and leverage are
configurable and unset until owner approval. Suggested Telegram leverage never
silently changes account settings.

## Corpus and acceptance

Owner-supplied channel history is the parser source of truth. Preserve channel,
message/reply IDs, dates, edits, captions, media and labels. Build grammar
fixtures and replay against every historical message, including non-signals.
Unknown text is skipped; optional offline/shadow AI may suggest new rules but
cannot promote its own output to a trade. Caption parsing precedes optional
separate image ROI/OCR; heavyweight vision never blocks text or the execution owner.

Engineering target: local receipt-to-socket-write should eventually be
comfortably sub-millisecond to low single milliseconds, evaluated at p99 and
p99.9. Bootstrap metrics omit Telegram, signing, journaling and exchange delivery.
End-to-end measurements precede any infrastructure purchase or location choice.
