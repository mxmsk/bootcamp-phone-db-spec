# storage-durability Specification

## Purpose
When a write is considered acknowledged and how the acknowledged survives a process crash, an OS failure and power loss; indivisibility
of an operation and of the version; recovery after a failure and start time; integrity checks at start and on read; the "read-only"
and "storage error" states; the clean-shutdown marker, version generations and the single owner of the database. Mechanics — ADR 0050–0054.

## Requirements

### Requirement: Acknowledgement only after flush to stable storage
The system SHALL return success of a write operation (including an identical add retry) only after its WAL entry has been flushed to stable storage without error. Operation checks
SHALL go by the latest writer state, including the unflushed; a response that depends on an unflushed operation SHALL wait for its flush, and if the flush fails be replaced by a batch refusal.
Trace: CUJ14, CUJ1, CUJ3, CUJ6, CUJ9, CUJ12; ADR 0050 items 1–2; Q-57.

#### Scenario: Response after the flush
- **WHEN** a write operation is executed and its WAL entry has not yet been flushed to stable storage
- **THEN** the client did not get success; success is returned after the flush returns without error

#### Scenario: Group acknowledgment
- **WHEN** several operations are executed at the same time and their entries are flushed by one flush
- **THEN** each gets success after the success of this flush, and on a flush error all operations of the batch get "storage error"

#### Scenario: An identical retry acknowledges the durable
- **GIVEN** an add of a record was executed but not yet flushed
- **WHEN** an identical add retry arrives
- **THEN** the writer sees the unflushed record and does not create a second one, does not change the version; the "unchanged" response is returned after the flush of the batch with the first
  add, and if the flush fails the retry gets the same refusal as the add

#### Scenario: Two writes against one version while the first is unflushed
- **GIVEN** a number rename was executed and not flushed, and a second rename of the same number arrives with the earlier expected version
- **WHEN** the writer executes the second
- **THEN** "version conflict" with the version of the unflushed operation; the response is returned after the flush, the data of the second operation is not written

### Requirement: Acknowledged data survives process crash, OS failure and power loss
The system SHALL preserve every acknowledged write operation (add, rename, delete) through a process crash, an OS
failure and power loss, if the disk honestly performs the flush to stable storage; a record acknowledged as deleted SHALL NOT reappear.
A "lying" disk and media corruption are outside the guarantee. Trace: CUJ14; ADR 0014 items 1–3, ADR 0050; Q-16, Q-20.

#### Scenario: Process crash after acknowledgment
- **WHEN** the client received the add acknowledgment and the process was killed by SIGKILL
- **THEN** after the start the record is found by search by number and by word prefix of the name

#### Scenario: Power loss, the disk does not keep the unflushed
- **WHEN** a test simulates power loss: everything written but not flushed is discarded; a rename and a delete of another record were acknowledged
- **THEN** after the start the rename is in place, the deleted record is absent

#### Scenario: Boundary of the guarantee
- **WHEN** the disk reported a successful flush without performing it
- **THEN** durability is not promised, detection of such a disk is not promised (ADR 0014 item 3)

### Requirement: What counts as a flush to stable storage
The system SHALL count as a flush a successful return of `fdatasync` or `fsync` on Linux, and a successful return of `fcntl(F_FULLFSYNC)` on macOS;
a plain `fsync` on macOS SHALL NOT count as a flush. File creation and renaming SHALL be durable only after the file and the directory are flushed.
Trace: CUJ14; ADR 0050 items 2–3; F-64…F-70.

#### Scenario: macOS
- **WHEN** the server flushes the WAL on macOS
- **THEN** `F_FULLFSYNC` is called (through `File::sync_data`/`sync_all`), not only `fsync`

#### Scenario: New WAL segment
- **WHEN** a new WAL segment is created
- **THEN** it is filled with zeros and flushed together with its directory before the first operation is written into it

### Requirement: All or nothing for an unacknowledged operation
After a failure the system SHALL leave an unacknowledged write operation either whole (record, index derivatives and version) or
absent; there SHALL NOT be a partial operation. Trace: CUJ14, X-5; ADR 0029 item 1, ADR 0051 item 2.

