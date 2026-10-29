## MODIFIED Requirements

### Requirement: Opening a snapshot is cheap
Opening a snapshot SHALL be done without copying data, without waiting for the writer, and in a time independent of the database size; closing — without scanning data. Trace: X-1, CUJ5, CUJ8; ADR 0055.

#### Scenario: Open time does not depend on size
- **WHEN** snapshot opening is measured on an empty database and on a database of 10^7 records
- **THEN** the results are of the same order and do not grow with the database size

#### Scenario: Opening under write load
- **WHEN** the writer continuously executes operations
- **THEN** snapshot opening does not wait for an operation to complete

### Requirement: Latency growth during copying
The backup SHALL cap the source read rate (default 64 MiB/s, ADR 0059), read data without evicting the working set of the cache with it, and run at low priority. Search and write latencies during the backup SHALL stay within the targets Q-47, Q-58, Q-59; there must be no refusals. Trace: CUJ15; OD-3, Q-11; ADR 0016 item 3.

#### Scenario: A load check
- **WHEN** a backup runs on a database of 10^7 records under typical load
- **THEN** p99 search and write latencies are within the targets; there are no refusals caused by the backup

#### Scenario: Rate limiting
- **WHEN** a backup runs on a fast disk
- **THEN** the read rate does not exceed the set cap
