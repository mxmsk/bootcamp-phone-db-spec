# storage-durability

## Purpose
When a write counts as acknowledged and how the acknowledged survives a process crash, an OS failure and power loss; the indivisibility
of an operation and its version; recovery after a failure and start time; the integrity check at start and on read; the "read-only"
and "storage error" states; the clean-shutdown marker, version generations and the single database owner. The mechanics are ADR 0050–0054.

## ADDED Requirements

### Requirement: Acknowledgement only after flush to stable storage
The system SHALL return the success of a write operation (including an identical retry of an add) only after its WAL entry has been flushed to stable storage without error. The operation's checks
SHALL run against the writer's latest state, including the unflushed; a response that depends on an unflushed operation SHALL wait for its flush, and if the flush fails be replaced by a batch refusal.
Trace: CUJ14, CUJ1, CUJ3, CUJ6, CUJ9, CUJ12; ADR 0050 items 1–2; Q-57.

#### Scenario: Response after the flush
- **WHEN** a write operation has run and its WAL entry has not yet been flushed to stable storage
- **THEN** the client has not received success; success is returned after the flush returns without error

#### Scenario: Group acknowledgment
- **WHEN** several operations run simultaneously and their entries are flushed by one flush
- **THEN** each gets success after the success of that flush, and on a flush error all operations of the batch get "storage error"

#### Scenario: An identical retry acknowledges the durable
- **GIVEN** an add has run but is not yet flushed
- **WHEN** an identical retry of the add arrives
- **THEN** the writer sees the unflushed record and creates no second record, does not change the version; the "unchanged" response is returned after the flush of the batch with the first
  add, and if the flush fails the retry gets the same refusal as the add

#### Scenario: Two writes against one version with the first unflushed
- **GIVEN** a number rename has run and is not flushed, and a second rename of the same number arrives with the earlier expected version
- **WHEN** the writer runs the second
- **THEN** "version conflict" with the version of the unflushed operation; the response is returned after the flush, the second operation's data is not written

### Requirement: Acknowledged data survives process crash, OS failure and power loss
The system SHALL keep every acknowledged write operation (add, rename, delete) across a process crash, an OS
failure and power loss, if the disk honestly performs the flush to stable storage; a record acknowledged as deleted SHALL NOT reappear.
A "lying" disk and medium corruption are outside the guarantee. Trace: CUJ14; ADR 0014 items 1–3, ADR 0050; Q-16, Q-20.

#### Scenario: Process crash after acknowledgment
- **WHEN** a client received the add acknowledgment and the process is killed with SIGKILL
- **THEN** after start the record is found by search by number and by word prefix of the name

#### Scenario: Power loss, the disk does not keep the unflushed
- **WHEN** a test simulates power loss: everything written but not flushed is discarded; a rename and a delete of another record were acknowledged
- **THEN** after start the rename is in place, the deleted record is absent

#### Scenario: The guarantee boundary
- **WHEN** a disk reported a successful flush without performing it
- **THEN** durability is not promised, detection of such a disk is not promised (ADR 0014 item 3)

### Requirement: What counts as a flush to stable storage
The system SHALL count as a flush on Linux a successful return of `fdatasync` or `fsync`, on macOS a successful return of `fcntl(F_FULLFSYNC)`;
a plain `fsync` on macOS SHALL NOT count as a flush. The creation and renaming of a file SHALL be durable only after the file and its directory are flushed.
Trace: CUJ14; ADR 0050 items 2–3; F-64…F-70.

#### Scenario: macOS
- **WHEN** the server flushes the WAL on macOS
- **THEN** `F_FULLFSYNC` is called (via `File::sync_data`/`sync_all`), not just `fsync`

#### Scenario: A new WAL segment
- **WHEN** a new WAL segment is created
- **THEN** it is filled with zeros and flushed together with its directory before the first operation is written into it