#### Scenario: Interruption in the middle of a WAL write
- **WHEN** the process is killed while a WAL entry is partially written
- **THEN** after the start the operation is absent, the earlier record and version are unchanged

#### Scenario: Order on the medium is violated inside an unflushed batch
- **WHEN** the medium kept a later entry of the batch and not an earlier one, nothing from the batch was acknowledged
- **THEN** after the start only the entries up to the first invalid one are applied; the later entry of the batch is not applied (including at subsequent starts)

### Requirement: Record and number version are one WAL entry
The system SHALL write the change of the number record, of all derived indexes and the new number version value in one WAL
entry, so that "record exists, no version" and the reverse are impossible. Trace: CUJ14, X-5; ADR 0028 item 6 (a), ADR 0051 items 2, 7.

#### Scenario: Failure after acknowledgment
- **WHEN** a rename was acknowledged, the process was killed and started
- **THEN** the changed name and the new number version are present together

#### Scenario: Indexes are consistent
- **WHEN** after a crash recovery start an acknowledged record is searched by number, by name word prefix and by whole name
- **THEN** the record is found by all three searches

### Requirement: Readers see only durable state
The system SHALL publish a change for search only after it has been flushed to stable storage; a search started after the response to the client SHALL see
the result of the operation in full, and a concurrent one — the state before or after in full. Trace: CUJ1, CUJ6, CUJ12, CUJ14; ADR 0029 item 4, ADR 0050 item 6.

#### Scenario: Search during a flush
- **WHEN** a search runs while the operation's WAL entry has not yet been flushed
- **THEN** the operation is not visible in the result

#### Scenario: A flush error leaves no trace for readers
- **WHEN** a WAL flush ended with an error
- **THEN** no search showed the changes of this batch and none will

### Requirement: WAL before data, protection of pages from torn writes
The system SHALL write a page to the data file only after the WAL entries with its changes are durable, and SHALL protect
every write of checkpoint pages with the doublewrite buffer, so that a torn page write does not corrupt the database. Trace: CUJ14; ADR 0051 items 3–4.

#### Scenario: Torn page write during a checkpoint
- **WHEN** the process is killed (or power is lost) at the moment a page is being written to the data file
- **THEN** at start the page is restored from the doublewrite buffer and the WAL is applied; acknowledged operations are present

#### Scenario: Page with an LSN above the durable one
- **WHEN** a page was modified by an operation whose WAL entry has not yet been flushed
- **THEN** the checkpoint does not write this version of the page to the data file

### Requirement: Checkpoints bound the WAL
The system SHALL perform a checkpoint when the WAL volume from `redo_start_lsn` is at least 256 MiB, after 5 minutes with unflushed pages, at clean shutdown and at the modified-page cap ("Memory for modified pages is bounded"); during a backup checkpoints are suspended. The WAL to be recovered is ≤ 256 MiB plus a batch. A checkpoint page write error is "read-only" or "storage error" by `errno`, the WAL is not truncated. Trace: X-1, CUJ14, CUJ15; ADR 0051 item 4, 0052 item 3, 0095–0097; Q-42.

#### Scenario: By volume
- **WHEN** the WAL since the last checkpoint reached 256 MiB
- **THEN** a checkpoint starts; after it the segments before the new `redo_start_lsn` are freed

#### Scenario: Idle
- **WHEN** 5 minutes passed with changes and without reaching 256 MiB
- **THEN** a checkpoint is performed

#### Scenario: Checkpoint failed
- **WHEN** the checkpoint page write ended with `ENOSPC`
- **THEN** the state is "read-only" (with `EIO` — "storage error"); acknowledged operations are kept in the WAL, new ones are rejected, the WAL is not truncated

#### Scenario: Modified-page cap
- **WHEN** modified pages in memory reached the cap with a WAL below 256 MiB
- **THEN** a checkpoint starts

