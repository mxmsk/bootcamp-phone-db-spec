## MODIFIED Requirements

### Requirement: Write operations are strictly sequential
Every acknowledged write operation that changed data SHALL get a commit number — the LSN of its WAL entry: strictly increasing, continuous across restarts, without repeats; "unchanged" operations and rejected ones get no number and are not written to the WAL. Operations on one number SHALL execute as if in a queue; the expected-version check and the change are one writer step, with no other write between them. Trace: CUJ9, CUJ12, X-5; ADR 0029 items 2–3.

#### Scenario: Two writes against one version
- **GIVEN** two rename operations on one number arrived with the same expected version
- **WHEN** the writer executes both
- **THEN** the first succeeds, the second gets a version conflict with the current version, the second's data is not written

#### Scenario: Commit numbers without gaps
- **WHEN** N acknowledged data-changing operations were executed, among them an identical retry and a rename to the same name
- **THEN** every changing operation got a number, the numbers strictly increase in execution order, including after a restart; "unchanged" and rejected operations got no number

### Requirement: One snapshot per request
Every read request (number search, a name-search page, a point version lookup) SHALL execute on one snapshot: the database state after all acknowledged operations with a commit number not greater than one value C. The records and number version returned by the request SHALL be from that snapshot. Trace: CUJ5, CUJ6, CUJ9, CUJ12; ADR 0029 items 4–5, ADR 0032.

#### Scenario: Version consistent with records
- **WHEN** while a number search is executing the writer changes this number's records
- **THEN** the response contains the records and version of one state: either before the change or after it

#### Scenario: Name-search page and number versions
- **WHEN** a name-search page reads the records of several numbers during a concurrent write
- **THEN** the version of each number on the page is taken from the same snapshot as the page's records

#### Scenario: No partial operation
- **WHEN** a reader opens a snapshot at any moment of the writer's work
- **THEN** it does not see an operation partially: it sees the record and the version change together or sees neither

### Requirement: Snapshot retains earlier page versions
While a snapshot with number N is open, the engine SHALL keep the data needed to read state N and not reuse it; after the last snapshot that needs it is closed, the space
SHALL return to circulation. Retained are pages removed from the trees after N and earlier published versions of pages changed after N: in memory, beyond the budget (by default
256 MiB) in the temporary file `retain` (D-35). Trace: CUJ5, CUJ15; ADR 0056 item 1.

#### Scenario: Reader sees its own state
- **GIVEN** a snapshot is open, then writes changed and deleted the same records
- **WHEN** the reader continues reading
- **THEN** it sees the state as of the opening

#### Scenario: Page rewritten by a checkpoint
- **GIVEN** a snapshot is open, then a page was changed twice by operations, and a checkpoint wrote its latest version to `data`
- **WHEN** the snapshot's reader reads this page
- **THEN** it sees the version as of the opening: the `data` file is not needed for the snapshot, the writer's unpublished versions are not part of the retention

#### Scenario: Freed page
- **GIVEN** a snapshot is open, then an operation removed a page from the tree
- **WHEN** a page allocation for new data is requested
- **THEN** the page is not allocated while a snapshot that sees it is alive (ADR 0044 item 2), and is allocated after it closes

#### Scenario: Retention beyond the memory budget
- **GIVEN** a snapshot is open, and writes changed pages by more than the retention memory budget
- **WHEN** the reader reads the changed pages
- **THEN** it sees the state as of the opening, some earlier versions lie in the temporary file `retain`

#### Scenario: Returning space
- **WHEN** all snapshots that retained earlier versions are closed
- **THEN** the retained volume becomes zero, memory and space in the temporary file are freed, and the freed pages are used again

#### Scenario: Temporary file error
- **WHEN** a write to the retention temporary file ended with any I/O error (`ENOSPC`, `EIO`, …)
- **THEN** snapshots are revoked in the order of the "Cap on retained volume" requirement, the database files are not affected, the database state does not change, write operations are acknowledged

### Requirement: Backup point is a snapshot at command receipt
A backup SHALL contain the state on the snapshot opened at command receipt: every operation acknowledged to a client before command receipt is included in full; operations after the snapshot are not included; there are no partial operations. The commit number and the snapshot time SHALL be recorded in the backup and available to the operator. Trace: CUJ15; ADR 0016, ADR 0057 (Q-83).

