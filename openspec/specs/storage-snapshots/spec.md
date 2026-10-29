# storage-snapshots Specification

## Purpose
The concurrency model of the storage engine (one writer, many readers on consistent snapshots), the implementation of the accepted
isolation (ADR 0029), snapshots for the duration of a request and of the by-number stream, a backup without stopping the server with a
"backup" marker and a start from it as a new generation, and the rebuild of derived indexes on a key format change.

## Requirements

### Requirement: One writer, many readers
The engine SHALL perform all write operations in a single writer thread in a single order; there may be many readers, and they
run in parallel with the writer and with each other. A reader SHALL NOT hold a lock that the writer waits for, and the writer SHALL NOT hold a lock that a reader waits for. Trace: CUJ1, CUJ6, CUJ9, CUJ12, X-2; ADR 0029 items 1–3, ADR 0055.

#### Scenario: Reader does not wait for the writer
- **GIVEN** the writer is stopped in the middle of an operation (a test breakpoint)
- **WHEN** a search request arrives
- **THEN** it runs and returns the state before that operation, without waiting for the writer

#### Scenario: Writer does not wait for a reader
- **GIVEN** a reader holds an open snapshot and reads no further
- **WHEN** a write operation arrives
- **THEN** it runs and is acknowledged, without waiting for the reader

#### Scenario: Single order of writes
- **WHEN** write operations on different numbers arrive simultaneously
- **THEN** they are lined up in one commit order, and each is visible to readers entirely relative to the others

### Requirement: Write operations are strictly sequential
Every acknowledged write operation that changed data SHALL receive a commit number — the LSN of its WAL entry: strictly increasing, continuing across restarts, without repeats; "unchanged" operations and rejected ones do not receive one and are not written to the WAL. Operations on one number SHALL execute as if in a queue; the expected-version check and the change are one step of the writer, with no other write between them. Trace: CUJ9, CUJ12, X-5; ADR 0029 items 2–3.

#### Scenario: Two writes against one version
- **GIVEN** two rename operations on one number arrived with the same expected version
- **WHEN** the writer executes both
- **THEN** the first succeeds, the second gets a version conflict with the current version, and the data of the second is not written

#### Scenario: Commit numbers without gaps
- **WHEN** N acknowledged operations that changed data have been executed, among which there was an identical retry and a rename to the same name
- **THEN** every changing operation received a number, the numbers strictly increase in execution order, including after a restart; "unchanged" and rejected operations did not receive one

### Requirement: Operation is visible only after it is durable
The result of an operation SHALL become visible to readers no earlier than it is durable (survives a failure), and the acknowledgment to the client SHALL be sent no earlier than the result is visible to new snapshots. A reader SHALL NOT see a change that could disappear on a failure. Trace: CUJ1, CUJ6, CUJ14; ADR 0014 item 1, ADR 0029 items 1, 4, ADR 0055.

#### Scenario: Read after write
- **WHEN** a client received success for a write operation and then started a search
- **THEN** the search sees the result of the operation entirely

#### Scenario: Non-durable is not visible
- **GIVEN** an operation is written into the writer's memory but not flushed to stable storage
- **WHEN** a search request arrives
- **THEN** it does not see the result of the operation

#### Scenario: Failure after a read
- **WHEN** a reader saw the result of an operation and the process crashed
- **THEN** after a restart the result is in place

### Requirement: One snapshot per request
Every read request (search by number, a name search page, a point version lookup) SHALL execute on one snapshot: the state of the database after all acknowledged operations with a commit number not greater than one value C. The records and the number version returned by the request SHALL come from this snapshot. Trace: CUJ5, CUJ6, CUJ9, CUJ12; ADR 0029 items 4–5, ADR 0032.

#### Scenario: Version consistent with the records
- **WHEN** while a search by number is running the writer changes the records of this number
- **THEN** the response contains the records and the version of one state: either before the change or after

#### Scenario: Name page and number versions
- **WHEN** a name search page reads the records of several numbers during a concurrent write
- **THEN** the version of each number on the page is taken from the same snapshot as the records of the page

#### Scenario: No partial operation
- **WHEN** a reader opens a snapshot at any moment of the writer's work
- **THEN** it does not see an operation partially: it sees the record and the version change together or neither

