# Telegram ingestion research — checked 2026-10-07

The two signal channels require an MTProto USER session, not Telegram Bot API
polling. The old homelab bot remains completely separate. No account was
connected and no credentials/session requested or created during bootstrap.

The original [Lonami/grammers GitHub repository](https://github.com/Lonami/grammers)
was archived on 2026-02-10 and points to the maintained
[Codeberg repository](https://codeberg.org/Lonami/grammers). Crates.io identifies
Codeberg as the repository for grammers-client 0.10.0 (updated 2026-07-02).
The current upstream client manifest also reports 0.10.0/edition 2024. This is
a repository migration, not evidence that the project stopped development.

[Current client documentation](https://docs.rs/grammers-client/0.10.0/grammers_client/)
uses Client/SenderPool and persistent Session storage. It warns that the
default configuration sleeps for short flood waits; that behavior must be
reviewed before placing RPC work near the ingestion/execution path. Friendly
wrappers expose raw generated Telegram layer types whose minor-version
compatibility must be reviewed. No grammers dependency is selected yet.

Phase 1 selection should compare the maintained grammers release with TDLib
through Rust bindings, considering native build footprint, layer/update
coverage, reconnect/edit/delete/reply provenance, update-queue bounds and
session security. TDLib's official [source](https://github.com/tdlib/td) is the
alternative reference, not an installed dependency. No claim of completed
TDLib compatibility testing is made. Prefer the smaller native Rust candidate
if measured update behavior and authentication/session review justify it.

## Adapter contract

Persist a dedicated user session in owner-only storage. Filter exact channel
IDs first. Keep source/message/reply IDs, edited/forwarded origin, source date,
local Instant receipt and conservative source-age estimate. Derive allowed
symbols from channel history, not generic substring ticker guesses.

Persistent updates are the normal path; history is for research/state repair.
On reconnect, distinguish live updates from history/replay and enforce maximum
live age. Backlog messages never become catch-up orders. Clock rollback, unknown
source age or uncertain provenance must reject execution. Login, peer lookup,
downloads and flood-wait sleeps stay outside the submit path.

Parse caption/text first. Complete valid captions proceed without image work.
Otherwise enqueue media to a separate bounded worker (future), with ROI/template
OCR preferred for repeated layouts. Text/caption/image conflicts and decimal/
ticker OCR ambiguity require manual review. AI-image trading remains disabled.