### Requirement: All or nothing for an unacknowledged operation
After a failure the system SHALL leave an unacknowledged write operation either whole (the record, the derived index entries and the version) or
absent; there SHALL NOT be a partial operation. Trace: CUJ14, X-5; ADR 0029 item 1, ADR 0051 item 2.

#### Scenario: A break in the middle of a WAL write
- **WHEN** the process is killed while a WAL entry is partially written
- **THEN** after start the operation is absent, the earlier record and version are unchanged

#### Scenario: Order on the medium broken inside an unflushed batch
- **WHEN** the medium kept a later entry of a batch and not an earlier one, and nothing from the batch was acknowledged
- **THEN** after start only entries up to the first invalid one are applied; the later entry of the batch is not applied (including at subsequent starts)

### Requirement: Record and number version are one WAL entry
The system SHALL write the change of a number's record, of all derived indexes and the new number version value in one WAL
entry, so that "record exists, no version" and the reverse are impossible. Trace: CUJ14, X-5; ADR 0028 item 6 (a), ADR 0051 items 2, 7.

#### Scenario: Failure after acknowledgment
- **WHEN** a rename is acknowledged, the process is killed and started
- **THEN** the changed name and the new number version are present together

#### Scenario: Indexes are consistent
- **WHEN** after a crash start an acknowledged record is searched by number, by name word prefix and by whole name
- **THEN** the record is found by all three searches

### Requirement: Readers see only durable state
The system SHALL publish a change for search only after it is flushed to stable storage; a search started after the response to the client SHALL see
the operation's result in full, and a concurrent one the state before or after in full. Trace: CUJ1, CUJ6, CUJ12, CUJ14; ADR 0029 item 4, ADR 0050 item 6.

#### Scenario: Search during the flush
- **WHEN** a search runs while the operation's WAL entry has not yet been flushed
- **THEN** the operation is not visible in the result

#### Scenario: A flush error leaves no trace for readers
- **WHEN** a WAL flush ended with an error
- **THEN** no search showed the changes of this batch and none will

### Requirement: WAL before data, protection of pages from torn writes
The system SHALL write a page to the data file only after the WAL entries with its changes are durable, and SHALL protect
every write of checkpoint pages with the doublewrite buffer, so that a torn page write does not corrupt the database. Trace: CUJ14; ADR 0051 items 3–4.

#### Scenario: A torn page write at a checkpoint
- **WHEN** the process is killed (or power is lost) at the moment a page is written to the data file
- **THEN** at start the page is restored from the doublewrite buffer and the WAL applied; acknowledged operations are present

#### Scenario: A page with an LSN above the durable one
- **WHEN** a page is changed by an operation whose WAL entry is not yet flushed
- **THEN** the checkpoint does not write this version of the page to the data file

### Requirement: Checkpoints bound the WAL
The system SHALL run a checkpoint when the WAL volume from `redo_start_lsn` is at least 256 MiB, after 5 minutes if there are
unflushed pages, and on clean shutdown; the WAL volume that recovery must apply SHALL NOT exceed 256 MiB
plus the current batch. Trace: X-1, CUJ14; ADR 0051 item 4, ADR 0052 item 3; Q-42.

#### Scenario: By volume
- **WHEN** the WAL since the last checkpoint reached 256 MiB
- **THEN** a checkpoint starts; after it the segments up to the new `redo_start_lsn` are freed

#### Scenario: Idle
- **WHEN** 5 minutes have passed with changes and without reaching 256 MiB
- **THEN** a checkpoint is run

### Requirement: Crash recovery by WAL replay
On start after a failure the system SHALL apply the WAL from `redo_start_lsn` in order to pages whose LSN is less than the entry's LSN, discard the
torn tail and write "recovery end" with a new WAL epoch before accepting operations; an entry of an old epoch at the needed
offset SHALL NOT be applied. Trace: CUJ14; ADR 0051 items 5–6.

