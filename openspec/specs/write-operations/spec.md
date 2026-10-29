# write-operations Specification

## Purpose
Observable semantics of the operations that add a record, rename it and delete it, for records (number, date) → name and for number versions:
inputs, results, error kinds and their check order, idempotency, version properties, atomicity and visibility.
No API format, protocol or UI look; the storage mechanics are requirements in the section "Requirements for other areas"
of `design.md`.

## Requirements

### Requirement: Adding a record
Given an add operation with the inputs "number", "name" and an optional "date", the system SHALL create the record (number, date) →
name if there is no record for this (number, date); an expected version is not required. Without a date: if the number has no records, 1876-03-07 is taken (`records`); if there is a record for 1876-03-07 with the same name, "unchanged"; otherwise "number known" ("Adding without a date for a known number"). Trace: CUJ1, CUJ2, CUJ9; ADR 0002, 0005, 0025, 0026; OD-6, OD-8.

#### Scenario: New number
- **WHEN** a record is added with a number that has no records
- **THEN** the record is created, the number is found by name, and the name is found by number (CUJ1)
- **AND** the number version changed from the initial version of the current generation to a different one

#### Scenario: Known number, new date
- **WHEN** a record is added for a date the number does not have yet
- **THEN** the record is created, the number's records for other dates are unchanged, and the number version changed (CUJ2)

#### Scenario: Same name for another date
- **WHEN** a name that already holds a number for one date is added to the same number for another date
- **THEN** a separate record is created; both records are returned in the number history

#### Scenario: Date not given
- **WHEN** an add is performed without a date, and the number has no records
- **THEN** the record is created for 1876-03-07; an explicitly given 1876-03-07 yields the same record

#### Scenario: Date not given, number known
- **GIVEN** the number has a record for 2020-01-01
- **WHEN** an add is performed without a date
- **THEN** "number known"; no record is created, the data and the number version are unchanged

#### Scenario: Date not given, retry on 1876-03-07
- **GIVEN** the number has a record for 1876-03-07 «Ann» (and possibly other records)
- **WHEN** an add of «Ann» is performed without a date
- **THEN** success "unchanged"; the data and the number version are unchanged (OD-8)

### Requirement: Name conflict and add idempotency
When an add with a date targets an occupied (number, date) with a different name, the system SHALL reject the operation as a "name conflict" carrying the current
record name and the current number version, without changing data; with the same name, it SHALL answer with success "unchanged". Names
are compared by exact spelling after normalization (`records`, "Name and query normalization": NFC and edge trimming). An add without a date is not covered by this requirement ("Adding without a date for a known number"). Trace: CUJ3, CUJ9; ADR 0003, 0005. Decisions: ADR 0025 (owner decision, OD-5, D-6).

#### Scenario: Different name
- **WHEN** (number, date) holds «Иван», and «Пётр» is added
- **THEN** "name conflict"; the response contains the current name «Иван» and the number version; the record and the version are unchanged

#### Scenario: Different spelling
- **WHEN** (number, date) holds «ANN», and «Ann» is added
- **THEN** "name conflict" (comparison by exact spelling, case is distinguished)

#### Scenario: Identical retry
- **WHEN** an add with a date is performed for a record that already exists with the same number, date and name
- **THEN** success "unchanged"; the record and the number version are unchanged

#### Scenario: Difference only in edge whitespace and NFC form
- **WHEN** the added name differs from the stored one only in edge whitespace or normalization form
- **THEN** it is the same name: success "unchanged"

#### Scenario: Concurrent adds for one (number, date)
- **WHEN** two adds for one free (number, date) with different names run at the same time
- **THEN** exactly one creates the record, the other gets a "name conflict" with the first one's name

### Requirement: Renaming a record
Given a rename operation with the inputs "number", "date", "new name", "expected version", the system SHALL replace
the name of the existing record (number, date), without keeping the previous one. The date is mandatory: without it, "invalid input" (date, "missing"),
with no substitution (`records`, "Zero date when no date is given"; owner decision, OD-5, Q-90). An add does not overwrite a name.
Trace: CUJ3, CUJ9, CUJ12; ADR 0005. Decisions: ADR 0025 (owner decision, OD-5, D-6), 0026 (owner decision, OD-5, D-7), 0027.

