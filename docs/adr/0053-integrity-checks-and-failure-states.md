# 0053. Integrity checks at start and at read; the "read-only" and "storage error" states
- Status: Accepted ([OURS]: items 1–4, 6, 7, 9); Accepted (OD-6) ([DEFAULT]: items 5 (errno classification), 8)
- Label: [OURS] (items 1–4, 5 except errno classification, 6, 7, 9); [DEFAULT] (item 5 — D-34; item 8 — D-29)
- Date: 2026-10-27
- CUJ: CUJ13, CUJ14, CUJ15, CUJ16, X-1

## Context
The owner decided (ADR 0014 [USER], items 4–7; OD-4, item 3): at start it is checked that the database opens as consistent (structure
and recent changes), otherwise it does not open; corruption of the remaining data is detected at read (checksums) and leads to the
"storage error" state; in "read-only" all searches work, in storage error — those for which the readable data is enough.
Not defined: what "structure" and "recent changes" include (Q-43), where the states come from and how they are exited
(Q-71). Start on 10^8 records is bounded by ADR 0052 (≤ 30 s), so the check cannot walk all records.

## Decision
1. [OURS] The start check (Q-43) runs before requests are accepted and consists of: (a) `control`: at least one of the two slots is valid
   (magic number, checksum, number); the `control` layout label is not newer than supported (the `data` format version lives in META and
   is checked when `data` is opened, ADR 0040 items 4, 6); the database identifier (`db_id`) is the same in `control`,
   the `data` header, `dwb` and the WAL segment headers; the page size matches; (b) a `dwb` pass (up to 128 pages): a page in
   `data` with a wrong checksum and a valid copy in `dwb` is restored, without a valid copy — refusal; (c) the WAL from `redo_start_lsn`
   to the end: each entry is checked (ADR 0051 items 5–6), these are the "recent changes"; (d) every page read while applying the
   WAL is checked by checksum, number and type; (e) "structure": the `data` service page, the root pages of every index and the
   placement metadata — checksum, number, type; the `data` length is not less than the placement boundary from the metadata. The cost is O(number of indexes +
   WAL since the last checkpoint), not O(number of records).
2. [OURS] Start refusal (the server does not start, `data`/`wal` are not modified, except restoring a torn page from `dwb`;
   the "end of recovery" entry and tail truncation — only after all checks succeed): inconsistency under items 1 (a), (b), (e);
   WAL corruption in the middle (ADR 0051 item 6); a page needed to apply the WAL with a wrong checksum; a format newer than the
   program. Operator message: cause, file, offset or LSN (channel — Q-66, area 3/4). The database does not open, records are not
   silently discarded (ADR 0014 item 4). The operator restores the database from a backup (CUJ14, CUJ15).
3. [OURS] A torn WAL tail is not corruption: it is discarded, the server starts, and the operator is told the number of discarded bytes and
   the kind of the previous stop (ADR 0054). A "lying" disk is not detected (ADR 0014; OD-4, item 3).
4. [OURS] Check at read: every page read from the `data` file into the cache is checked (CRC-32C, the page number
   in the header equals the requested one, the type is expected). A page in the cache is not rechecked. A mismatch or a read error (`EIO`):
   the page is marked corrupted in memory (repeated reads do not try again), the server moves to "storage error" (item 6); a search
   that needs the page ends with a "storage error" refusal; a search that does not touch corrupted pages is executed (Q-14).
   "Readable data is enough" = "the search does not touch pages from the set of corrupted ones". The same path covers a violation of a structure
   invariant found at read (key order, a reference to a page outside the file).
5. [OURS; errno classification — DEFAULT, D-34] The "read-only" state (CUJ16: «база доступна только для чтения» ("the database is available for reading only")). The owner's decision Q-20 ("disk write error — 'storage error'")
   is refined by this item: a failed flush and any error outside the list below (item 6) remain a storage error, while the listed errors, in which nothing is lost and the fix depends
   on the operator (space, permissions, mounting), give "read-only". Alternative: all write errors — "storage error" (then a restart is needed after space is freed). Sources: a write operation, opening for write,
   creating a WAL segment, extending a file, a rename failed with `EROFS`, `EACCES`, `EPERM`, `ENOSPC` or
   `EDQUOT` (the file system is mounted read-only or remounted by the kernel, permissions, a full disk, a quota). The membership
   condition: no acknowledged operation is lost and no page is corrupted — the error occurred before the flush. The operation
   is not acknowledged and not published. Readers work without restrictions (all searches). Start: if the files are not writable and
   the previous stop was clean (the WAL after the checkpoint is empty), the server starts in "read-only" (writing the "running" marker
   is impossible until it succeeds); if the stop was a crash or this is a backup — recovery requires writing: the server does not
   start, the message gives the cause (the alternative "apply the WAL in memory only" is rejected for complexity, see below).