### Requirement: Snapshot is not kept between pages
The engine SHALL NOT keep a snapshot or cursor state between requests: the next name search page reads the state as of its own request. Consistency between pages SHALL NOT be promised. Trace: CUJ5, CUJ12; ADR 0029 item 5, ADR 0032.

#### Scenario: Write between pages
- **GIVEN** a client received the first results page
- **WHEN** a write is acknowledged before the second page is requested
- **THEN** the second page reads the new state, and the client holds nothing of the earlier one

#### Scenario: Snapshot released after the response
- **WHEN** the response for a page has been sent
- **THEN** the snapshot of this request is released

### Requirement: Opening a snapshot is cheap
Opening a snapshot SHALL be done without copying data, without waiting for the writer and in a time independent of the database size; closing — without traversing data. Trace: X-1, CUJ5, CUJ8; ADR 0055.

#### Scenario: Open time does not depend on size
- **WHEN** snapshot opening is measured on an empty database and on a database of 10^7 records
- **THEN** the results are of one order and do not grow with the database size

#### Scenario: Opening under write load
- **WHEN** the writer continuously executes operations
- **THEN** snapshot opening does not wait for an operation to finish

### Requirement: Snapshot retains earlier page versions
While a snapshot with number N is open, the engine SHALL keep the data needed to read state N and not reuse it; after the last snapshot that needs it is closed, the space
SHALL return to circulation. Retained are the pages removed from the trees after N and the earlier published versions of pages changed after N: in memory, beyond the budget (default
256 MiB) in the temporary file `retain` (D-35). Trace: CUJ5, CUJ15; ADR 0056 item 1.

#### Scenario: Reader sees its own state
- **GIVEN** a snapshot is open, then writes changed and deleted the same records
- **WHEN** the reader continues reading
- **THEN** it sees the state as of the opening

#### Scenario: Page rewritten by a checkpoint
- **GIVEN** a snapshot is open, then a page is changed twice by operations, and a checkpoint wrote its last version to `data`
- **WHEN** the snapshot's reader reads this page
- **THEN** it sees the version as of the opening: the `data` file is not needed for the snapshot, the writer's unpublished versions are not part of retention

#### Scenario: Freed page
- **GIVEN** a snapshot is open, then an operation removed a page from a tree
- **WHEN** a page allocation for new data is requested
- **THEN** the page is not handed out while a snapshot that sees it is alive (ADR 0044 item 2), and is handed out after it is closed

#### Scenario: Retention beyond the memory budget
- **GIVEN** a snapshot is open, and writes changed pages by a volume greater than the memory budget for retention
- **WHEN** the reader reads the changed pages
- **THEN** it sees the state as of the opening, part of the earlier versions lie in the temporary file `retain`

#### Scenario: Return of space
- **WHEN** all snapshots that retained earlier versions are closed
- **THEN** the retained volume becomes zero, memory and space in the temporary file are freed, and the freed pages are used again

#### Scenario: Temporary file error
- **WHEN** a write to the temporary retention file ended with any I/O error (`ENOSPC`, `EIO`, …)
- **THEN** snapshots are revoked in the order of the requirement "Cap on retained volume", the database files are not affected, the database state does not change, write operations are acknowledged

### Requirement: Snapshot is always released
A snapshot SHALL be released on request completion, error, cancellation, client connection drop and handler panic. A snapshot leak SHALL be considered a defect: the retained volume and the number of open snapshots are available to tests and to the server log. Trace: CUJ5, CUJ8, CUJ15; ADR 0056.

#### Scenario: Connection drop
- **WHEN** a client dropped the connection in the middle of the results
- **THEN** the snapshot is released immediately, and if the drop is not noticed — no later than the deadline of the requirement "Snapshot retention ceilings" (the short-read ceiling, stream idle)

#### Scenario: Handler panic
- **WHEN** a request handler ended with a panic
- **THEN** the number of open snapshots decreases, as on normal completion

### Requirement: Snapshot retention ceilings
A short read SHALL have a duration ceiling of 30 s (default, D-30; not extended by client activity): when it expires the snapshot is revoked and the read is aborted with a "snapshot revoked" error.
A by-number stream SHALL NOT have a general ceiling and is revoked only when the client is idle (does not accept data for 120 s) and, at the retention cap, last; a revocation is a read refusal by the storage without
the server switching to "storage error". Trace: CUJ4, CUJ8, X-1; ADR 0035 item 6, ADR 0056.