#### Scenario: Confirmed overwrite
- **WHEN** the name of an existing record is renamed with a current version and a new name different from the current one
- **THEN** the name is replaced, the previous name of this record is returned by no search, and the number version changed (CUJ3)
- **AND** the number's records for other dates are unchanged

#### Scenario: Same name with a current version
- **WHEN** the new name equals the current one after normalization (`records`, "Name and query normalization") and the version is current
- **THEN** success "unchanged"; the number version does not change

#### Scenario: Record absent
- **WHEN** the record (number, date) is absent and the number version is current
- **THEN** "not found"; the data is unchanged

#### Scenario: Invalid new name
- **WHEN** the new name is invalid under the `records` requirements: empty or only whitespace, with a control character, or
  longer than 200 Unicode scalar values of the normalized text ("Text length and length cap")
- **THEN** "invalid input" with a (name, reason) pair from the `records` dictionary: "empty", "control character" or "too
  long"; the data is unchanged, there is no truncation

#### Scenario: Date not given
- **WHEN** a rename is sent without a date (the version is current, the number has a record for 1876-03-07)
- **THEN** "invalid input" with the pair (date, "missing") from the `records` dictionary; the record for 1876-03-07 and the records for other dates
  are not affected, the number version does not change (owner decision, OD-5, Q-90; ADR 0025 item 6)

#### Scenario: Explicit zero date
- **WHEN** a rename is sent with the explicit date 1876-03-07, a record for this date exists and the version is current
- **THEN** the record for 1876-03-07 is changed (the same key as with the substitution on add)

### Requirement: Deleting a record
Given a delete operation with the inputs "number", "date", "expected version", the system SHALL delete exactly one
record (number, date), without touching the other records of the number; the date is mandatory: without it, "invalid input" (date,
"missing"), with no substitution (`records`, "Zero date when no date is given"; owner decision OD-5, Q-90);
an absent record with a current version gives "not found". Trace: CUJ6, CUJ9, CUJ12; ADR 0005. Decisions: ADR 0025, 0026 (OD-5: D-6, D-7), 0027.

#### Scenario: Deleting one record
- **WHEN** an existing record is deleted with a current version
- **THEN** only this record is deleted; the other records of the number are intact; the number version changed (CUJ6)

#### Scenario: Deleting the last record
- **WHEN** the only record of a number is deleted
- **THEN** the number is like an unknown one: its history is empty, its version is the initial version of the current generation (ADR 0027)

#### Scenario: Record absent
- **WHEN** the record is absent and the number version is current
- **THEN** "not found"; the data is unchanged

#### Scenario: Date not given
- **WHEN** a delete is sent without a date, the number has records for 1876-03-07 and for other dates, and the version is current
- **THEN** "invalid input" with the pair (date, "missing"); no record of the number is deleted, the version does not change (owner
  decision, OD-5, Q-90; ADR 0025 item 6)

#### Scenario: Explicit zero date
- **WHEN** a delete is sent with the explicit date 1876-03-07, a record for this date exists and the version is current
- **THEN** the record for 1876-03-07 is deleted; the other records of the number are intact

#### Scenario: Deleted does not come back
- **WHEN** a record is deleted and the acknowledgment is received
- **THEN** no search returns it; a deleted record does not reappear without a new add

### Requirement: Write error kinds
On writes the system SHALL distinguish five error kinds: "invalid input" (naming the field: number, date, name, expected
version), "name conflict" (add with a date), "number known" (add without a date, the number has records but no identical one on 1876-03-07), "version conflict" (rename, delete) and "not found" (rename, delete);
on any error the data and the versions do not change. Trace: CUJ1, CUJ2, CUJ3, CUJ6, CUJ9, CUJ12; OD-6, OD-8; ADR 0025.

#### Scenario: Content of a version conflict
- **WHEN** the expected version is not equal to the current number version
- **THEN** "version conflict"; the response contains the current number version (CUJ9, CUJ12)

#### Scenario: Content of "number known"
- **WHEN** an add without a date received "number known"
- **THEN** the response contains the current number version; the current name is not reported (there are several records)