#### Scenario: Backup
- **WHEN** a backup is in progress
- **THEN** a new checkpoint does not start until the backup ends; one in progress at the moment of the command finishes before the backup commit number is fixed

### Requirement: Crash recovery by WAL replay
The system SHALL, on a start after a failure, apply the WAL from `redo_start_lsn` in order to pages whose LSN is lower than the entry's LSN, discard
the torn tail and write "end of recovery" with a new WAL epoch before accepting operations; an entry of an old epoch at the required
offset SHALL NOT be applied. Trace: CUJ14; ADR 0051 items 5–6.

#### Scenario: Replay is idempotent
- **WHEN** the process is killed during recovery and started again
- **THEN** the result is the same as with a single application

#### Scenario: Ghost of a torn batch
- **GIVEN** in the previous run a later entry of a batch ended up on the medium and an earlier one did not, and after recovery less was written than the batch occupied
- **WHEN** the server is crash-stopped right after "end of recovery" and started again
- **THEN** the ghost entry is not applied: its epoch is lower than the epoch of the "end of recovery" entry

### Requirement: Torn tail is discarded, mid-WAL corruption is a refusal
The system SHALL treat an invalid WAL entry as a torn tail if no valid entry after it has a `flushed_lsn`
greater than its position, and discard the tail, reporting the number of bytes to the operator; otherwise it SHALL treat this as WAL corruption and not open the database. Trace: CUJ14; ADR 0051 item 6, ADR 0053 items 2–3.

#### Scenario: Torn tail
- **WHEN** the last batch was written incompletely and nothing from it was acknowledged
- **THEN** the server starts, the tail is discarded, the operator is told the number of discarded bytes

#### Scenario: Corruption of an acknowledged entry in the middle
- **WHEN** a byte is changed in the WAL in an entry followed by entries acknowledged later
- **THEN** the server does not start, reports the reason and position, entries are not discarded, files are not changed

#### Scenario: Detectability boundary
- **WHEN** the last acknowledged WAL entry is corrupted and there is nothing after it
- **THEN** detection is not promised: it is indistinguishable from a torn tail (media corruption, ADR 0014 item 3)

### Requirement: A failed flush is not retried
The system SHALL, on any `fsync`, `fdatasync`, `F_FULLFSYNC` error, not acknowledge the batch operations, not retry the flush on the same file
and go to the "storage error" state. Trace: CUJ16, CUJ14; ADR 0050 item 5; F-71, F-72.

#### Scenario: WAL flush error
- **WHEN** the WAL flush returned `EIO`
- **THEN** the batch operations get "storage error", there is no repeated flush, the state is "storage error", the operator gets a message

#### Scenario: What is on the disk is decided by the start
- **WHEN** the server is restarted after such an error
- **THEN** the batch entry is present in full or absent in full by the WAL checksums

### Requirement: Batch failure rolls back the writer's unpublished state
The system SHALL, when a batch write or flush fails, discard all unpublished writer state (page versions, allocated and freed pages, `next_S`, commit number) and
reject with the same refusal all operations executed after the first failed one. The next accepted operation SHALL be built on the last published state and written from the last
durable LSN. Trace: CUJ14, CUJ16, X-5; ADR 0051 item 9; ADR 0053 item 8.

#### Scenario: Operations on top of a failed one
- **GIVEN** operation A was executed in the writer's memory, then operation B over the same pages, and the batch write with A ended with `ENOSPC`
- **WHEN** the writer rolls back the state
- **THEN** A and B are rejected with the "read-only" refusal, readers see the state before A, the next accepted operation C is executed on the state before A

#### Scenario: Allocations and counter returned
- **GIVEN** the failed operation took pages from the free list and advanced `next_S`
- **WHEN** the state is rolled back
- **THEN** these pages are in the free list again, `next_S` is as before, the next operation gets the same pages and the same `next_S`

#### Scenario: Checkpoint and recovery agree
- **GIVEN** after the failed operation, operation C over the same pages was executed and acknowledged
- **WHEN** a checkpoint is performed, then the server is killed and recovered from the WAL
- **THEN** the page contents after recovery equal the contents written by the checkpoint and contain no trace of the rejected operation

