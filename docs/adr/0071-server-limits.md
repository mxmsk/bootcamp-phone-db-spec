# 0071. Server limits: revising the brief §4 values for network clients and long requests
- Status: Accepted (OD-6)
- Label: [DEFAULT] (values; brief §4 asks to propose them anew and ask the owner; closes Q-52 and Q-102 with a default value: D-43, D-52)
- Date: 2026-10-27
- CUJ: CUJ4, CUJ5, CUJ7, CUJ8, CUJ9, X-1; items 1.3, 1.5 of the original wording

## Context
Brief §4: 256 connections, a 32 KiB body, a request timeout of 10 s, a connection idle of 60 s — accepted for "loopback only". Now there are network clients
(CUJ8, CUJ9), and areas 1 and 2 have accumulated conditions: (a) `records` requires not rejecting a valid name because of input size before normalization (20,000
whitespace characters on each edge, Q-102); (b) the number history stream is up to 2,967,164 records (≈ 300 MB) without truncation (ADR 0035 item 6), it has no snapshot
lifetime ceiling, only a client idle of 120 s (ADR 0056 item 2, D-30); (c) word search up to ≈ 14 s on 10^8 without a deadline in storage (D-24), a short read
has a snapshot ceiling of 30 s; (d) a write cannot be cancelled once accepted by the writer (ADR 0055), its timeout leaves the outcome unknown (P-12).
Hyper provides only `header_read_timeout` (30 s by default), the other deadlines are server code (F-101).

## Decision
A table of default values (all [DEFAULT]):

| Parameter | Value | Was (brief §4) | Why |
|---|---|---|---|
| Connections per listener | 256 for loopback and 256 for network (pools are separate) | 256 total | a network influx (P-7f) must not take slots from the UI and local programs |
| Connections from one network address | 32 | none | one service does not occupy the pool; there are few network services, 32 is a margin per client pool |
| Anonymous connections (class U) from one network address | 16 | none | beyond that the oldest anonymous one of the same address is evicted, others' are not affected (ADR 0072 item 2) |
| Protection period of a new anonymous connection from eviction | 1 s | none | at an anonymous stream below 256/s a legitimate connection has time to send a token (ADR 0072 item 2) |
| Request head (request line and headers) | 16 KiB; exceeding — 431 `headers_too_large`, one measure for target and headers | none | the token, paths and service headers fit in 2 KiB; the worst valid search request is ≈ 12,900 bytes (ADR 0064 item 1) |
| Request body | 32 KiB (OD-6, D-43; was 256 KiB); exceeding — 413 `payload_too_large` | 32 KiB | the owner's decision: as in brief §4; a valid name without edges is ≈ 11,600 bytes of body |
| The whole first request of a connection (including the TLS handshake) | 10 s | none (hyper: 30 s) | a connection without a request is class U (ADR 0072) |
| Idle between requests (keep-alive) | 60 s | 60 s | unchanged |
| Request body — from the end of headers to the end of the body | 10 s | within the 10 s timeout | at 32 KiB — not slower than ≈ 3.3 KiB/s |
| Read processing until the response begins | 20 s | 10 s | covers ≈ 14 s of word search (ADR 0049 item 3) and waiting in the pool queue with margin, but is shorter than the 30 s snapshot ceiling: the client gets "request timeout", not "snapshot revoked" |
| Write processing until the response | 10 s | 10 s | on expiry without an outcome — "write outcome unknown" (item 4) |
| Response write: no data accepted by the socket | 10 s ordinary response; 120 s number stream | none | 120 s equals the stream client's idle (ADR 0056 item 2); P-6, P-10 |
| Grace period for reads on stop | 5 s | none | ADR 0074; together with ≲ 30 s of storage it fits in 60 s (ADR 0054 item 8) |
| Delivering a refusal before closing | no more than 1 s and 64 KiB of draining | none | RFC 9112 §9.6 (F-107), P-9 |
| Simultaneous refusals by the connection limit | 16 | none | a refusal must not become a way to occupy resources |