#### Scenario: Slow but reading stream client
- **GIVEN** a client accepts a number history slowly but continuously, and the total delivery time is more than 120 s
- **WHEN** delivery continues and the retained volume is at the cap
- **THEN** the stream is not revoked, delivery completes in full with the completeness marker

#### Scenario: Stream client stopped reading
- **GIVEN** a stream client does not accept data
- **WHEN** 120 s of idle have passed
- **THEN** the snapshot is revoked, the stream is cut off without the final completeness marker (`number-search`), the write is not delayed

#### Scenario: Short read did not finish
- **WHEN** a short read did not finish within 30 s
- **THEN** the snapshot is revoked, the read is aborted with a "snapshot revoked" error without a partial result that looks complete

#### Scenario: Fast request
- **WHEN** a request finished before the ceiling
- **THEN** the snapshot is released on completion, the ceiling is not used

### Requirement: By-number results are streamed on one snapshot
Reading a number history (up to 2 967 164 records) SHALL be done on one snapshot opened before the first record is sent; the records and the version SHALL come from it, and the engine's memory for the results SHALL be bounded by the chunk size and not depend on the number of records. There is no truncation or continuation. Trace: CUJ4, CUJ6, CUJ8, CUJ9; ADR 0007, ADR 0035 item 6.

#### Scenario: Write to the number during results
- **GIVEN** a number history of many records is being returned
- **WHEN** a record is added to this number
- **THEN** the results do not contain it, the version is as of the start, the write is acknowledged without waiting for the end of the results

#### Scenario: Memory does not grow with history
- **WHEN** a history of 10^6 records and of 10 records is returned
- **THEN** the engine's peak memory for the results is of the same order

#### Scenario: Completed results
- **WHEN** all records of the number are delivered without revoking the stream
- **THEN** the engine reports full completion to the API layer (the basis of the end marker, ADR 0035 item 6)

### Requirement: Cap on retained volume
The engine SHALL limit the volume retained by snapshots (page versions in memory and in `retain`, pages in the free list): by default at most the larger of 1 GiB and 25 % of the data, the on-disk part — at most the free space minus a write reserve of 1 GiB (ADR 0059). When exceeded, snapshots are revoked by class: short reads, then by-number streams, within a class the oldest first; a write SHALL NOT be rejected or slowed. Trace: CUJ1, CUJ16; ADR 0056, 0096 (OD-7).

#### Scenario: Backup under high write load
- **GIVEN** a backup is running (it creates no retention) with high write load, short reads are open
- **WHEN** the retained volume reaches the cap
- **THEN** the backup is not affected, the reads are revoked, writes continue to be acknowledged

#### Scenario: Streams are revoked last
- **GIVEN** the cap is reached, a short read and a by-number stream are open
- **WHEN** the engine needs to free space
- **THEN** the short read is revoked first, the by-number stream is revoked only if that is not enough

#### Scenario: Free space
- **WHEN** free space on the volume dropped to the write reserve
- **THEN** snapshots are revoked in the same order until the space is exhausted, so that the database does not go "read-only" because of snapshots

### Requirement: Snapshots live only in process memory
A snapshot and the data it retains SHALL NOT be preserved between runs; after any restart there are no open snapshots, the retention space is returned to circulation at start without traversing the whole database. Trace: CUJ14; ADR 0056.

#### Scenario: Restart with an open snapshot
- **GIVEN** the process is killed while a snapshot is open
- **WHEN** the server is started again
- **THEN** there are no snapshots, the database is consistent, the temporary retention file is deleted, no extra space is retained

### Requirement: Backup is made inside the server process
Copying SHALL be performed by the server process, the owner of the database (X-2), without a snapshot and without retaining versions: with checkpoints suspended, `data` and the WAL up to number X are copied. The server SHALL NOT stop; reads and writes — without refusals caused by the copying, except an abort by the cap on modified pages. At most one copy at a time; a second one is rejected. Trace: CUJ15, X-2; ADR 0016, ADR 0057, ADR 0096 (OD-7).

#### Scenario: Copying while the server is running
- **WHEN** copying is in progress
- **THEN** searches and write operations are handled and acknowledged