#### Scenario: Replay is idempotent
- **WHEN** the process is killed during recovery and started again
- **THEN** the result is the same as with a single application

#### Scenario: A ghost of a torn batch
- **GIVEN** in the previous run a later entry of a batch ended up on the medium, an earlier one did not, and after recovery less was written than the batch occupied
- **WHEN** the server is crash-stopped right after "recovery end" and started again
- **THEN** the ghost entry is not applied: its epoch is less than the epoch of the "recovery end" entry

### Requirement: Torn tail is discarded, mid-WAL corruption is a refusal
The system SHALL treat an invalid WAL entry as a torn tail if no valid entry after it has a `flushed_lsn`
greater than its position, and discard the tail, telling the operator the number of bytes; otherwise it SHALL treat it as WAL corruption and not open the database. Trace: CUJ14; ADR 0051 item 6, ADR 0053 items 2–3.

#### Scenario: A torn tail
- **WHEN** the last batch was written incompletely and nothing from it was acknowledged
- **THEN** the server starts, the tail is discarded, the operator is told the number of discarded bytes

#### Scenario: Corruption of an acknowledged entry in the middle
- **WHEN** a byte is changed in the WAL in an entry followed by entries acknowledged later
- **THEN** the server does not start, reports the reason and position, entries are not discarded, files are unchanged

#### Scenario: The detectability boundary
- **WHEN** the last acknowledged WAL entry is corrupted, and nothing follows it
- **THEN** detection is not promised: it is indistinguishable from a torn tail (medium corruption, ADR 0014 item 3)

### Requirement: A failed flush is not retried
On any error of `fsync`, `fdatasync`, `F_FULLFSYNC` the system SHALL not acknowledge the batch's operations, not repeat the flush on the same file
and enter the "storage error" state. Trace: CUJ16, CUJ14; ADR 0050 item 5; F-71, F-72.

#### Scenario: A WAL flush error
- **WHEN** the WAL flush returned `EIO`
- **THEN** the batch's operations get "storage error", there is no repeat flush, the state is "storage error", the operator gets a message

#### Scenario: What is on the disk is decided by the start
- **WHEN** the server is restarted after such an error
- **THEN** the batch's entry is either whole or entirely absent by the WAL checksums

### Requirement: Batch failure rolls back the writer's unpublished state
On a failure to write or flush a batch the system SHALL discard all the writer's unpublished state (page versions, pages handed out and freed, `next_S`, the commit number) and
reject with the same refusal all operations run after the first failed one. The next accepted operation SHALL be built on the last published state and written from the last
durable LSN. Trace: CUJ14, CUJ16, X-5; ADR 0051 item 9; ADR 0053 item 8.

#### Scenario: Operations on top of a failed one
- **GIVEN** operation A has run in the writer's memory, then operation B on the same pages, and the write of the batch with A ended with `ENOSPC`
- **WHEN** the writer rolls back the state
- **THEN** A and B are rejected with a "read-only" refusal, readers see the state before A, the next accepted operation C runs on the state before A

#### Scenario: Allocations and the counter returned
- **GIVEN** a failed operation took pages from the free list and advanced `next_S`
- **WHEN** the state is rolled back
- **THEN** these pages are again in the free list, `next_S` is as before, the next operation gets the same pages and the same `next_S`

#### Scenario: Checkpoint and recovery agree
- **GIVEN** after a failed operation, an operation C on the same pages is run and acknowledged
- **WHEN** a checkpoint is run, then the server is killed and recovered from the WAL
- **THEN** the page contents after recovery equal the contents written by the checkpoint and contain no trace of the rejected operation

#### Scenario: A flush error
- **WHEN** the WAL flush ended with `EIO`
- **THEN** the unpublished state is discarded, the batch's operations and those run after them got "storage error", the writer accepts no new operations until restart

