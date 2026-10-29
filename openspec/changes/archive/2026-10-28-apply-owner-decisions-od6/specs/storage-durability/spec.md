## RENAMED Requirements

- FROM: `### Requirement: Start time at 10^8 records [DEFAULT]`
- TO: `### Requirement: Start time at 10^7 records`

## REMOVED Requirements

### Requirement: Target write latency [DEFAULT]
**Reason**: OD-6
**Migration**: the requirement is replaced by "Target write latency" (values and scenarios recalculated per OD-6; the scenario names contained outdated numbers or wording).

### Requirement: Version does not require a number scan
**Reason**: OD-6
**Migration**: the requirement is replaced by "Version does not require a number scan on generation change" (values and scenarios recalculated per OD-6; the scenario names contained outdated numbers or wording).

## MODIFIED Requirements

### Requirement: Start time at 10^7 records
The system SHALL start on a database of 10^7 records, including the consistency check at start, in no more than 5 s after a clean shutdown and no more than 30 s after a crash shutdown or from a backup, without scanning
records or rebuilding indexes; a start with an index rebuild or a format conversion is not covered by these numbers. The values were accepted by the owner (OD-6, D-28; they do not depend on the number of records, so they are the same as before). Trace: CUJ14, X-1; ADR 0052 item 2; Q-42; OD-6.

#### Scenario: After a clean shutdown
- **WHEN** the server starts on a database of 10^7 records after a clean shutdown
- **THEN** it is ready to accept requests within 5 s

#### Scenario: After a crash shutdown
- **WHEN** the server starts after SIGKILL with a WAL of 256 MiB since the last checkpoint
- **THEN** it is ready to accept requests within 30 s

#### Scenario: A start with a rebuild or a format conversion
- **WHEN** a server of a new v1 version starts on a database of 10^7 records after a clean shutdown, and the index label or the format version differs
- **THEN** the 5 s and 30 s targets do not apply to this start; the rebuild or conversion is completed before requests are accepted; the rebuild time is at most 6 minutes (`storage-indexes`, "Index format label and rebuild at start")

### Requirement: Consistency check at start
The system SHALL check before accepting requests: the `control` slots and format; the database identifier in all files; the doublewrite
buffer pages; the WAL from `redo_start_lsn` (every entry); the pages read during WAL replay; the service page, the index roots and the placement metadata.
The volume of the check SHALL NOT depend on the number of records. Trace: CUJ14; ADR 0014 item 5, ADR 0053 item 1; Q-43.

#### Scenario: The check does not scan records
- **WHEN** the server starts on a database of 10^7 records
- **THEN** the number of pages read is bounded by the number of indexes and the pages touched by the WAL

#### Scenario: The database identifier does not match
- **WHEN** the database directory contains a `wal/` or `dwb` from a different database
- **THEN** the server does not start and names the file

## ADDED Requirements

### Requirement: Target write latency
The system SHALL acknowledge a write operation on a database of 10^7 records under a load of up to 100 operations per second with latency p50 ≤ 20 ms and p99 ≤ 100 ms
(with p99 of flush to stable storage ≤ 40 ms), and during a backup with p99 ≤ 250 ms and without refusals. The values were accepted by the owner (OD-6, D-27: the same numbers at 10^7). Trace: CUJ1, CUJ15, X-1; ADR 0052 item 1; Q-58, Q-11; OD-6.

#### Scenario: Write latency at 10^7
- **WHEN** on a database of 10^7 records with a cold cache, writes are executed at up to 100 per second
- **THEN** p50 ≤ 20 ms and p99 ≤ 100 ms on the reference rig

#### Scenario: During a backup
- **WHEN** a backup is running
- **THEN** write operations are not rejected, p99 ≤ 250 ms

### Requirement: Version does not require a number scan on generation change
The system SHALL store for a number only the sequence value S (u64; in the number header a varint, at most 5 bytes for S < 2^35) from a counter shared by all numbers, restore `next_S` from the WAL and `control`, and change the version generation
without changing records and without scanning numbers. Trace: X-5, X-1; ADR 0028 item 6 (g, e), ADR 0051 item 7; Q-78.

#### Scenario: A generation change at 10^7 numbers
- **WHEN** a crash recovery start on a database of 10^7 numbers
- **THEN** no number record is read or changed for the sake of the generation change

#### Scenario: The counter after a failure
- **WHEN** the server is restarted after a crash
- **THEN** `next_S` is not lower than the largest S among applied WAL entries and the value in `control`
