## MODIFIED Requirements

### Requirement: Adding a record
The system SHALL, for an add operation with inputs "number", "name" and an optional "date", create the record (number, date) →
name if there is no record on this (number, date); an expected version is not required. Without a date: if the number has no records, 1876-03-07 is taken (`records`); if there is a record on 1876-03-07 with the same name — "unchanged"; otherwise — "number known" ("Adding without a date for a known number"). Trace: CUJ1, CUJ2, CUJ9; ADR 0002, 0005, 0025, 0026; OD-6, OD-8.

#### Scenario: A new number
- **WHEN** a record is added with a number that has no records
- **THEN** the record is created, the number is found by name, and the name is found by number (CUJ1)
- **AND** the number version changed from the initial version of the current generation to a different one

#### Scenario: A known number, a new date
- **WHEN** a record is added on a date the number does not have yet
- **THEN** the record is created, the number's records on other dates are not changed, and the number version changed (CUJ2)

#### Scenario: The same name on a different date
- **WHEN** a name that already owns the number on one date is added to the same number on another
- **THEN** a separate record is created; both records appear in the number history

#### Scenario: No date given
- **WHEN** an add is performed without a date and the number has no records
- **THEN** the record is created on 1876-03-07; an explicitly given 1876-03-07 yields the same record

#### Scenario: No date given, number known
- **GIVEN** the number has a record on 2020-01-01
- **WHEN** an add is performed without a date
- **THEN** "number known"; no record is created, and the data and the number version are not changed

#### Scenario: No date given, a retry on 1876-03-07
- **GIVEN** the number has a record on 1876-03-07 "Ann" (and possibly other records)
- **WHEN** an add of "Ann" is performed without a date
- **THEN** success "unchanged"; the data and the number version are not changed (OD-8)

### Requirement: Write error kinds
The system SHALL distinguish five kinds of errors on write: "invalid input" (naming the field: number, date, name, expected
version), "name conflict" (add with a date), "number known" (add without a date, the number has records, but none identical on 1876-03-07), "version conflict" (rename, delete) and "not found" (rename, delete);
on any error, data and versions do not change. Trace: CUJ1, CUJ2, CUJ3, CUJ6, CUJ9, CUJ12; OD-6, OD-8; ADR 0025.

#### Scenario: Contents of a version conflict
- **WHEN** the expected version does not equal the current number version
- **THEN** "version conflict"; the response contains the current number version (CUJ9, CUJ12)

#### Scenario: Contents of "number known"
- **WHEN** an add without a date received "number known"
- **THEN** the response contains the current number version; the current name is not reported (there are several records)

#### Scenario: Invalid input is named by field
- **WHEN** the number has an invalid shape, or the date is missing (rename, delete), or the date is not a calendar day (for example, 1875-02-30), or the date is outside the range
  1876-03-07…9999-12-31 (for example, 1876-03-06 or 10000-01-01 when the year is written with five digits), or the name is invalid
- **THEN** "invalid input" with a (field, reason) pair from `records`, "Value error kind dictionary": (date,
  "missing"), (date, "nonexistent day"), (date, "out of range"), etc.; the reason dictionary is not duplicated here; the "expected
  version" field is this capability's own field and is not part of the `records` dictionary; data is not changed

#### Scenario: An error changes nothing
- **WHEN** an operation ended with any write error
- **THEN** the number version and all its records are in the same state as before the operation

#### Scenario: Other kinds of refusals are out of scope
- **WHEN** a request is rejected because of permissions (CUJ8), the request limit, or the database state "read-only"/"storage error"
  (CUJ16)
- **THEN** these are other kinds of refusals, defined by other areas; data is not changed

### Requirement: Check order on write
The system SHALL check an operation in this order: input (number; date — mandatory for rename and delete, not for add; name; expected version — presence and kind, for rename and delete; the first invalid field is reported); for rename and delete — version ("version conflict"), then record existence ("not found"); for an add with a date — occupancy of (number, date), without a date — an identical record on 1876-03-07 ("unchanged"), otherwise the number's records ("number known"). Access and database state come earlier. Trace: CUJ1–CUJ3, CUJ9, CUJ12; OD-8; ADR 0025.

#### Scenario: A stale version and a missing record
- **WHEN** a record that does not exist is renamed or deleted with a stale version
- **THEN** "version conflict" with the current version, not "not found"

#### Scenario: The record vanished because of someone else's delete
- **WHEN** the record was deleted concurrently, and a client renames or deletes it with a version read earlier
- **THEN** "version conflict" (the delete changed the number version), not "not found" (CUJ12)

#### Scenario: The current version and a missing record
- **WHEN** the record is absent and the version is current
- **THEN** "not found"

#### Scenario: The same name with a stale version
- **WHEN** a rename to the same name is sent with a stale version
- **THEN** "version conflict"; success "unchanged" is returned only with a current version (OD-4, item 7)

#### Scenario: Invalid input before version
- **WHEN** a request contains both an invalid name and a stale version
- **THEN** "invalid input" (field "name"), not "version conflict"

#### Scenario: Several invalid fields
- **WHEN** both the number and the name are invalid
- **THEN** the field "number" is reported (order: number, date, name, expected version)

#### Scenario: A missing date comes before name and version
- **WHEN** a rename is sent without a date, with an invalid name and without an expected version
- **THEN** "invalid input" with the pair (date, "missing"): the date is checked before the name and the expected version

#### Scenario: A missing date comes before a version conflict
- **WHEN** a delete is sent without a date and with a stale version
- **THEN** "invalid input" (date, "missing"), not "version conflict": input is checked before version

