# Security

No bootstrap credentials are required. `.env.example` contains comments only;
no `.env` was created. Never commit Telegram API hash/session, master wallet
key/seed, API-wallet key, GitHub credentials, tokens, cookies, private exports
or journals. Config has no secret fields. Credentials must not appear in CLI
arguments, remote URLs, synchronous logs or error dumps.

Repository access uses `/home/server/.ssh/hypertele_scalp_ed25519` exclusively
through `github-hypertele-scalp`, `IdentitiesOnly yes`, strict host checking and
a dedicated known-hosts file pinned to GitHub's official public server key.
Existing keys were not overwritten. Private key stays owner-only on beelink;
write deploy-key scope is this repository, not the whole GitHub account.
Git identity is set repository-locally. No token/helper-store authentication.

Keep source/corpus operational privacy: channel exports are paid/private,
may identify members, and must remain outside Git unless explicitly redacted
and authorized. Exports need owner-only access and retention/backups policy.
Application keys do not grant permission to redistribute provider content.

Future runtime should use a dedicated `hypertele` user, no login shell, sudo or
Docker group. This is an intentional security deviation from the homelab's
`server` convention and requires owner approval at deployment. No user was
created. Prefer systemd credentials or owner-only mode 600 files; a dedicated
API wallet with limited funded exposure, never a master key. Do not reuse
deregistered wallets. No exchange credential is requested before an approved phase.

One owner must hold an OS-level process lock for the API-wallet identity.
Rust `&mut` ownership within one process does not prevent a second daemon.
Native protection survives the local daemon; ambiguous exchange delivery stops
actions until reconciled. External manual account changes invalidate readiness.

Unsafe Rust is forbidden. Prices use checked exact arithmetic; input/config/
record/queue/dedupe bounds reject overflow instead of fallback. CI uses no
secrets and does not persist checkout credentials. Review resolved dependencies
and a future cargo-audit/deny gate before network/signing release; bootstrap's
dependency review is not a clean vulnerability-audit claim.

The authoritative homelab manual contains plaintext credential examples in
unrelated runbooks. Do not copy those into this repository or repeat them in
reports. They were not used, rotated or edited during this project work.
