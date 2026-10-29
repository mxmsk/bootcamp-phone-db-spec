# 0072. Protecting connection slots from slow and anonymous clients
- Status: Accepted ([OURS]: all items except the values in item 2); Accepted (OD-6) ([DEFAULT]: item 2 — protection period 1 s and 16 anonymous per address, D-53)
- Label: [OURS]; [DEFAULT] (item 2, values)
- Date: 2026-10-27
- CUJ: CUJ7, CUJ8, CUJ13, X-2 (pitfalls P-6, P-7f, P-7g, P-9, P-10, P-12)

## Context
P-6: any local user without a token can occupy all slots with idling, slow headers or slow reading of a response; without a response write deadline
a slot is occupied indefinitely. P-7f: with network clients this is possible from the network. P-7g: the per-token request limit and the common connection limit protect against
different things. P-9: closing with unread input gives an RST, the client may lose the refusal (RFC 9112 §9.6, F-107). P-10: `SO_SNDTIMEO` behaves
differently on Linux and macOS. Hyper provides `header_read_timeout`, but no idle, body or write deadlines (F-101). All local OS users
come from one address 127.0.0.1, they cannot be told apart by address.

## Decision
1. [OURS] Three connection classes: U — not yet a single request with a valid token (a new connection until the first such request; an "unauthorized" refusal closes the connection); I —
   there was a valid token, now idle between requests; A — a request of a valid token is being processed. The class changes to A on a successful token
   check and returns to I after the response.