#### Scenario: Flush error
- **WHEN** the WAL flush ended with `EIO`
- **THEN** the unpublished state is discarded, the batch operations and those executed after them got "storage error", the writer accepts no new operations until a restart

### Requirement: Start time at 10^7 records
The system SHALL start on a database of 10^7 records, including the start check, in no more than 5 s after a clean shutdown and no more than 30 s after a crash shutdown or on a backup, without scanning
records and rebuilding indexes; a start with an index rebuild or a format conversion is not included in these figures. The values were accepted by the owner (OD-6, D-28; they do not depend on the number of records, so they are the earlier ones). Trace: CUJ14, X-1; ADR 0052 item 2; Q-42; OD-6.

#### Scenario: After a clean shutdown
- **WHEN** the server starts on a database of 10^7 records after a clean shutdown
- **THEN** it is ready to accept requests no later than 5 s

#### Scenario: After a crash shutdown
- **WHEN** the server starts after SIGKILL with a 256 MiB WAL since the last checkpoint
- **THEN** it is ready to accept requests no later than 30 s

#### Scenario: Start with a rebuild or format conversion
- **WHEN** a server of a new v1 version starts on a database of 10^7 records after a clean shutdown, and the index label or the format version differs
- **THEN** the 5 s and 30 s targets do not apply to this start; a rebuild or conversion is performed before accepting requests; the rebuild time is no more than 6 minutes (`storage-indexes`, "Index format label and rebuild at start")

### Requirement: Consistency check at start
The system SHALL check before accepting requests: the `control` slots and the format; the database identifier in all files; the doublewrite buffer
pages; the WAL from `redo_start_lsn` (every entry); the pages read while applying the WAL (except a page of all zeros that a WAL entry initializes fully: it is accepted as empty both inside the file and beyond its end); the service page, the index roots and the placement metadata.
The scope of the check SHALL NOT depend on the number of records. Trace: CUJ14; ADR 0014 item 5, ADR 0053 item 1; Q-43.

#### Scenario: The check does not scan records
- **WHEN** the server starts on a database of 10^7 records
- **THEN** the number of pages read is bounded by the number of indexes and the pages touched by the WAL

#### Scenario: Ordinary failure after file growth
- **GIVEN** the file grew by a chunk after the last checkpoint, and the process was killed before the allocated pages were written
- **WHEN** the server starts and applies the WAL
- **THEN** zero pages initialized by WAL entries are accepted as empty, the database starts (CUJ14)

#### Scenario: Database identifier mismatch
- **WHEN** the database directory contains `wal/` or `dwb` from another database
- **THEN** the server does not start and names the file

### Requirement: Inconsistent database is not opened
The system SHALL not start, not modify `data` and the WAL (except restoring a torn page from the doublewrite buffer) and report to the operator the reason, the file and
the position if the start check found an inconsistency: no valid `control` slot, format newer than the program, a page without a valid copy, mid-WAL corruption,
an invalid root page. Trace: CUJ14; ADR 0014 item 4, ADR 0053 item 2.

#### Scenario: Both control slots invalid
- **WHEN** both `control` slots fail the checksum check
- **THEN** the server does not start, reporting «нет верного управляющего файла» ("no valid control file"); entries are not discarded

#### Scenario: Index root page
- **WHEN** the checksum of an index root page is invalid
- **THEN** the server does not start, the message names the index and the page

#### Scenario: Format newer than the program
- **WHEN** `format_version` in META (ADR 0040 item 4) or the `control` layout label is higher than supported
- **THEN** the server does not start with this reason; the database is not changed

#### Scenario: Restore from a backup
- **WHEN** the operator starts the server on a backup after a refusal
- **THEN** the server starts on the backup data (CUJ14, CUJ15)

