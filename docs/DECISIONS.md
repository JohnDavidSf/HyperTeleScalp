# Decisions — 2026-10-07

| ID | Decision | Why / revisiting condition |
|---|---|---|
| D001 | One Rust crate, edition 2024; pinned stable 1.99.0 | Clear boundaries without speculative workspace |
| D002 | Offline bootstrap only, no Live/Testnet enum variant | Make real submission impossible, rather than relying on a default flag |
| D003 | Deterministic channel grammar; A/B stubs ignore all | History must prove formats; unknown text cannot become a trade |
| D004 | Clearly marked synthetic grammar for tests/replay only | Exercise plumbing without inventing canonical paid-channel signals |
| D005 | Checked u128 fixed decimal, 12 fractional places; positive price/quantity | Exact comparisons and bounded precision; exchange formatter still required |
| D006 | Preserve Range/CMP/MultipleRanges | Do not flatten entry semantics; missing directional CMP cap and malformed/overlapping ambiguous zones reject |
| D007 | Bounded single-owner queue and source-level simulation ledger | No unbounded backlog or parser-version duplicate; durable live design remains future |
| D008 | One future actor/key; capped IOC; native protection for actual fills | No signer races, unbounded market entry or assumed full fill |
| D009 | Halt/reconcile ambiguous delivery, never blind retry | Socket failure cannot prove exchange rejection |
| D010 | Durable-before-post is planned baseline; async durability unapproved | Millisecond sync measurements matter, but equivalence is unproven |
| D011 | Caption first, separate bounded OCR/AI worker later | Images cannot slow text or promote unknown text to trades |
| D012 | Grammers 0.10.0/Codeberg is a candidate, no dependency yet | Upstream migration and update/flood-wait/session behavior need validation |
| D013 | No official HL Rust SDK dependency for hot path | Inspected SDK uses HTTP actions, native TLS and float helpers; evaluate minimal rustls transport later |
| D014 | Immutable coherent snapshot; ArcSwap vs safe locks undecided | Freshness and consistency before unsafe optimization |
| D015 | Monotonic stages, bounded nonblocking events, HDR distributions | Tail latency and drops observable without formatting in submit |
| D016 | Repository-scoped SSH key and local Git identity | Least scope; preserve existing keys/global config |
| D017 | Margin/leverage and alpha/beta remain configurable/unset | Owner has not authorized live values; backtest first |
| D018 | No host tuning, apt/global package changes or service deployment | Baseline before tuning; isolate project from homelab |

Unresolved: corpus access/retention/schema; channel grammars and alias mapping;
same-symbol/opposing signals; source age under clock changes; catalog snapshots;
precision and CLOID scheme; crash journal format/retention; IOC partial native
protection; full account risk; strategy sizing/TP/SL; telemetry task integration;
runtime user/backups; dependency/signing certification. Record future decisions
with evidence and owner authorization when a phase boundary changes.
