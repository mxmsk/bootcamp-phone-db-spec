## Why

Area 2 under `docs/process.md` ("Storage") turns the semantics of data, operations and searches accepted in area 1 into a
dedicated storage engine: without it we can neither implement the server (area 3) nor the operator (area 4), nor verify
the scale of 10^8 records and reliability (CUJ14–CUJ16, X-1, X-2, X-5). Off-the-shelf storages and search engines are forbidden (ADR 0017),
so the format, indexes, WAL, snapshots and backup must be defined by requirements and ADR 0040–0059.

## What Changes

- File format: the database directory (`data`, `control`, `dwb`, `wal/`, `LOCK`), the header and format version in v1, the 16 KiB page,
  spilling large keys to overflow, record and number header encoding, CRC-32C checksums, free pages and
  file growth, the size at 10^8 records (ADR 0040–0044).
- Indexes: records R and derived trees of order O, words W and dictionary D on a single B+-tree; search by number, results by name with
  continuation from any position, word-prefix search without full materialization, exact search, the label and rebuild, size
  and latency targets (ADR 0045–0049).
- Durability: acknowledgment after flush to stable storage, the write-ahead log and recovery, checks at start and on
  read, the "read-only" and "storage error" states, a single owner, the clean-shutdown marker and version generations
  (ADR 0050–0054).
- Snapshots and backup: one writer and readers on snapshots, the retention timeout and ceilings, a backup from a snapshot, start from backup,
  a blocking rebuild of derived indexes on format change (ADR 0055–0059).
- The seams between subtasks are reconciled (`design.md`, "Subtask reconciliation"): the spill threshold, the place and representation of versions, the rebuild estimate,
  the page layout and the WAL, retention under snapshots; ADR 0040–0059 and the delta-specs are fixed point by point.
- New verified facts F-52…F-85 in `facts.md`; 22 questions closed, 4 narrowed, new Q-110…Q-125 in `docs/open-questions.md`;
  17 defaults (D-20…D-36) await the owner as a package at the end of the area.
- Nothing BREAKING: v1 has not been released. Client-visible novelties awaiting the owner's decision: word search without a timeout and a new error
  (D-24), the snapshot timeout and stream revocation (D-30), classifying `ENOSPC` as "read-only" (D-34), a blocking rebuild on format change (D-26).

## Capabilities

### New Capabilities

- `storage-format`: the database directory, the header and format version, the page and its header, keys and values of any length, record
  and number version encoding, checksums, free pages, file growth and size.
- `storage-indexes`: indexes as derived data of records, search by number, the number version from a snapshot, order and cursor,
  exact search, word-prefix search without a search timeout, the format label and rebuild, size and latency targets.
- `storage-durability`: write acknowledgment, flush to stable storage, the WAL and recovery, the integrity check, the "read-only"
  and "storage error" states, the database owner, clean shutdown, version generations.
- `storage-snapshots`: one writer and readers on snapshots, the retention timeout and cap, backup and start from backup,
  the rebuild of derived indexes on format change.

### Modified Capabilities

None: the requirements of area 1 and `user-journeys` do not change. The numeric placeholders of CUJ1, CUJ5, CUJ15, X-1 (Q-47, Q-58, Q-59, Q-104)
are replaced after the owner answers the [DEFAULT] package (a separate change at archiving).

## Impact

- ADR 0040–0059 (new, 20); they build on ADR 0013–0017, 0019, 0022, 0027–0029, 0035, 0039; the index is `docs/adr/README.md`.
- Requirements for areas 3–5 and for the build are collected in `design.md`, "Requirements for other areas".
- Documents: `facts.md` (F-52…F-85), `docs/open-questions.md` (22 closed, 4 narrowed, 16 new, 17 [DEFAULT]), `docs/adr/README.md`;
  calculations and measurements are in `research/{file-format,indexes,durability,snapshots}/`.
- No code: the change defines requirements, not implementation.
