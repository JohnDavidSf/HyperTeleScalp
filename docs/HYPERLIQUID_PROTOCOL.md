# Hyperliquid external contract — checked 2026-10-07

Documentation inspection only: no signing, account query, WebSocket order or
testnet validation was performed. These assumptions require rechecking before
implementation and actual bounded testnet acceptance afterward.

## Transport and order contract

Mainnet and testnet expose `/ws` at their respective API hosts; automated clients
must handle unsolicited disconnects. Reconnect subscriptions/snapshots repair
missed state, not permission to resend pending actions.
[WebSocket documentation](https://hyperliquid.gitbook.io/hyperliquid-docs/for-developers/api/websocket).

Action requests use `method: post`, a unique correlation `id`, and a request
with `type: action` plus the signed payload. Replies arrive on `channel: post`
and must match the ID, response type and action shape. Transport correlation
IDs are distinct from CLOIDs and signer nonces.
[Post requests](https://hyperliquid.gitbook.io/hyperliquid-docs/for-developers/api/websocket/post-requests).

The HTTP action endpoint is `/exchange`. Order fields encode asset, buy side,
price/size strings, reduceOnly, limit TIF or trigger and optional CLOID.
`Ioc` cancels unfilled quantity rather than resting. CLOID is 128-bit hexadecimal.
Grouping values include `na`, `normalTpsl`, `positionTpsl`; trigger types use
`isMarket`, `triggerPx`, and `tpsl` (`tp`/`sl`). Leverage actions include
`isCross`; optional `expiresAfter` has signing/rate-limit implications.
[Exchange endpoint](https://hyperliquid.gitbook.io/hyperliquid-docs/for-developers/api/exchange-endpoint).

Our future entry is an aggressive IOC whose price remains capped by source
entry/slippage policy. Unknown/unexpected replies and delivery after possible
socket write are ambiguous. HTTP 200 or a socket write is not proof of a fill.

## Signing, wallet ownership and nonces

API wallets sign for the account; account queries use the real account address,
not the agent address. Nonces are tracked per signer even across subaccounts;
the highest 100 are retained and a new nonce must be unique and larger than
the smallest retained. Documented block-time window is T−2 days to T+1 day.
Agent pruning can remove nonce state; do not reuse deregistered wallet keys.
[Nonces/API wallets](https://hyperliquid.gitbook.io/hyperliquid-docs/for-developers/api/nonces-and-api-wallets).

Official signing guidance warns about distinct L1/user-signed schemes,
MessagePack field ordering, numeric trailing zeros and address case. A correct
local recovery test alone does not prove exchange acceptance. Build byte-level
vectors against the official Python SDK plus Rust/reference implementations
before adding a signer. Keep exactly one actor/nonce owner per key.
[Signing](https://hyperliquid.gitbook.io/hyperliquid-docs/for-developers/api/signing).

## Precision and metadata

Perp prices allow at most five significant figures and at most
`6 - szDecimals` fractional places; integer prices are exempt from the
significant-figure restriction. Quantity respects metadata `szDecimals`.
Remove trailing zeros before signing. A magnitude-dependent valid price grid
cannot be reduced to an arbitrary fixed global tick. Round long caps down and
short floors up to a certified legal price, reject if that cannot execute safely.
Bootstrap exact decimals/tick conversion do not yet certify this contract.
[Tick/lot rules](https://hyperliquid.gitbook.io/hyperliquid-docs/for-developers/api/tick-and-lot-size).

Native perp action IDs derive from the selected meta universe. Builder perps
use `100000 + perp_dex_index * 10000 + index_in_meta` and qualified `{dex}:{coin}`
names. Mainnet/testnet catalogs differ. HIP-3 is excluded from bootstrap symbol
acceptance until explicit mapping and account support are proven.
[Asset IDs](https://hyperliquid.gitbook.io/hyperliquid-docs/for-developers/api/asset-ids).

## Market/account state and recovery

Relevant subscriptions: `bbo`, `l2Book`, `orderUpdates`, `userFills` and account
state. BBO has nullable bid/ask levels with decimal price/size strings and
exchange time; it updates only when BBO changes on a block. Snapshot fills are
marked `isSnapshot`; snapshots and live fills require dedupe. A quiet BBO needs
separate connection-liveness reasoning, not automatic timestamp refresh.
[Subscriptions](https://hyperliquid.gitbook.io/hyperliquid-docs/for-developers/api/websocket/subscriptions).

For quiet streams, application `method: ping` produces `channel: pong`; the
documented server timeout is 60 seconds without a sent message. Track pong
deadlines and invalidate affected generations on disconnect. Receipt of a pong
alone cannot certify account position correctness.
[Timeouts/heartbeats](https://hyperliquid.gitbook.io/hyperliquid-docs/for-developers/api/websocket/timeouts-and-heartbeats).

`orderStatus` accepts OID or CLOID with the account user. Open orders, fills,
positions and journal context are complementary reconciliation evidence.
`unknownOid` alone cannot prove non-delivery, especially around reconnect or
visibility delay. Preserve requested/filled/remaining quantity, average price,
fees, OID/CLOID and deduped fill identity. Correlate attributable fills, never
infer a fill solely from an order disappearing.
[Info endpoint](https://hyperliquid.gitbook.io/hyperliquid-docs/for-developers/api/info-endpoint).

## Protection hazard

TP/SL triggers use mark price. The official parent-order protection description
warns that children may remain unplaced when the parent is not fully filled;
canceling a partially filled parent cancels its children. Therefore native
grouping is NOT assumed to protect an IOC partial fill. This particular IOC
combination must be tested; separate reduce-only protection for attributed filled
size is the conservative design. Trigger-market slippage and trigger-limit
nonfill are separate risks. Entry fill + uncertain protection requires
DEGRADED PROTECTION and safe reconciliation/retry or explicitly configured flattening.
[TP/SL behavior](https://hyperliquid.gitbook.io/hyperliquid-docs/trading/take-profit-and-stop-loss-orders-tp-sl).

## Limits and certification

Documented per-IP caps: 1200 REST weight/minute; 10 WebSocket connections;
30 new connections/minute; 1000 subscriptions; 10 unique subscribed users;
2000 outgoing WS messages/minute; 100 simultaneous inflight posts. Account
action limits also depend on cumulative traded volume; they are independent
of IP limits. Reuse sockets, bound subscription churn and reserve protection/
reconciliation capacity. Share public-IP budget awareness with the existing
homelab read-only consumer without reusing its credentials/state.
[Rate/user limits](https://hyperliquid.gitbook.io/hyperliquid-docs/for-developers/api/rate-limits-and-user-limits).

Still unproven: CLOID reuse/retention/idempotency boundaries; IOC grouped-child
activation after partial cancellation; order/fill observation ordering;
precision edge cases; account-mode readiness; exchange-side protection gaps;
crash-safe authoritative absence; signer restart nonces. All require fixtures,
mock fault injection and approved testnet evidence before any Live proposal.
