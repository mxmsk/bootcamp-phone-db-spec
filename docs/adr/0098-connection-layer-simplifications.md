# 0098. Connection layer: TE+CL per RFC 9112, closing without a response on overflow and shutdown
- Status: Accepted (OD-7)
- Label: [OURS] (owner decisions OD-7 items 1, 3, 5 — [USER]; wording — [OURS])
- Date: 2026-10-28
- CUJ: CUJ7, CUJ8, CUJ13, CUJ14

## Context
The Rust expert (`docs/reviews/rust-expert.md`, items 1, 6, 15): the requirement of 400 for a request with `Transfer-Encoding` and `Content-Length` is infeasible on hyper without a second HTTP parser (F-120); 503 `too_many_connections` over HTTPS is possible only after the TLS handshake (asymmetric cryptography for every rejected client); hyper's `graceful_shutdown` immediately closes idle connections (F-122).

## Decision
1. (OD-7 item 1.) A request with `Transfer-Encoding` and `Content-Length` is handled per RFC 9112 §6.1 as hyper does: `Content-Length` is ignored, the body is chunked, the connection is closed after the response; 400 is not required. There is no boundary substitution (F-107, F-120). A request that cannot be parsed as HTTP/1.1 is still 400.
2. (OD-7 item 3.) When slots overflow (there is no one to evict or the per-address cap of 32 is reached), the connection is closed without a response — on both HTTP and HTTPS (one branch). The code `too_many_connections` and "no more than 16 concurrent refusals" are removed. Eviction remains: an anonymous one (class U) older than 1 s, then an idle one (class I); classes U/I/A, the limit of 16 of class U per address and the protection period are unchanged. The client sees a reset/close and retries; the UI shows "no connection".
3. (OD-7 item 5.) On shutdown idle connections (request headers not yet received) are closed without a response; 503 `server_stopping` is received only by requests whose headers were already received and by writes not taken by the writer. The race "the client writes into a closing connection" cannot be eliminated and is acceptable (the request was not executed).

## Alternatives considered
- Pre-parsing headers under hyper for 400: a second parser, rejected (expert's item 1).
- 503 on HTTP and closing on HTTPS: two branches, rejected as added complexity.
- Our own flag and dropping connections in order to give everyone 503 on shutdown: rejected (expert's item 15).

## Consequences
- "Changed by OD-7" in ADR 0062, 0067, 0071, 0072 (items 1–2, 5, 6, 8), 0074 (item 7), 0095 (item 9).
- The requirements of `server-runtime`, `http-api`, `api-access`, `web-ui-shell` are changed; the CUJ7/CUJ13 goal under a flood is met by eviction.

## Sources
- OD-7 items 1, 3, 5; `docs/reviews/rust-expert.md` items 1, 6, 15; RFC 9112 §6.1 (F-120); F-107, F-122.