### Requirement: Page check on read
The system SHALL check every page read from the file into the cache (CRC-32C, page number, type); on a mismatch or read error mark the page
as corrupted, go to "storage error" and refuse the searches that need it; other searches SHALL be executed. Trace: CUJ14, CUJ16; ADR 0014 item 5, ADR 0053 item 4; Q-14.

#### Scenario: Corruption on read
- **WHEN** a search reads a page with an invalid checksum
- **THEN** the search refuses with "storage error", the server is in the "storage error" state, the page is reported to the operator

#### Scenario: Search bypassing the corrupted page
- **GIVEN** the server is in the "storage error" state because of a corrupted page
- **WHEN** a search that does not touch it is executed
- **THEN** it returns a result

#### Scenario: Repeated access
- **WHEN** another search touches the same corrupted page
- **THEN** it refuses without re-reading the file

### Requirement: "Read-only" state
The system SHALL go to the "read-only" state when a write, an open for writing, a segment creation, a file extension or a rename ended with the error `EROFS`, `EACCES`,
`EPERM`, `ENOSPC`, `EDQUOT`; the operation is not acknowledged and not published, all searches work. Errors of the `retain` temporary file do not lead to "read-only" (they only revoke snapshots, `storage-snapshots`). The clarification of Q-20 was accepted by the owner (D-34, OD-6): other write and read errors and a failed
flush — "storage error". Trace: CUJ16; ADR 0053 item 5; Q-71, Q-20.

#### Scenario: Full disk
- **WHEN** creating a new WAL segment ended with `ENOSPC`
- **THEN** the operation is not acknowledged, data is not changed, the state is "read-only", searches work

#### Scenario: Boundary with "storage error"
- **WHEN** a WAL write ended with `ENOSPC`, and in another case — with `EIO`
- **THEN** in the first case "read-only" (nothing was lost, the fix is up to the operator), in the second "storage error" (CUJ16, "Disk write error", in the clarification of Q-20, D-34)

#### Scenario: Start on files that are not writable
- **WHEN** the database files are not writable and the previous stop was clean
- **THEN** the server starts in the "read-only" state, all searches work

#### Scenario: Crash shutdown and files not writable
- **WHEN** the files are not writable and the previous stop was a crash shutdown
- **THEN** the server does not start, reports the reason (recovery requires writing)

#### Scenario: Start from backup and files not writable
- **WHEN** a directory with `run_state = backup` (a backup) is not writable, for example an archive on a read-only volume
- **THEN** the server does not start and reports the reason: starting from a backup requires writing (generation change, clearing the marker; ADR 0053 item 5); a backup is not started in "read-only" on the earlier generation

### Requirement: "Storage error" state
The system SHALL go to the "storage error" state on a failed flush, a read or write error outside the "read-only" list (including `EIO`; except the `retain` temporary file: its errors only revoke snapshots), page corruption,
a structure invariant violation and unavailability of the generation counter location or the randomness source; changes are rejected, search works as long as it can; the operator gets a message
with the source, `errno` and place. "Storage error" SHALL be stronger than "read-only". Trace: CUJ16, CUJ14; ADR 0014 item 6, ADR 0053 items 6–7; Q-71.

#### Scenario: Disk write error
- **WHEN** a WAL write ended with `EIO`
- **THEN** the operation is not acknowledged; there is no partial change; the state is "storage error"

#### Scenario: State in memory
- **WHEN** the server is in "storage error" and is restarted
- **THEN** the state is determined by the start checks, not by a stored value

#### Scenario: Reproducibility
- **WHEN** a test injects a failure with a given `errno` on a given file operation
- **THEN** the state set by the cause table is observed, without a real disk overflow

### Requirement: Leaving the states [DEFAULT]
The system SHALL leave "storage error" only on a restart, and "read-only" by itself, with a probe write; if the cause is a checkpoint failure, the probe is a repeated checkpoint, and a successful WAL write does not give an exit. The state SHALL be checked before the operation checks (`write-operations`,
"Check order on write"); in "read-only" a write operation SHALL start with a probe — a durable service WAL entry: if it fails — the "read-only" refusal, if it succeeds — the state is
normal, an operator message, the ordinary operation path. A default (D-29). Trace: CUJ16; ADR 0053 item 8; Q-71.

