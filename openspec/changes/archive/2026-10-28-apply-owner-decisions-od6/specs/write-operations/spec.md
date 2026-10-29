## MODIFIED Requirements

### Requirement: Adding a record
Given an add operation with inputs "number", "name" and an optional "date", the system SHALL create the record (number, date) →
name if no record exists for that (number, date); an expected version is not required. Adding without a date is allowed only if the number has no records at all: the zero date 1876-03-07 is used
(`records`, "Zero date when no date is given"); otherwise the outcome is "number known" ("Adding without a date for a known number"). Trace: CUJ1, CUJ2, CUJ9; ADR 0002 (items 4–5), 0005; OD-6 (Q-160). Decisions: ADR 0025 (owner decision, OD-5, D-6),
0026 (owner decision, OD-5, D-7).

#### Scenario: New number
- **WHEN** a record is added with a number that has no records
- **THEN** the record is created, the number is found by name, and the name is found by number (CUJ1)
- **AND** the number version changed from the initial version of the current generation to a different one

#### Scenario: Known number, new date
- **WHEN** a record is added for a date the number does not have yet
- **THEN** the record is created, the number's records for other dates are unchanged, the number version changed (CUJ2)

#### Scenario: The same name on a different date
- **WHEN** a name that already owns a number on one date is added to the same number on another date
- **THEN** a separate record is created; both records appear in the number history

#### Scenario: Date not specified
- **WHEN** an add is performed without a date and the number has no records
- **THEN** the record is created on 1876-03-07; an explicitly specified 1876-03-07 yields the same record

#### Scenario: Date not specified, number known
- **GIVEN** the number has a record on 2020-01-01
- **WHEN** an add is performed without a date
- **THEN** "number known"; no record is created, the data and the number version are unchanged

### Requirement: Name conflict and add idempotency
When adding with a date to an occupied (number, date) with a different name, the system SHALL reject the operation with a "name conflict" carrying the record's current
name and the current number version, without changing data; with the same name, it SHALL respond with success "unchanged". Names
are compared by exact spelling after normalization (`records`, "Name and query normalization": NFC and edge trimming). Adding without a date is not covered by this requirement ("Adding without a date for a known number"). Trace: CUJ3, CUJ9; ADR 0003, 0005. Decisions: ADR 0025 (owner decision, OD-5, D-6).

#### Scenario: Different name
- **WHEN** (number, date) holds "Иван" and "Пётр" is being added
- **THEN** "name conflict"; the response contains the current name "Иван" and the number version; the record and version are unchanged

#### Scenario: Different spelling
- **WHEN** (number, date) holds "ANN" and "Ann" is being added
- **THEN** "name conflict" (comparison by exact spelling, case is distinguished)

#### Scenario: Identical retry
- **WHEN** a record with a date is added that already exists with the same number, date and name
- **THEN** success "unchanged"; the record and number version are unchanged

#### Scenario: Difference only in edge whitespace and NFC form
- **WHEN** the name being added differs from the stored one only in edge whitespace or normalization form
- **THEN** it is the same name: success "unchanged"

#### Scenario: Concurrent adds of the same (number, date)
- **WHEN** two adds to the same free (number, date) with different names run concurrently
- **THEN** exactly one creates the record, the other gets "name conflict" with the first one's name

### Requirement: Write error kinds
On writes, the system SHALL distinguish five error kinds: "invalid input" (naming the field: number, date, name, expected
version), "name conflict" (add with a date), "number known" (add without a date, the number has records), "version conflict" (rename, delete) and "not found" (rename, delete);
on any error, data and versions do not change. Trace: CUJ1, CUJ2, CUJ3, CUJ6, CUJ9, CUJ12; OD-6 (Q-160). Decisions: ADR 0025 (owner decision, OD-5, D-6).

#### Scenario: Contents of a version conflict
- **WHEN** the expected version is not equal to the current number version
- **THEN** "version conflict"; the response contains the current number version (CUJ9, CUJ12)

#### Scenario: Contents of "number known"
- **WHEN** an add without a date got "number known"
- **THEN** the response contains the current number version; the current name is not reported (there are several records)

#### Scenario: Invalid input named by field
- **WHEN** the number has an invalid shape, or the date is missing (rename, delete), or the date is not a calendar day (for example, 1875-02-30), or the date is out of the range
  1876-03-07…9999-12-31 (for example, 1876-03-06 or 10000-01-01 when a year is written with five digits), or the name is invalid
