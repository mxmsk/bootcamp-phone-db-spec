# 0057. Backup: cut, point, parameters, "backup" marker, start
- Status: Accepted
- Label: [OURS]
- Date: 2026-10-27
- CUJ: CUJ10, CUJ14, CUJ15, CUJ16, X-2, X-5

## Context
ADR 0016 [USER]: a backup by operator command, without stopping the server and without refusals, a consistent cut, restore is a start
from the backup; latency growth is acceptable within the targets (OD-3, Q-11). ADR 0028 items 5–7: the backup carries the "backup" marker; a start on it is a
new version generation. Q-83: which point the backup captures; Q-41: parameters. The database is owned by one process (X-2): an external
program cannot open it.

## Decision
1. The backup is made by the server process (the database owner) from a snapshot (ADR 0055) opened when the command is accepted. The command channel is area
   4 (Q-60); the server is stopped — no backup is made (files of a stopped database do not count as a backup, ADR 0028 item 7). At most
   one backup at a time; a second command is rejected («копирование уже выполняется» ("backup already in progress")). A clean shutdown cancels
   the backup, does not wait for it and stays clean (ADR 0028 item 10: backup is not a write operation).
2. Q-83. The backup point is the snapshot at the moment the server accepts the command: all operations acknowledged to clients before the command is accepted are included
   entirely (the operator command is issued after the client received success, so the snapshot is opened no earlier than publication); operations
   after the snapshot are not included; no partial ones. The commit number and the snapshot time (UTC) are recorded in the backup description; area 4 prints them to
   the operator. The alternative "the moment the backup finishes" (the backup contains everything acknowledged up to the end): rejected, it requires
   a "fuzzy" backup with WAL appending and gives a nondeterministic point.
3. Content: records, versions, indexes of one snapshot (start without rebuild, Q-42); tokens, settings and the generation counter are not
   included (ADR 0016 item 5, ADR 0028 item 9). The alternative "records only, rebuild indexes at start": a smaller backup, but
   a start from a backup of 10^8 records is minutes of rebuild (ADR 0058, calculation), restore after a failure takes longer; rejected.
4. Q-41, integrity check: every page of the snapshot is read and checked by checksum while copying; a mismatch —
   the backup is not created, the operator gets a message, the server goes to "storage error" (ADR 0014 item 6). The checksum is not a bottleneck
   (measured crc32c 8–12 GiB/s versus I/O of minutes for 40 GB). A separate backup verification command is not introduced (not in the brief); corruption of
   the archive after creation is detected on reading pages and at start (ADR 0014 item 5).
5. Q-41, completeness: the backup is written to a temporary place (in the target path); the last step is a flush of the data and the directory to
   stable storage and an atomic write of the description (the "backup" marker, format version and index labels, commit number, time, sizes and
   file checksums). A directory with `run_state = backup` without a full and valid description does not start ("backup incomplete"); without `control` it is not a database. The description is checked only for a backup: an ordinary database does not have one.
6. Q-41, target: the path does not exist or is empty; a non-empty target is a refusal, nothing is overwritten (like database creation, ADR 0015
   item 2). Before the start the free space of the target is checked ≥ the snapshot size plus a margin (default 5%); a shortage or a write error
   in progress is a refusal, the partial backup is deleted, the server continues working without refusing clients.
7. A "read-only" source: the backup proceeds (reads the snapshot, does not write to the source). "Storage error": the backup is made
   only if all pages are readable and pass the check; otherwise a refusal (item 4). Q-71 is a durability subtask, here only the behavior of
   the backup.
8. The "backup" marker is in the backup description and in the backup's `control` (not in `data`, ADR 0040 item 5; the composition of the backup's `control` — item 13); the source database does not have it. It is preserved on byte-for-byte duplication
   of the backup directory; cleared only on starting the server on it (ADR 0028 item 5). To the operator (area 4) document: to
   start repeatedly from one archive, duplicate the backup directory before the first start.
