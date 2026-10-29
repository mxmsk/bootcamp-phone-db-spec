# storage-snapshots Specification

## Purpose
The concurrency model of the storage engine (one writer, many readers on consistent snapshots), the implementation of the accepted
isolation (ADR 0029), snapshots for the duration of a request and of a number results stream, a backup without stopping the server with
the "backup" marker and a start from it as a new generation, and the rebuild of derived indexes on a key format change.

## ADDED Requirements

### Requirement: One writer, many readers
The engine SHALL perform all write operations in a single writer thread in one order; there may be many readers, they
run in parallel with the writer and with each other. A reader SHALL NOT hold a lock the writer waits for, and the writer SHALL NOT hold a lock a reader waits for. Trace: CUJ1, CUJ6, CUJ9, CUJ12, X-2; ADR 0029 items 1–3, ADR 0055.

#### Scenario: A reader does not wait for the writer
- **GIVEN** the writer is stopped in the middle of an operation (a test breakpoint)
- **WHEN** a search request arrives
- **THEN** it runs and returns the state before that operation, without waiting for the writer

#### Scenario: The writer does not wait for a reader
- **GIVEN** a reader holds an open snapshot and does not read further
- **WHEN** a write operation arrives
- **THEN** it runs and is acknowledged, without waiting for the reader

#### Scenario: A single order of writes
- **WHEN** write operations on different numbers arrive simultaneously
- **THEN** they are lined up in one commit order, and each is visible to readers as a whole relative to the others

### Requirement: Write operations are strictly sequential
Each acknowledged write operation SHALL get a commit number: strictly increasing, without gaps or repeats within the lifetime of the process. Operations on one number SHALL run as if in a queue; the check of the expected version and the change are one writer step, with no other write between them. Trace: CUJ9, CUJ12, X-5; ADR 0029 items 2–3.

#### Scenario: Two writes against one version
- **GIVEN** two rename operations on one number arrived with the same expected version
- **WHEN** the writer runs both
- **THEN** the first succeeds, the second gets a version conflict with the current version, the second's data is not written

#### Scenario: Commit numbers without gaps
- **WHEN** N acknowledged operations have run
- **THEN** their commit numbers go consecutively in execution order, rejected operations get no number

### Requirement: Operation is visible only after it is durable
An operation's result SHALL become visible to readers no earlier than it is durable (survives a failure), and the acknowledgment to the client SHALL be sent no earlier than the result is visible to new snapshots. A reader SHALL NOT see a change that could vanish in a failure. Trace: CUJ1, CUJ6, CUJ14; ADR 0014 item 1, ADR 0029 items 1, 4, ADR 0055.

#### Scenario: Read after write
- **WHEN** a client received the success of a write operation and then started a search
- **THEN** the search sees the operation's result in full

#### Scenario: Non-durable is not visible
- **GIVEN** an operation is written to the writer's memory but not flushed to stable storage
- **WHEN** a search request arrives
- **THEN** it does not see the operation's result

#### Scenario: Failure after a read
- **WHEN** a reader saw an operation's result and the process crashed
- **THEN** after restart the result is in place

### Requirement: One snapshot per request
Each read request (search by number, a page of search by name, a point fetch of a version) SHALL run on one snapshot: the database state after the first N acknowledged operations for a single N. The records and the number version returned by the request SHALL be from this snapshot. Trace: CUJ5, CUJ6, CUJ9, CUJ12; ADR 0029 items 4–5, ADR 0032.

#### Scenario: Version consistent with records
- **WHEN** during a search by number the writer changes the records of that number
- **THEN** the response contains the records and version of one state: either before the change or after

#### Scenario: A page by name and number versions
- **WHEN** a page of search by name reads the records of several numbers during a parallel write
- **THEN** the version of each number on the page is taken from the same snapshot as the page's records

#### Scenario: No partial operation
- **WHEN** a reader opens a snapshot at any moment of the writer's work
- **THEN** it does not see an operation partially: it sees the record and the version change together or sees neither