2. [OURS] Accepting when the listener's pool is full: (a) the oldest connection of class U **older than the protection period** is evicted by closing — 1 s [DEFAULT];
   (b) if there are none — the longest-idle I; (c) if there is no I either (the pool holds only A and U younger than the protection period) — the new connection gets
   a `too_many_connections` refusal with `Connection: close` and is closed per item 5, at most 16 such refusals at a time (beyond that — closing
   without a response). Connections of class A and U younger than the protection period are not evicted. [DEFAULT] On the network listener from one address — at most 16
   connections of class U: a new U beyond that evicts the oldest U of the same address (not someone else's), the protection period does not apply here, so
   a source from one address does not evict others'. The limit of the guarantee: under a stream of new anonymous connections r, the age of the oldest U in a full pool
   of 256 is ≈ 256 / r s, it exceeds 1 s while r < 256 per second; then a new legitimate connection finds a victim, and is itself evicted no sooner than
   after the protection period and has time to send a request with a valid token (then it is A, then I; I is evicted only when there is no U). At r ≥ 256/s (both on
   loopback without distinguishing sources, and on the network from many addresses) anonymous connections younger than the protection period fill the pool, and new connections,
   including legitimate ones, get refusal (c): this is expected behavior, not a "silent" refusal; the client retries. There is no unconditional guarantee: it is
   narrowed to r < 256/s. Alternatives:
   reserving slots (rejected below); a protection period of 5 s — with the same 256 slots the threshold is 51/s; without a protection period and without a per-address limit —
   anonymous connections evict fresh legitimate ones. Residual: the CPU load of accepting and evicting (item 6).
3. [OURS] Deadlines before the token: the first request — 10 s for everything, including the TLS handshake (ADR 0071); an "unauthorized" refusal closes the connection after
   the response (item 5), a class U slot is not held. Deadlines after: idle between requests 60 s, body 10 s, response write without progress 10 s (stream — 120 s). Write progress is acceptance by the
   socket (a write to the kernel buffer completed) of at least part of the data; the deadline is counted by server code, not by `SO_SNDTIMEO` (P-10: an exact "N seconds without
   progress" cannot be had with it). The write deadline expired: the connection is closed without finishing the write (the response can no longer be delivered), the reader's snapshot is released.
4. [OURS] Stream back-pressure: the next portion of records is read from the engine only after the previous one is accepted by the socket; the engine does not accumulate
   results beyond a portion (ADR 0035 item 6, 0056 item 6). A write idle of 120 s is exactly "the client does not accept data for 120 s" from ADR 0056 item 2: the server closes
   the connection, the snapshot is released, the stream is cut without the final marker.
5. [OURS] Stepwise closing (RFC 9112 §9.6, P-9): after a refusal, a response without reading the whole body, and before closing with unread input the server
   first closes the write side (half-close), then drains the input for no more than 1 s and 64 KiB and closes the connection. The refusals 400, 401, 403, 413, 429, 431,
   503 come before the body is read (by headers) and reach the client. A connection after a refusal with an undrained body is not reused.
6. [OURS] One network address holds at most 32 connections of the network listener (ADR 0071); beyond that — a refusal as in item 2 (c). On loopback there is no limit
   by address: OS users cannot be told apart by it. Residual risk: a local user without a token can create connections at a rate at
   which the server spends processor time on accepting and evicting; access to data and the slots of authorized clients are not affected by this. Protection
   against a connection influx at the OS level is outside the specification.
7. [OURS] A refusal by the per-token request limit (`access`, CUJ8) is issued by headers, before the body is read and without calling storage or the reader pool: it
   costs no more than an "unauthorized" refusal, and another's limit does not affect other tokens (P-7g). The loopback and network listener pools are separate: an influx from the network
   does not take slots from the UI and local programs.
8. [OURS] Framing (P-8): a request with both `Transfer-Encoding` and `Content-Length` is not executed: the response is "invalid request" (`invalid_request`), the connection is closed
   after the response (RFC 9112 §6.3: "ought to be handled as an error"). A request not parseable as HTTP/1.1 — the same. How this is achieved in hyper — checked at
   implementation (F-108); without that, the item is checked by a test on a raw socket.

## Alternatives considered
- Only idle and header deadlines, without classes: rejected, `P-6` is not closed — slots are occupied by fast frequent anonymous connections.
- Waiting for a place in the accept queue instead of a refusal: rejected, the client does not get a definite response, the influx grows in the kernel queue.
- Reserving part of the slots for valid tokens: rejected, evicting class U solves the same task without a permanent slot expense.
- `SO_SNDTIMEO` and `TCP_USER_TIMEOUT`: rejected, differing OS behavior (P-10), and the difference between "idle" and "slow but reading" is lost.
- Distinguishing local users by socket uid: rejected for v1, not portable between Linux and macOS (the source was not checked), while the token remains
  the main protection.
- Revoking tokens on suspected brute force: belongs to `access`.

## Consequences
- Requirements for `access`: the token check and limit are by headers; the "valid token" marker is reported to the connection layer (classes U/I/A).
- CUJ8 "other clients do not notice" refers to the token's request limit (ADR 0068); for a stream of anonymous connections the guarantee is conditional (item 2, threshold 256/s
  per pool) — this is a narrowing visible to the owner, in the package (D-53). Protection against a stream above the threshold is at the OS and network level, outside the specification (item 6).
- Verified by implementation tests: eviction (including the 1 s protection period and the cap of 16 anonymous per address), a stream of anonymous connections below and above the threshold, separate pools, delivery of 401/413/429 with an unread body, slow headers, slow reading.
- The numeric values — ADR 0071 (Proposed).

## Sources
- Pitfalls P-6, P-7f, P-7g, P-8, P-9, P-10; RFC 9112 §6.3, §9.6 (F-107); hyper docs (F-101).
- ADR 0035 item 6, 0056 items 2, 6, 0071.

- **Changed by OD-7 (2026-10-28).** Items 2 (c), 6, 8, 5: when there is no eviction victim and at the address cap the connection is closed without a response, `too_many_connections` and "at most 16 refusals" are removed; item 8 — a request with both `Transfer-Encoding` and `Content-Length` is handled per RFC 9112 §6.1 (chunked body, closing after the response), 400 only for an unparseable request; item 5 — the list of refusals without "too many connections"; item 4 — read "accepted by the socket" as "accepted by the connection buffer" (ADR 0098, 0100).