9. Start from backup: the start kind is determined from the medium; "backup" outranks "clean shutdown" and "crash". Steps: the change of version generation
   (ADR 0028 items 6, 8–9) is durably written before the marker is cleared and before operations are accepted; then the marker is cleared
   durably. A failure between the steps — the next start is also from backup (an extra generation change is harmless, ADR 0028 item 10).
10. A backup of the earlier version v1 is opened by the new program itself; index labels that differ from the server's labels lead to a
    rebuild (ADR 0058).
11. Impact on latency (OD-3, Q-11): a read speed limit on the source (value — ADR 0059), reads without evicting the cache
    working set, low priority; the backup takes no locks that readers or the writer wait on; the goal is p99
    of search and write within the targets Q-47, Q-58, Q-59 during backup on a 10^8 database (a load test; the target figures are set by other
    subtasks). Cost: retention (ADR 0056, item 7) and I/O; at the retention cap the backup is revoked, the write is not.
12. A "snapshot revoked" error during backup: the backup is rejected with a message, the partial backup is deleted; the operator
    retries (under lower load or at a higher speed).
13. Backup composition and the backup's `control` (closes Q-121). The backup: `data` (snapshot pages, `page_count` and META of the same snapshot), `control`, `dwb` of 128 empty slots, `wal/` with one zero
    segment without entries, `LOCK`. The backup's `control`: `run_state = backup`; the source's `db_id`; `next_S` ≥ the largest S in the snapshot's headers (the source's `next_S` at the snapshot LSN is taken, it is not less);
    the backup WAL start LSN and `redo_start_lsn` ≥ the largest `page_lsn` of the copied pages (the snapshot LSN rounded up to an entry boundary is taken; without this a start from backup would number
    entries below the pages' `page_lsn`, and recovery would skip acknowledged entries under the rule "`page_lsn` < LSN", ADR 0051 item 5 (d)); the WAL epoch ≥ the source's epoch at the snapshot; the source's `G_run`, `E_run`
    at the snapshot. Start from backup: the checks (ADR 0053), a recovery pass over an empty WAL (applies nothing), "end of recovery", then item 9. A cut at one LSN
    is given by the snapshot (item 1), so neither a source checkpoint nor WAL appending for the backup is needed. A backup in "storage error" — item 7.
14. **Changed by OD-6 (2026-10-28).** A start from a backup of 10^7 records is minutes (≈ 1 min) of rebuild; the load test is on 10^7 (OD-6).
15. **Changed by the final fix pass (2026-10-28, ADR 0095).** The commit number is the LSN of the entry of the last changing operation in the snapshot; the backup description is the `backup.desc` file; the backup's `next_S` is strictly greater than the largest S.

## Alternatives considered
- Byte-for-byte file copying under a write lock: rejected, this is a write stop of minutes (ADR 0016 item 1).
- A fuzzy file copy with subsequent WAL appending (as in some DBMSs): rejected, the point is undefined (item 2); at start
  WAL replay is needed, start time grows.
- Continuous restart of the backup when the source changes: rejected, in SQLite on a write from outside the backup
  restarts and "may never run to completion" (F-83); for us a write does not cancel the snapshot.
- Backup by a separate program over files: impossible, the database is owned by one process (X-2).
- A backup without page checks: rejected, a backup cannot be relied on (CUJ14: data is not silently lost).
- For Q-83 the sources had no other variants.

## Consequences
- Q-83, Q-41 and Q-121 are closed for storage (the command parameters in the CLI — area 4).
- Requirements for areas 3–4 — in the `design.md` of the change `add-storage-engine`, section "Requirements for other areas".

## Sources
- ADR 0014, 0015, 0016, 0028, 0055, 0056; OD-3 Q-11; `research/snapshots` (crcbench); SQLite backup API (F-83).

- **Changed by OD-7 (2026-10-28).** Items 1–3, 11–13: the backup is made without a snapshot and without retention — a checkpoint pause, fixing LSN X, copying `data` and the WAL up to X, a start from backup brings it by the same recovery up to X (ADR 0096); the "fuzzy backup" rejected here becomes accepted with a fixed point X. Item 11: "low priority" and "without cache eviction" are removed (ADR 0101); at the cap of changed pages the backup is aborted (item 12, ADR 0097).
