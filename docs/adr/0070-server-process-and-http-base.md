# 0070. Server process: database ownership, HTTP base and request execution model
- Status: Accepted
- Label: [OURS] (hyper and serde_json — the owner's decision [USER], brief §3/Implementation; the rest is ours)
- Date: 2026-10-27
- CUJ: CUJ7, CUJ8, CUJ9, CUJ13, CUJ15, X-1, X-2

## Context
ADR 0011 item 5 and ADR 0015 [USER]: one server process owns the database, the UI and API are its clients. Brief §3/Implementation [USER]: HTTP and JSON — the crates
hyper and serde_json. Area 2 gave the model "one writer, readers on snapshots" (ADR 0055, 0056) and required of area 3: a per-request snapshot
is released on any outcome (ADR 0056 item 4), the number stream is not buffered (ADR 0035 item 6), the kind of the "snapshot revoked" and "server
starting" refusals is set by area 3. We need to define exactly what hyper does, what is left to the server, and how requests map to threads.

## Decision
1. [OURS] The process: the single database owner per ADR 0054 item 1 (`LOCK` is the first action). The server process serves the API (`/v1/…`), the UI
   (static resources on the loopback listener, area 5), and the operator control channel (area 4, Q-60). There are no other processes opening the database
   files. One process per database.
2. [OURS] The HTTP base: hyper 1.x, HTTP/1.1 only (`hyper::server::conn::http1`), on top of tokio; TLS on the network listener — area 3 "access",
   ALPN only `http/1.1` (HTTP/2 is not offered in v1: it would add separate stream and connection limits, and there is no gain for a small number
   of long-lived clients). hyper does not accept connections itself: accepting, connection limits, idle and response-write timeouts, the body cap,
   graceful shutdown — server code (F-101…F-105). hyper's gain is parsing and forming HTTP/1.1, `header_read_timeout`, a cap on the number of headers.
3. [OURS] Listeners: loopback (UI, file token, UI token) and, if the network listener is enabled (ADR 0012 item 3), network HTTPS. Addresses, ports and
   certificate are settings (area 4, Q-45). Listeners are opened before requests are accepted (item 5); a bind error is a start refusal with the address and `errno`,
   there is no fallback port; the server does not set `SO_REUSEPORT`, so that a foreign process cannot share the port (P-7).
4. [OURS] Threads: (a) the I/O runtime (tokio, the number of worker threads equals the number of cores) parses HTTP, checks access and deadlines and does not call
   blocking storage operations; (b) the reader pool — separate threads (the number P = max(4, number of cores)) that execute storage reads on snapshots;
   (c) the writer — one engine thread (ADR 0055 item 1), write requests are put in its queue, the response is returned after publication (ADR 0055 item 4);
   (d) engine background threads (checkpoint, backup) — area 2. Waiting for the writer or a reader does not occupy an I/O runtime worker thread.
5. [OURS] The request processing order is single, ADR 0063 item 7 and spec `http-api` ("Request check order"); here is the consequence for execution: access
   (token, permissions, request limit) is checked only by headers, before the body is read; for writes the database state comes before the operation's checks (ADR 0053 item 8,
   `write-operations`), then input, version, existence; for reads — execution on a snapshot. An error at an early step does not require executing a later one: no token —
   the body is not read and there are no calls to storage.
6. [OURS] Read classes in the reader pool: word-prefix search (up to ≈ 14 s on 10^8, ADR 0049 item 3) occupies at most ⌈P/2⌉ threads at a time; other
   reads (by number, exact name search, number version) use any threads. Excess word searches wait in a queue; waiting counts toward the request
   deadline (ADR 0071). This way heavy searches do not deprive other requests of threads.
7. [OURS] Cancellation and release: every request has a cancellation token. It fires on a client connection drop, request deadline expiry and
   server stop; a read checks it between page reads (at least once per page) and finishes, releasing the snapshot via a scope
   guard (ADR 0056 item 4). A write already accepted by the writer is not subject to cancellation: it is acknowledged or rejected (item 8, ADR 0071 item 4).
8. [OURS] Panic: in a read handler — the request ends with an internal error, the snapshot is released, the process keeps running. A panic in the writer thread
   or in the checkpoint thread — the process terminates at once (the stop is a crash, ADR 0028 item 10; the state after a panic is not trusted, recovery
   at the next start, CUJ14).
9. [OURS] The server log for the operator contains no tokens, `Authorization` header values, request bodies, numbers or names; a request entry is method,
   path without the query string, status, duration, client name (P-5, CUJ13).
10. **Changed by OD-6 (2026-10-28).** Word-prefix search — up to ≈ 1.4 s on 10^7 (was ≈ 14 s on 10^8, OD-6); the share of ⌈P/2⌉ threads did not change.

## Alternatives considered
- Another HTTP library: rejected, hyper is named by the owner (brief §3/Implementation).
- A ready high-level framework on top of hyper (router, limits): rejected as an extra dependency; the needed limits are set
  by hand anyway (hyper does not provide them, F-101), there are few routes. Not forbidden by the owner; the crate question — at implementation (ADR 0017 item 3, cargo deny).
- Execute storage reads directly on I/O runtime threads: rejected, reading cold pages and a search of up to ≈ 14 s would stop accepting and the responses
  "unauthorized" and "too many requests".
- One common pool without a word-search class: rejected, heavy searches would crowd out exact reads (CUJ4: p99 ≤ 10 ms, ADR 0049).
- HTTP/2: rejected for v1 (above); adding it later is compatible with HTTP/1.1 clients.

## Consequences
- The number of threads and the "word search" class are default settings, verified by the implementation's load test (not measured).
- Cancellation channels and snapshot guards are part of the implementation, their leak is a defect visible in the number of open snapshots (ADR 0056 item 4).
- hyper behaviors not confirmed by documentation (F-108) are checked by implementation tests.

## Sources
- `docs/source/brief.md`, §3 "Interfaces", "Implementation"; ADR 0011, 0015, 0054, 0055, 0056, 0049.
- Verified against docs.rs: F-101…F-106.

- **Changed by OD-7 (2026-10-28).** Item 3: the listener is created via `socket2` without `SO_REUSEADDR`/`SO_REUSEPORT`, not `tokio::net::TcpListener::bind` (ADR 0101).
