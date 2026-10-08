# Reference engineering audit — 2026-10-07

Inspected source (read-only reference clones under `/tmp`):

- [bahmgit/hlcli](https://github.com/bahmgit/hlcli), commit
  `e62a745ecc8b7569d5fd7963c1ec1e82edcf5c0e` (2026-10-05).
- [official Rust SDK](https://github.com/hyperliquid-dex/hyperliquid-rust-sdk),
  commit `aac75585daf12d0a3761126cc7da7a5e035b5853` (2025-10-20), package 0.6.0.
- Current official contract links in `HYPERLIQUID_PROTOCOL.md`.

No reference implementation was built/run with credentials. No code was copied
or adapted into HyperTeleScalp. Future copying from hlcli requires Apache-2.0
license/NOTICE/attribution compliance; SDK source is MIT. Architecture learning
does not confer exchange certification or a supply-chain audit.

## hlcli findings

| Area / inspected files | Decision | Reason |
|---|---|---|
| Daemon ownership (`docs/architecture.md`, `core.rs`, `server.rs`) | USE concept | One resident owner separates clients/state/execution; our source is Telegram and one bounded actor |
| Nonces (`core.rs::next_nonce`) | ADAPT | Atomic monotonic allocation against wall ms under serialized execution; add process exclusivity and tested restart/clock behavior |
| Action WS (`exchange.rs`) | ADAPT | Persistent post IDs/pending replies, timeout classification, ping/pong, disconnect retirement; reference worker channel is unbounded, ours MUST be bounded |
| Wire/signing (`protocol.rs`) | VERIFY AGAINST CURRENT HL | Typed action/hash/signature structures and field omission matter; require official SDK cross-vectors and testnet before selecting signer stack |
| Journal (`execution.rs::Journal::append`) | USE safety; ADAPT format | JSON context plus pending/terminal phases, append then `sync_data`; record enough source/account/quantity/protection context for our recovery |
| Submit order (`core.rs`, `execution.rs`) | VERIFY detail | Reference signs before entering journaled submit; pending record is durable before post. Do not misdescribe this as journal-before-sign |
| Ambiguous handling (`TransportError`, execution kernel) | USE invariant | Possible delivery/timeout/decode uncertainty halts instead of blind retry; terminal journal failures also require conservative handling |
| Reconnect/reconcile (`core.rs`, `info.rs`) | ADAPT | Context-bearing unresolved records query authoritative state; old context-free rows remain audit-only |
| Freshness (`state.rs`, `feed.rs`) | ADAPT | Explicit known/fresh state, account/book/capacity validation, snapshot times and application-pong liveness; distinguish feed liveness from quote changes |
| Protection (`managed.rs`, architecture) | USE invariant; ADAPT lifecycle | Only owned entry fills arm/resize exits; partial and degraded protection persist identity; no assumption requested == filled |
| CLOIDs (`protocol.rs`, `core.rs`) | ADAPT | Explicit 16-byte identities support correlation; our stable source/template/intent mapping and crash-proof recovery are not yet specified |
| Catalog/metadata (`info.rs`, `planner.rs`, `protocol.rs`) | ADAPT | Cache market/size/price rules; direct validated perps only initially, no blind aliases |
| Subscriptions (`feed.rs`) | ADAPT | Background user/book/asset state and liveness; narrow configured native perp universe |
| CLI/IPC/HTTP/grid/encrypted profiles | DO NOT NEED now | No UI server, shell sessions, multi-terminal command language or encrypted key manager in bootstrap |
| Spot, TWAP, chase, native trails, builder extras | DO NOT NEED now | Initial goal is perp signal entry + basic mandatory protection; separately verify new protocol features |
| Tests (`tests/*_contract.rs`, docs/validation.md) | USE strategy | Mock exchange boundaries, exact submitted actions, journals, ambiguous outcomes, restart identity, protection and readiness; independent testnet certification |

Reference dependencies include Alloy signing/ABI crates, MessagePack,
rust_decimal, Tokio, rustls-backed reqwest/tungstenite, plus CLI/IPC/security
libraries. They do not all belong in our crate. Its own validation document
reports unresolved dependency advisories; those are upstream-reported findings,
not an audit performed here. Do not inherit the reference lockfile.

## Official Rust SDK findings

`exchange/exchange_client.rs` posts signed actions through its HTTP client;
`ws/ws_manager.rs` manages subscriptions. Inspection found no public action-WS
post execution equivalent to hlcli's transport. Existing BBO examples and
order/CLOID/trigger types are useful cross-references, not our selected hot client.
Its market-order helpers use f64 and may perform info requests; those patterns
are inappropriate for our exact cached-price hot path. Manifest defaults use
reqwest's TLS defaults and explicitly native-tls tungstenite, plus broad Tokio
features. Avoid importing this dependency wholesale without feature/transport review.

Recommendation: a small direct rustls action transport may be preferable,
keeping wire/signing behavior independently certified against official
documentation and SDK vectors. Do not hand-roll cryptography. No SDK, signer,
Telegram, TLS or WebSocket dependency is installed in our crate today.
