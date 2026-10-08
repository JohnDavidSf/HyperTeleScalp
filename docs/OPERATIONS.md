# Operations

Bootstrap is an offline CLI, not a running daemon. No port, runtime user,
systemd unit, credential, container or persistent exchange connection was created.

```bash
cd /opt/homelab/hypertele-scalp
source /home/server/.cargo/env
cargo run --locked -- check-config config/config.example.toml
cargo run --locked -- replay-fixtures tests/fixtures/synthetic.ndjson
```

Rustup was installed as `server` using minimal profile, rustfmt and clippy with
`--no-modify-path`; source the environment per shell. Existing shell/global
Node/Python/system packages were not modified. The exact project toolchain is
pinned to 1.99.0. Development directory is 775 server:server.

## Verified host snapshot, 2026-10-07

beelink: Ubuntu 24.04.3 LTS, Intel N150/4 cores, 15 GiB RAM with about 12 GiB
available at inspection, 42 MiB swap used; kernel 7.0.0-30-generic, uptime
36 days. Root had 68 GiB free, `/media` 551 GiB. Fourteen existing containers
were running; no failed systemd units at this inspection (old docs said one).
Exactly one existing bot.py process was verified by exact /proc argument
matching, with 13 threads. Deleted finance/planrai/invest/dead-mount paths and
qbit-port-sync units are absent. Its one documented HL monitor is unrelated.

Source rules 5300/5301 route WiFi/Ethernet through tables 101/100 respectively;
the dual-NIC timer is active with a scheduled next firing. UFW permits LAN and
Tailscale only. This is local inspection, not an external public-exposure scan.
Docker default log rotation is configured (20m/3), but inspected containers
have empty log options and no CPU/memory limits. A reboot marker remains for
installed kernels/libc. No discrepancies were repaired during bootstrap.

## Future production service

`systemd/hypertele-scalp.service.example` is documentation ONLY, with a future
daemon command/binary path that does not exist in bootstrap. Do not install or
enable it. Prefer a separately approved unprivileged `hypertele` user with no
sudo/Docker/login; otherwise `server` needs strong confinement. Test every
sandbox directive with actual credentials/session/state access and networking.
No unit hardening or memory-lock guarantee is claimed today.

Deploy a verified binary outside `target/`; validate config, metadata, account
mode/leverage/margin and authoritative open orders/positions before enabling
ingestion. Start disabled/read-only. Restart must reconcile journal/actions and
existing exchange protection before admission. Never clear ambiguous state by
restarting, manually deleting the journal or blindly resending a client ID.

Logs eventually use journald; latency uses separate fixed events/HDR. Metrics
bind loopback if added. No new firewall rule or public exposure is needed.
Bulk historical datasets need an explicitly approved separate storage path;
do not write unbounded recordings to root. Off-host backups for session/journal
and retention/encryption policy remain unresolved; Git backs up source only.

Host maintenance, pending reboot, Docker resource/log policy and SMART/backups
belong to a separately authorized task. Do not change them while developing
the project. Compare end-to-end home/VPS Telegram->HL p99 before buying infrastructure.