### Requirement: Snapshot is not kept between pages
The engine SHALL NOT keep a snapshot or cursor state between requests: the next page of search by name reads the state as of its own request. Consistency between pages SHALL NOT be promised. Trace: CUJ5, CUJ12; ADR 0029 item 5, ADR 0032.

#### Scenario: A write between pages
- **GIVEN** a client received the first page of results
- **WHEN** a write is acknowledged before the second page is requested
- **THEN** the second page reads the new state, and the client holds nothing of the earlier one

#### Scenario: The snapshot is released after the response
- **WHEN** the response for a page has been sent
- **THEN** the snapshot of that request is released

### Requirement: Opening a snapshot is cheap
Opening a snapshot SHALL be done without copying data, without waiting for the writer and in a time independent of the database size; closing it — without scanning data. Trace: X-1, CUJ5, CUJ8; ADR 0055.

#### Scenario: Open time does not depend on size
- **WHEN** snapshot opening is measured on an empty database and on a database of 10^8 records
- **THEN** the results are of the same order and do not grow with the database size

#### Scenario: Opening under write load
- **WHEN** the writer continuously runs operations
- **THEN** opening a snapshot does not wait for an operation to finish

### Requirement: Snapshot retains earlier page versions
While a snapshot with number N is open, the engine SHALL keep the data needed to read state N and not reuse it; after the last snapshot that needs them is closed, the space
SHALL return to circulation. What is retained: pages removed from the trees after N, and earlier published versions of pages changed after N: in memory, beyond the budget (by default
256 MiB) in the temporary file `retain` (D-35). Trace: CUJ5, CUJ15; ADR 0056 item 1.

#### Scenario: A reader sees its own state
- **GIVEN** a snapshot is open, then writes changed and deleted the same records
- **WHEN** the reader continues reading
- **THEN** it sees the state as of the opening

#### Scenario: A page rewritten by a checkpoint
- **GIVEN** a snapshot is open, then a page is changed twice by operations, and a checkpoint wrote its latest version to `data`
- **WHEN** the snapshot's reader reads this page
- **THEN** it sees the version as of the opening: the `data` file is not needed for the snapshot, the writer's unpublished versions are not part of the retention

#### Scenario: A freed page
- **GIVEN** a snapshot is open, then an operation removed a page from the tree
- **WHEN** a page is requested for new data
- **THEN** the page is not handed out while a snapshot that sees it is alive (ADR 0044 item 2), and is handed out after it closes

#### Scenario: Retention beyond the memory budget
- **GIVEN** a snapshot is open, and writes changed pages in a volume larger than the retention memory budget
- **WHEN** the reader reads the changed pages
- **THEN** it sees the state as of the opening, part of the earlier versions lies in the temporary file `retain`

#### Scenario: Return of space
- **WHEN** all snapshots that retained earlier versions are closed
- **THEN** the retained volume becomes zero, memory and space in the temporary file are freed, and the freed pages are used again

#### Scenario: A temporary file error
- **WHEN** a write to the temporary retention file ended with `ENOSPC`
- **THEN** snapshots are revoked in the order of the requirement "Cap on retained volume", the database files are untouched, the database state does not change, write operations are acknowledged

### Requirement: Snapshot is always released
A snapshot SHALL be released on request completion, error, cancellation, client connection drop and handler panic. A snapshot leak SHALL be considered a defect: the retained volume and the number of open snapshots are available to tests and the server log. Trace: CUJ5, CUJ8, CUJ15; ADR 0056.

#### Scenario: Connection drop
- **WHEN** a client dropped the connection in the middle of results
- **THEN** the snapshot is released immediately, and if the drop is not noticed — no later than the timeout under the requirement "Snapshot retention ceilings" (the short-read ceiling, stream idle)

#### Scenario: Handler panic
- **WHEN** a request handler ended with a panic
- **THEN** the number of open snapshots decreases, as on normal completion

