## MODIFIED Requirements

### Requirement: Checkpoints bound the WAL
The system SHALL perform a checkpoint when the WAL volume from `redo_start_lsn` is at least 256 MiB, after 5 minutes when there are
unflushed pages, and on clean shutdown; the WAL volume that recovery must apply SHALL NOT exceed 256 MiB
plus the current batch. A checkpoint page-write error (including `ENOSPC` on a previously extended file: growth does not reserve blocks) SHALL move the database to "read-only" or "storage error" by `errno`; the WAL is not truncated in that case. Trace: X-1, CUJ14; ADR 0051 item 4, ADR 0052 item 3, ADR 0095; Q-42.

#### Scenario: By volume
- **WHEN** the WAL since the last checkpoint reached 256 MiB
- **THEN** a checkpoint starts; after it, segments before the new `redo_start_lsn` are freed

#### Scenario: Idle
- **WHEN** 5 minutes passed with changes and without reaching 256 MiB
- **THEN** a checkpoint is performed

#### Scenario: Checkpoint failed
- **WHEN** the checkpoint page write ended with `ENOSPC`
- **THEN** the state is "read-only" (with `EIO` — "storage error"); acknowledged operations are preserved in the WAL, new ones are rejected, the WAL is not truncated

### Requirement: Consistency check at start
Before accepting requests, the system SHALL check: the `control` slots and format; the database identifier in all files; the doublewrite
buffer pages; the WAL from `redo_start_lsn` (every entry); the pages read during WAL replay (except an all-zero page that a WAL entry initializes in full: it is accepted as empty both inside the file and beyond its end); the service page, index roots and placement metadata.
The check volume SHALL NOT depend on the number of records. Trace: CUJ14; ADR 0014 item 5, ADR 0053 item 1; Q-43.

#### Scenario: The check does not walk records
- **WHEN** the server starts on a database of 10^7 records
- **THEN** the number of pages read is bounded by the number of indexes and the pages touched by the WAL

#### Scenario: Ordinary crash after file growth
- **GIVEN** the file grew by a chunk after the last checkpoint, and the process was killed before the allocated pages were written
- **WHEN** the server starts and applies the WAL
- **THEN** zero pages initialized by WAL entries are accepted as empty, the database starts (CUJ14)

#### Scenario: Database identifier mismatch
- **WHEN** the database directory holds a `wal/` or `dwb` from a different database
- **THEN** the server does not start and names the file

### Requirement: "Read-only" state
The system SHALL move to the "read-only" state when a write, an open for writing, a segment creation, a file extension or a rename ended with the error `EROFS`, `EACCES`,
`EPERM`, `ENOSPC`, `EDQUOT`; the operation is not acknowledged and not published, all searches work. `retain` temporary file errors do not lead to "read-only" (they only revoke snapshots, `storage-snapshots`). The Q-20 refinement was accepted by the owner (D-34, OD-6): other write and read errors and a failed
flush — "storage error". Trace: CUJ16; ADR 0053 item 5; Q-71, Q-20.

#### Scenario: Full disk
- **WHEN** creating a new WAL segment ended with `ENOSPC`
- **THEN** the operation is not acknowledged, data is unchanged, the state is "read-only", searches work

#### Scenario: Boundary with "storage error"
- **WHEN** a WAL write ended with `ENOSPC`, and in another case with `EIO`
- **THEN** in the first case "read-only" (nothing is lost, the remedy depends on the operator), in the second "storage error" (CUJ16, "Disk write error", in the Q-20 refinement, D-34)

#### Scenario: Start on files not writable
- **WHEN** the database files are not writable and the previous shutdown was clean
- **THEN** the server starts in the "read-only" state, all searches work

#### Scenario: Crash shutdown and files not writable
- **WHEN** the files are not writable and the previous shutdown was a crash
- **THEN** the server does not start and reports the reason (recovery requires writing)

#### Scenario: Start from a backup and files not writable
- **WHEN** a directory with `run_state = backup` (a backup) is not writable, for example an archive on a read-only volume
- **THEN** the server does not start and reports the reason: a start from a backup requires writing (generation change, clearing the marker; ADR 0053 item 5); a backup is not started "read-only" on the earlier generation

### Requirement: "Storage error" state
The system SHALL move to the "storage error" state on a failed flush, a read or write error outside the "read-only" list (including `EIO`; except the `retain` temporary file: its errors only revoke snapshots), a corrupted page,
a violated structure invariant, and unavailability of the generation counter location or the randomness source; changes are rejected, search works while it can; the operator gets a message
with the source, `errno` and location. "Storage error" SHALL be stronger than "read-only". Trace: CUJ16, CUJ14; ADR 0014 item 6, ADR 0053 items 6–7; Q-71.