#### Scenario: Acknowledged before the command
- **GIVEN** a client got success of an operation, then the operator started a backup
- **WHEN** the backup is ready and started
- **THEN** the operation is in the backup

#### Scenario: Write during the backup
- **WHEN** an operation is acknowledged after the backup snapshot was opened
- **THEN** it is not in the backup, and the backup equals the state at the commit number from its descriptor

#### Scenario: Check against the operation sequence
- **WHEN** a test replays the recorded data-changing operations with a commit number not greater than the backup's number
- **THEN** the result equals the backup contents (including for a database that survived a restart)

### Requirement: Contents of the backup control file
The system SHALL record in the backup's `control` `run_state = backup`, the source `db_id`, `next_S` strictly greater than the largest S in the snapshot headers, the WAL start LSN and a `redo_start_lsn` not less than
the largest `page_lsn` of the copied pages, a WAL epoch not less than the source's epoch, and the source's `G_run`, `E_run`. The backup descriptor is checked only when `run_state = backup`.
Trace: CUJ15, X-5; ADR 0051 items 5, 7; ADR 0054 items 5, 9; ADR 0057 item 13.

#### Scenario: Backup version counter
- **GIVEN** the largest S in the headers in the snapshot is 500
- **WHEN** the server is started on the backup and accepts the first operation on a number
- **THEN** the issued S is greater than 500: the number version within the new generation is never repeated, a stale change fails the version check

#### Scenario: Backup LSN above the pages
- **GIVEN** the backup has pages with `page_lsn` up to L
- **WHEN** the server is started on the backup, an operation that changed such a page is acknowledged, and the server is killed
- **THEN** the operation's WAL entry has an LSN greater than L and is applied during recovery, not skipped by the rule "`page_lsn` is less than the entry's LSN"

#### Scenario: Backup without a WAL
- **WHEN** the server starts on a backup with an empty `dwb` and a zero WAL segment
- **THEN** the recovery pass applies nothing, "end of recovery" is recorded, the generation change is performed, the database accepts operations

#### Scenario: Ordinary database without a backup descriptor
- **WHEN** the server starts on an ordinary database with `run_state` `clean` or `running`
- **THEN** the backup descriptor is not required and not checked

### Requirement: Backup does not exist until complete
A backup SHALL be written to a temporary place and become a backup by the last step: data and the directory are flushed, then the descriptor file `backup.desc` is written atomically (a temporary file and `rename`) in the backup directory (the "backup" marker, format version, index labels, commit number, snapshot time, file sizes and checksums; `key=value` lines and a total checksum). A directory with `run_state = backup` without a complete and valid `backup.desc` SHALL NOT be started (refusal "backup incomplete"). Trace: CUJ14, CUJ15; ADR 0057 (Q-41).

#### Scenario: Crash in the middle of a backup
- **WHEN** the process or power failed before the descriptor was written
- **THEN** on starting the server on this directory, refusal "backup incomplete" (if there is no `control` either — "not a database"); the source database is not affected

#### Scenario: Completed backup
- **WHEN** the descriptor is written
- **THEN** the directory starts as a backup

### Requirement: Start from a backup is a new generation
The server SHALL determine the start kind from the medium: the "backup" marker overrides the clean-shutdown and crash markers. On a start from a backup, the version generation change SHALL be durably written before the marker is cleared and before operations are accepted, then the marker is cleared durably and `backup.desc` is deleted; a crash between the steps makes the next start also a new generation, and a leftover descriptor file is deleted by the start. Trace: CUJ15, X-5; ADR 0028 items 5–6, 9–10.

#### Scenario: First start from a backup
- **WHEN** the server is started on a backup
- **THEN** the start kind is "start from backup", versions of earlier generations do not pass, operations are accepted after the generation change is written

#### Scenario: Crash between steps
- **WHEN** the process crashed after the generation change was written but before the marker was cleared
- **THEN** the next start is again from a backup, the generation changes once more

#### Scenario: Repeated start from an archive
- **GIVEN** a backup archive is saved bytewise before the first start
- **WHEN** the server is started a second time on a copy of the archive
- **THEN** it is again a new generation
