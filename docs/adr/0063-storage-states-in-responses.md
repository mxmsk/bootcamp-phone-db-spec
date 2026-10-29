# 0063. Server refusals in API responses: storage states, start and stop, check order
- Status: Accepted ([OURS]: items 1–7); Accepted (OD-6) ([DEFAULT]: item 8 — `Retry-After`, D-42)
- Label: [OURS] (items 1–7; per the owner's decisions ADR 0011 item 6 [USER] and ADR 0014 [USER]); [DEFAULT] (item 8)
- Date: 2026-10-27
- CUJ: CUJ4, CUJ5, CUJ7, CUJ8, CUJ9, CUJ14, CUJ16, X-5

## Context
CUJ16 and ADR 0011 item 6 [USER]: the user learns of "read-only" and "storage error" when their action fails; the state is not shown
in advance; search works as long as it can. Area 2 set the sources and exits of the states (ADR 0053), snapshot revocation (ADR 0056 item 2), the rebuild
of indexes at start (ADR 0058 item 2), the database state before the operation's checks (`write-operations`). Q-44: which refusal the API client gets.
This ADR replaces two earlier ones that duplicated each other (0063 "codes and order" and 0073 "kinds of refusals"; the latter is Superseded by 0063): here is
one decision — which condition corresponds to which refusal, what the server does not show, and a single check order. Codes and statuses — ADR 0062 items 4 and 8,
spec `http-api`.

## Decision
1. [OURS] The server does not disclose the storage state by anything except the refusal of the action it prevented: there is no state or health resource, no
   state field or header in successful responses, no ready-made flags for the UI. ("Server starting" and "server stopping",
   ADR 0074, are lifecycle states, not storage states.) The state is reported to the operator in the message channel (ADR 0053 item 7, ADR 0074 item 4).
2. [OURS] Correspondence of conditions to refusals:
   - a write operation in "read-only" (the durable write probe failed, ADR 0053 item 8) — 503 `read_only`: the state is temporary and
     is cleared by the operator without a restart, a retry is safe;
   - a write operation in "storage error" and a disk write error of the operation itself (ADR 0053 item 6) — 500 `storage_error`: persists until
     restart; `storage_error` takes priority when both states hold (ADR 0053 item 7); both refusals come before value and version checks
     (ADR 0025 item 4, 0053 item 8): an identical retry and a stale version also get the state refusal;
   - a read that needs a corrupted page, or a read error — 500 `storage_error` (ADR 0053 item 4); a read that does not touch
     them, and all searches in "read-only", are executed;
   - snapshot revoked (30 s ceiling, idle, retention cap; ADR 0056) — 503 `snapshot_revoked`, if before the response begins: a repeatable
     error, not `storage_error` and not `internal_error`; in a stream after sending has begun — a cut without a marker (item 5);
   - a tree key above 65,535 bytes (D-36) — 500 `internal_error`, the state does not change; an unavailable generation counter — `storage_error`
     (ADR 0053 item 6 (e)); other unexpected engine errors — `internal_error`;
   - request deadline (ADR 0071): a read that did not begin a response or a write not accepted by the writer — 503 `request_timeout` (not executed); a write accepted,
     no outcome — 504 `write_outcome_unknown` (ADR 0071 item 4).
3. [OURS] The server does not cache the storage state at the HTTP level: every write goes to the engine, which decides by the current state and by the probe
   (ADR 0053 item 8): after space is freed the next change passes without a restart and without a "stuck" refusal.
4. [OURS] The server does not tell the client `errno`, file names, offsets, page numbers or the text of system errors: these go to the operator (ADR 0053 item 6,
   ADR 0074 item 4). The client gets a code, a status and a message of fixed composition.
5. [OURS] A refusal possible before the response begins is issued as a full error response. For a non-stream response this is always so: a page or result
   is prepared entirely before the first byte. For the number history stream the status cannot be changed after headers are sent: on snapshot revocation and a storage
   error in the middle of the stream the server breaks the connection without the final chunk and without the completeness marker (ADR 0035 item 6, ADR 0056 item 2);
   a cut stream never looks complete. The UI gets the same responses as an API client; texts («Сохранение не выполнено: база доступна только для чтения» ("Saving failed: the database is read-only") and others) — area 5 by code; there is nothing to poll for the state.
6. [OURS] Start and stop (ADR 0058 item 2, ADR 0074): for the whole period from opening the listeners to the working mode — 503 `server_starting` on any request
   on any path, before the route and token checks (there are no tokens of this start yet, ADR 0074 item 3). "Refusal" is chosen over "connections are not accepted":
   the client distinguishes "starting" from "not running". After a stop signal, on an open connection — 503 `server_stopping`.
7. [OURS] A single check order (source — spec `http-api`, "Request check order"; other ADRs and capabilities refer here):
   (1) the listener's connection limit; (2) header parsing (400 `invalid_request`, 431); (3) start and stop; (4) loopback listener Host; (5) route and
   method; (6) token → request limit → permissions, by headers only; (7) body: type, cap, JSON parsing; (8) database state (for writes); (9) values in field
   order; (10) the operation. Rationale: (a) start before the token — there are no tokens of this start yet, and an old token must not get "unauthorized"
   instead of "starting" (the client would re-read a file that is not there yet, CUJ7); (b) Host before the token — a foreign name must not trigger a check (ADR 0067 item 1);
   (c) access before the body — a refusal costs the minimum, the body is not read, and a client without a token learns nothing of sizes or of the database state (CUJ8); (d) the limit before
   permissions — a permissions refusal also spends the token's limit (ADR 0068 item 2), and exceeding the limit is seen earlier; (e) access before the database state (ADR 0025 item 4:
   the order between them is an area decision). An error at an early step does not require executing a later one: the body is not read, storage is not called (steps 1–6).
