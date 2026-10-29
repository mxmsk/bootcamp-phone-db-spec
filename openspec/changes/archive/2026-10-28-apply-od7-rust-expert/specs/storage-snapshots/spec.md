## RENAMED Requirements

- FROM: `### Requirement: Backup is made from a snapshot inside the server process`
- TO: `### Requirement: Backup is made inside the server process`

- FROM: `### Requirement: Backup point is a snapshot at command receipt`
- TO: `### Requirement: Backup point is the LSN at command receipt`

## MODIFIED Requirements

### Requirement: Cap on retained volume
The engine SHALL cap the volume retained by snapshots (page versions in memory and in `retain`, pages in the free list): by default at most the larger of 1 GiB and 25% of the data, with the on-disk part at most the free space minus the write reserve of 1 GiB (ADR 0059). When the cap is exceeded, snapshots are revoked by class: short reads first, then by-number streams, and within a class the oldest first; writes SHALL NOT be rejected or slowed. Trace: CUJ1, CUJ16; ADR 0056, 0096 (OD-7).

#### Scenario: Backup under heavy writes
- **GIVEN** a backup is in progress (it creates no retention) and heavy writes, with short reads open
- **WHEN** the retained volume reaches the cap
- **THEN** the backup is not affected, reads are revoked, and writes continue to be acknowledged

#### Scenario: Streams are revoked last
- **GIVEN** the cap is reached, with a short read and a by-number stream open
- **WHEN** the engine needs to free space
- **THEN** the short read is revoked first, and the by-number stream is revoked only if that is not enough

#### Scenario: Free space
- **WHEN** free space on the volume has dropped to the write reserve
- **THEN** snapshots are revoked in the same order until space runs out, so that the database does not go "read-only" because of snapshots

### Requirement: Backup is made inside the server process
The backup SHALL be performed by the server process, the owner of the database (X-2), without a snapshot and without retaining versions: with checkpoints suspended, `data` and the WAL up to number X are copied. The server SHALL NOT stop; reads and writes proceed without refusals caused by the backup, except for an abort by the modified-page cap. At most one backup runs at a time; a second one is rejected. Trace: CUJ15, X-2; ADR 0016, ADR 0057, ADR 0096 (OD-7).

#### Scenario: A backup while the server runs
- **WHEN** a backup is in progress
- **THEN** searches and write operations are processed and acknowledged

#### Scenario: A second backup
- **GIVEN** a backup is in progress
- **WHEN** a second backup command arrives
- **THEN** it is rejected with the message «копирование уже выполняется» ("backup already in progress"), and the first is not affected

#### Scenario: Server shutdown during a backup
- **WHEN** a clean shutdown is requested during a backup
- **THEN** the backup is cancelled, the partial backup is deleted, and the shutdown does not wait for its end and remains clean

### Requirement: Backup point is the LSN at command receipt
The backup SHALL contain the state at commit number X, the LSN of the last acknowledged operation after checkpoints are stopped: every operation acknowledged before the command was received is included in full, operations after X are not included, and there are no partial ones. X and the commit time SHALL be recorded in the backup and available to the operator. Trace: CUJ15; ADR 0016, ADR 0057 (Q-83), ADR 0096 (OD-7).

#### Scenario: Acknowledged before the command
- **GIVEN** a client received a success for an operation, then the operator started a backup
- **WHEN** the backup is ready and started from
- **THEN** the operation is in the backup

#### Scenario: A write during the backup
- **WHEN** an operation is acknowledged after commit X is fixed
- **THEN** it is not in the backup, and the backup equals the state at the commit number from its description

#### Scenario: Cross-check against the operation sequence
- **WHEN** a test replays the recorded data-changing operations with a commit number not above the backup number
- **THEN** the result equals the backup contents (including for a database that survived a restart)

### Requirement: Backup includes data and indexes
The backup SHALL contain everything needed to start: `data` as is (at the last checkpoint), `wal/` with the WAL from `redo_start_lsn` to X, its own `control`, `dwb` with no occupied slots, and `LOCK`; records, versions and indexes are brought up to X by a recovery pass at start. Tokens, settings and the generation counter SHALL NOT be included. Trace: CUJ15, CUJ10; ADR 0016 item 5, ADR 0028, ADR 0057, ADR 0096 (OD-7).

#### Scenario: Start without a rebuild
- **WHEN** the server starts from a backup of the same format version
- **THEN** the indexes are used as is, and searches return the data as of the backup point

#### Scenario: Tokens are not included
- **WHEN** the backup is ready
- **THEN** it contains no tokens, settings or generation counter

#### Scenario: WAL up to X
- **GIVEN** operations were acknowledged after the source's last checkpoint, and a backup was made
- **WHEN** the server is started from the backup
- **THEN** those operations are visible, and none acknowledged after X

### Requirement: Contents of the backup control file
The system SHALL write into the backup's `control`: the source `control` as of the moment checkpoints were suspended (`db_id`, `redo_start_lsn`, WAL epoch, `next_S`, `G_run`, `E_run`), `run_state = backup` and `backup_end_lsn = X`. The backup description is checked only when `run_state = backup`. Trace: CUJ15, X-5; ADR 0051 items 5, 7; ADR 0054 items 5, 9; ADR 0057 item 13; ADR 0096 (OD-7).