#### Scenario: Invalid input named by field
- **WHEN** the number has an invalid shape, or the date is missing (rename, delete), or the date is not a calendar day (for example, 1875-02-30), or the date is out of the range
  1876-03-07…9999-12-31 (for example, 1876-03-06 or 10000-01-01 when a year is written with five digits), or the name is invalid
- **THEN** "invalid input" with a (field, reason) pair from `records`, "Value error kind dictionary": (date,
  "missing"), (date, "nonexistent day"), (date, "out of range") and so on; the reason dictionary is not duplicated here; the field "expected
  version" is this capability's own field and is not part of the `records` dictionary; the data is unchanged

#### Scenario: An error changes nothing
- **WHEN** an operation ended with any write error
- **THEN** the number version and all its records are in the same state as before the operation

#### Scenario: Other refusal kinds out of scope
- **WHEN** a request is rejected because of permissions (CUJ8), the request limit, or the "read-only"/"storage error" database state
  (CUJ16)
- **THEN** these are other refusal kinds, defined by other areas; the data is unchanged

### Requirement: Check order on write
The system SHALL check an operation in this order: input (number; date, which is mandatory for rename and delete and not for add; name; expected version, presence and kind, for rename and delete; the first invalid field is reported); for rename and delete, the version ("version conflict"), then the existence of the record ("not found"); for an add with a date, whether (number, date) is occupied, and without a date, an identical record on 1876-03-07 ("unchanged"), otherwise the number's records ("number known"). Earlier than all that come access and database state. Trace: CUJ1–CUJ3, CUJ9, CUJ12; OD-8; ADR 0025.

#### Scenario: Stale version and absent record
- **WHEN** a record that does not exist is renamed or deleted with a stale version
- **THEN** "version conflict" with the current version, not "not found"

#### Scenario: Record vanished because of another client's delete
- **WHEN** the record was deleted in parallel, and a client renames or deletes it with a version read before that
- **THEN** "version conflict" (the delete changed the number version), not "not found" (CUJ12)

#### Scenario: Current version and absent record
- **WHEN** the record is absent and the version is current
- **THEN** "not found"

#### Scenario: Same name with a stale version
- **WHEN** a rename to the same name is sent with a stale version
- **THEN** "version conflict"; success "unchanged" is returned only with a current version (OD-4, item 7)

#### Scenario: Invalid input before the version
- **WHEN** a request contains both an invalid name and a stale version
- **THEN** "invalid input" (field "name"), not "version conflict"

#### Scenario: Several invalid fields
- **WHEN** both the number and the name are invalid
- **THEN** the field "number" is reported (order: number, date, name, expected version)

#### Scenario: Date missing before the name and the version
- **WHEN** a rename is sent without a date, with an invalid name and without an expected version
- **THEN** "invalid input" with the pair (date, "missing"): the date is checked before the name and the expected version

#### Scenario: Date missing before a version conflict
- **WHEN** a delete is sent without a date and with a stale version
- **THEN** "invalid input" (date, "missing"), not "version conflict": input is checked before the version

#### Scenario: Add without a date
- **WHEN** an add is performed without a date, and the number has no records
- **THEN** there is no date error: 1876-03-07 is taken (`records`, "Zero date when no date is given")

#### Scenario: Input before "number known"
- **WHEN** an add without a date contains an invalid name, and the number is known
- **THEN** "invalid input", not "number known"