### Requirement: Target write latency [DEFAULT]
The system SHALL acknowledge a write operation on a database of 10^8 records under a load of up to 100 operations per second with latency p50 ≤ 20 ms and p99 ≤ 100 ms
(with a flush to stable storage p99 ≤ 40 ms), and during a backup with p99 ≤ 250 ms and without refusals. Values are defaults, awaiting the owner's decision. Trace: CUJ1, CUJ15, X-1; ADR 0052 item 1; Q-58, Q-11.

#### Scenario: Write latency at 10^8
- **WHEN** on a database of 10^8 records with a cold cache writes run at up to 100 per second
- **THEN** p50 ≤ 20 ms and p99 ≤ 100 ms on the reference bench

#### Scenario: During a backup
- **WHEN** a backup is running
- **THEN** write operations are not rejected, p99 ≤ 250 ms

### Requirement: Start time at 10^8 records [DEFAULT]
The system SHALL start on a database of 10^8 records, including the start check, in no more than 5 s after a clean shutdown and no more than 30 s after a crash shutdown or on a backup, without scanning
records or rebuilding indexes; a start with an index rebuild or format conversion is not included in these numbers. Values are defaults, awaiting the owner's decision. Trace: CUJ14, X-1; ADR 0052 item 2; Q-42.

#### Scenario: After a clean shutdown
- **WHEN** the server starts on a database of 10^8 records after a clean shutdown
- **THEN** it is ready to accept requests within 5 s

#### Scenario: After a crash shutdown
- **WHEN** the server starts after SIGKILL with a 256 MiB WAL since the last checkpoint
- **THEN** it is ready to accept requests within 30 s

#### Scenario: A start with a rebuild or format conversion
- **WHEN** a server of a new v1 version starts on a database of 10^8 records after a clean shutdown, and the index label or format version differs
- **THEN** the 5 s and 30 s targets do not apply to this start; the rebuild or conversion is done before accepting requests; the rebuild time is no more than 60 minutes (`storage-indexes`, "Index format label and rebuild at start")

### Requirement: Consistency check at start
Before accepting requests the system SHALL check: the `control` slots and format; the database identifier in all files; the doublewrite
buffer pages; the WAL from `redo_start_lsn` (every entry); the pages read while applying the WAL; the service page, the index roots and the layout metadata.
The volume of the check SHALL NOT depend on the number of records. Trace: CUJ14; ADR 0014 item 5, ADR 0053 item 1; Q-43.

#### Scenario: The check does not scan records
- **WHEN** the server starts on a database of 10^8 records
- **THEN** the number of pages read is bounded by the number of indexes and the pages touched by the WAL

#### Scenario: Database identifier mismatch
- **WHEN** the database directory contains a `wal/` or `dwb` from another database
- **THEN** the server does not start and names the file

### Requirement: Inconsistent database is not opened
The system SHALL not start, not modify `data` and the WAL (except restoring a torn page from the doublewrite buffer) and tell the operator the reason, file and
position if the start check found an inconsistency: no valid `control` slot, a format newer than the program, a page without a valid copy, WAL corruption in the middle,
an invalid root page. Trace: CUJ14; ADR 0014 item 4, ADR 0053 item 2.

#### Scenario: Both control slots are invalid
- **WHEN** both `control` slots fail the checksum check
- **THEN** the server does not start, reporting «нет верного управляющего файла» ("no valid control file"); records are not discarded

#### Scenario: Index root page
- **WHEN** the checksum of an index root page is invalid
- **THEN** the server does not start, the message names the index and the page

#### Scenario: Format newer than the program
- **WHEN** `format_version` in META (ADR 0040 item 4) or the `control` layout label is above the supported one
- **THEN** the server does not start with this reason; the database is unchanged

#### Scenario: Restore from a backup
- **WHEN** the operator starts the server on a backup after a refusal
- **THEN** the server starts on the backup data (CUJ14, CUJ15)

