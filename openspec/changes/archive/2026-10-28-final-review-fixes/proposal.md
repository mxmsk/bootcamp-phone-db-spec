## Why

The final review of four groups (`docs/reviews/review-data.md`, `review-storage.md`, `review-server.md`, `review-admin-ui.md`) found 28 items: BLOCKERs and contradictions between capabilities. In addition, `user-journeys` still had open-question placeholders and marks of waiting for the owner's answer even though all [DEFAULT] items were accepted by the owner (OD-6), and the Purpose of several capabilities still had the 10^8 scale.

## What Changes

Closing the findings (file#item → fix):

- **data#1** retry of an add without a date in CUJ1/CUJ9: `user-journeys` (CUJ1, CUJ9) — scenarios "with a date" and "without a date" (outcome "number known").
- **data#2** `write-operations`, "Retrying an operation after an unknown outcome": a rename to the same name — "unchanged"; a caveat about an add with and without a date.
- **data#3** `records`, "Value error kind dictionary": protocol-level refusals with the `http-api` reasons; `http-api` — a new requirement "Absent member and `null`" (ADR 0062 item 3).
- **storage#1** zero page on recovery: `storage-format` ("Common page header and types", "Checks when opening the file"), `storage-durability` ("Consistency check at start").
- **storage#2** `next_S` strictly greater than the largest S: `storage-format` ("Number header and number version"), `storage-durability` ("Version does not require a number scan on generation change"), `storage-snapshots` ("Contents of the backup control file").
- **storage#3** checkpoint and `ENOSPC`: `storage-durability` ("Checkpoints bound the WAL", "Leaving the states"), `storage-format` ("File growth and out of space"); ADR 0095.
- **storage#4** `storage-indexes`, "Word search always completes with a result, with no cap on word count": the exceptions are snapshot revocation and request timeout.
- **storage#5** `storage-format`, "Data file header": the signature is bytes 32…39.
- **storage#6** `retain` does not move the database to "read-only" or "storage error": `storage-durability`, `storage-snapshots`.
- **storage#7** start from a backup with unwritable files is a start refusal: `storage-durability`.
- **storage#8** layout of `control`, `dwb`, `wal/` and the `backup.desc` file: `storage-durability` (new requirement), `storage-format`, `storage-snapshots`; ADR 0095.
- **storage#9** commit number = the LSN of the data-changing operation's WAL entry: `storage-snapshots` ("Write operations are strictly sequential", "One snapshot per request", the check); ADR 0095.
- **server#1** refusals produced by hyper parsing: status only: `http-api` ("Error body", "Client-failure codes and statuses", "Header size cap"), `server-runtime`; the number of headers is not limited separately; the hyper check is Q-167.
- **server#2** a network token on loopback with the network enabled: `api-access`, "UI and file tokens are accepted only on the loopback listener".
- **server#3** 401 always closes the connection: `server-runtime` (new "Refusals that close the connection"), `http-api` ("Failure headers"), `api-access`.
- **server#4** order of the 32-connection cap and eviction of anonymous ones: `server-runtime` ("Connection limits", "Anonymous connections from one network address").
- **server#5** token file format: `api-access`, "Local-program token in a file".
- **server#6** Host without a port on port 80: `api-access`, "Loopback protection: Host check".
- **server#7** search query edges versus the header cap: `records` ("Input size before normalization and request body cap"), `http-api` ("Header size cap").
- **server#8** `null` is equivalent to an absent member: `http-api`, a new requirement.
- **server#9** a write not taken by the writer at shutdown is 503 `server_stopping`: `server-runtime` ("Clean shutdown: HTTP side"), `http-api` ("Snapshot revocation, server start and shutdown").
- **admin-ui#1** server without a control channel: `operator-control` ("Channel works under load and storage failures", "Command output and exit codes"; code 37), `operator-lifecycle`, `server-runtime` ("Startup order").
- **admin-ui#2** `serve` on an empty directory: `operator-lifecycle` ("Server does not start on a directory without a database") — a leftover `LOCK` is allowed; `server-runtime` — directory check before `LOCK`.
- **admin-ui#3** stream break: `web-ui-shell` ("Server stopping" and connection drop, "Read-only" and "storage error" texts…), `web-ui-flows` ("Search by number (CUJ4)", "Known number is determined by the server response", "Flow message texts") — «ответ получен не полностью» ("the response was received incompletely") with «Повторить» ("Retry").
- **admin-ui#4** texts for a failed re-read: `web-ui-shell` refers to `web-ui-flows`; `web-ui-flows` ("What to show after saving") — the text "The number history was not loaded…".
- **admin-ui#5** "outcome unknown" without auto re-read: `web-ui-flows`, "Entered input is kept when a write fails".
- **admin-ui#6** `operator-control`, "Backup cancellation": a connection drop on the command is always code 24.
- **admin-ui#7** Purpose of `operator-lifecycle` (10^8 → 10^7) — a direct edit of the Purpose (deltas do not change the Purpose); likewise `storage-format`, `storage-indexes`.

OD-6 placeholders: in `user-journeys` all open-question placeholders were replaced with accepted values (D-x, ADR); the stale "Open: Q-x" were replaced with "Decided: …"; the placeholder mechanism was removed from the Purpose (direct edit); the marks of waiting for the owner were removed (`api-access`, `server-runtime`, `storage-durability`).

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `user-journeys`, `records`, `write-operations`, `http-api`, `api-access`, `server-runtime`, `storage-format`, `storage-durability`, `storage-snapshots`, `storage-indexes`, `operator-control`, `operator-lifecycle`, `web-ui-shell`, `web-ui-flows`.

## Impact

No new operations or capabilities are introduced. New ADR: 0095 (storage and startup decisions). A new exit code 37 for control commands (ADR 0084, range 20–39). New questions (T): Q-167. There are no findings that require an owner decision.