#### Scenario: An add without a date
- **WHEN** an add is performed without a date and the number has no records
- **THEN** there is no date error: 1876-03-07 is taken (`records`, "Zero date when no date is given")

#### Scenario: Input before "number known"
- **WHEN** an add without a date contains an invalid name and the number is known
- **THEN** "invalid input", not "number known"

#### Scenario: An add with a version of the wrong kind
- **WHEN** an add is received with an expected version that does not belong to the version type (wrong kind), or with a version
  of the right kind not issued by the server (an add's expected version is not checked at all)
- **THEN** the version is ignored, neither "invalid input" nor "version conflict" is returned; the operation is processed
  in the ordinary way

#### Scenario: Input before occupancy
- **WHEN** an add contains an invalid name for an occupied (number, date)
- **THEN** "invalid input", not "name conflict"

### Requirement: Retrying an operation after an unknown outcome
The system SHALL, when a write operation whose result the client did not receive is retried, produce no second effect if the number's records have not changed: an add with a date and an add without a date with a record on 1876-03-07 with the same name (OD-8) — "unchanged"; a delete and a rename that changed the name, with the earlier version — "version conflict"; a rename to the same name — "unchanged". If the records changed, the ordinary rules apply. Trace: CUJ1, CUJ9, CUJ12, CUJ14; OD-6, OD-8; ADR 0026.

#### Scenario: A retry of an add without a date
- **WHEN** an add without a date of a new number was performed, the response was lost, and the client retries it
- **THEN** success "unchanged" with the number version (OD-8); there is one record, and the retry is safe

#### Scenario: A retry without a date after records changed
- **WHEN** after an add without a date, the record on 1876-03-07 was deleted or renamed, and the client retries the add
- **THEN** the ordinary rules apply: a number with no records — the record is created; otherwise "number known"

#### Scenario: A retry of an add
- **WHEN** an add with a date was performed, the response was lost, the number's records did not change, and the client retries the add
- **THEN** success "unchanged"; there is one record in the database

#### Scenario: A retry of an add after someone else's delete
- **WHEN** an add was performed, the response was lost, another client deleted this record, and the client retries the add
- **THEN** the record is created anew (outcome "changed"): an add carries no version and does not distinguish a "first attempt" from a new one
  (owner decision "add without a version"); a client that needs not to resurrect the deleted checks by number search

#### Scenario: A retry of an add after someone else's rename
- **WHEN** an add of «Иван» was performed, the response was lost, another client replaced the name of this record with «Пётр», and the client
  retries the add of «Иван»
- **THEN** "name conflict" with the current name «Пётр» and the number version; data is not changed

#### Scenario: A retry of a delete
- **WHEN** a delete was performed, the response was lost, and the client retries the delete with the earlier version
- **THEN** "version conflict" with the current version; the client re-reads the number by number search and sees the record is gone

#### Scenario: A retry of a rename
- **WHEN** a rename was performed, the response was lost, and the client retries it with the earlier version
- **THEN** "version conflict"; after re-reading the history, the client sees its own name and does not retry

#### Scenario: A retry of a rename to the same name
- **WHEN** a rename replaced the name with the same one (the number version did not change), the response was lost, and the client retries it with the earlier version
- **THEN** success "unchanged"; the number version does not change

#### Scenario: A retry of a rename after someone else's delete
- **WHEN** a rename was performed, the response was lost, another client deleted the record, and the client retries the rename with the earlier version
- **THEN** "version conflict" (the delete changed the version); the record is not resurrected

### Requirement: Adding without a date for a known number
The system SHALL, on an add without a date where the number has a record on 1876-03-07 with the same name (names are compared as on an identical retry of an add with a date), answer "unchanged" with the current version; otherwise, if there are records on any date — "number known" with the current version; data and version do not change. The check and the record creation SHALL be one writer step: of two concurrent adds without a date for a new number, one creates the record. Trace: CUJ1, CUJ2, CUJ9; OD-6, OD-8; ADR 0005, 0025.

#### Scenario: Number known
- **GIVEN** the number has records on 2020-01-01 "Bob" and 2010-05-05 "Ann"
- **WHEN** "Eve" is added without a date
- **THEN** "number known" with the current number version; records and version are not changed

#### Scenario: The only record on 1876-03-07 with the same name
- **GIVEN** the number has one record on 1876-03-07 named "Ann"
- **WHEN** "Ann" is added without a date
- **THEN** success "unchanged" (OD-8); the record and the number version are not changed

#### Scenario: A record on 1876-03-07 with the same name and other records
- **GIVEN** the number has records on 1876-03-07 "Ann" and on 2020-01-01 "Bob"
- **WHEN** "Ann" is added without a date
- **THEN** success "unchanged"; data and version are not changed

#### Scenario: A record on 1876-03-07 with a different name
- **GIVEN** the number has one record on 1876-03-07 named "Bob"
- **WHEN** "Ann" is added without a date
- **THEN** "number known" with the current version; the record is not changed, and there is no name conflict

#### Scenario: The same form of the name
- **GIVEN** the number has a record on 1876-03-07 "Ann"
- **WHEN** " Ann " is added without a date (spaces at the edges)
- **THEN** the names are compared after normalization (NFC, edge trimming): success "unchanged"

#### Scenario: The CUJ1/CUJ2 race
- **WHEN** two adds without a date for the same new number are executed concurrently
- **THEN** one creates the record on 1876-03-07, and the other gets "unchanged" if the name is the same, otherwise "number known", with the version after the first; there is no silent date substitution for a known number

#### Scenario: A number with no records after a delete
- **GIVEN** all the number's records were deleted
- **WHEN** a record is added without a date
- **THEN** the record is created on 1876-03-07 (a number with no records is unknown)