### Requirement: Page check on read
The system SHALL check every page read from the file into the cache (CRC-32C, page number, type); on a mismatch or read error it marks the page
corrupted, enters "storage error" and refuses the searches that need it; other searches SHALL run. Trace: CUJ14, CUJ16; ADR 0014 item 5, ADR 0053 item 4; Q-14.

#### Scenario: Corruption on read
- **WHEN** a search reads a page with an invalid checksum
- **THEN** the search refuses with "storage error", the server is in the "storage error" state, the operator is told the page

#### Scenario: A search bypassing the corrupted page
- **GIVEN** the server is in the "storage error" state because of a corrupted page
- **WHEN** a search that does not touch it runs
- **THEN** it returns a result

#### Scenario: A repeated access
- **WHEN** another search touches the same corrupted page
- **THEN** it refuses without reading the file again

### Requirement: "Read-only" state
The system SHALL enter the "read-only" state when a write, an open for writing, the creation of a segment, a file extension or a rename ended with the error `EROFS`, `EACCES`,
`EPERM`, `ENOSPC`, `EDQUOT`; the operation is not acknowledged and not published, all searches work. [DEFAULT] This is a clarification of Q-20 (D-34, awaiting the owner): other write and read errors and a failed
flush are "storage error". Trace: CUJ16; ADR 0053 item 5; Q-71, Q-20.

#### Scenario: A full disk
- **WHEN** the creation of a new WAL segment ended with `ENOSPC`
- **THEN** the operation is not acknowledged, data is unchanged, the state is "read-only", searches work

#### Scenario: The boundary with "storage error"
- **WHEN** a WAL write ended with `ENOSPC`, and in another case with `EIO`
- **THEN** in the first case "read-only" (nothing is lost, the fix depends on the operator), in the second "storage error" (CUJ16, "Disk write error", in the Q-20 clarification, D-34)

#### Scenario: A start on files not writable
- **WHEN** the database files are not writable and the previous shutdown was clean
- **THEN** the server starts in the "read-only" state, all searches work

#### Scenario: Crash shutdown and files not writable
- **WHEN** the files are not writable and the previous shutdown was a crash
- **THEN** the server does not start, reports the reason (recovery requires writing)

### Requirement: "Storage error" state
The system SHALL enter the "storage error" state on a failed flush, a read or write error outside the "read-only" list (including `EIO`), page corruption,
a violated structure invariant and the unavailability of the generation counter location or the randomness source; changes are rejected, search works while it can; the operator gets a message
with the source, `errno` and location. "Storage error" SHALL be stronger than "read-only". Trace: CUJ16, CUJ14; ADR 0014 item 6, ADR 0053 items 6–7; Q-71.

#### Scenario: A disk write error
- **WHEN** a WAL write ended with `EIO`
- **THEN** the operation is not acknowledged; there is no partial change; the state is "storage error"

#### Scenario: In-memory state
- **WHEN** the server is in "storage error" and is restarted
- **THEN** the state is determined by the start checks, not by a saved value

#### Scenario: Reproducibility
- **WHEN** a test injects a failure with a given `errno` on a given file operation
- **THEN** the state given by the table of causes is observed, without a real disk overflow

### Requirement: Leaving the states [DEFAULT]
The system SHALL leave "storage error" only on restart, and "read-only" by itself, by a trial write. The state SHALL be checked before the operation checks (`write-operations`,
"Check order on write"); in "read-only" a write operation SHALL begin with a probe — a durable service WAL entry: if it fails — a "read-only" refusal, if it succeeds — the state is
normal, a message to the operator, the operation's ordinary path. Default (D-29). Trace: CUJ16; ADR 0053 item 8; Q-71.

#### Scenario: Space freed
- **GIVEN** the server is in the "read-only" state because of `ENOSPC`
- **WHEN** the operator freed space and the user repeated the save
- **THEN** the probe succeeded, the save was performed and acknowledged, the state is normal, the operator gets a message

