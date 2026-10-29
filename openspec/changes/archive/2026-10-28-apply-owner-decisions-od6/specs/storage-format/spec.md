## RENAMED Requirements

- FROM: `### Requirement: File size at 10^8 records`
- TO: `### Requirement: File size at 10^7 records`

## MODIFIED Requirements

### Requirement: File size at 10^7 records
The `data` file (without `wal/`, `dwb` and backups) of a database of 10^7 records of the reference profile P0 SHALL be at most 8 × 10^9 bytes.
Profile P0: average name 22 characters (42.1 bytes), 2.5 words per name, 1.25 records per number, inserts in random order,
the index layout of ADR 0045.
The value is an owner decision (OD-6, D-21): the scale of 10^7 and the target of ≤ 8 GB replace "10^8 and ≤ 40 GB" from brief §4 (ADR 0044 item 6). Trace: X-1; OD-6; brief §4. Decisions:
ADR 0044 (items 6–8), 0017, 0019 (item 1).

#### Scenario: A reference-profile database
- **WHEN** the test load-generation tool (Q-53) creates 10^7 records of profile P0 by inserts in random order
- **THEN** the length of `data` is at most 8 × 10^9 bytes (the calculated expectation is 4.6–5.3 GB with prefix compression of 25–45%, 6.1 GB without
  compression; ADR 0044 item 8; scaled proportionally from 10^8, to be confirmed with a prototype)

#### Scenario: Names longer than the profile
- **WHEN** a database with a different average name length is checked
- **THEN** the size is estimated by the table in ADR 0044 item 8; the 8 GB figure does not apply to such a database as a target
