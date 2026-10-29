## ADDED Requirements

### Requirement: Memory for modified pages is bounded
The volume of modified pages not yet written to `data` (together with version chains) SHALL be at most 1 GiB (default). When the cap is reached, a checkpoint is started, and the writer waits for it to finish before the next operation: the operation is neither rejected nor lost. If the cap is reached during a backup, the backup is aborted and checkpoints resume. Trace: X-1, CUJ14, CUJ15; ADR 0097 (OD-7).

#### Scenario: The cap is reached
- **GIVEN** modified pages have taken 1 GiB
- **WHEN** a write operation arrives
- **THEN** it waits for the checkpoint to finish, then is executed and acknowledged; there is no refusal

#### Scenario: The cap during a backup
- **GIVEN** a backup is in progress, checkpoints are suspended, and the cap is reached
- **WHEN** a write operation arrives
- **THEN** the backup is aborted, checkpoints resume, and the operation waits for the checkpoint, then is acknowledged

#### Scenario: Write deadline while waiting
- **WHEN** the write deadline expired while the operation was waiting for a checkpoint before execution began
- **THEN** "request timeout", and the data is not changed

## MODIFIED Requirements

### Requirement: Checkpoints bound the WAL
The system SHALL perform a checkpoint when the WAL volume from `redo_start_lsn` reaches at least 256 MiB, after 5 minutes with unflushed pages, on clean shutdown, and at the modified-page cap ("Memory for modified pages is bounded"); checkpoints are suspended for the duration of a backup. The WAL to be recovered is at most 256 MiB plus a batch. A checkpoint page-write error leads to "read-only" or "storage error" depending on `errno`, and the WAL is not truncated. Trace: X-1, CUJ14, CUJ15; ADR 0051 item 4, 0052 item 3, 0095–0097; Q-42.

#### Scenario: By volume
- **WHEN** the WAL since the last checkpoint has reached 256 MiB
- **THEN** a checkpoint is started; after it, the segments before the new `redo_start_lsn` are released

#### Scenario: Idle
- **WHEN** 5 minutes have passed with changes and without reaching 256 MiB
- **THEN** a checkpoint is performed

#### Scenario: The checkpoint failed
- **WHEN** the checkpoint page write ended with `ENOSPC`
- **THEN** the state is "read-only" (on `EIO`, "storage error"); acknowledged operations are preserved in the WAL, new ones are rejected, and the WAL is not truncated

#### Scenario: Modified-page cap
- **WHEN** modified pages in memory have reached the cap while the WAL is under 256 MiB
- **THEN** a checkpoint is started

#### Scenario: Backup
- **WHEN** a backup is in progress
- **THEN** a new checkpoint does not begin until the backup ends; one running at the moment of the command completes before the backup number is fixed