### Requirement: Snapshot retention ceilings
A short read SHALL have a timeout ceiling of 30 s (default, D-30; not extended by client activity): when it expires the snapshot is revoked, and the read is interrupted with a "snapshot revoked" error.
A number stream SHALL NOT have a general ceiling and is revoked only on client idle (does not accept data for 120 s) and, at the retention cap, last; revocation is a storage read refusal without
the server entering "storage error". Trace: CUJ4, CUJ8, X-1; ADR 0035 item 6, ADR 0056.

#### Scenario: A slow but reading stream client
- **GIVEN** a client accepts number history slowly but continuously, and the total delivery time exceeds 120 s
- **WHEN** delivery continues, and the retained volume is at the cap
- **THEN** the stream is not revoked, delivery completes in full with the completeness marker

#### Scenario: A stream client stopped reading
- **GIVEN** a stream client does not accept data
- **WHEN** 120 s of idle have passed
- **THEN** the snapshot is revoked, the stream is cut off without the final completeness marker (`number-search`), writes are not delayed

#### Scenario: A short read did not finish
- **WHEN** a short read did not finish within 30 s
- **THEN** the snapshot is revoked, the read is interrupted with a "snapshot revoked" error with no partial result that looks complete

#### Scenario: A fast request
- **WHEN** a request finished before the ceiling
- **THEN** the snapshot is released on completion, the ceiling is not used

### Requirement: By-number results are streamed on one snapshot
Reading a number's history (up to 2,967,164 records) SHALL run on one snapshot opened before the first record is sent; the records and the version SHALL be from it, the engine's memory for the results SHALL be bounded by the chunk size and independent of the number of records. There is no truncation or continuation. Trace: CUJ4, CUJ6, CUJ8, CUJ9; ADR 0007, ADR 0035 item 6.

#### Scenario: A write to the number during results
- **GIVEN** a history of a number with many records is being returned
- **WHEN** a record is added to this number
- **THEN** the results do not contain it, the version is as of the start, the write is acknowledged without waiting for the end of the results

#### Scenario: Memory does not grow with history
- **WHEN** a history of 10^6 records and of 10 records is returned
- **THEN** the engine's peak memory for the results is of the same order

#### Scenario: Completed results
- **WHEN** all records of the number are delivered without stream revocation
- **THEN** the engine reports full completion to the API layer (the basis of the end marker, ADR 0035 item 6)

### Requirement: Cap on retained volume
The engine SHALL bound the volume retained by snapshots (page versions in memory and in `retain`, pages waiting in the free list): by default no more than the larger of 1 GiB and 25% of the data,
the on-disk part — no more than the free space minus the write reserve of 1 GiB (ADR 0059). When it is exceeded snapshots are revoked by class: backup, short reads, number streams,
within a class the oldest first; writes SHALL NOT be rejected or slowed. Trace: CUJ1, CUJ15, CUJ16; ADR 0056.

#### Scenario: Backup under high write rate
- **GIVEN** a backup and writes at a rate such that the retained volume reaches the cap
- **WHEN** the cap is exceeded
- **THEN** the backup is revoked with an error, writes continue to be acknowledged

#### Scenario: Streams are revoked last
- **GIVEN** the cap is reached, a backup, a short read and a number stream are open
- **WHEN** the engine needs to free space
- **THEN** the backup is revoked first, then the short read, the number stream is revoked only if that is not enough

#### Scenario: Free space
- **WHEN** free space on the volume fell to the write reserve
- **THEN** snapshots are revoked in the same order until the space is exhausted, so the database does not go "read-only" because of snapshots

### Requirement: Snapshots live only in process memory
A snapshot and the data it retains SHALL NOT persist between runs; after any restart there are no open snapshots, the retention space is returned to circulation at start without scanning the whole database. Trace: CUJ14; ADR 0056.

#### Scenario: Restart with an open snapshot
- **GIVEN** the process is killed with a snapshot open
- **WHEN** the server is started again
- **THEN** there are no snapshots, the database is consistent, the temporary retention file is deleted, no extra space is retained

### Requirement: Backup is made from a snapshot inside the server process
Copying SHALL be done by the server process, the database owner (X-2), from the snapshot opened at command receipt; the server SHALL NOT stop, and read and write requests SHALL be handled without refusals caused by the copying. At most one backup runs at a time; a second command is rejected. Trace: CUJ15, X-2; ADR 0016, ADR 0057.