6. [OURS] The "storage error" state (CUJ16, ADR 0014 item 6). Sources: (a) a failed flush (`fsync`, `fdatasync`, `F_FULLFSYNC`)
   of the WAL, data, `dwb`, `control`, directory (ADR 0050 item 5); (b) a read or write error not covered by item 5 (`EIO` and any
   `errno` not listed in item 5 — conservatively); (c) corruption found under item 4, and an invariant violation; (d) a checkpoint refusal
   for reason (b); (e) the generation counter or randomness source being unavailable at a start that requires a new generation
   (ADR 0054 items 5–6). Changes are rejected; search works as long as it can (item 4); the operator gets a message: source, `errno`, file,
   offset or page number.
7. [OURS] Priority and memory: "storage error" is stronger than "read-only"; the state is kept in memory (not on disk) and is reset only by
   a restart; every transition is written as an operator message.
8. [DEFAULT, D-29] Exit from the states (Q-71): (a) from "storage error" — only a server restart: after a flush error
   and corruption the state of the medium is unknown, the start checks decide (ADR 0053 items 1–3, ADR 0051 item 5) and, on start refusal, a backup;
   (b) from "read-only" — by itself, via a probe write. The state is checked before the operation's checks (area 1, "Check order for writes": access and database state checks come first):
   while the state is "read-only", every accepted write operation begins with a probe — a durable service WAL entry (ADR 0051 item 10), independent of the outcome of the operation's checks.
   The probe failed — "read-only" refusal (input, version and retry are not considered), the state is unchanged. The probe succeeded — the state is normal, the operator gets a message, the operation goes through
   the normal path (checks, then write). There is no background polling. An operator who freed space or restored permissions does not restart the server. Safe: an error before the flush does not change the acknowledged state,
   the unpublished state is rolled back (ADR 0051 item 9), and the probe and the new write go from the previous durable LSN over the failed one (ADR 0051 item 6: validity by CRC and LSN).
9. [OURS] The CUJ16 test (reproducibility): file operations go through an interface with injected failures (an error with a given `errno` on a given
   operation and call), without a real disk overflow; a read failure — a corrupted byte in a page of a test file.
10. **Changed by OD-6 (2026-10-28).** The scale is 10^7 records (ADR 0017); the numbers and estimates for 10^8 in this ADR are the history of the calculation, the effective values for 10^7 are in ADR 0044, 0049, 0052, 0058, 0059; non-linear estimates are recomputed on the prototype (T, `docs/open-questions.md`).
11. **Changed by the final fix pass (2026-10-28, ADR 0095).** A zero page initialized by a WAL entry is accepted as empty (inside the file and past its end); a checkpoint failure gives "read-only"/"storage error" by `errno`, and exit from "read-only" caused by it is via a successful checkpoint; `retain` errors do not change the state; start from a backup with non-writable files is a refusal.

## Alternatives considered
- Check the checksum of every page at start (a full walk): 40 GB ≈ 10 s … 270 s (ADR 0052), grows with the database; contradicts the
  owner's decision (structure and recent changes). Rejected.
- Do not check the WAL at start, only apply it: corruption in the middle of the WAL would silently truncate acknowledged changes (ADR 0014 item 4).
  Rejected.
- "Storage error" as a self-healing state (retry on a timer): after a flush error a retry is misleading
  (F-71, F-72); after page corruption a retry does not fix anything. Rejected.
- Restart only, also from "read-only" (variant (a) of item 8 for both): simpler and more predictable, but after freeing the disk the operator
  has to restart the server despite having no data error. An alternative for the owner.
- Background polling "has writing become possible": adds a thread and writes without a user request. Rejected.
- In-memory recovery at start on a database not writable: the WAL is applied to the cache only, the files are not changed;
  search works on the "WAL applied" state. Cost: a second WAL application mode, the inability to flush pages when
  space is freed without moving to the normal mode, testing complexity. Not introduced (a new function outside the brief).
- A "read-only" mode on operator command: rejected by the owner (ADR 0018, Q-17).

## Consequences
- Q-43 and Q-71 are closed (Q-71 — item 8 as [DEFAULT], the owner's answer may change item 8 (b)). Q-107 — ADR 0054.
- Requirements for other areas: operator messages contain the source and location (Q-66); API codes and UI texts for "read-only" and
  "storage error" (Q-44, Q-65) — areas 3 and 5; distinguishability of the two states in an API response — area 3.
- Backup during "storage error": a recommendation to the backup subtask — do not make a backup (consistency is not guaranteed); during "read-only"
  — make one, if no write to the source directory is required (Q-71, CUJ15).
- A full disk turns into "read-only" less often: WAL segments are created in advance (ADR 0050 item 3), while an `ENOSPC` error on
  deferred allocation under `write` on copy-on-write file systems is not excluded (not verified).

## Sources
- ADR 0014 items 4–7, 0018; OD-4 item 3, OD-3 Q-12, Q-14, Q-17 (`docs/owner-decisions.md`); `docs/open-questions.md`, Q-43, Q-71.
- F-65, F-67, F-71, F-72 (`facts.md`); `research/durability/read-bench.py`.