#### Scenario: Add with a version of the wrong kind
- **WHEN** an add is received with an expected version that does not belong to the version type (wrong kind), or with a version
  of the right kind that the server did not issue (an add's expected version is not checked at all)
- **THEN** the version is ignored, neither "invalid input" nor "version conflict" is returned; the operation is handled
  as usual

#### Scenario: Input before occupancy
- **WHEN** an add contains an invalid name for an occupied (number, date)
- **THEN** "invalid input", not "name conflict"

### Requirement: Expected version is mandatory
The system SHALL require the expected number version on rename and delete and refuse with "invalid input" (field
"expected version") if it is absent or of the wrong kind; on add it is not required, not checked and does not affect
the result; a value of the version type that is not equal to the current number version gives "version conflict". Trace: CUJ9, CUJ12;
owner decision "Operations: the expected version is mandatory". Decisions: ADR 0025 (owner decision, OD-5, D-6, item 7).

#### Scenario: No version on rename or delete
- **WHEN** a rename or delete is received without an expected version
- **THEN** "invalid input" (field "expected version"); the data is unchanged

#### Scenario: Add without a version
- **WHEN** an add is performed without a version
- **THEN** the operation is handled as usual

#### Scenario: Not a version
- **WHEN** a value that does not belong to the version type is passed as the expected version (wrong kind: the kind is the syntax
  and format of the version type, set by the API area; it is checked by format and does not depend on database state)
- **THEN** "invalid input" (field "expected version"), not "version conflict"

#### Scenario: Version of the right kind that the server did not issue
- **WHEN** a rename or delete passes a value of the version type that the server never issued to this number
  (a version of another number, of a previous generation, or invented by the client)
- **THEN** "version conflict" with the current number version (any value of the version type not equal to the current number
  version is a conflict); "invalid input" is not returned; the data is unchanged

### Requirement: Result of a successful operation
On success of an add, rename or delete the system SHALL return the outcome ("changed" or "unchanged") and
the number version current after the operation; the format of the version representation is outside this area. Trace: CUJ1, CUJ3, CUJ6,
CUJ9, X-5. Decisions: ADR 0026 (owner decision, OD-5, D-7).

#### Scenario: Version in a successful response
- **WHEN** an operation changed data
- **THEN** the result: "changed" and the new number version, usable as the expected version for the next rename without
  re-reading

#### Scenario: Identical retry
- **WHEN** an add of an identical record with a date, or a rename to the same name with a current version
- **THEN** the result: "unchanged" and the current number version, equal to the previous one

#### Scenario: Deleting the last record
- **WHEN** the only record of a number is deleted
- **THEN** the result: "changed" and the initial version of the current generation

### Requirement: Number version in search results
The system SHALL return the number version: in a search by number, once per number, as a result field (also for an empty history, the initial one), not in every record (OD-6, Q-128); in a name search, in every result record, the version of its number. The version is consistent with the records of this number in the result and refers to the whole number. Trace: CUJ6, CUJ9, CUJ12, X-5; OD-6.
Decisions: ADR 0029 (item 5, owner decision, OD-5, D-8), 0026.

#### Scenario: Results by number
- **WHEN** a search by number with records is performed
- **THEN** the result contains all records of the number and its version once; the version is accepted as the expected one in rename and delete (CUJ12)

#### Scenario: Version is not repeated in records
- **WHEN** a number has three records and a search by number is performed
- **THEN** the version is passed once per result, and the records contain no versions

#### Scenario: Number without records
- **WHEN** a search by number without records is performed
- **THEN** the result is empty and contains the initial version of the current generation

#### Scenario: Results by name
- **WHEN** a name search is performed, and one of a number's three records is in the result
- **THEN** the record contains the version of the whole number; it is enough to delete this record from the results (CUJ6) without re-reading

#### Scenario: One version per number in a result
- **WHEN** one page of a name search has several records of one number
- **THEN** they have one and the same version, and it matches the state of the number from which these records were taken

#### Scenario: Re-read, then retry
- **WHEN** an action is rejected with a "version conflict", the client performs a search by number and repeats the action with the version from the
  result, and the number did not change in the meantime
- **THEN** the action is performed

#### Scenario: Stale version between pages
- **WHEN** a number is changed between getting two pages of name-search results, and the client acts on the version from the first
- **THEN** "version conflict"; the data is unchanged (consistency between pages is not promised: owner decision, OD-5, D-8, Q-56)

### Requirement: Retrying an operation after an unknown outcome
When a write operation whose result the client did not receive is retried, the system SHALL not produce a second effect if the number's records did not change: an add with a date, and an add without a date when a record for 1876-03-07 with the same name exists (OD-8), give "unchanged"; a delete, and a rename that changed the name, with the old version give "version conflict"; a rename to the same name gives "unchanged". If the records did change, the ordinary rules apply. Trace: CUJ1, CUJ9, CUJ12, CUJ14; OD-6, OD-8; ADR 0026.

#### Scenario: Retry of an add without a date
- **WHEN** an add without a date of a new number was performed, the response was lost, and the client retries it
- **THEN** success "unchanged" with the number version (OD-8); there is one record, the retry is safe

#### Scenario: Retry without a date after the records changed
- **WHEN** after an add without a date the record for 1876-03-07 was deleted or renamed, and the client retries the add
- **THEN** the ordinary rules apply: a number without records, the record is created; otherwise "number known"

#### Scenario: Retry of an add
- **WHEN** an add with a date was performed, the response was lost, the number's records did not change, and the client retries the add
- **THEN** success "unchanged"; the database has one record

#### Scenario: Retry of an add after another client's delete
- **WHEN** an add was performed, the response was lost, another client deleted this record, and the client retries the add
- **THEN** the record is created again (outcome "changed"): an add carries no version and does not distinguish a "first attempt" from a new one
  (owner decision "add without a version"); a client that must not resurrect deleted data checks with a search by number

#### Scenario: Retry of an add after another client's rename
- **WHEN** an add of «Иван» was performed, the response was lost, another client replaced this record's name with «Пётр», and the client
  retries the add of «Иван»
- **THEN** "name conflict" with the current name «Пётр» and the number version; the data is unchanged

#### Scenario: Retry of a delete
- **WHEN** a delete was performed, the response was lost, and the client retries the delete with the old version
- **THEN** "version conflict" with the current version; the client re-reads the number by a search by number and sees that the record is gone

#### Scenario: Retry of a rename
- **WHEN** a rename was performed, the response was lost, and the client retries it with the old version
- **THEN** "version conflict"; after re-reading the history, the client sees its own name and does not retry

#### Scenario: Retry of a rename to the same name
- **WHEN** a rename replaced the name with the very same one (the number version did not change), the response was lost, and the client retries it with the old version
- **THEN** success "unchanged"; the number version does not change

#### Scenario: Retry of a rename after another client's delete
- **WHEN** a rename was performed, the response was lost, another client deleted the record, and the client retries the rename with the old version
- **THEN** "version conflict" (the delete changed the version); the record is not resurrected

### Requirement: When the number version changes
Within one version generation (between crash-restart and start-from-backup events) the system SHALL change
a number's version if and only if the records of that number change: creating a record, replacing a name with a different one,
deleting a record. An identical retry, a rename to the same name, an error, a read, a planned restart and operations on other
numbers do not change the version. Trace: CUJ1, CUJ3, CUJ6, CUJ12, X-5; owner decision "versioning belongs to the number". Decisions:
ADR 0027, 0028.

#### Scenario: Creation, rename, delete
- **WHEN** any of these operations is performed with a data change
- **THEN** the number version became a value different from the previous one

#### Scenario: Another date of the same number changed
- **WHEN** a record of the number for another date is changed or deleted
- **THEN** the number version changed, and an action with the old version is rejected with a "version conflict" (CUJ12)

#### Scenario: Another number
- **WHEN** the records of another number are changed
- **THEN** this number's version did not change

#### Scenario: Operations without data changes
- **WHEN** a read, an erroneous operation, an identical retry or a planned restart is performed
- **THEN** the number version did not change

#### Scenario: Exception: generation change
- **WHEN** the server is restarted after a crash or started from a backup (ADR 0028)
- **THEN** the versions of all numbers, including those with unchanged records, get values different from those issued before, within
  the bounds of the guarantee in "Version is never repeated" (owner decision, OD-5, D-17); this is the only event that changes a version without
  changing records

### Requirement: Initial version and a number with no records
The system SHALL treat the version of a number with no records as the initial version of the current generation: one for all numbers,
unchanged across planned restarts; a number with records never has the initial version. A number whose records have all been deleted
is indistinguishable from an unknown one. This refines X-5 for a number with no records (ADR 0027, item 4). Trace: CUJ1, CUJ6,
X-5. Decisions: ADR 0027, 0028.

#### Scenario: Unknown number
- **WHEN** a number has no records and never had any
- **THEN** its version is the initial version of the current generation

#### Scenario: After deleting all records
- **WHEN** all records of a number are deleted
- **THEN** the number version is the initial version of the current generation, a search by number gives an empty list

#### Scenario: Planned restart
- **WHEN** a client read the initial version of an empty number, the server was restarted in a planned way, and the number is still empty
- **THEN** the initial version is the same

#### Scenario: Action with the initial version
- **WHEN** a rename or delete is sent with the initial version, and the number has records
- **THEN** "version conflict" (the version of a number with records is not the initial one)

#### Scenario: Rename with no records, with the initial version
- **WHEN** a record of a number with no records is renamed with the initial version of the current generation
- **THEN** "not found"; the data is unchanged

### Requirement: Version is never repeated
Within a version generation the system SHALL not issue to a number with records a version previously issued to that number
(deterministically); between generations it is also deterministic if the server settings are preserved (a location outside the backup where
tokens are kept, OD-4 item 9), otherwise a version coincides with a previously issued one with probability at most 2^-64 per generation-change event
(owner decision, OD-5, D-17, ADR 0028 items 6(c), 9); the version is opaque.
Trace: X-5, CUJ12. Decisions: ADR 0027, 0028.

#### Scenario: Deleted everything and added again
- **WHEN** a client read version V of a number with records, then all records were deleted, the number was added again, and the client
  sends a rename or delete with V
- **THEN** the current number version is not equal to V, the action is rejected with a "version conflict", the data is unchanged (the guarantee
  applies to a database continuing its own history and to a backup created by the command; on files copied in another way a repeat
  is possible, owner decision, OD-5, D-13)

#### Scenario: Empty number and another client's add
- **WHEN** a client read the initial version, then the number was added by another client, and the first client renames a record with the
  initial version
- **THEN** "version conflict"; the data is unchanged

#### Scenario: Initial version and another client's add followed by a full delete
- **WHEN** a client read the initial version, then the number was added and fully deleted again, and the client renames a record with the
  initial version
- **THEN** "not found"; the data is unchanged (a repeat of the initial version within a generation is harmless: there is no target record)

#### Scenario: Comparison for equality only
- **WHEN** a client received two versions of one number
- **THEN** the specification does not promise which of them is newer and does not require the client to order them

### Requirement: Record and version change are indivisible
The system SHALL perform the change of a number's records and the change of its version as one indivisible change: neither on failure nor
for a reader is there a state where the record is changed and the version is not, or the other way round. Trace: X-5, CUJ14; ADR 0006 item 6
(b). Decisions: ADR 0028, 0029 (isolation is an owner decision, OD-5, D-8).

#### Scenario: Failure at the moment of an operation
- **WHEN** the server crashed during a write operation that was not acknowledged
- **THEN** after restart the operation is either fully present (record and new version) or fully absent (the previous
  record and version) (CUJ14)

#### Scenario: Reader sees a consistent pair
- **WHEN** a reader gets a number's records together with its version (search by number)
- **THEN** the version corresponds to exactly these records: there is no "new history, old version" pair or the reverse

### Requirement: Version check and change are indivisible
The system SHALL perform the check of the expected version and the change as one action: of several concurrent operations
on one number with one and the same expected version, at most one changes data successfully, and the rest get
"version conflict". Trace: CUJ9, CUJ12, X-5. Decisions: ADR 0029 (isolation is an owner decision, OD-5, D-8).

#### Scenario: Two renames with one version
- **WHEN** two clients simultaneously rename the names of different dates of one number with one and the same expected version
- **THEN** one is performed; the other gets a "version conflict" with the version after the first, and its data is not applied

#### Scenario: Rename and delete
- **WHEN** a rename and a delete of one record arrive simultaneously with one version
- **THEN** exactly one operation is performed; the result corresponds to some order "first one, then the other"

#### Scenario: Different numbers do not interfere
- **WHEN** records of different numbers are changed simultaneously
- **THEN** the operations do not affect each other and both can be performed

### Requirement: Visibility of an acknowledged operation
The system SHALL show every search started after the client received a successful response to a write operation
the result of that operation in full, and show readers running concurrently with it the state either before or after
the operation in full, with no intermediate state. Trace: CUJ1, CUJ6, CUJ12; ADR 0014. Decisions: ADR 0029 (isolation is an owner decision, OD-5, D-8).

#### Scenario: Find right after saving
- **WHEN** a client received an add success and then searches by number and by name word prefix
- **THEN** the record is present in both searches (CUJ1)

#### Scenario: Re-read after delete
- **WHEN** a client received a delete success and re-queries the list
- **THEN** the deleted record is not in the list (CUJ6)

#### Scenario: Reader during an operation
- **WHEN** a search runs concurrently with a write operation
- **THEN** it sees either the state before the operation or after it; it does not see a partial result (for example, a record without a version
  change)

### Requirement: Clean and crash shutdown
The system SHALL treat as a clean shutdown a stop by operator command or SIGTERM/SIGINT after which every accepted operation
is acknowledged or rejected, the data and the clean-shutdown marker are durably written, and the process exits with code 0; any other
termination (SIGKILL, including when idle, power loss, panic, OOM) is a crash. The marker is cleared durably before
the first operation after start. Trace: CUJ14, CUJ15; ADR 0028 items 6(b), 10.

#### Scenario: Clean shutdown with an operation in flight
- **WHEN** during a write in flight the server receives SIGTERM
- **THEN** the operation is acknowledged to the client or rejected before the process ends; exit code 0 only after that; after
  start, the version issued in the acknowledgment is valid

#### Scenario: Shutdown did not finish in time
- **WHEN** a write in flight did not finish before the shutdown deadline, and the process was terminated forcibly
- **THEN** the shutdown is a crash: after start, a rename or delete with a version issued before the stop gives a "version conflict"

#### Scenario: SIGKILL when idle
- **WHEN** the process is killed with SIGKILL when no operations were in flight, and the data is intact
- **THEN** the shutdown is a crash (there is no clean-shutdown marker): after start, versions issued before the stop give
  a "version conflict"; the data is unchanged and not lost

#### Scenario: Marker cleared before the first operation
- **WHEN** the server is started after a clean shutdown, accepted the first write operation and crashed
- **THEN** the next start is a crash start: there is no clean-shutdown marker on the medium

### Requirement: Versions after planned restart, crash restart and restore
After a planned restart the system SHALL keep previously issued versions valid; after a crash
restart and a start from a backup created by the command, it SHALL reject with a "version conflict" any rename or
delete with a version issued before the event, for any number (including an empty one). Whether a shutdown is clean follows "Clean and
crash shutdown", the boundary of the guarantee follows "Version is never repeated" (OD-5, D-17). Trace: CUJ14, CUJ15, X-5;
OD-3 Q-77, OD-4 item 5. Decisions: ADR 0028, 0027.

#### Scenario: Planned restart
- **WHEN** the server was stopped cleanly (operator command or SIGTERM; the process exited with code 0 after answering all
  accepted operations) and started again, and a client sends a rename with a version read before the stop, with current
  data
- **THEN** the version is valid; the action is performed

#### Scenario: Crash restart
- **WHEN** the server crashed (SIGKILL, power loss, panic, OOM) and was restarted, and a client sends a rename or delete with a version read before the crash
- **THEN** "version conflict" for any number; the data is unchanged; after re-reading (search by number) the action goes through

#### Scenario: Number is empty before and after the event
- **WHEN** a client read the initial version of an empty number, the server was restarted after a crash or started from a backup, the number
  is still empty, and the client renames or deletes a record of this number with the old initial version
- **THEN** "version conflict" (not "not found"); the data is unchanged; the current version in the response is the initial version of the new
  generation

#### Scenario: Start from a backup
- **WHEN** the server is started from a backup created by the backup command, and a client sends a rename with a version
  read before the restore (including one equal to the version stored in the backup)
- **THEN** "version conflict"; the data is unchanged

#### Scenario: Two starts from one backup
- **WHEN** the same backup is brought up twice (from one archive) on one installation (server settings preserved), and
  an operation on one number is performed in each start
- **THEN** the number version after the first start is not equal to the version after the second; a rename with a version from the first start in the
  second start gives a "version conflict" (deterministically, D-17, ADR 0028 item 9 (c))

#### Scenario: Backup on a machine without the previous settings
- **WHEN** a backup created by the command is started on a machine without the settings of the previous installation (or the settings are restored from
  an old settings copy)
- **THEN** there is no deterministic guarantee between generations; the guarantee is probabilistic, a version coincides with a previously issued one with probability
  at most 2^-64 per event (D-17); the scenario is verified by reviewing the mechanism (the capacity of the random part of the version is at least
  128 bits from the OS source), not by observing a probability; the mechanism test uses a substitutable randomness source
  (ADR 0028 item 9, tests)

#### Scenario: Files copied another way
- **WHEN** the server is started on files copied by something other than the backup command
- **THEN** this is not a "backup" (owner decision, OD-5, D-13, ADR 0028 item 7): restore this way is not supported; behavior is determined by the
  file state at start: after a clean shutdown, like a planned restart, otherwise like a crash

#### Scenario: Cold copy and the source database that kept running
- **WHEN** the directory of a stopped database is copied with `cp`, the source database kept running and issued version V of a number, and then
  the copy is started and changes the same number
- **THEN** the "version is never repeated" guarantee does not apply on such files (the price of D-13): a change with V is accepted if and
  only if the number's current version on the copy equals V; with a backup created by the command this is ruled out within the bounds of
  D-17; the scenario is verified by reviewing the mechanism, not by a test

#### Scenario: Mass conflicts are acceptable
- **WHEN** after a restore or a crash restart many clients get a "version conflict"
- **THEN** each gets a "version conflict" (not another refusal kind); after re-reading the number by a search by number, a retry of the
  action with the current version is performed

#### Scenario: Versions do not repeat through lost changes
- **WHEN** after a restore from a backup or a restart a new operation is performed on a number, and earlier versions had been issued
  for changes that were not preserved in the database
- **THEN** the number version is equal to none previously issued to this number, including the unpreserved ones: deterministically
  on the same installation (settings preserved), otherwise within the 2^-64 bound (D-17); within one generation, always

### Requirement: Adding without a date for a known number
When an add is made without a date and the number has a record for 1876-03-07 with the same name (names are compared as for an identical retry of an add with a date), the system SHALL answer "unchanged" with the current version; otherwise, if there are records for any date, "number known" with the current version; the data and the version do not change. The check and the record creation SHALL be one writer step: of two concurrent adds without a date of a new number, one creates the record. Trace: CUJ1, CUJ2, CUJ9; OD-6, OD-8; ADR 0005, 0025.

#### Scenario: Number known
- **GIVEN** the number has records for 2020-01-01 «Bob» and 2010-05-05 «Ann»
- **WHEN** «Eve» is added without a date
- **THEN** "number known" with the current number version; the records and the version are unchanged

#### Scenario: Only record on 1876-03-07 with the same name
- **GIVEN** the number has one record for 1876-03-07 with the name «Ann»
- **WHEN** «Ann» is added without a date
- **THEN** success "unchanged" (OD-8); the record and the number version are unchanged

#### Scenario: Record on 1876-03-07 with the same name and other records
- **GIVEN** the number has records for 1876-03-07 «Ann» and for 2020-01-01 «Bob»
- **WHEN** «Ann» is added without a date
- **THEN** success "unchanged"; the data and the version are unchanged

#### Scenario: Record on 1876-03-07 with a different name
- **GIVEN** the number has one record for 1876-03-07 with the name «Bob»
- **WHEN** «Ann» is added without a date
- **THEN** "number known" with the current version; the record is unchanged, there is no name conflict

#### Scenario: Same form of the name
- **GIVEN** the number has a record for 1876-03-07 «Ann»
- **WHEN** « Ann » is added without a date (spaces at the edges)
- **THEN** names are compared after normalization (NFC, edge trimming): success "unchanged"

#### Scenario: CUJ1/CUJ2 race
- **WHEN** two adds without a date of one new number run at the same time
- **THEN** one creates the record for 1876-03-07, the other gets "unchanged" if the name is the same, otherwise "number known", with the version after the first; there is no silent date substitution for a known number

#### Scenario: Number with no records after deletes
- **GIVEN** all records of the number were deleted
- **WHEN** a record is added without a date
- **THEN** the record is created for 1876-03-07 (a number with no records is an unknown one)