#### Scenario: Disk write error
- **WHEN** a WAL write ended with `EIO`
- **THEN** the operation is not acknowledged; there is no partial change; the state is "storage error"

#### Scenario: State in memory
- **WHEN** the server is in "storage error" and is restarted
- **THEN** the state is determined by the start checks, not by a stored value

#### Scenario: Reproducibility
- **WHEN** a test injects a failure with a given `errno` on a given file operation
- **THEN** the state given by the reason table is observed, without a real disk overflow

### Requirement: Leaving the states [DEFAULT]
The system SHALL leave "storage error" only on restart, and "read-only" on its own, by a probe write; if the cause is a checkpoint failure, the probe is a repeated checkpoint, and a successful WAL write does not give an exit. The state SHALL be checked before the operation checks (`write-operations`,
"Check order on write"); in "read-only" a write operation SHALL begin with a probe — a durable service WAL entry: if it fails — refusal "read-only", if it succeeds — the state
is normal, an operator message, the operation's usual path. A default (D-29). Trace: CUJ16; ADR 0053 item 8; Q-71.

#### Scenario: Space freed
- **GIVEN** the server is in "read-only" because of `ENOSPC`
- **WHEN** the operator freed space and the user retried the save
- **THEN** the probe succeeded, the save is performed and acknowledged, the state is normal, the operator gets a message

#### Scenario: State before operation checks
- **GIVEN** the server is in "read-only", space is not freed
- **WHEN** a rename with a stale version arrives, and then an identical retry of a previously acknowledged add
- **THEN** both get "read-only", not "version conflict" and not "unchanged"; data and state are unchanged

#### Scenario: Probe succeeded, operation rejected by a check
- **GIVEN** the server is in "read-only", space is freed
- **WHEN** a rename with a stale version arrives
- **THEN** the probe succeeded, the state is normal, the operator gets a message, and the operation gets "version conflict" by the usual check order

#### Scenario: Read-only because of a checkpoint
- **GIVEN** the "read-only" state was caused by a checkpoint failure (`ENOSPC`), WAL writes are still possible
- **WHEN** a write operation arrives and space is not freed
- **THEN** the probe — a repeated checkpoint — failed: refusal "read-only", the WAL does not grow beyond the cap; after a successful checkpoint the state is normal

#### Scenario: Storage error persists
- **GIVEN** the server is in "storage error"
- **WHEN** the cause is removed but the server is not restarted
- **THEN** changes are still rejected, there are no probes or background polling; when starting in "read-only" the probe includes writing `run_state = running`

#### Scenario: Retry after a failed write
- **WHEN** a WAL write failed with `ENOSPC`, then the probe and a new write succeeded
- **THEN** the new write is placed from the previous durable LSN; after startup the WAL is read without error

### Requirement: Version does not require a number scan on generation change
The system SHALL store for a number only an ordinal value S (u64; a varint in the number header, at most 5 bytes for S < 2^35) from a counter shared by all numbers, restore `next_S` from the WAL and `control`, and change the version generation
without changing records and without scanning numbers. Trace: X-5, X-1; ADR 0028 item 6 (d, f), ADR 0051 item 7; Q-78.

#### Scenario: Generation change on 10^7 numbers
- **WHEN** a crash recovery start on a database of 10^7 numbers
- **THEN** no number's record is read or changed for the generation change

#### Scenario: Counter after a crash
- **WHEN** the server is restarted after a crash
- **THEN** `next_S` is greater than the largest S among the headers and not lower than the largest `next_S` among the applied WAL entries and the value in `control` (each of them is already greater than the issued S), so a version is never repeated

## ADDED Requirements

### Requirement: Layout of control, dwb and WAL
The system SHALL keep state in `control` (two 4 KiB slots with a layout label, slot number, `db_id`, ADR 0040 item 1 fields and CRC-32C), `dwb` (page slots with `db_id`) and `wal/` segments (a header with `db_id`, segment number and starting LSN; entries per ADR 0051 item 2). A layout label newer than supported means start refusal; v1 labels are read by all v1 programs. Trace: CUJ11, CUJ14, CUJ15; ADR 0040 item 1, ADR 0051 item 2, ADR 0095.

#### Scenario: `control` slots
- **WHEN** both `control` slots are valid
- **THEN** the slot with the larger number is in effect; if only one is valid, it is used

#### Scenario: WAL segment of another database
- **WHEN** the `db_id` in a `wal/` segment header is not equal to the `db_id` in `control`
- **THEN** the server does not start and names the file; a segment filled with zeros is not this database's WAL

#### Scenario: WAL of an earlier v1 version
- **GIVEN** a v1 server was crash-stopped
- **WHEN** a new v1 program with a newer WAL layout label starts
- **THEN** a WAL with the old label is read and applied (CUJ11, CUJ14)