#### Scenario: Backup while the server runs
- **WHEN** a backup is in progress
- **THEN** searches and write operations are handled and acknowledged

#### Scenario: A second backup
- **GIVEN** a backup is in progress
- **WHEN** a second backup command arrives
- **THEN** it is rejected with the message «копирование уже выполняется» ("backup already in progress"), the first is unaffected

#### Scenario: Server shutdown during a backup
- **WHEN** a clean shutdown is requested during a backup
- **THEN** the backup is cancelled, the partial backup is deleted, the shutdown does not wait for its end and remains clean

### Requirement: Backup point is the snapshot at command receipt
A backup SHALL contain the state on the snapshot opened at command receipt: every operation acknowledged to a client before the command was received is included in full; operations after the snapshot are not included; there are no partial operations. The commit number and the snapshot time SHALL be recorded in the backup and available to the operator. Trace: CUJ15; ADR 0016, ADR 0057 (Q-83).

#### Scenario: Acknowledged before the command
- **GIVEN** a client received the success of an operation, then the operator started a backup
- **WHEN** the backup is ready and started
- **THEN** the operation is in the backup

#### Scenario: A write during the backup
- **WHEN** an operation is acknowledged after the backup snapshot was opened
- **THEN** it is not in the backup, and the backup equals the state at the commit number from its description

#### Scenario: Checking against the operation sequence
- **WHEN** a test replays the recorded operations up to the backup's commit number
- **THEN** the result equals the backup's contents

### Requirement: Backup includes data and indexes
A backup SHALL contain everything needed to start without a rebuild: the records, versions and indexes of one snapshot (`data`, read on the snapshot), as well as its own `control`, a `dwb` without occupied slots, `wal/` with one
zero segment without entries, and `LOCK`. Tokens, server settings and the generation counter SHALL NOT be included in the backup. Trace: CUJ15, CUJ10; ADR 0016 item 5, ADR 0028, ADR 0057.

#### Scenario: Start without a rebuild
- **WHEN** the server starts on a backup of the same format version
- **THEN** the indexes are used as they are, searches return the data of the backup point

#### Scenario: Tokens are not included
- **WHEN** a backup is ready
- **THEN** it contains no tokens, settings or generation counter

### Requirement: Contents of the backup control file
The system SHALL write to the backup's `control` `run_state = backup`, the source's `db_id`, `next_S` no less than the largest S in the snapshot headers, the WAL start LSN and `redo_start_lsn` no less than
the largest `page_lsn` of the copied pages, a WAL epoch no less than the source's epoch, and the source's `G_run`, `E_run`. The backup description is checked only when `run_state = backup`.
Trace: CUJ15, X-5; ADR 0051 items 5, 7; ADR 0054 items 5, 9; ADR 0057 item 13.

#### Scenario: Backup version counter
- **GIVEN** the largest S in the snapshot's headers is 500
- **WHEN** the server is started on the backup and accepts the first operation on a number
- **THEN** the issued S is greater than 500: the number version within the new generation is not repeated, a stale change does not pass the version check

#### Scenario: Backup LSN above the pages
- **GIVEN** the backup has pages with `page_lsn` up to L
- **WHEN** the server is started on the backup, an operation that changed such a page is acknowledged, and the server is killed
- **THEN** the operation's WAL entry has an LSN greater than L and is applied during recovery, not skipped by the rule "`page_lsn` is less than the entry's LSN"

#### Scenario: A backup without a WAL
- **WHEN** the server starts on a backup with an empty `dwb` and a zero WAL segment
- **THEN** the recovery pass applies nothing, "recovery end" is written, the generation change is done, the database accepts operations

#### Scenario: An ordinary database without a backup description
- **WHEN** the server starts on an ordinary database with `run_state` `clean` or `running`
- **THEN** the backup description is not required and not checked

