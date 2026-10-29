# 0074. Server start and clean shutdown: what clients and the operator see
- Status: Accepted
- Label: [OURS] (per the owner's decisions ADR 0015 [USER], ADR 0028 item 10; deadline values — ADR 0071, Proposed)
- Date: 2026-10-27
- CUJ: CUJ7, CUJ13, CUJ14, CUJ15, CUJ16, X-2; item 1.5 of the original wording

## Context
ADR 0058 item 2 leaves to area 3 the form of the "server starting" response during an index rebuild (up to ≈ 60 min on 10^8, D-26). ADR 0053 items 2, 3 and ADR 0054 item 4
require telling the operator the cause of a refusal, the kind of the previous stop and the number of discarded WAL bytes; CUJ13 — printing the UI address with an embedded token; the message
channel (Q-66) and the command format are area 4. ADR 0028 item 10 and ADR 0054 item 8 define clean shutdown on SIGTERM/SIGINT; a deadline of at least 60 s
(Q-108) is an area 2 requirement to area 4. Here is only the behavior of the HTTP side and the content of process messages.

## Decision
1. [OURS] Start order: (1) the `LOCK` lock (ADR 0054 item 1; busy — refusal «база уже открыта другим процессом» ("the database is already opened by another process")); (2) reading the settings and certificate
   (network listener enabled and no certificate — refusal, ADR 0012 item 3); (3) `control` is missing — refusal "there is no database" (ADR 0015 item 1); (4) opening the listeners
   (item 2); (5) start checks and recovery (ADR 0053, 0051 item 5), an index rebuild on label mismatch (ADR 0058), a new generation on an
   abnormal start (ADR 0054 items 5–6); (6) the local-program token file and the UI token anew (area 3 "access"); (7) the transition to working mode;
   (8) the «готов» ("ready") messages and the UI address (item 5). On a refusal at steps (1)–(3) there are no listeners. On a refusal at step (5) the listeners are closed, the operator gets the cause,
   the process ends with a non-zero code (codes — area 4).
   An area 4 addition (ADR 0077 item 1, ADR 0081 item 1): between steps (3) and (4) the server takes `control.lock` of the settings directory, parses `tokens` and opens the operator control channel; the other steps and their order do not change.
2. [OURS] Listeners are opened at step (4), before the checks and the rebuild. Reason: the client gets a definite response instead of "connection refused"
   (ADR 0058 item 2 allows both), the port is taken by us before another OS user can take it (P-7), and "server starting" is the same for all stages of (5).
   A bind error is a start refusal: address and `errno` (ADR 0070 item 3).
3. [OURS] The "server starting" mode (steps 4–6, until the working mode at step 7): every request on any path and method, on any listener, gets the refusal
   "server starting" (`server_starting`, `Retry-After: 5`, ADR 0062 item 8, 0063 items 6, 8) without reading the body, without checking the route and token. Storage is not
   called. The tokens of this start do not exist yet (step 6 ends before the working mode), so a token check makes no sense for the whole start period;
   an old token gets the same refusal, not "unauthorized" (the client retries and after readiness gets "unauthorized", re-reads the file,
   CUJ7). Only the connection limit and header parsing (invalid request, headers too large) come before `server_starting`: ADR 0063 item 7. The connection limits of
   ADR 0071 apply in this mode.
4. [OURS] Start messages (channel and format — area 4, Q-66; here the content, one entry per event, as text, without secrets, except item 5):
   (a) the database directory and the server version; (b) the kind of the previous stop: "clean", "crash" or "start from backup" (ADR 0054 item 4); (c) on a crash —
   the number of discarded WAL tail bytes and the recovery result (ADR 0053 item 3, 0051 item 5); (d) the index rebuild: the cause, the share done, an estimate of
   the remaining time — at least once per 10 s, on completion the duration (ADR 0058 item 4); (e) a start refusal: the cause, file, offset or LSN, `errno` (ADR 0053 item 2);
   (f) listener addresses; (g) the storage state if the database started in "read-only" or "storage error" (ADR 0053 items 5–7, 0054 item 7) — to the operator only;
   (h) «готов». Every storage state transition at run time is also an operator entry (ADR 0053 item 7).
5. [OURS] The UI address with an embedded token is printed once, after «готов» and only after all listeners are successfully bound, to the process's standard output
   and is not written to the server log or to files (P-5, CUJ13). The form of the address and the place of the token in it (fragment or otherwise) — area 3 "access" and area 5 (Q-38, P-1…P-5).
   After a restart a new address is printed; the old one does not work (CUJ13).
6. [OURS] The local-program token file is ready before the working mode (item 1: step 6 before step 7). Listeners accept connections from step 4, but requests before step 7
   get `server_starting` without a token check (item 3), so there is no window in which tokens are checked while the file holds the old token.
7. [OURS] Clean shutdown, the HTTP side (ADR 0054 item 8; SIGTERM, SIGINT, operator command): (1) the listeners are closed, new connections are not accepted;
   (2) a request that arrives on an already open connection after the signal gets the refusal "server stopping" with the connection closed and is not executed;
   idle connections are closed; (3) a write accepted by the writer completes with acknowledgment or rejection (none is left in flight; no later than the deadline of
   ADR 0071 item 4); (4) reads in progress at the moment of the signal get 5 s to finish, then are cancelled, connections are closed, snapshots are released; number
   streams are cut without the final marker; (5) then the storage steps (ADR 0054 item 8 (3)–(6)); a backup is cancelled (ADR 0057 item 1). A client whose write
   request was interrupted by closing the connection without a response acts as after a connection drop (ADR 0026 items 5–6).
8. [OURS] A repeated signal during shutdown neither speeds it up nor cancels it: the message «остановка уже выполняется» ("shutdown already in progress"). A stop during start cancels
   the rebuild (the earlier index is intact, ADR 0058 item 3) and ends the process; storage writes the marker per ADR 0054 item 8 if `run_state = running` is already written,
   and writes nothing otherwise.
9. [OURS] The shutdown deadline budget: step (4) ≤ 5 s (in parallel with step (3)) + storage steps ≲ 30 s (ADR 0054 item 8) ≈ 35 s; the area 4 deadline — at least 60 s
   (Q-108). A longer deadline is not required, but less than ≈ 35 s is not allowed either: a forced end before the marker is a crash shutdown.
10. **Changed by OD-6 (2026-10-28).** The index rebuild — up to ≈ 6 min on 10^7 (was ≈ 60 min on 10^8, D-26, OD-6).

## Alternatives considered
- Listeners after the rebuild (before that — "connection refused"): rejected, the client cannot tell a stopped server from a starting one, and the port stays
  free for a foreign process (P-7).
- Accept requests during the rebuild on the old index: rejected in ADR 0058 (a background rebuild — two code formats).
- Answer "unauthorized" in the start mode: rejected, the same refusal for a valid and an invalid token would hide the cause, and the client would re-read a file
  that is not there yet.
- Listeners stay open during shutdown and answer "server stopping" to new connections: rejected, the server lives up to 35 s without use, and the client
  gets the same "unavailable" effect; the refusal on existing connections is kept.
- Wait for all number streams to finish: rejected, 300 MB at 2 Mbit/s is hours, a clean shutdown would not fit the deadline (ADR 0054 item 8).
- Print the UI address before «готов» ("ready"): rejected, a link in the start mode would get "server starting" with no benefit.
- A second signal as an immediate exit: rejected, it turns a clean shutdown into a crash by an accidental keypress; for an immediate stop there is SIGKILL (ADR 0028 item 10).

## Consequences
- Requirements for area 4: the channel and format of messages, exit codes, a deadline ≥ 60 s, the listener settings, a stop during backup, SIGHUP handling.
- The kinds "server starting" and "server stopping" are `server_starting`, `server_stopping`, both 503 (ADR 0062 item 8); the first has `Retry-After: 5`.
- hyper's behavior on graceful close (`graceful_shutdown`, F-102…F-104) is checked by a test; our own stop flag is acceptable instead of it.

## Sources
- ADR 0012 item 3, 0014, 0015, 0026, 0028 item 10, 0051, 0053, 0054, 0057, 0058, 0071; `openspec/specs/user-journeys` (CUJ13, CUJ7);
  F-102…F-104; pitfalls P-5, P-7.

- **Changed by OD-7 (2026-10-28).** Item 7: on shutdown idle connections are closed without a response, 503 `server_stopping` — to requests with headers already accepted and to writes not yet taken (ADR 0098).
