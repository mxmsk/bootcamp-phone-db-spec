## Why

The owner accepted the results of the expert assessment of feasibility in Rust (`docs/owner-decisions.md`, OD-7; the source is `docs/reviews/rust-expert.md`, verdict "feasible with reservations"). The spec has to be brought into line: some requirements cannot be met on hyper as written, some cost more than the CUJs need.

## What Changes

Eight OD-7 decisions (the simplest change, the CUJs are kept, no new operations, OD-1…OD-6 are not violated):

1. TE+CL (item 1) — per RFC 9112 §6.1, as in hyper: `Content-Length` is ignored, the connection is closed after the response; 400 remains for an unparseable request (`server-runtime`, `http-api`).
2. Backup without retention (item 2) — pause the checkpoints, fix LSN X, copy `data` and the WAL up to X, and start from the backup brings it up by the same restore; the "backup" class is removed from snapshot revocation (`storage-snapshots`, `storage-durability`, `operator-control`).
3. Slot overflow (item 3) — closing without a response, the "16 refusals" cap and the code `too_many_connections` are removed; eviction is kept (`server-runtime`, `http-api`, `api-access`, `web-ui-shell`).
4. Word search (item 4) — D without aggregates, G2 and G3 first, G1 by profile (`storage-indexes`).
5. Shutdown (item 5) — idle connections are closed without a response, 503 `server_stopping` only when headers have been accepted (`server-runtime`, `http-api`).
6. Memory for modified pages (item 6) — a cap of 1 GiB [OURS], the writer waits for a checkpoint, during a backup the cap interrupts the backup (`storage-durability`).
7. UI and the stream by number (item 7) — responsiveness up to about 10^4 records, the stream buffer is bounded (`web-ui-flows`, `server-runtime`).
8. The crates `rustix` and `socket2` (item 8) — ADR 0101; "low priority" and "without evicting the OS cache" are removed.

Also: ADR 0096–0101, the "Changed by OD-7" items in the affected ADRs, facts F-120…F-127, 13 new (T) Q-168…Q-180 in `docs/open-questions.md`.

## Capabilities

### Modified Capabilities
- `server-runtime`: limits and slot protection, TE+CL, shutdown, the stream by number.
- `http-api`: failure codes, check order, shutdown.
- `api-access`, `web-ui-shell`, `web-ui-flows`: wording of refusals on overflow, number history.
- `storage-indexes`: dictionary D and the search methods.
- `storage-durability`: checkpoints, the new requirement "Memory for modified pages is capped".
- `storage-snapshots`: backup without a snapshot, the retention cap without the backup class.
- `operator-control`: space, cancellation and outcomes of backup.

## Impact

32 MODIFIED (3 of them with a rename), 1 ADDED, 0 REMOVED, 3 RENAMED. The expert's items that remain open are (T) Q-168…Q-180.
