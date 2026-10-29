## MODIFIED Requirements

### Requirement: Zero date when no date is given
The system SHALL substitute 1876-03-07 if no date is passed on add and the number has no records (otherwise — "unchanged" for a record on 1876-03-07 with the same name, or "number known", `write-operations`, OD-8); an explicit 1876-03-07 is the same key, and the two cannot be told apart after saving. On rename and delete the date is mandatory: its absence is a value error (date, "missing"), and there is no substitution (OD-5, Q-90; ADR 0025 item 6). Trace: CUJ1–CUJ3, CUJ6, CUJ9; OD-6, OD-8; ADR 0002.

#### Scenario: No date passed on add
- **WHEN** a record is added without a date and the number has no records
- **THEN** the record is stored with the date 1876-03-07

#### Scenario: No date passed, number known
- **WHEN** a record is added without a date and the number has a record on any date
- **THEN** no record is created and there is no substitution: "number known" or, if there is a record on 1876-03-07 with the same name, "unchanged" (`write-operations`)

#### Scenario: No date passed on rename or delete
- **WHEN** a rename or delete is sent without a date
- **THEN** a value error (date, "missing"); data is not changed, and the record on 1876-03-07 is not touched

#### Scenario: Explicit zero date on rename and delete
- **WHEN** a rename or delete is sent with an explicit date of 1876-03-07
- **THEN** the date is valid and the operation applies to the record (number, 1876-03-07)

#### Scenario: Explicit and substituted dates are one key
- **GIVEN** there is a record (number, substituted 1876-03-07) → "Ann"
- **WHEN** (the same number, explicit 1876-03-07) → "Ann" is added
- **THEN** an identical record, and data is not changed