#### Scenario: State before the operation checks
- **GIVEN** the server is in the "read-only" state, space is not freed
- **WHEN** a rename with a stale version arrives, and then an identical retry of a previously acknowledged add
- **THEN** both get "read-only", not "version conflict" and not "unchanged"; data and state are unchanged

#### Scenario: The probe succeeded, the operation was rejected by a check
- **GIVEN** the server is in the "read-only" state, space is freed
- **WHEN** a rename with a stale version arrives
- **THEN** the probe succeeded, the state is normal, the operator gets a message, and the operation gets "version conflict" by the ordinary check order

#### Scenario: Storage error persists
- **GIVEN** the server is in the "storage error" state
- **WHEN** the cause is eliminated but the server is not restarted
- **THEN** changes are still rejected, there are no probes or background polling; on a start in "read-only" the probe includes writing `run_state = running`

#### Scenario: A retry after a failed write
- **WHEN** a WAL write failed with `ENOSPC`, then the probe and a new write succeeded
- **THEN** the new entry is laid down from the earlier durable LSN; after start the WAL is read without error

### Requirement: Single owner of the database
The system SHALL take an exclusive non-blocking lock on the `LOCK` file of the database directory as the first action at start and hold it until the process ends; if the lock is held, the start SHALL
be rejected with the message «база уже открыта другим процессом» ("database already open by another process"), and any other lock error with a refusal giving its reason. Trace: X-2, CUJ13; ADR 0015 item 3, ADR 0054 item 1.

#### Scenario: A second process
- **WHEN** a second server process opens the same database while the first is alive
- **THEN** the second does not start, the message «база уже открыта другим процессом» ("database already open by another process"), data is unchanged

#### Scenario: The first killed
- **WHEN** the first process is killed with SIGKILL and a new one is started
- **THEN** the lock is free and the new process starts (a crash start)

#### Scenario: Lock before reading
- **WHEN** the server starts
- **THEN** before the lock no other database file is read or modified

### Requirement: Database creation does not overwrite an existing one
The system SHALL create a database so that `control` is created last and with overwrite forbidden, and on a directory with an existing `control` or when the lock is held the
create command refuses without changing the database. A server on a directory without `control` SHALL NOT start. Trace: CUJ13, X-2; ADR 0015 items 1–2, ADR 0054 item 2.

#### Scenario: Repeated creation
- **WHEN** the create command is run on a directory with a database
- **THEN** refusal, files unchanged

#### Scenario: An interrupted creation
- **WHEN** creation is interrupted before `control` is written
- **THEN** the directory is not a database: the server does not start, repeated creation is allowed

### Requirement: Clean-shutdown marker and clean shutdown
The system SHALL write the clean-shutdown marker to `control` as the last step of a clean shutdown, after the acknowledgment or rejection of every accepted operation, the WAL flush and a full
checkpoint; termination before this step SHALL leave the marker unwritten (a crash shutdown), and the process on a clean shutdown SHALL end with code 0. Trace: CUJ14, CUJ15; ADR 0028 item 10, ADR 0054 item 8; Q-108.

#### Scenario: A clean shutdown
- **WHEN** the server receives SIGTERM with a write in flight
- **THEN** the operation is acknowledged or rejected, the checkpoint is done, the marker is written, code 0

#### Scenario: The shutdown deadline expired
- **WHEN** the process is forcibly terminated before the marker is written
- **THEN** there is no marker, the next start is a crash start

### Requirement: Start kind is determined by the marker on the medium
The system SHALL determine the start kind from `run_state` in `control`: `clean` — planned, `running` — crash, `backup` — start from a backup; before accepting the first operation after start
`run_state = running` SHALL be written durably. For files copied not by the command, the start kind is determined by the same marker. Trace: CUJ14, CUJ15; ADR 0028 items 7, 10, ADR 0054 items 4, 6.

#### Scenario: The marker is cleared before the first operation
- **WHEN** the server is started after a clean shutdown, accepted the first operation and crashed
- **THEN** the next start is a crash start