#### Scenario: Second copy
- **GIVEN** copying is in progress
- **WHEN** a second backup command arrives
- **THEN** it is rejected with the message «копирование уже выполняется» ("backup already in progress"), the first is not affected

#### Scenario: Server stop during copying
- **WHEN** a clean shutdown is issued during copying
- **THEN** the copying is cancelled, the partial backup is deleted, the shutdown does not wait for its end and remains a clean one

### Requirement: Backup point is the LSN at command receipt
A backup SHALL contain the state at commit number X — the LSN of the last acknowledged operation after the checkpoints are stopped: every operation acknowledged before the command was received is included entirely, operations after X are not included, there are no partial ones. X and the commit time SHALL be recorded in the backup and available to the operator. Trace: CUJ15; ADR 0016, ADR 0057 (Q-83), ADR 0096 (OD-7).

#### Scenario: Acknowledged before the command
- **GIVEN** a client received success for an operation, then the operator started a backup
- **WHEN** the backup is ready and started
- **THEN** the operation is in the backup

#### Scenario: Write during copying
- **WHEN** an operation is acknowledged after X is fixed
- **THEN** it is not in the backup, and the backup equals the state at the commit number from its description

#### Scenario: Check against the operation sequence
- **WHEN** a test replays the recorded data-changing operations with a commit number not greater than the backup number
- **THEN** the result equals the contents of the backup (including for a database that survived a restart)

### Requirement: Backup includes data and indexes
A backup SHALL contain everything needed to start: `data` as is (as of the last checkpoint), `wal/` with the WAL from `redo_start_lsn` to X, its own `control`, `dwb` without occupied slots, and `LOCK`; records, versions and indexes are brought up to X by the recovery pass at start. Tokens, settings and the generation counter SHALL NOT be included. Trace: CUJ15, CUJ10; ADR 0016 item 5, ADR 0028, ADR 0057, ADR 0096 (OD-7).

#### Scenario: Start without a rebuild
- **WHEN** the server starts on a backup of the same format version
- **THEN** the indexes are used as is, searches return the data of the backup point

#### Scenario: Tokens are not included
- **WHEN** the backup is ready
- **THEN** it contains no tokens, settings or generation counter

#### Scenario: WAL up to X
- **GIVEN** after the source's last checkpoint operations were acknowledged, and a backup was made
- **WHEN** the server is started on the backup
- **THEN** these operations are visible, there are none acknowledged after X

### Requirement: Contents of the backup control file
The system SHALL write into the backup's `control`: the source's `control` as of the moment the checkpoints were suspended (`db_id`, `redo_start_lsn`, the WAL epoch, `next_S`, `G_run`, `E_run`), `run_state = backup` and `backup_end_lsn = X`. The backup description is checked only when `run_state = backup`. Trace: CUJ15, X-5; ADR 0051 items 5, 7; ADR 0054 items 5, 9; ADR 0057 item 13; ADR 0096 (OD-7).

#### Scenario: Version counter of the backup
- **GIVEN** after restoring a backup the largest S is 500
- **WHEN** the server is started on the backup and accepts the first operation on a number
- **THEN** the issued S is greater than 500: the number version within the new generation is never repeated, a stale change does not pass the version check

#### Scenario: Backup LSN above the pages
- **GIVEN** the backup has pages with `page_lsn` up to X
- **WHEN** the server is started on the backup, an operation that changed such a page is acknowledged, and the server is killed
- **THEN** the operation's WAL entry has an LSN greater than X and during recovery is applied, not skipped by the rule "`page_lsn` less than the entry's LSN"

#### Scenario: Backup without WAL
- **WHEN** the server starts on a backup whose WAL from `redo_start_lsn` to X is empty (there were no changes after the checkpoint)
- **THEN** the recovery pass applies nothing, "end of recovery" is written, the generation change is done, the database accepts operations

#### Scenario: Ordinary database without a backup description
- **WHEN** the server starts on an ordinary database with `run_state` `clean` or `running`
- **THEN** the backup description is not required and not checked

#### Scenario: Recovery up to X
- **WHEN** the server starts on a backup with an empty `dwb` and a WAL up to X
- **THEN** the recovery pass applies the WAL up to X, "end of recovery" is written, the generation change is done, the database accepts operations