#### Scenario: The backup's version counter
- **GIVEN** after restoring a backup, the largest S is 500
- **WHEN** the server is started from the backup and accepts the first operation on a number
- **THEN** the issued S is greater than 500: the number version within the new generation is never repeated, and a stale change fails the version check

#### Scenario: Backup LSN above the pages
- **GIVEN** the backup has pages with `page_lsn` up to X
- **WHEN** the server is started from the backup, an operation that changed such a page is acknowledged, and the server is killed
- **THEN** the operation's WAL entry has an LSN greater than X and is applied at recovery rather than skipped by the "`page_lsn` less than the entry LSN" rule

#### Scenario: A backup without a WAL
- **WHEN** the server starts from a backup whose WAL from `redo_start_lsn` to X is empty (there were no changes after the checkpoint)
- **THEN** the recovery pass applies nothing, "end of recovery" is recorded, the generation change is performed, and the database accepts operations

#### Scenario: An ordinary database without a backup description
- **WHEN** the server starts on an ordinary database with `run_state` `clean` or `running`
- **THEN** the backup description is neither required nor checked

#### Scenario: Recovery up to X
- **WHEN** the server starts from a backup with an empty `dwb` and a WAL up to X
- **THEN** the recovery pass applies the WAL up to X, "end of recovery" is recorded, the generation change is performed, and the database accepts operations

### Requirement: Checks during copying
The backup SHALL read and verify the checksum of every `data` page. On a mismatch, the backup SHALL NOT be created, the operator is told about the corruption, and the server goes to the "storage error" state (ADR 0014 item 6). Trace: CUJ14, CUJ15; ADR 0014 items 5–6, ADR 0057 (Q-41), ADR 0096.

#### Scenario: A corrupted page
- **GIVEN** a page on the medium is corrupted
- **WHEN** the backup reaches it
- **THEN** the backup is rejected with a corruption message, the partial backup is deleted, and there is no valid backup

#### Scenario: An intact database
- **WHEN** all pages pass the check
- **THEN** the backup is created and marked as verified

### Requirement: Backup target and location
The backup command SHALL accept a target path that does not exist or is empty; an existing non-empty target SHALL be rejected without overwriting anything. Before starting, the target's free space SHALL be checked to be at least the backup size (`data` plus the WAL up to X) plus a margin; on a shortage or a write error the backup is rejected, the partial backup is deleted, and the server keeps running. Trace: CUJ15; ADR 0016, ADR 0057 (Q-41), ADR 0096.

#### Scenario: The target is occupied
- **WHEN** the target path contains files
- **THEN** refusal, and the target contents are not changed

#### Scenario: No space
- **WHEN** there is not enough space on the target
- **THEN** refusal before starting (or, on a write error midway, an abort), and the server serves clients without refusals

### Requirement: Copying under source constraints
The backup SHALL run even when the source is in the "read-only" state, since it reads files and does not write to the source. In the "storage error" state the backup SHALL run if all pages can be read and pass the check, and otherwise be rejected. A server unavailable to the command makes no backup. Trace: CUJ15, CUJ16; ADR 0014, ADR 0028 item 7 (Q-71, Q-60); ADR 0096.

#### Scenario: A read-only database
- **GIVEN** the database is available only for reading
- **WHEN** a backup is started
- **THEN** the backup is created

#### Scenario: A stopped server
- **WHEN** the server is stopped and the operator starts a backup
- **THEN** no backup is created; a message that the server is unavailable

### Requirement: Latency growth during copying
The backup SHALL cap the source read rate (default 64 MiB/s, ADR 0059); there is no stream priority or OS cache management. Search and write latencies during the backup SHALL stay within the targets Q-47, Q-58, Q-59; there must be no refusals, except an abort of the backup by the modified-page cap. Trace: CUJ15; OD-3, Q-11; ADR 0016 item 3; ADR 0101 (OD-7).

#### Scenario: A load check
- **WHEN** a backup runs on a database of 10^7 records under typical load
- **THEN** p99 search and write latencies are within the targets; there are no refusals caused by the backup

#### Scenario: Rate limiting
- **WHEN** a backup runs on a fast disk
- **THEN** the read rate does not exceed the set cap

### Requirement: Start from a backup is a new generation
The server SHALL determine the start kind from the medium: the "backup" marker takes precedence over the clean-shutdown and crash markers. On a start from a backup, a recovery pass SHALL run up to `backup_end_lsn`, then the generation change SHALL be durably recorded before the marker is removed and before operations are accepted, then the marker is removed and `backup.desc` is deleted; a failure between steps makes the next start also a start from a backup. Trace: CUJ15, X-5; ADR 0028 items 5–6, 9–10; ADR 0096 (OD-7).

#### Scenario: The first start from a backup
- **WHEN** the server is started from a backup
- **THEN** the start kind is "start from backup", versions of earlier generations do not pass, and operations are accepted after the generation change is recorded

#### Scenario: A failure between steps
- **WHEN** the process crashed after the generation change was recorded but before the marker was removed
- **THEN** the next start is again from a backup, and the generation changes once more

#### Scenario: A repeated start from an archive
- **GIVEN** the backup archive was saved byte-for-byte before the first start
- **WHEN** the server is started a second time on a duplicate of the archive
- **THEN** it is a new generation again