#### Scenario: Space freed
- **GIVEN** the server is in the "read-only" state because of `ENOSPC`
- **WHEN** the operator freed space and the user repeated the save
- **THEN** the probe succeeded, the save is executed and acknowledged, the state is normal, the operator gets a message

#### Scenario: State before the operation checks
- **GIVEN** the server is in the "read-only" state, space is not freed
- **WHEN** a rename with a stale version arrives, and then an identical retry of a previously acknowledged add
- **THEN** both get "read-only", not "version conflict" and not "unchanged"; data and state are not changed

#### Scenario: Probe succeeded, operation rejected by a check
- **GIVEN** the server is in the "read-only" state, space is freed
- **WHEN** a rename with a stale version arrives
- **THEN** the probe succeeded, the state is normal, the operator gets a message, and the operation gets "version conflict" by the ordinary check order

#### Scenario: Read-only because of a checkpoint
- **GIVEN** the "read-only" state is caused by a checkpoint failure (`ENOSPC`), writing to the WAL is still possible
- **WHEN** a write operation arrives and space is not freed
- **THEN** the probe — a repeated checkpoint — failed: the "read-only" refusal, the WAL does not grow beyond the cap; after a successful checkpoint the state is normal

#### Scenario: Storage error persists
- **GIVEN** the server is in the "storage error" state
- **WHEN** the cause is eliminated but the server is not restarted
- **THEN** changes are still rejected, there are no probes or background polling; on a start in "read-only" the probe includes writing `run_state = running`

#### Scenario: Retry after a failed write
- **WHEN** a WAL write failed with `ENOSPC`, then the probe and a new write succeeded
- **THEN** the new write lands from the earlier durable LSN; after a start the WAL is read without error

### Requirement: Single owner of the database
The system SHALL take an exclusive non-blocking lock on the `LOCK` file of the database directory as the first action at start and hold it until the process ends; if the lock is held, the start SHALL
be refused with the message «база уже открыта другим процессом» ("database is already open by another process"), and any other lock error — with a refusal giving its reason. Trace: X-2, CUJ13; ADR 0015 item 3, ADR 0054 item 1.

#### Scenario: Second process
- **WHEN** a second server process opens the same database while the first is alive
- **THEN** the second does not start, the message «база уже открыта другим процессом» ("database is already open by another process"), data is not changed

#### Scenario: First one killed
- **WHEN** the first process is killed by SIGKILL and a new one is started
- **THEN** the lock is free and the new process starts (crash recovery start)

#### Scenario: Lock before reading
- **WHEN** the server starts
- **THEN** before the lock no other database file is read or changed

### Requirement: Database creation does not overwrite an existing one
The system SHALL create a database so that `control` is created last and with overwriting forbidden, and on a directory with an existing `control` or with the lock held,
the create command refuses without changing the database. A server on a directory without `control` SHALL NOT start. Trace: CUJ13, X-2; ADR 0015 items 1–2, ADR 0054 item 2.

#### Scenario: Repeated creation
- **WHEN** the create command is run on a directory with a database
- **THEN** refusal, files are not changed

#### Scenario: Interrupted creation
- **WHEN** creation is interrupted before `control` is written
- **THEN** the directory is not a database: the server does not start, repeated creation is allowed

### Requirement: Clean-shutdown marker and clean shutdown
The system SHALL write the clean-shutdown marker to `control` as the last step of a clean shutdown, after acknowledging or rejecting every accepted operation, flushing the WAL and a full
checkpoint; termination before this step SHALL leave the marker unwritten (crash shutdown), and the process on a clean shutdown SHALL exit with code 0. Trace: CUJ14, CUJ15; ADR 0028 item 10, ADR 0054 item 8; Q-108.

#### Scenario: Clean shutdown
- **WHEN** the server receives SIGTERM with a write in flight
- **THEN** the operation is acknowledged or rejected, the checkpoint is performed, the marker is written, code 0