8. [DEFAULT] (D-42) `Retry-After`: for `server_starting` — 5 s; for `rate_limited` — whole seconds, at least 1 (ADR 0068 item 3); for `snapshot_revoked` — 0
   (retry at once); for the rest it is not set (the time is unknown). The rebuild duration (ADR 0058, D-26) was not taken for the estimate. Alternative: do not
   send a hint.

## Alternatives considered
- The database state before access: rejected — it discloses the state to the unauthenticated.
- A state resource (`/v1/status`) or a state header in every response: rejected — a new operation outside the brief and a direct violation of "the UI is not shown
  the server state in advance" (CUJ16).
- 507 for `read_only`: rejected — it refers to a shortage of space, while the causes are broader (permissions, mounting); 500 for `read_only`: the client cannot tell unrecoverable
  from temporary.
- "Snapshot revoked" as "storage error": rejected, ADR 0056 item 2 explicitly separates revocation, and the client needs to retry, not call the operator.
- Do not listen on the port during a rebuild: rejected — ECONNREFUSED is indistinguishable from a non-running server.
- Check the token at `server_starting` (the earlier wording of item 4): rejected — there are no tokens of this start yet; an old token would get "unauthorized"
  instead of "starting".
- Size before access (the earlier wording): rejected — a client without a token would get 413, and a "read" token with a large body — not «нет прав».
- Give the client `errno`: rejected, disclosure of environment details. Buffer the whole stream for the sake of the status: rejected, 300 MB of memory per request
  (ADR 0056 item 6).

## Consequences
- The request timeout of 10 s (Q-52) does not apply to the number stream: ADR 0071 items 2–3 (20 s to the first byte, client idle 120 s).
- Tests: injecting a failure with a given `errno` (ADR 0053, `storage-durability`, "\"Read-only\" state") at the HTTP level checks item 2; the check order — one
  test for each pair of adjacent steps.

## Sources
- ADR 0011 item 6, 0014, 0025, 0035 item 6, 0049, 0053, 0054, 0055, 0056, 0058, 0062, 0071, 0074; `openspec/specs/user-journeys` (CUJ16);
  `openspec/specs/storage-durability`; `docs/open-questions.md` Q-44, Q-52, Q-66, Q-71.