1. The limits are operator settings (area 4); the defaults are the table. A body cap below 32 KiB cannot be configured (OD-6).
2. The processing deadline is counted from the end of request parsing (headers and body received) to the first byte of the response. For a response that is not a stream,
   the "first byte" comes after the page is fully prepared (it is bounded by the page size of the results).
3. The number history stream is exempt from the processing deadline after the first byte (ADR 0056 item 2: it lives as long as the client accepts data). The deadline
   to the first byte (20 s) applies to it: the snapshot is opened at once, the first portion is ready quickly.
4. Write timeout: while a write operation has not been accepted by the writer, it is withdrawn without consequences and the client gets "request timeout" (not executed);
   accepted and the outcome unknown by 10 s (the writer is waiting for a flush) — the client gets "write outcome unknown", the operation stays in the writer's queue and
   is acknowledged or rejected; the client re-reads the number history and, as after a connection drop, acts by version (ADR 0026 items 5–6, Q-54).
   The server does not report "not executed" for an accepted operation.
5. A refusal for exceeding any limit is a distinguishable kind of refusal, code and status — ADR 0062 item 8 (`payload_too_large`, `headers_too_large`,
   `too_many_connections`, `request_timeout`, `write_outcome_unknown`); "too many requests" per token (CUJ8) is a separate refusal
   `rate_limited` (ADR 0068), not a connection limit.
6. **Changed by OD-6 (2026-10-28).** D-43 (OD-6): the body cap is 32 KiB instead of 256 KiB; the lower bound of the setting is 32 KiB; the worst-case memory for bodies is 32 KiB × 512 connections = 16 MiB (was 128 MiB); D-52 accepted by the owner. The body time of 10 s at 32 KiB — not slower than ≈ 3.3 KiB/s. The read time of 20 s and the 14 s class are recomputed: word search ≈ 1.4 s on 10^7 (T: the 20 s deadline is excessive, return the brief's 10 s — decide on the prototype). Q-102 is closed differently: `records` stipulates the body cap, edges above the cap — 413.

## Alternatives considered
- Keep 32 KiB: rejected, `records` requires accepting a valid name with large edges; at 32 KiB the scenario "20,000 whitespace characters on each edge"
  would be refused. Alternative: a ceiling on edge whitespace, but "however many" in `records` is an area 1 decision (not to be changed).
- A body of 1 MiB and more: rejected, 256 KiB × 512 connections = 128 MiB in the worst case, beyond that it grows without benefit.
- A request timeout of 10 s for all reads: rejected, word search up to ≈ 14 s would get a deterministic refusal, diverging from "a valid request
  always ends with a result" (ADR 0048 item 6); 30 s: coincides with the snapshot ceiling, two errors for one condition; without a deadline: a slot is occupied arbitrarily long.
- A write timeout as "not executed": rejected, a lie if the writer accepted the operation (P-12).
- One common pool of 256: rejected (P-7f).
- The same 10 s timeout for the stream: rejected, the history of the maximum number would be cut (Q-52, area 2).
- A minimum stream speed instead of an idle of 120 s: not chosen, it would change the accepted semantics of area 2 (a "slow but reading" client); the risk of a
  "dripping" client remains — question Q-135.

## Consequences
- Goes to the owner as a package at the end of area 3 (brief §4): body and head sizes — D-43, the other values — D-52. If the values change, the numbers in
  `server-runtime` change.
- Q-52 is closed with default values; Q-102 is closed: X = 256 KiB.
- The response time of 20 s and the 14 s class are estimates; the actual search time on 10^8 was not measured (ADR 0049 item 3).

## Sources
- `docs/source/brief.md`, §4; ADR 0019 item 4, 0035 item 6, 0048, 0049, 0055, 0056, 0059; `openspec/specs/records` ("Input size before normalization and request body cap");
  F-101, F-107; pitfalls P-6, P-9, P-10, P-12.

- **Changed by OD-7 (2026-10-28).** the code `too_many_connections` is not used: overflow is a close without a response (ADR 0098).