#### Scenario: Shutdown deadline expired
- **WHEN** the process is forcibly terminated before the marker is written
- **THEN** there is no marker, the next start is a crash recovery start

### Requirement: Start kind is determined by the marker on the medium
The system SHALL determine the start kind by `run_state` in `control`: `clean` — planned, `running` — crash recovery, `backup` — start from backup; before accepting the first operation after the start,
`run_state = running` SHALL be written durably. For files copied not by the command, the start kind is determined by the same marker. Trace: CUJ14, CUJ15; ADR 0028 items 7, 10, ADR 0054 items 4, 6.

#### Scenario: The marker is cleared before the first operation
- **WHEN** the server was started after a clean shutdown, accepted the first operation and crashed
- **THEN** the next start is a crash recovery start

#### Scenario: Start from backup
- **WHEN** the server starts on a backup with the `backup` marker
- **THEN** this is a start from backup; the marker is cleared in the running database, and remains in the backup archive

### Requirement: Version generation: counter outside the backup and random epoch
The system SHALL, on a crash recovery start and a start from backup, before accepting operations durably write the generation counter `G = max(G_s, G_run) + 1` to the `generation` file next to the settings (outside the backup), choose the epoch `E`
from 16 bytes of the OS randomness source and durably write `G`, `E` to `control`; the observable number version SHALL be (G, E, S), the initial one — (G, E, 0). On a planned start
`G` and `E` SHALL be the earlier ones. Trace: CUJ14, CUJ15, X-5; ADR 0028 items 8–9, ADR 0054 items 5–6.

#### Scenario: Crash recovery start
- **WHEN** the server is restarted after a crash shutdown
- **THEN** `G` is greater than before, `E` is new; the number version read before the failure does not match the current one

#### Scenario: Two starts from one archive
- **WHEN** a backup is brought up twice on one installation
- **THEN** `G` differs between the starts; a rename by the version of the first start in the second is a "version conflict"

#### Scenario: Settings rollback
- **WHEN** `G_s` in the settings is less than `G_run` in the database
- **THEN** the new `G` is greater than `G_run`, the versions do not match the earlier ones

#### Scenario: Planned start
- **WHEN** the server is started after a clean shutdown
- **THEN** `G`, `E` are the same, issued versions are valid, the `generation` file is neither read nor written

### Requirement: Unavailable counter location or randomness source
The system SHALL, on a crash recovery start and a start from backup, if the settings directory is not writable, both `generation` slots are invalid, a flush failed or the randomness source returned
an error, not accept write operations, go to "storage error", not change `run_state` and execute search with versions of the earlier generation. Trace: CUJ16; ADR 0028 item 8, ADR 0054 item 7; Q-107.

#### Scenario: Read-only settings directory
- **WHEN** after a crash shutdown the settings directory is not writable
- **THEN** changes are rejected ("storage error"), search works, `run_state` is not changed

#### Scenario: Randomness source error
- **WHEN** an injected randomness source returns an error on a crash recovery start
- **THEN** the same: changes are rejected, search works, no version of the new generation is issued

#### Scenario: Exit
- **WHEN** the cause is eliminated and the server is restarted
- **THEN** an ordinary crash recovery start with a new generation is performed

### Requirement: Substituting the randomness source and counter storage in tests
The system SHALL obtain randomness and access to `generation` through injectable interfaces; tests with a deterministic source check the growth of `G` and the change of `E` on a crash recovery start and a start from backup,
the invariance on a planned one, the refusal on a source or storage error, and the capacity of `E` of at least 128 bits. Trace: CUJ14, CUJ15, X-5; ADR 0028 item 9, ADR 0054 item 10.

#### Scenario: Epoch capacity
- **WHEN** a test reads the epoch length
- **THEN** it is at least 128 bits

#### Scenario: Deterministic source
- **WHEN** a test substitutes the randomness source and the counter storage
- **THEN** the change of `G` and `E` is checked exactly, without touching the OS

