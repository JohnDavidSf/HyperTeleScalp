# Hot path

Current hot-path microbenchmarks use synthetic in-memory inputs only. There is
no live ingestion, signing or socket-write path. The intended submit path is:
receive -> filter -> exact parser -> identity/admission -> snapshot/risk ->
journal/sign -> write on an existing action socket.

Before admission, connection/authentication, TLS, subscriptions, metadata,
config parsing and dependency initialization must already be complete. Reject
if any cache, queue, exchange readiness or account state is unknown/stale.
At actor dequeue, re-evaluate source age and current executable price; price
movement during parsing or queuing cannot reuse a previously favorable quote.

Forbidden normal submit work: DNS, TCP/TLS/WebSocket handshake, Telegram login,
HTTP/REST metadata/market fetch, SQL/database lookup, filesystem/config read,
model inference, OCR/image work, sleep/backoff, slow fallback, unbounded
allocation and synchronous formatted logging. Execution durability is the
single deliberate synchronous exception under investigation.

Exact decimals use checked unsigned integers. Parser length is bounded at 4096
bytes, synthetic targets at 16, multiple entry zones at 8. The synthetic parser
does allocate a symbol, target vector and boxed signal. No zero-allocation or
lock-free claim is made. Direct scanning/token tables/memchr/small collections
may be considered only after correctness and measured before/after p99 evidence.

Local T5-T1 is the primary future metric. Record drop counts and distributions;
never fabricate T4/T5/T6 from the simulation backend. Queue saturation, stale
state and unknown formats are counted rejections. No host governor/affinity/
IRQ/LTO/native-CPU tuning was applied to this baseline.
