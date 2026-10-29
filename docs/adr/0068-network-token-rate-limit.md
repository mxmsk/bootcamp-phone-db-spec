# 0068. Request limit per network token
- Status: Accepted (OD-6)
- Label: [DEFAULT] (items 1–3 — Q-46, D-51); [OURS] (items 4–6)
- Date: 2026-10-27
- CUJ: CUJ8, CUJ9, CUJ10

## Context
The brief: a per-token request limit; requests that are too frequent get «слишком много запросов, повторите позже», other clients
do not notice (CUJ8). Decided: the limit applies only to network tokens (OD-3, Q-24). Not settled: the value, the window, the model, the retry hint,
whether one limit applies to all or is set at issuance (Q-46). Pitfall P-7g: the request limit and the connection limit protect against
different things.

## Decision
1. [DEFAULT] The model is a token bucket separately for each network token: capacity 100, refill 100 per second
   (that is, up to 100 requests per second steadily, a burst up to 100). The value is set in the server settings and is common to all
   network tokens; it is not set at token issuance and not shown in the list. Rationale: the search latency target p99 ≤ 10 ms
   (ADR 0049; brief §4) means that one sequential client makes on the order of 100 requests per second; the limit equals a
   fully loaded sequential client and restricts parallel "ramp-up" and erroneous loops. Alternatives:
   a fixed window (allows a double burst at the window boundary); a limit at token issuance (needs command fields and
   the question Q-40); a smaller value (10/s) — cuts legitimate batch searches. The number is subject to the owner's confirmation.
2. [DEFAULT] All requests of a token spend one reserve: every request is 1 unit, reads and writes are equal. A request with «нет прав» also
   spends (the token is recognized). A request for a page of the number stream spends 1 unit per request, not per record of the stream.
3. [DEFAULT] The hint is the `Retry-After` header (whole seconds, at least 1; RFC 6585 §4 allows it, F-92) (value — ADR 0063 item 8). The refusal is «слишком много запросов, повторите позже»;
   `rate_limited`, status 429 (RFC 6585, F-92; ADR 0062 item 8).
4. [OURS] The order is single, ADR 0063 item 7 (token → limit → permissions after Host and route). Requests with «не авторизован» and «хост не разрешён» spend no token's reserve (there is no identifier). A refusal by limit does not spend the reserve (otherwise a client
   repeating the request would not get out of refusal). A request does not wait and is not queued: the refusal is immediate, the connection's thread is occupied
   minimally.
5. [OURS] Isolation: the limit state is per token, in memory, without a shared lock between tokens (atomic counters or
   a per-token lock); exceeding the limit by one token does not change the reserve of others or others' latency beyond the targets (CUJ8). The reserve
   starts anew on restart; on token revocation it is deleted; a new token with the same name gets a full reserve.
6. [OURS] The connection cap (P-6, P-7f, P-7g): the request limit does not limit the number of connections, and one client within its own
   limit can occupy slots (slow clients, open connections). Slots must not go entirely to one
   source or one token, otherwise "other clients do not notice" is violated: ADR 0072 applies (loopback and network pools are separate, at most 32
   connections from a network address, at most 16 anonymous from an address, eviction of idle ones).

## Alternatives considered
See item 1. Additionally: a limit also on local programs and the UI — rejected by the owner (OD-3, Q-24); a limit in a queue with
waiting — rejected (it occupies a connection).

## Consequences
- The value 100/s (D-51) is a question to the owner as an area package (Q-46 is closed with a default value until confirmed).
- Tests: two tokens, one exceeds the limit, the other without refusals; a token with the limit then returns to normal.

## Sources
- ADR 0012 item 7; `docs/owner-decisions.md`, OD-3 (Q-18, Q-24); ADR 0049.
- Fact F-92; pitfalls P-6, P-7f, P-7g.