- **THEN** "invalid input" with a (field, reason) pair from `records`, "Value error kind dictionary": (date,
  "missing"), (date, "nonexistent day"), (date, "out of range") and so on; the reason dictionary is not duplicated here; the "expected
  version" field belongs to this capability alone and is not part of the `records` dictionary; data is unchanged

#### Scenario: An error changes nothing
- **WHEN** an operation ended with any write error
- **THEN** the number version and all its records are in the same state as before the operation

#### Scenario: Other refusal kinds are out of scope
- **WHEN** a request is rejected because of permissions (CUJ8), the request limit, or the database state "read-only"/"storage error"
  (CUJ16)
- **THEN** these are other refusal kinds, defined by other areas; data is unchanged

### Requirement: Check order on write
The system SHALL check an operation in this order: input (number, date — mandatory for rename and delete, optional
for add, name, and for rename and delete the presence and shape of the expected version; the first invalid field is reported); for rename and delete, the version ("version conflict"), then
record existence ("not found"); for add with a date, whether (number, date) is occupied, without a date, whether the number has records ("number known"). Access and database state checks come
earlier. Trace: CUJ1, CUJ2, CUJ3, CUJ9, CUJ12; OD-6 (Q-160). Decisions: ADR 0025 (owner decision, OD-5, D-6).

#### Scenario: Stale version and a missing record
- **WHEN** a record that does not exist is renamed or deleted with a stale version
- **THEN** "version conflict" with the current version, not "not found"

#### Scenario: Record disappeared because of another client's delete
- **WHEN** the record was deleted concurrently, and a client renames or deletes it with a version read earlier
- **THEN** "version conflict" (the delete changed the number version), not "not found" (CUJ12)

#### Scenario: Current version and a missing record
- **WHEN** the record is missing and the version is current
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

#### Scenario: Missing date before name and version
- **WHEN** a rename is sent without a date, with an invalid name and without an expected version
- **THEN** "invalid input" with the pair (date, "missing"): the date is checked before the name and the expected version

#### Scenario: Missing date before version conflict
- **WHEN** a delete is sent without a date and with a stale version
- **THEN** "invalid input" (date, "missing"), not "version conflict": input is checked before the version

#### Scenario: Add without a date
- **WHEN** an add is performed without a date and the number has no records
- **THEN** there is no date error: 1876-03-07 is used (`records`, "Zero date when no date is given")

#### Scenario: Input before "number known"
- **WHEN** an add without a date contains an invalid name and the number is known
- **THEN** "invalid input", not "number known"