### Requirement: Checks during copying
Copying SHALL read and verify the checksum of every snapshot page. On a mismatch the backup SHALL NOT be created, the operator is told about the corruption, and the server enters the "storage error" state (ADR 0014 item 6). Trace: CUJ14, CUJ15; ADR 0014 items 5–6, ADR 0057 (Q-41).

#### Scenario: A corrupted page
- **GIVEN** a page on the medium is corrupted
- **WHEN** copying reaches it
- **THEN** the backup is rejected with a corruption message, the partial backup is deleted, there is no valid backup

#### Scenario: An intact database
- **WHEN** all pages pass the check
- **THEN** the backup is created and marked verified

### Requirement: Backup does not exist until complete
A backup SHALL be written to a temporary location and become a backup in the last step: data and directory flushed, then the description atomically written (the "backup" marker, the format version, the commit
number, the snapshot time, the sizes and checksums of files). A directory with `run_state = backup` without a complete description SHALL NOT be started (the refusal "backup incomplete"). Trace: CUJ14, CUJ15; ADR 0057 (Q-41).

#### Scenario: Failure in the middle of copying
- **WHEN** the process or power failed before the description was written
- **THEN** on starting the server on this directory the refusal is "backup incomplete" (if `control` is also absent — "not a database"); the source database is untouched

#### Scenario: A completed backup
- **WHEN** the description is written
- **THEN** the directory starts as a backup

### Requirement: Backup target and location
The backup command SHALL accept a target path that does not exist or is empty; an existing non-empty target SHALL be rejected without overwriting anything. Before starting, the free space of the target SHALL be checked to be no less than the snapshot size plus a margin; on a shortage or write error the backup is rejected, the partial backup is deleted, the server keeps working. Trace: CUJ15; ADR 0016, ADR 0057 (Q-41).

#### Scenario: Target is busy
- **WHEN** the target path contains files
- **THEN** refusal, the target contents are unchanged

#### Scenario: Out of space
- **WHEN** the target lacks space
- **THEN** refusal before the start (or, on a write error in progress, an abort), the server without refusals to clients

### Requirement: Copying under source constraints
Copying SHALL run also when the source is "read-only", since it reads a snapshot and does not write to the source. Under a "storage error" copying SHALL run if all pages are readable and pass the check, otherwise it is rejected. A server unavailable to the command makes no backup: the files of a stopped database do not count as a backup. Trace: CUJ15, CUJ16; ADR 0014, ADR 0028 item 7 (Q-71, Q-60).

#### Scenario: A read-only database
- **GIVEN** the database is available for reading only
- **WHEN** a backup is started
- **THEN** the backup is created

#### Scenario: A stopped server
- **WHEN** the server is stopped and the operator starts a backup
- **THEN** no backup is created; a message that the server is unavailable

### Requirement: Latency growth during copying
Copying SHALL limit the source read speed (by default 64 MiB/s, ADR 0059), read data without evicting the cache working set with it, and run at low priority. Search and write latencies during copying SHALL stay within the targets of Q-47, Q-58, Q-59; there must be no refusals. Trace: CUJ15; OD-3, Q-11; ADR 0016 item 3.

#### Scenario: Load check
- **WHEN** a backup runs on a database of 10^8 records under typical load
- **THEN** the p99 of search and write latencies is within targets; there are no refusals caused by the copying

#### Scenario: Speed limit
- **WHEN** copying runs on a fast disk
- **THEN** the read speed does not exceed the set cap

### Requirement: "Backup" marker in the backup itself
A backup SHALL carry the "backup" marker in the backup description and in the backup's `control` (not in `data`); in the source database the marker SHALL NOT appear. The marker is preserved when the backup directory is duplicated bytewise and is cleared only when the server starts on it. Trace: CUJ15, X-5; ADR 0028 items 5, 7.

#### Scenario: A copy of a backup
- **GIVEN** the backup directory is duplicated unchanged
- **WHEN** the server is started on the duplicate
- **THEN** the start goes as on a backup

#### Scenario: The source database without a marker
- **WHEN** a backup is made
- **THEN** the source database has no "backup" marker, planned restarts go as before