### Requirement: Checks during copying
Copying SHALL read and verify the checksum of every `data` page. On a mismatch the backup SHALL NOT be created, the operator is told about the corruption, and the server goes to the "storage error" state (ADR 0014 item 6). Trace: CUJ14, CUJ15; ADR 0014 items 5–6, ADR 0057 (Q-41), ADR 0096.

#### Scenario: Corrupted page
- **GIVEN** a page on the medium is corrupted
- **WHEN** the copying reaches it
- **THEN** the copying is rejected with a corruption message, the partial backup is deleted, there is no valid backup

#### Scenario: Intact database
- **WHEN** all pages pass the check
- **THEN** the backup is created and marked as verified

### Requirement: Backup does not exist until complete
A backup SHALL be written to a temporary place and become a backup by the last step: the data and the directory are flushed, then the description file `backup.desc` is written atomically (temporary file and `rename`) in the backup directory (the "backup" marker, format version, index labels, commit number, snapshot time, file sizes and checksums; `key=value` lines and a total checksum). A directory with `run_state = backup` without a complete and valid `backup.desc` SHALL NOT start (refusal "backup is incomplete"). Trace: CUJ14, CUJ15; ADR 0057 (Q-41).

#### Scenario: Failure in the middle of copying
- **WHEN** the process or power failed before the description was written
- **THEN** when the server starts on this directory the refusal is "backup is incomplete" (if `control` is also missing — "not a database"); the source database is not affected

#### Scenario: Completed backup
- **WHEN** the description is written
- **THEN** the directory starts as a backup

### Requirement: Backup target and location
The backup command SHALL accept a target path that does not exist or is empty; an existing non-empty target SHALL be rejected, overwriting nothing. Before the start, the free space of the target SHALL be checked to be at least the size of the backup (`data` plus WAL up to X) plus a reserve; on insufficient space or a write error the copying is rejected, the partial backup is deleted, the server keeps working. Trace: CUJ15; ADR 0016, ADR 0057 (Q-41), ADR 0096.

#### Scenario: Target occupied
- **WHEN** the target path contains files
- **THEN** refusal, the contents of the target are unchanged

#### Scenario: No space
- **WHEN** the target does not have enough space
- **THEN** refusal before the start (or, on a write error in progress, an abort), the server without refusals to clients

### Requirement: Copying under source constraints
Copying SHALL be performed also in the source's "read-only" state, since it reads files and does not write to the source. In "storage error" copying SHALL be performed if all pages can be read and pass the check, otherwise it is rejected. A server that is unavailable for the command makes no backup. Trace: CUJ15, CUJ16; ADR 0014, ADR 0028 item 7 (Q-71, Q-60); ADR 0096.

#### Scenario: Read-only database
- **GIVEN** the database is available only for reading
- **WHEN** copying is started
- **THEN** the backup is created

#### Scenario: Stopped server
- **WHEN** the server is stopped and the operator starts copying
- **THEN** no backup is created; a message that the server is unavailable

### Requirement: Latency growth during copying
Copying SHALL limit the source read rate (default 64 MiB/s, ADR 0059); there is no thread priority or OS cache control. Search and write latencies during copying SHALL stay within the targets of Q-47, Q-58, Q-59; there must be no refusals, except an abort of the copying by the cap on modified pages. Trace: CUJ15; OD-3, Q-11; ADR 0016 item 3; ADR 0101 (OD-7).

#### Scenario: Load check
- **WHEN** copying runs on a database of 10^7 records under typical load
- **THEN** p99 of search and write latencies is within the targets; there are no refusals caused by the copying

#### Scenario: Rate limit
- **WHEN** copying runs on a fast disk
- **THEN** the read rate does not exceed the set limit

### Requirement: "Backup" marker in the backup itself
A backup SHALL carry the "backup" marker in the backup description and in the backup's `control` (not in `data`); the marker SHALL NOT appear in the source database. The marker is preserved on a bytewise duplication of the backup directory and is cleared only when the server starts on it. Trace: CUJ15, X-5; ADR 0028 items 5, 7.

#### Scenario: Copy of a backup
- **GIVEN** the backup directory is duplicated unchanged
- **WHEN** the server is started on the duplicate
- **THEN** the start proceeds as on a backup