### Requirement: Target write latency
The system SHALL acknowledge a write operation on a database of 10^7 records under a load of up to 100 operations per second with latency p50 ≤ 20 ms and p99 ≤ 100 ms
(with a stable-storage flush p99 ≤ 40 ms), and during a backup with p99 ≤ 250 ms and without refusals. The values were accepted by the owner (OD-6, D-27: the same figures at 10^7). Trace: CUJ1, CUJ15, X-1; ADR 0052 item 1; Q-58, Q-11; OD-6.

#### Scenario: Write latency at 10^7
- **WHEN** on a database of 10^7 records with a cold cache, writes are performed at up to 100 per second
- **THEN** p50 ≤ 20 ms and p99 ≤ 100 ms on the reference rig

#### Scenario: During a backup
- **WHEN** a backup is in progress
- **THEN** write operations are not rejected, p99 ≤ 250 ms

### Requirement: Version does not require a number scan on generation change
The system SHALL store for a number only the sequence value S (u64; in the number header a varint, at most 5 bytes at S < 2^35) from a counter shared by all numbers, restore `next_S` from the WAL and `control` and change the version generation
without changing records and without scanning numbers. Trace: X-5, X-1; ADR 0028 item 6 (d, f), ADR 0051 item 7; Q-78.

#### Scenario: Generation change on 10^7 numbers
- **WHEN** a crash recovery start on a database of 10^7 numbers
- **THEN** no number record is read or changed for the sake of the generation change

#### Scenario: Counter after a failure
- **WHEN** the server is restarted after a crash shutdown
- **THEN** `next_S` is greater than the largest S among the headers and no lower than the largest `next_S` among the applied WAL entries and the value in `control` (each of them is already greater than the issued S), so a version is never repeated

### Requirement: Layout of control, dwb and WAL
The system SHALL keep state in `control` (two 4 KiB slots with a layout label, a slot number, `db_id`, the fields of ADR 0040 item 1 and CRC-32C), `dwb` (page slots with `db_id`) and `wal/` segments (a header with `db_id`, the segment number and the starting LSN; entries per ADR 0051 item 2). A layout label newer than supported is a start refusal; v1 labels are read by all v1 programs. Trace: CUJ11, CUJ14, CUJ15; ADR 0040 item 1, ADR 0051 item 2, ADR 0095.

#### Scenario: Control slots
- **WHEN** both `control` slots are valid
- **THEN** the slot with the greater number is in effect; if only one is valid, it is used

#### Scenario: WAL segment of another database
- **WHEN** `db_id` in the `wal/` segment header is not equal to `db_id` in `control`
- **THEN** the server does not start and names the file; a segment filled with zeros is not a WAL of this database

#### Scenario: WAL of an earlier v1 version
- **GIVEN** a v1 server was crash-stopped
- **WHEN** a new v1 program with a newer WAL layout label is started
- **THEN** the WAL with the old label is read and applied (CUJ11, CUJ14)

### Requirement: Memory for modified pages is bounded
The volume of modified pages not yet written to `data` (together with version chains) SHALL be no more than 1 GiB (by default). When the cap is reached a checkpoint starts, and the writer waits for its end before the next operation: the operation is neither rejected nor lost. If the cap is reached during a backup, the backup is aborted and checkpoints resume. Trace: X-1, CUJ14, CUJ15; ADR 0097 (OD-7).

#### Scenario: Cap reached
- **GIVEN** modified pages occupied 1 GiB
- **WHEN** a write operation arrives
- **THEN** it waits for the end of the checkpoint, then is executed and acknowledged; there is no refusal

#### Scenario: Cap during a backup
- **GIVEN** a backup is in progress, checkpoints are suspended, and the cap is reached
- **WHEN** a write operation arrives
- **THEN** the backup is aborted, checkpoints resume, the operation waits for a checkpoint, then is acknowledged

#### Scenario: Write deadline while waiting
- **WHEN** the write deadline expired while the operation waited for a checkpoint before starting execution
- **THEN** "request timeout", data is not changed
