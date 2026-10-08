# Signal format and identity

No provider grammar has been supplied. Channel A/B use `ChannelParser` with
their respective IDs and currently return Ignore for every message.

`MessageClassification` represents Ignore, NewSignal, Update, Close and Cancel.
A Signal retains channel/message identity, template version, reply/reference,
receive Instant, source date/age, origin, edit/forward flags, symbol, direction,
EntryPolicy, multiple targets, optional stop, suggestion and parse evidence.
Missing fields can be represented in the model for research; simulation risk
rejects missing/invalid protection. Production templates must establish exact
evidence; no probability or LLM guess is sufficient.

EntryPolicy preserves a bounded range, explicit current-price reference with
directional bound, or multiple zones. The fixture parser currently emits a
single range. Unbounded CMP is rejected by simulation risk. Real examples such
as `ENTRY 7.06 - 7.55` or `CMP till 7.55` require actual channel templates and
direction-specific interpretation; they are not accepted by the stub.

Source identity `(channel_id, message_id)` is the entry dedupe root. SignalId
adds template_version for audit/reproducibility. The simulation ledger blocks
a second entry even after a parser-version change. Edits/updates need a
separate event/revision identity linked to that root; they are not new entries.
Cross-channel reposts, forwarded copies and simultaneous opposing signals need
explicit policy beyond a source ID; no fuzzy content dedupe is implemented.
The ledger is bounded, has no silent eviction and is not durable across restart.

## Artificial fixture grammar

```text
HTS_FIXTURE_V1|BTC|LONG|100|101|110,120|90
```

Fields: marker, symbolic token, LONG/SHORT, entry lower/upper, comma-delimited
targets, stop. Values use ASCII unsigned decimals; comma is a target separator,
never a decimal separator. Missing/extra fields, malformed decimals and unknown
direction are ignored. This grammar is ONLY for explicitly offline replay/tests.
It is not channel A or B. Hype and ordinary ticker mentions are ignored.

Decimal has twelve fractional places in a checked u128 representation. Price
and Quantity must be positive. No binary float, scientific notation, minus,
comma decimal or Unicode digit fallback. `exact_ticks` requires a supplied
validated quantum; exchange significant-figure rules are not implemented, and
the fixed-point scale must not be mistaken for a universal Hyperliquid tick.

Normalized NDJSON retains IDs, source timestamp, reference, text/caption,
media, edit/forward flags and optional label. The reader streams bounded 64 KiB
records, skips blank lines and reports physical error line numbers. Replay
classifies only; source age remains unknown instead of becoming zero at replay.
Raw Telegram JSON/HTML normalization, edits graph, clustering, labeling UI,
statistics and strategy replay remain future work. Store exports outside Git.