#### Scenario: Add with a version of the wrong shape
- **WHEN** an add is received with an expected version that does not belong to the version type (wrong shape), or with a version
  of the right shape not issued by the server (an add's expected version is not checked at all)
- **THEN** the version is ignored, neither "invalid input" nor "version conflict" is returned; the operation is processed
  as usual

#### Scenario: Input before occupancy
- **WHEN** an add contains an invalid name for an occupied (number, date)
- **THEN** "invalid input", not "name conflict"

### Requirement: Number version in search results
The system SHALL return the number version: in a number search, once per number, as a result field (also for an empty history, the initial one), not in every record (OD-6, Q-128); in a name search, in every result record, the version of its number. The version is consistent with the records of that number in the result and applies to the whole number. Trace: CUJ6, CUJ9, CUJ12, X-5; OD-6.
Decisions: ADR 0029 (item 5 — owner decision, OD-5, D-8), 0026.

#### Scenario: Number search results
- **WHEN** a number search is performed and there are records
- **THEN** the result contains all the number's records and its version once; the version is accepted as the expected one in rename and delete (CUJ12)

#### Scenario: Version is not repeated in records
- **WHEN** a number has three records and a number search is performed
- **THEN** the version is passed once per result, the records do not contain versions

#### Scenario: Number without records
- **WHEN** a number search is performed for a number without records
- **THEN** the result is empty and contains the initial version of the current generation

#### Scenario: Name search results
- **WHEN** a name search is performed and one of a number's three records is in the result
- **THEN** the record contains the version of the whole number; it is enough to delete this record from the results (CUJ6) without re-reading

#### Scenario: One version per number in a result
- **WHEN** one name-search page has several records of the same number
- **THEN** they have the same version, and it corresponds to the state of the number the records were taken from

#### Scenario: Re-read, retry
- **WHEN** an action was rejected with a "version conflict", the client performs a number search and repeats the action with the version from the
  result, and the number did not change in the meantime
- **THEN** the action succeeds

#### Scenario: Stale version between pages
- **WHEN** a number is changed between getting two name-search result pages, and the client acts on the version from the first
- **THEN** "version conflict"; data is unchanged (consistency between pages is not promised: owner decision, OD-5, D-8, Q-56)

### Requirement: Retrying an operation after an unknown outcome
When a write operation whose result the client did not receive is retried, and the number's records did not
change between attempts, the system SHALL not produce a second effect: adding an identical record with a date — success "unchanged"; adding without a date — "number known" (the record was already created by the first attempt; the client finds out the outcome with a number search); rename and delete with the
old version — "version conflict". If the number's records did change between attempts, the usual rules apply
(rename and delete — "version conflict"; add — by key occupancy). Trace: CUJ1, CUJ9, CUJ12, CUJ14; OD-6 (Q-160). Decisions: ADR 0026 (owner decision, OD-5, D-7).

#### Scenario: Retrying an add without a date
- **WHEN** an add without a date of a new number completed, the response was lost, and the client retries it
- **THEN** "number known" with the number version; there is one record, the number search shows it

#### Scenario: Retrying an add
- **WHEN** an add with a date completed, the response was lost, the number's records did not change, and the client retries the add
- **THEN** success "unchanged"; the database has one record

#### Scenario: Retrying an add after another client's delete
- **WHEN** an add completed, the response was lost, another client deleted this record, and the client retries the add
- **THEN** the record is created anew (outcome "changed"): an add carries no version and does not distinguish the "first attempt" from a new one
  (owner decision "add without a version"); a client that must not resurrect deleted data checks with a number search

#### Scenario: Retrying an add after another client's rename
- **WHEN** an add of "Иван" completed, the response was lost, another client renamed this record to "Пётр", and the client
  retries the add of "Иван"
- **THEN** "name conflict" with the current name "Пётр" and the number version; data is unchanged

#### Scenario: Retrying a delete
- **WHEN** a delete completed, the response was lost, and the client retries the delete with the old version
- **THEN** "version conflict" with the current version; the client re-reads the number with a number search and sees that the record is gone

#### Scenario: Retrying a rename
- **WHEN** a rename completed, the response was lost, and the client retries it with the old version
- **THEN** "version conflict"; after re-reading the history, the client sees its own name and does not retry

#### Scenario: Retrying a rename after another client's delete
- **WHEN** a rename completed, the response was lost, another client deleted the record, and the client retries the rename with the old version
- **THEN** "version conflict" (the delete changed the version); the record is not resurrected

### Requirement: Result of a successful operation
On the success of an add, rename or delete, the system SHALL return the outcome ("changed" or "unchanged") and the
number version current after the operation; the version's representation format is outside this area. Trace: CUJ1, CUJ3, CUJ6,
CUJ9, X-5. Decisions: ADR 0026 (owner decision, OD-5, D-7).

#### Scenario: Version in a successful response
- **WHEN** an operation changed data
- **THEN** the result: "changed" and the new number version, usable as the expected one for the next rename without
  re-reading

#### Scenario: Identical retry
- **WHEN** an add of an identical record with a date or a rename to the same name with a current version
- **THEN** the result: "unchanged" and the current number version, equal to the previous one

#### Scenario: Deleting the last record
- **WHEN** the number's only record is deleted
- **THEN** the result: "changed" and the initial version of the current generation

## ADDED Requirements

### Requirement: Adding without a date for a known number
The system SHALL reject an add without a date if the number has at least one record on any date (including on 1876-03-07 with the same name), with a "number known" conflict carrying the current number version, without changing data or the version. The check and the record creation SHALL be one writer step: of two concurrent adds without a date of a new number, exactly one creates the record. Trace: CUJ1, CUJ2, CUJ9; OD-6 (Q-160); ADR 0005, 0025.

#### Scenario: Number known
- **GIVEN** the number has records on 2020-01-01 "Bob" and 2010-05-05 "Ann"
- **WHEN** "Eve" is added without a date
- **THEN** "number known" with the current number version; records and version are unchanged

#### Scenario: Only record on 1876-03-07 with the same name
- **GIVEN** the number has one record on 1876-03-07 with the name "Ann"
- **WHEN** "Ann" is added without a date
- **THEN** "number known", not success "unchanged"; the client finds out the state with a number search

#### Scenario: CUJ1/CUJ2 race
- **WHEN** two adds without a date of the same new number run concurrently
- **THEN** one creates the record on 1876-03-07, the other gets "number known" with the version after the first; there is no silent substitution of a date for a known number

#### Scenario: Number without records after deletion
- **GIVEN** all of the number's records were deleted
- **WHEN** a record is added without a date
- **THEN** the record is created on 1876-03-07 (a number without records is unknown)