### Requirement: Start from a backup is a new generation
The server SHALL determine the start kind from the medium: the "backup" marker overrides the clean-shutdown marker and the crash marker. On a start from a backup the version generation change SHALL be durably recorded before the marker is cleared and before operations are accepted, then the marker is cleared durably; a failure between the steps makes the next start also a new generation. Trace: CUJ15, X-5; ADR 0028 items 5–6, 9–10.

#### Scenario: The first start from a backup
- **WHEN** the server is started on a backup
- **THEN** the start kind is "start from backup", versions of earlier generations do not pass, operations are accepted after the generation change is recorded

#### Scenario: Failure between steps
- **WHEN** the process crashed after the generation change was recorded but before the marker was cleared
- **THEN** the next start is again from a backup, the generation changes once more

#### Scenario: A repeated start from an archive
- **GIVEN** a backup archive is saved bytewise before the first start
- **WHEN** the server is started a second time on a duplicate of the archive
- **THEN** it is again a new generation

### Requirement: Backup of the earlier v1 format
The backup description SHALL contain the format version and the derived-index format labels. A server of a new version within v1 SHALL open a backup of an earlier v1 version itself, without an operator command; a label mismatch triggers a rebuild under the requirement "Rebuild of derived indexes on format change". Trace: CUJ11, CUJ15; ADR 0013, ADR 0016 item 6, ADR 0039 item 3.

#### Scenario: A backup from before an upgrade
- **WHEN** a server of a new v1 version is started on a backup of an earlier one
- **THEN** the database is opened without a command; if the index labels differ, they are rebuilt before accepting requests

### Requirement: Rebuild of derived indexes on format change
Each index derived from the name (order, words, exact search) SHALL carry a format label (the key format version, the `icu_collator` version, the word table version, the hash identifier; ADR 0045 item 7); when the label does not match the running server the index SHALL be rebuilt from the stored names before accepting requests. The order and matches before and after SHALL be the same. Trace: CUJ5, CUJ11; ADR 0036, ADR 0039 items 3, 6; ADR 0058 (Q-82).

#### Scenario: Matching labels
- **WHEN** the index labels equal the server's labels
- **THEN** there is no rebuild, the start has no delay for it

#### Scenario: A mismatched label
- **WHEN** the order index label differs
- **THEN** before accepting requests the server builds the index anew from the names and only then accepts requests

#### Scenario: The same order
- **WHEN** the rebuild is complete
- **THEN** the order reference tests (ADR 0039 item 5) give the same result as before it

### Requirement: Rebuild is failure-tolerant and verified
A rebuild SHALL read names from one snapshot, build the new index next to the old one and swap in one atomic step; a failure in the middle of a rebuild leaves the earlier index untouched, the rebuild starts anew. After building, the number of index entries SHALL be checked against the number of data records and the order of adjacent keys. Trace: CUJ5, CUJ14; ADR 0039 item 3, ADR 0058.

#### Scenario: Failure during a rebuild
- **WHEN** the process is killed in the middle of a rebuild
- **THEN** at the next start the database opens, the earlier index is intact, the rebuild starts anew

#### Scenario: Count check
- **WHEN** the number of cells of the new index is not equal to the expected one (order — the number of data records; words and exact search — the number of postings computed from the records)
- **THEN** the new index does not replace the old one, the start ends with a refusal and a message to the operator

### Requirement: Rebuild is visible to the operator
While a rebuild runs before accepting requests, the server SHALL tell the operator the reason, the share done and the kind of action, and SHALL NOT give clients results of an incomplete index: connections are not accepted or get a "server starting" refusal. A rebuild requires no operator action; there is no separate command. Trace: CUJ11, CUJ13; ADR 0013, OD-4 item 8, ADR 0058 (Q-82).

#### Scenario: Progress
- **WHEN** a rebuild is in progress
- **THEN** the operator sees the reason, the share and an estimate of the remaining time

#### Scenario: A client during a rebuild
- **WHEN** a client sends a request during a rebuild
- **THEN** it does not get results of an incomplete index