#### Scenario: Source database without the marker
- **WHEN** a backup is made
- **THEN** the source database has no "backup" marker, planned restarts go as before

### Requirement: Start from a backup is a new generation
The server SHALL determine the start kind from the medium: the "backup" marker takes precedence over the clean-shutdown and failure markers. On a start from a backup the recovery pass up to `backup_end_lsn` SHALL be performed, then the generation change SHALL be written durably before the marker is cleared and before operations are accepted, then the marker is cleared and `backup.desc` is deleted; a failure between the steps makes the next start also a start from a backup. Trace: CUJ15, X-5; ADR 0028 items 5–6, 9–10; ADR 0096 (OD-7).

#### Scenario: First start from a backup
- **WHEN** the server is started on a backup
- **THEN** the start kind is "start from backup", versions of earlier generations do not pass, operations are accepted after the generation change is written

#### Scenario: Failure between steps
- **WHEN** the process crashed after the generation change was written but before the marker was cleared
- **THEN** the next start is again from a backup, the generation is changed once more

#### Scenario: Repeated start from an archive
- **GIVEN** a backup archive was saved bytewise before the first start
- **WHEN** the server is started a second time on a duplicate of the archive
- **THEN** it is again a new generation

### Requirement: Backup of the earlier v1 format
The backup description SHALL contain the format version and the format labels of the derived indexes. A server of a new version within v1 SHALL open a backup of an earlier v1 version by itself, without an operator command; a label mismatch starts a rebuild per the requirement "Rebuild of derived indexes on format change". Trace: CUJ11, CUJ15; ADR 0013, ADR 0016 item 6, ADR 0039 item 3.

#### Scenario: Backup from before the upgrade
- **WHEN** a server of a new v1 version is started on a backup of an earlier one
- **THEN** the database is opened without a command; if the index labels differ, they are rebuilt before requests are accepted

### Requirement: Rebuild of derived indexes on format change
Every index derived from the name (order, words, exact search) SHALL carry a format label (key format version, `icu_collator` version, word table version, hash identifier; ADR 0045 item 7); when the label does not match the running server the index SHALL be rebuilt from the stored names before requests are accepted. The order and the matches before and after SHALL be the same. Trace: CUJ5, CUJ11; ADR 0036, ADR 0039 items 3, 6; ADR 0058 (Q-82).

#### Scenario: Matching labels
- **WHEN** the index labels equal the server's labels
- **THEN** there is no rebuild, the start has no delay for it

#### Scenario: Mismatched label
- **WHEN** the label of the order index differs
- **THEN** before accepting requests the server builds the index anew from the names and only then accepts requests

#### Scenario: Same order
- **WHEN** the rebuild is complete
- **THEN** the order references (ADR 0039 item 5) give the same result as before it

### Requirement: Rebuild is failure-tolerant and verified
A rebuild SHALL read the names from one snapshot, build the new index next to the old one and swap it in one atomic step; a failure in the middle of a rebuild leaves the old index untouched, and the rebuild starts over. After building, the index's record count SHALL be checked against the data record count, as well as the order of adjacent keys. Trace: CUJ5, CUJ14; ADR 0039 item 3, ADR 0058.

#### Scenario: Failure during rebuild
- **WHEN** the process is killed in the middle of a rebuild
- **THEN** at the next start the database opens, the old index is intact, the rebuild starts over

#### Scenario: Count check
- **WHEN** the number of cells of the new index is not equal to the expected one (order — the number of data records; words and exact search — the number of postings computed from the records)
- **THEN** the new index does not replace the old one, the start ends with a refusal and a message to the operator

### Requirement: Rebuild is visible to the operator
While a rebuild runs before requests are accepted, the server SHALL tell the operator the reason, the fraction done and the kind of action, and SHALL NOT give clients results of an incomplete index: connections are not accepted or get the "server starting" refusal. A rebuild requires no operator action; there is no separate command. Trace: CUJ11, CUJ13; ADR 0013, OD-4 item 8, ADR 0058 (Q-82).

#### Scenario: Progress
- **WHEN** a rebuild is running
- **THEN** the operator sees the reason, the fraction and the estimate of the remaining time

#### Scenario: Client during a rebuild
- **WHEN** a client sends a request during a rebuild
- **THEN** it does not get results from an incomplete index