#### Scenario: A start from a backup
- **WHEN** the server starts on a backup with the `backup` marker
- **THEN** it is a start from a backup; the marker is cleared in the running database, and remains in the backup archive

### Requirement: Version generation: counter outside the backup and random epoch
On a crash start and a start from a backup, before accepting operations, the system SHALL durably write the generation counter `G = max(G_s, G_run) + 1` to the `generation` file next to the settings (outside the backup), choose the epoch `E`
of 16 bytes from the OS randomness source and durably write `G`, `E` to `control`; the observable number version SHALL be (G, E, S), the initial one (G, E, 0). On a planned start
`G` and `E` SHALL be unchanged. Trace: CUJ14, CUJ15, X-5; ADR 0028 items 8–9, ADR 0054 items 5–6.

#### Scenario: A crash start
- **WHEN** the server is crash-restarted
- **THEN** `G` is greater than before, `E` is new; a number version read before the failure does not match the current one

#### Scenario: Two starts from one archive
- **WHEN** a backup is brought up twice on one installation
- **THEN** `G` differs between the starts; a rename by the version of the first start in the second is a "version conflict"

#### Scenario: Settings rollback
- **WHEN** `G_s` in the settings is less than `G_run` in the database
- **THEN** the new `G` is greater than `G_run`, versions do not match the earlier ones

#### Scenario: A planned start
- **WHEN** the server is started after a clean shutdown
- **THEN** `G`, `E` are the same, issued versions are valid, the `generation` file is neither read nor written

### Requirement: Version does not require a number scan
The system SHALL store for a number only the sequence value S (u64; in the number header a varint, at most 5 bytes at S < 2^35) from a counter shared by all numbers, restore `next_S` from the WAL and `control`, and change the version generation
without changing records and without scanning numbers. Trace: X-5, X-1; ADR 0028 item 6 (d, f), ADR 0051 item 7; Q-78.

#### Scenario: Generation change at 10^8 numbers
- **WHEN** a crash start on a database of 10^8 numbers
- **THEN** no number's record is read or changed for the sake of the generation change

#### Scenario: The counter after a failure
- **WHEN** the server is crash-restarted
- **THEN** `next_S` is no lower than the largest S among the applied WAL entries and the value in `control`

### Requirement: Unavailable counter location or randomness source
On a crash start and a start from a backup, if the settings directory is not writable, both `generation` slots are invalid, the flush failed or the randomness source returned
an error, the system SHALL not accept write operations, enter "storage error", not change `run_state` and run search with the versions of the earlier generation. Trace: CUJ16; ADR 0028 item 8, ADR 0054 item 7; Q-107.

#### Scenario: The settings directory is read-only
- **WHEN** after a crash shutdown the settings directory is not writable
- **THEN** changes are rejected ("storage error"), search works, `run_state` is unchanged

#### Scenario: A randomness source error
- **WHEN** the injected randomness source returns an error on a crash start
- **THEN** the same: changes are rejected, search works, no version of the new generation is issued

#### Scenario: Exit
- **WHEN** the cause is eliminated and the server is restarted
- **THEN** an ordinary crash start with a new generation is performed

### Requirement: Substituting the randomness source and counter storage in tests
The system SHALL obtain randomness and access to `generation` through injectable interfaces; tests with a deterministic source check the growth of `G` and the change of `E` on a crash start and a start from a backup,
no change on a planned one, a refusal on a source or storage error, and an `E` capacity of at least 128 bits. Trace: CUJ14, CUJ15, X-5; ADR 0028 item 9, ADR 0054 item 10.

#### Scenario: Epoch capacity
- **WHEN** a test reads the epoch length
- **THEN** it is at least 128 bits

#### Scenario: A deterministic source
- **WHEN** a test substitutes the randomness source and the counter storage
- **THEN** the change of `G` and `E` is checked exactly, without calling the OS
