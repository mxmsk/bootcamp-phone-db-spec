# 0095. Final fix pass: decisions on storage, startup, access and the UI
- Status: Accepted
- Closes the findings of the final review (`docs/reviews/review-*.md`): storage#1–9, server#1–9, admin-ui#1–6, data#1–3; Q-166
- Label: [OURS]
- Date: 2026-10-28
- CUJ: CUJ1, CUJ7, CUJ9, CUJ11, CUJ13, CUJ14, CUJ15, CUJ16

## Context
The final review found places where two implementations of the same specification would give different observable behavior. The decisions below were chosen as the simplest ones, consistent with OD-1…OD-6 and the CUJs; they introduce no new operations or capabilities. None changes the brief or requires an owner decision. UI strings are quoted in Russian, the language of the interface; English glosses are in parentheses.

## Decision
1. [OURS] File growth does not reserve blocks (`set_len`, no `fallocate`): no crate beyond std is needed (storage#3). `ENOSPC` may surface when checkpoint pages are written, already after operations were acknowledged. A checkpoint failure: `ENOSPC`, `EDQUOT`, `EROFS`, `EACCES`, `EPERM` — "read-only", anything else (`EIO`) — "storage error" (ADR 0053 items 5–6); the WAL is not truncated. Exit from a "read-only" caused by a checkpoint is only by a successful checkpoint (the probe = a repeated checkpoint); a successful probe write to the WAL does not give an exit, otherwise the WAL would grow beyond 256 MiB.
2. [OURS] A page of all zeros that the first applied WAL entry initializes entirely (the first change after it was handed out) is accepted as empty on recovery — both inside the file and past its end (storage#1). A zero page referenced by a tree or list without such an entry is corruption. Otherwise an ordinary `SIGKILL` after file growth would lead to a start refusal (CUJ14).
3. [OURS] `next_S` is strictly greater than the largest S in the headers (storage#2): `next_S` is the next value to be issued; equality would give a repeated version (X-5).
4. [OURS] The commit number (storage#9) is the LSN of the WAL entry of the operation that changed data: it grows across restarts; "unchanged" operations and rejected ones do not get a number (they have no WAL entry). A snapshot is the state after all operations with a number not greater than C.
5. [OURS] Layout (storage#8): `control` — two 4 KiB slots (magic number, layout label, slot number, `db_id`, `run_state`, `redo_start_lsn`, `next_S`, `G_run`, `E_run`, WAL epoch, CRC-32C); `dwb` — page slots with `db_id`; a `wal/` segment — a header with `db_id`, number and starting LSN; an entry — the fields of ADR 0051 item 2. The v1 labels are read by all v1 programs. The backup description is the file `backup.desc` in the backup directory (`key=value` lines and a final sum), written as the last step atomically; it is absent in a working database, a start from backup deletes it after clearing the `backup` marker. "Backup incomplete" — there is a `control` with `run_state = backup` but no valid `backup.desc`; "not a database" — there is no `control`.
6. [OURS] A start from backup with unwritable files is a start refusal, as for a crash shutdown (ADR 0053 item 5): a start from backup requires writing (generation change, clearing the marker) (storage#7).
7. [OURS] Errors of the temporary `retain` file move the database neither to "read-only" nor to "storage error": any I/O error on `retain` only revokes snapshots in the order of ADR 0056 (storage#6). Snapshot revocation (`snapshot_revoked`) is an exception to "word-prefix search always completes with a result" (storage#4).
8. [OURS] Refusals produced by HTTP parsing (hyper) before the service is called — an unparsable request, `Transfer-Encoding` with `Content-Length`, 431 — are given by status only; a `{code, …}` body is not guaranteed, the client parses by status class (server#1). There is no separate cap on the number of headers: hyper's `max_headers` is lifted; a prototype check is Q-167.
9. [OURS] 401 `unauthorized` always closes the connection (`Connection: close`, graceful close); likewise 400 on a boundary error, 413, 431, 503 `server_stopping`, "too many connections"; other refusals do not close the connection (server#3). The eviction rule for 16 anonymous connections of an address takes precedence over the cap of 32 connections per address (server#4).
10. [OURS] A network token on the loopback listener (HTTP without TLS) is not recognized under any network listener setting (server#2, OD-3, Q-27). The local-program token file is 64 lowercase hexadecimal characters and LF (server#5). With loopback port 80 a Host without a port is acceptable (server#6). `null` for a known JSON member is equivalent to absence (server#8, ADR 0062 item 3). A write not taken by the writer by the time of the stop signal (queued, with an unread body) gets 503 `server_stopping` and is not executed (server#9).
11. [OURS] `serve`: the database directory is checked for existence before `LOCK` (missing — code 5, nothing is created); a leftover `LOCK` is acceptable in a directory without a database (`init` accepts such a directory) (admin-ui#2). A server that started in "storage error" without a control channel (the settings directory is not writable) holds the existing `control.lock` for reading; commands reply «сервер работает без канала команд; остановка — сигналом SIGTERM» ("the server is running without a control channel; stop it with SIGTERM"), code 37 (admin-ui#1). A connection drop on the backup command without a final message is always code 24 (admin-ui#6).
12. [OURS] UI: no status received — «нет связи с сервером» ("no connection to the server") and «Повторить» ("Retry"); 200 but a body without `complete` — «ответ получен не полностью» ("the response was not received in full") and «Повторить» (not "storage error") (admin-ui#3). "Outcome unknown" — without automatic re-reading and without the claim "a retry is safe" (admin-ui#5). A history failure after a successful save — «История номера не загружена: …» ("The number history was not loaded: …") (admin-ui#4).
13. [OURS] A retry after an unknown outcome: an add without a date — "number known"; a rename to the same name — "unchanged", one that changed the name — "version conflict" (data#1, data#2). Protocol-level refusals (`invalid_format` for a date, `wrong_type`, `missing`) belong to `http-api` and do not violate the `records` dictionary (data#3).

## Alternatives considered
- `fallocate`/`F_PREALLOCATE` for file growth (item 1): rejected — it needs `libc`, platform-specific code and one more crate decision; it is enough to describe the failure at a checkpoint honestly.
- The commit number as a separate counter in `control` (item 4): rejected — the LSN is already continuous and durable, a second counter would have to be synchronized with the WAL.
- The backup description as fields of a `control` slot (item 5): rejected — a 4 KiB slot with a sum must stay identical for the database and the backup; a separate file distinguishes "backup incomplete" from "not a database".
- Checking `control` before `LOCK` for all modes (item 11): rejected — it changes the order of refusal codes of ADR 0077 item 1; checking the directory and tolerating a leftover `LOCK` is enough.
- A start from backup on read-only media in "read-only" (item 6): rejected — versions of earlier generations would remain valid, a generation change is impossible (X-5).

## Consequences
- A new exit code 37 of the control commands (ADR 0084, range 20–39).
- Q-167 (T) is opened: verify on a prototype hyper (refusals without a body, `max_headers`) and `set_len`/`ENOSPC` on Linux and macOS.
- ADR 0053, 0054, 0057, 0062, 0077, 0080, 0084 are supplemented (the item "Changed by the final fix pass").

## Sources
- `docs/reviews/review-data.md`, `review-storage.md`, `review-server.md`, `review-admin-ui.md`; ADR 0040, 0044, 0050, 0051, 0053, 0054, 0056, 0057, 0062, 0065, 0067, 0072, 0075, 0077, 0080, 0081, 0083, 0084, 0090, 0094; OD-3 (Q-27), OD-6.

- **Changed by OD-7 (2026-10-28).** Item 9: "too many connections" does not close the connection after the response — there is no response; 400 "on a boundary error" — only for an unparsable request (ADR 0098).
