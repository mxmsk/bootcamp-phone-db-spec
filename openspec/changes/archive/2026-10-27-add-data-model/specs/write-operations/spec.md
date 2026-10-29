# write-operations

## Purpose

The observable semantics of the operations that add, rename and delete a record (number, date) → name, and of number
versions: inputs, results, error kinds and their check order, idempotency, version properties, atomicity and visibility.
No API format, protocol or UI form; the storage mechanics are requirements in the section "Requirements for other areas"
of `design.md`.

## ADDED Requirements

### Requirement: Adding a record
The system SHALL, on an add operation with the inputs "number", "name" and an optional "date", create the record (number,
date) → name if there is no record for this (number, date); an expected version is not required. Without a date the zero
date 1876-03-07 is taken (`records`, "Zero date when no date is given"). Trace: CUJ1, CUJ2, CUJ9; ADR 0002 (items 4–5),
0005. Decisions: ADR 0025 (owner decision, OD-5, D-6), 0026 (owner decision, OD-5, D-7).

#### Scenario: New number
- **WHEN** a record is added with a number that has no records
- **THEN** the record is created, the number is found by name, and the name is found by number (CUJ1)
- **AND** the number version changed from the initial version of the current generation to another one

#### Scenario: Known number, new date
- **WHEN** a record is added for a date the number does not have yet
- **THEN** the record is created, the number's records on other dates are unchanged, the number version changed (CUJ2)

#### Scenario: The same name on another date
- **WHEN** a name that already owns a number on one date is added to the same number on another date
- **THEN** a separate record is created; both records appear in the number history

#### Scenario: Date not given
- **WHEN** the add is performed without a date
- **THEN** the record is created on 1876-03-07; an explicitly given 1876-03-07 yields the same record

### Requirement: Name conflict and add idempotency
When adding to an occupied (number, date) with a different name, the system SHALL reject the operation with a "name
conflict" carrying the record's current name and the current number version, without changing data; with the same name,
respond with success "unchanged". Names are compared by exact spelling after normalization (`records`, "Name and query
normalization": NFC and edge trimming). Trace: CUJ3, CUJ9; ADR 0003, 0005. Decisions: ADR 0025 (owner decision, OD-5, D-6).

#### Scenario: A different name
- **WHEN** «Иван» is recorded on (number, date), and «Пётр» is added
- **THEN** "name conflict"; the response contains the current name «Иван» and the number version; the record and the
  version are unchanged

#### Scenario: A different spelling
- **WHEN** «ANN» is recorded on (number, date), and «Ann» is added
- **THEN** "name conflict" (comparison by exact spelling, case is distinguished)

#### Scenario: Identical retry
- **WHEN** a record is added that already exists with the same number, date and name
- **THEN** success "unchanged"; the record and the number version are unchanged

#### Scenario: Difference only in edge whitespace and NFC form
- **WHEN** the name being added differs from the recorded one only in edge whitespace or normalization form
- **THEN** it is the same name: success "unchanged"

#### Scenario: Concurrent adds of one (number, date)
- **WHEN** two adds to one free (number, date) with different names are performed at the same time
- **THEN** exactly one creates the record, the second gets a "name conflict" with the name of the first

### Requirement: Renaming a record
The system SHALL, on a rename operation with the inputs "number", "date", "new name", "expected version", replace the name
of the existing record (number, date), without keeping the previous one. The date is mandatory: without it — "invalid
input" (date, "missing"), with no substitution (`records`, "Zero date when no date is given"; owner decision, OD-5, Q-90).
An add does not overwrite the name. Trace: CUJ3, CUJ9, CUJ12; ADR 0005. Decisions: ADR 0025 (owner decision, OD-5, D-6),
0026 (owner decision, OD-5, D-7), 0027.

#### Scenario: Acknowledged overwrite
- **WHEN** the name of an existing record is renamed with the current version and a new name different from the current one
- **THEN** the name is replaced, the record's previous name is returned by no search, the number version changed (CUJ3)
- **AND** the number's records on other dates are unchanged

#### Scenario: The same name with the current version
- **WHEN** the new name equals the current one after normalization (`records`, "Name and query normalization") and the
  version is current
- **THEN** success "unchanged"; the number version does not change

#### Scenario: The record is absent
- **WHEN** the record (number, date) is absent and the number version is current
- **THEN** "not found"; the data is unchanged

#### Scenario: Invalid new name
- **WHEN** the new name is invalid under the `records` requirements: empty or only spaces, with a control character, or
  longer than 200 scalar values of the normalized text ("Text length and length cap")
- **THEN** "invalid input" with a (name, reason) pair from the `records` dictionary — "empty", "control character" or "too
  long"; the data is unchanged, there is no truncation

#### Scenario: Date not given
- **WHEN** the rename is sent without a date (the version is current, the number has a record on 1876-03-07)
- **THEN** "invalid input" with the (date, "missing") pair from the `records` dictionary; the record on 1876-03-07 and the
  records on other dates are untouched, the number version does not change (owner decision, OD-5, Q-90; ADR 0025 item 6)

#### Scenario: Explicit zero date
- **WHEN** the rename is sent with the explicit date 1876-03-07, a record exists on this date and the version is current
- **THEN** the record on 1876-03-07 is changed (the same key as with substitution on add)

### Requirement: Deleting a record
The system SHALL, on a delete operation with the inputs "number", "date", "expected version", delete exactly one record
(number, date), without touching the other records of the number; the date is mandatory: without it — "invalid input"
(date, "missing"), with no substitution (`records`, "Zero date when no date is given"; owner decision OD-5, Q-90); a missing
record with the current version — "not found". Trace: CUJ6, CUJ9, CUJ12; ADR 0005. Decisions: ADR 0025, 0026 (OD-5: D-6,
D-7), 0027.

#### Scenario: Deleting one record
- **WHEN** an existing record is deleted with the current version
- **THEN** only this record is deleted; the other records of the number are intact; the number version changed (CUJ6)

#### Scenario: Deleting the last record
- **WHEN** the only record of the number is deleted
- **THEN** the number is like an unknown one: its history is empty, the version is the initial version of the current
  generation (ADR 0027)

#### Scenario: The record is absent
- **WHEN** the record is absent and the number version is current
- **THEN** "not found"; the data is unchanged

#### Scenario: Date not given
- **WHEN** the delete is sent without a date, the number has records on 1876-03-07 and on other dates, the version is
  current
- **THEN** "invalid input" with the (date, "missing") pair; no record of the number is deleted, the version does not change
  (owner decision, OD-5, Q-90; ADR 0025 item 6)

#### Scenario: Explicit zero date
- **WHEN** the delete is sent with the explicit date 1876-03-07, a record exists on this date and the version is current
- **THEN** the record on 1876-03-07 is deleted; the other records of the number are intact

#### Scenario: The deleted does not come back
- **WHEN** a record is deleted and the acknowledgement is received
- **THEN** no search returns it; a deleted record does not reappear without a new add

### Requirement: Write error kinds
The system SHALL distinguish four kinds of errors on write: "invalid input" (naming the field: number, date, name, expected
version), "name conflict" (add), "version conflict" (rename, delete) and "not found" (rename, delete); on any error the data
and versions do not change. Trace: CUJ3, CUJ6, CUJ9, CUJ12. Decisions: ADR 0025 (owner decision, OD-5, D-6).

#### Scenario: Contents of a version conflict
- **WHEN** the expected version is not equal to the current number version
- **THEN** "version conflict"; the response contains the current number version (CUJ9, CUJ12)

#### Scenario: Invalid input is named by field
- **WHEN** the number has an invalid format, or the date is missing (rename, delete), or the date is not a calendar day
  (for example, 1875-02-30), or the date is outside the range 1876-03-07…9999-12-31 (for example, 1876-03-06 or 10000-01-01
  when a five-digit year is written), or the name is invalid
- **THEN** "invalid input" with a (field, reason) pair from `records`, "Value error kind dictionary": (date, "missing"),
  (date, "nonexistent day"), (date, "out of range"), etc.; the reason dictionary is not duplicated here; the "expected
  version" field is this capability's own field and is not part of the `records` dictionary; the data is unchanged

#### Scenario: An error changes nothing
- **WHEN** the operation ended with any write error
- **THEN** the number version and all its records are in the same state as before the operation

#### Scenario: Other kinds of refusal are out of the area
- **WHEN** a request is rejected because of permissions (CUJ8), a request limit, or the "read-only"/"storage error"
  database state (CUJ16)
- **THEN** these are other kinds of refusal, defined by other areas; the data is unchanged

### Requirement: Check order on write
The system SHALL check an operation in this order: input (number, date — mandatory for rename and delete, optional for add,
name, for rename and delete — the presence and kind of the expected version; the first invalid field is reported); for
rename and delete — the version ("version conflict"), then the existence of the record ("not found"); for add —
occupancy of (number, date). Access and database state checks go earlier. Trace: CUJ3, CUJ9, CUJ12. Decisions: ADR 0025
(owner decision, OD-5, D-6).

#### Scenario: A stale version and a missing record
- **WHEN** a record that does not exist is renamed or deleted with a stale version
- **THEN** "version conflict" with the current version, not "not found"

#### Scenario: The record disappeared because of someone else's delete
- **WHEN** the record was deleted concurrently, and a client renames or deletes it with a version read before that
- **THEN** "version conflict" (the delete changed the number version), not "not found" (CUJ12)

#### Scenario: Current version and a missing record
- **WHEN** the record is absent, and the version is current
- **THEN** "not found"

#### Scenario: The same name with a stale version
- **WHEN** a rename to the same name is sent with a stale version
- **THEN** "version conflict"; success "unchanged" is returned only with the current version (OD-4, item 7)

#### Scenario: Invalid input before the version
- **WHEN** the request contains both an invalid name and a stale version
- **THEN** "invalid input" (the "name" field), not "version conflict"

#### Scenario: Several invalid fields
- **WHEN** both the number and the name are invalid
- **THEN** the "number" field is reported (order: number, date, name, expected version)

#### Scenario: A missing date before the name and the version
- **WHEN** a rename is sent without a date, with an invalid name and without an expected version
- **THEN** "invalid input" with the (date, "missing") pair: the date is checked before the name and the expected version

#### Scenario: A missing date before a version conflict
- **WHEN** a delete is sent without a date and with a stale version
- **THEN** "invalid input" (date, "missing"), not "version conflict": the input is checked before the version

#### Scenario: Add without a date
- **WHEN** the add is performed without a date
- **THEN** there is no date error: 1876-03-07 is taken (`records`, "Zero date when no date is given")

#### Scenario: Add with a version of the wrong kind
- **WHEN** an add is received with an expected version that does not belong to the version type (wrong kind), or with a
  version of the right kind not issued by the server (an add does not check the expected version at all)
- **THEN** the version is ignored, neither "invalid input" nor "version conflict" is returned; the operation is processed
  as usual

#### Scenario: Input before occupancy
- **WHEN** an add contains an invalid name for an occupied (number, date)
- **THEN** "invalid input", not "name conflict"

### Requirement: Expected version is mandatory
The system SHALL require the expected number version on rename and delete and refuse with "invalid input" (the "expected
version" field) if it is absent or of the wrong kind; on add it is not required, not checked and does not affect the
result; a value of the version type that is not equal to the current number version — "version conflict". Trace: CUJ9,
CUJ12; owner decision "Operations: the expected version is mandatory". Decisions: ADR 0025 (owner decision, OD-5, D-6,
item 7).

#### Scenario: No version on rename or delete
- **WHEN** a rename or delete is received without an expected version
- **THEN** "invalid input" (the "expected version" field); the data is unchanged

#### Scenario: Add without a version
- **WHEN** the add is performed without a version
- **THEN** the operation is processed as usual

#### Scenario: Not a version
- **WHEN** a value that does not belong to the version type is passed as the expected version (wrong kind: the kind is the
  syntax and format of the version type, defined by the API area; checked by format, independent of the database state)
- **THEN** "invalid input" (the "expected version" field), not "version conflict"

#### Scenario: A version of the right kind that the server did not issue
- **WHEN** a value of the version type that the server never issued for this number (the version of another number, of a
  previous generation, invented by the client) is passed in a rename or delete
- **THEN** "version conflict" with the current number version (any value of the version type not equal to the current
  number version is a conflict); "invalid input" is not returned; the data is unchanged

### Requirement: Result of a successful operation
The system SHALL, on a successful add, rename or delete, return the outcome ("changed" or "unchanged") and the number
version current after the operation; the format of representing the version is outside this area. Trace: CUJ1, CUJ3,
CUJ6, CUJ9, X-5. Decisions: ADR 0026 (owner decision, OD-5, D-7).

#### Scenario: Version in a successful response
- **WHEN** the operation changed data
- **THEN** the result: "changed" and the new number version, usable as the expected one for the next rename without
  re-reading

#### Scenario: Identical retry
- **WHEN** an add of an identical record or a rename to the same name with the current version
- **THEN** the result: "unchanged" and the current number version, equal to the previous one

#### Scenario: Deleting the last record
- **WHEN** the only record of the number is deleted
- **THEN** the result: "changed" and the initial version of the current generation

### Requirement: Number version in search results
The system SHALL return in every record of the result of any search (by number and by name) the version of its number,
consistent with the records of this number in this result; search by number returns the number version also for an empty
history (the initial one). The version belongs to the whole number, not to the records shown. Trace: CUJ6, CUJ9, CUJ12,
X-5. Decisions: ADR 0029 (item 5 — owner decision, OD-5, D-8), 0026.

#### Scenario: Results by number
- **WHEN** a search by number with records is performed
- **THEN** the result contains all records of the number and its version; the version is accepted as the expected one in
  rename and delete (CUJ12)

#### Scenario: A number with no records
- **WHEN** a search by number with no records is performed
- **THEN** the result is empty and contains the initial version of the current generation

#### Scenario: Results by name
- **WHEN** a search by name is performed, and one of the number's three records entered the result
- **THEN** the record contains the version of the whole number; it is enough to delete this record from the results (CUJ6)
  without re-reading

#### Scenario: One version per number in a result
- **WHEN** one search result (one page) contains several records of one number
- **THEN** they have the same version, and it corresponds to the state of the number from which these records were taken

#### Scenario: Re-read, retried
- **WHEN** an action is rejected with a "version conflict", the client performs a search by number and repeats the action
  with the version from the result, and the number has not changed in the meantime
- **THEN** the action is performed

#### Scenario: A stale version between pages
- **WHEN** the number is changed between receiving two pages of results by name, and the client acts on the version from
  the first
- **THEN** "version conflict"; the data is unchanged (consistency between pages is not promised: owner decision, OD-5, D-8,
  Q-56)

### Requirement: Retrying an operation after an unknown outcome
The system SHALL, when a write operation whose result the client did not receive is retried, and if the number's records
did not change between attempts, not produce a second effect: adding an identical record — success "unchanged"; rename and
delete with the previous version — "version conflict". If the number's records changed between attempts, the usual rules
apply (rename and delete — "version conflict"; add — by key occupancy). Trace: CUJ9, CUJ12, CUJ14. Decisions: ADR 0026
(owner decision, OD-5, D-7).

#### Scenario: Retrying an add
- **WHEN** an add was performed, the response was lost, the number's records did not change, and the client retries the add
- **THEN** success "unchanged"; there is one record in the database

#### Scenario: Retrying an add after someone else's delete
- **WHEN** an add was performed, the response was lost, another client deleted this record, and the client retries the add
- **THEN** the record is created anew (outcome "changed"): an add carries no version and does not distinguish "the first
  attempt" from a new one (owner decision "add without a version"); a client that needs not to resurrect the deleted checks
  with a search by number

#### Scenario: Retrying an add after someone else's name replacement
- **WHEN** an add of «Иван» was performed, the response was lost, another client replaced the name of this record with
  «Пётр», and the client retries the add of «Иван»
- **THEN** "name conflict" with the current name «Пётр» and the number version; the data is unchanged

#### Scenario: Retrying a delete
- **WHEN** a delete was performed, the response was lost, and the client retries the delete with the previous version
- **THEN** "version conflict" with the current version; the client re-reads the number with a search by number and sees
  that the record is gone

#### Scenario: Retrying a rename
- **WHEN** a rename was performed, the response was lost, and the client retries it with the previous version
- **THEN** "version conflict"; after re-reading the history the client sees its name and does not repeat

#### Scenario: Retrying a rename after someone else's delete
- **WHEN** a rename was performed, the response was lost, another client deleted the record, and the client retries the
  rename with the previous version
- **THEN** "version conflict" (the delete changed the version); the record is not resurrected

### Requirement: When the number version changes
The system SHALL, within one version generation (between crash restart and start-from-backup events), change a number's
version if and only if the records of this number change: creating a record, replacing a name with a different one,
deleting a record. An identical retry, a rename to the same name, an error, a read, a planned restart and operations on
other numbers do not change the version. Trace: CUJ1, CUJ3, CUJ6, CUJ12, X-5; owner decision "versioning belongs to the
number". Decisions: ADR 0027, 0028.

#### Scenario: Create, replace a name, delete
- **WHEN** any of these operations that changes data is performed
- **THEN** the number version became a value different from the previous one

#### Scenario: Another date of the same number was changed
- **WHEN** the number's record on another date is renamed or deleted
- **THEN** the number version changed, and an action with the previous version is rejected with a "version conflict" (CUJ12)

#### Scenario: Another number
- **WHEN** the records of another number are changed
- **THEN** this number's version did not change

#### Scenario: Operations that do not change data
- **WHEN** a read, an erroneous operation, an identical retry or a planned restart is performed
- **THEN** the number version did not change

#### Scenario: Exception: generation change
- **WHEN** the server is restarted after a crash or started from a backup (ADR 0028)
- **THEN** the versions of all numbers, including those with unchanged records, receive values different from those issued
  earlier, within the guarantee of "Version is never repeated" (owner decision, OD-5, D-17); this is the only event that
  changes a version without changing records

### Requirement: Initial version and a number with no records
The system SHALL consider the version of a number with no records to be the initial version of the current generation: one
for all numbers, unchanged on planned restarts; a number with records never has the initial version. A number all of whose
records have been deleted is indistinguishable from an unknown one. This refines X-5 for a number with no records (ADR
0027, item 4). Trace: CUJ1, CUJ6, X-5. Decisions: ADR 0027, 0028.

#### Scenario: Unknown number
- **WHEN** the number has no records and never had any
- **THEN** its version is the initial version of the current generation

#### Scenario: After deleting all records
- **WHEN** all records of the number are deleted
- **THEN** the number version is the initial version of the current generation, a search by number gives an empty list

#### Scenario: Planned restart
- **WHEN** a client read the initial version of an empty number, the server was restarted in a planned way, the number is
  still empty
- **THEN** the initial version is the same

#### Scenario: An action by the initial version
- **WHEN** a rename or delete is sent with the initial version, and the number has records
- **THEN** "version conflict" (the version of a number with records is not the initial one)

#### Scenario: A rename with no records by the initial version
- **WHEN** a record of a number with no records is renamed with the initial version of the current generation
- **THEN** "not found"; the data is unchanged

### Requirement: Version is never repeated
The system SHALL, within a generation, not issue to a number with records a version previously issued to this number
(deterministically); between generations — also deterministically if the server settings are preserved (the place outside
the backup where tokens are kept, OD-4 item 9), otherwise the version coincides with a previously issued one with
probability no higher than 2^-64 per generation-change event (owner decision, OD-5, D-17, ADR 0028 items 6(c), 9); the
version is opaque. Trace: X-5, CUJ12. Decisions: ADR 0027, 0028.

#### Scenario: Deleted everything and added again
- **WHEN** a client read version V of a number with records, then all records are deleted, the number is added anew, and
  the client sends a rename or delete with V
- **THEN** the current number version is not equal to V, the action is rejected with a "version conflict", the data is
  unchanged (the guarantee applies to a database continuing its own history and to a backup made by the command; on files
  copied otherwise a repeat is possible, owner decision, OD-5, D-13)

#### Scenario: An empty number and someone else's add
- **WHEN** a client read the initial version, then the number was added by another client, and the first client renames a
  record with the initial version
- **THEN** "version conflict"; the data is unchanged

#### Scenario: Initial version and someone else's add followed by a delete
- **WHEN** a client read the initial version, then the number was added and again fully deleted, and the client renames a
  record with the initial version
- **THEN** "not found"; the data is unchanged (a repeat of the initial version within a generation is harmless: the target
  record does not exist)

#### Scenario: Comparison only for equality
- **WHEN** a client received two versions of one number
- **THEN** the specification does not promise which of them is newer and does not require the client to order them

### Requirement: Record and version change are indivisible
The system SHALL perform the change of a number's records and the change of its version as one indivisible change: neither
on failure nor for a reader is there a state where the record is changed and the version is not, or vice versa. Trace: X-5,
CUJ14; ADR 0006 item 6 (b). Decisions: ADR 0028, 0029 (isolation — owner decision, OD-5, D-8).

#### Scenario: Failure at the moment of the operation
- **WHEN** the server crashed during a write operation that had not been acknowledged
- **THEN** after restart the operation is either fully present (the record and the new version) or fully absent (the
  previous record and version) (CUJ14)

#### Scenario: A reader sees a consistent pair
- **WHEN** a reader receives a number's records together with the version (search by number)
- **THEN** the version corresponds to exactly these records: there is no "new history — old version" pair or vice versa

### Requirement: Version check and change are indivisible
The system SHALL perform the check of the expected version and the change as one action: of several simultaneous operations
on one number with the same expected version, at most one successfully changes data, the others get a "version conflict".
Trace: CUJ9, CUJ12, X-5. Decisions: ADR 0029 (isolation — owner decision, OD-5, D-8).

#### Scenario: Two renames with one version
- **WHEN** two clients simultaneously rename the names of different dates of one number with the same expected version
- **THEN** one is performed; the second gets a "version conflict" with the version after the first, and its data is not applied

#### Scenario: A rename and a delete
- **WHEN** a rename and a delete of one record arrive simultaneously with one version
- **THEN** exactly one operation is performed; the result corresponds to some order "one first, then the other"

#### Scenario: Different numbers do not interfere
- **WHEN** the records of different numbers are changed simultaneously
- **THEN** the operations do not affect each other and both can be performed

### Requirement: Visibility of an acknowledged operation
The system SHALL show to any search begun after the client received a success response for a write operation the result of
this operation in full, and to readers running concurrently with it — the state either fully before or fully after the
operation, with no intermediate states. Trace: CUJ1, CUJ6, CUJ12; ADR 0014. Decisions: ADR 0029 (isolation — owner decision,
OD-5, D-8).

#### Scenario: Found right after saving
- **WHEN** a client received the success of an add and then searches by number and by a word prefix of the name
- **THEN** the record is present in both searches (CUJ1)

#### Scenario: Re-read after a delete
- **WHEN** a client received the success of a delete and re-queries the list
- **THEN** the deleted record is not in the list (CUJ6)

#### Scenario: A reader during an operation
- **WHEN** a search runs concurrently with a write operation
- **THEN** it sees either the state before the operation or after; it does not see a partial result (for example, a record
  without a version change)

### Requirement: Clean and crash shutdown
The system SHALL consider a shutdown a clean one if it is by operator command or SIGTERM/SIGINT, after which every accepted
operation is acknowledged or rejected, the data and the clean-shutdown marker are durably written, and the process has
exited with code 0; any other termination (SIGKILL, including at idle, power loss, panic, OOM) is a crash. The marker is
cleared durably before the first operation after start. Trace: CUJ14, CUJ15; ADR 0028 items 6(b), 10.

#### Scenario: Clean shutdown with an operation in flight
- **WHEN** during a write in flight the server receives SIGTERM
- **THEN** the operation is acknowledged to the client or rejected before the process ends; exit code 0 only after that;
  after start the version issued in the acknowledgement is valid

#### Scenario: The shutdown did not meet the deadline
- **WHEN** a write in flight did not finish before the shutdown deadline, and the process was terminated forcibly
- **THEN** the shutdown is a crash: after start a rename or delete by a version issued before the shutdown — "version
  conflict"

#### Scenario: SIGKILL at idle
- **WHEN** the process was killed with SIGKILL when there were no operations in flight, and the data is intact
- **THEN** the shutdown is a crash (there is no clean-shutdown marker): after start the versions issued before the shutdown
  give a "version conflict"; the data is unchanged and not lost

#### Scenario: The marker is cleared before the first operation
- **WHEN** the server was started after a clean shutdown, accepted the first write operation and crashed
- **THEN** the next start is a crash start: there is no clean-shutdown marker on the medium

### Requirement: Versions after planned restart, crash restart and restore
The system SHALL after a planned restart keep previously issued versions valid; after a crash restart and a start from a
backup created by the command — reject with a "version conflict" any rename or delete by a version issued before the event,
for any number (including an empty one). Whether a shutdown is clean is per "Clean and crash shutdown", the guarantee
boundary is per "Version is never repeated" (OD-5, D-17). Trace: CUJ14, CUJ15, X-5; OD-3 Q-77, OD-4 item 5. Decisions: ADR
0028, 0027.

#### Scenario: Planned restart
- **WHEN** the server was stopped cleanly (operator command or SIGTERM; the process exited with code 0 after responding to
  all accepted operations) and started again, and a client sends a rename with a version read before the shutdown, with
  current data
- **THEN** the version is valid; the action is performed

#### Scenario: Crash restart
- **WHEN** the server crashed (SIGKILL, power loss, panic, OOM) and was restarted, and a client sends a rename or delete
  with a version read before the crash
- **THEN** "version conflict" for any number; the data is unchanged; after re-reading (search by number) the action passes

#### Scenario: The number is empty before and after the event
- **WHEN** a client read the initial version of an empty number, the server was restarted after a crash or started from a
  backup, the number is still empty, and the client renames or deletes a record of this number with the previous initial
  version
- **THEN** "version conflict" (not "not found"); the data is unchanged; the current version in the response is the initial
  version of the new generation

#### Scenario: Start from a backup
- **WHEN** the server is started from a backup created by the backup command, and a client sends a rename with a version
  read before the restore (including one equal to the version stored in the backup)
- **THEN** "version conflict"; the data is unchanged

#### Scenario: Two starts from one backup
- **WHEN** one and the same backup is brought up twice (from one archive) on one installation (server settings preserved),
  and an operation on one number is performed in each start
- **THEN** the number version after the first start is not equal to the version after the second; a rename by a version
  from the first start in the second start — "version conflict" (deterministically, D-17, ADR 0028 item 9 (c))

#### Scenario: A backup on a machine without the previous settings
- **WHEN** a backup created by the command is started on a machine without the settings of the previous installation (or
  the settings are restored from an old copy of the settings)
- **THEN** there is no deterministic guarantee between generations; the guarantee is probabilistic, a coincidence of the
  version with one issued earlier is no higher than 2^-64 per event (D-17); the scenario is verified by a review of the
  mechanism (the capacity of the random part of the version is at least 128 bits from an OS source), not by observing
  probability; the mechanism test uses a substitutable randomness source (ADR 0028 item 9, tests)

#### Scenario: Files copied otherwise
- **WHEN** the server is started on files copied not by the backup command
- **THEN** this is not a "backup" (owner decision, OD-5, D-13, ADR 0028 item 7): restore is not supported this way; the
  behavior is determined by the state of the files at start: after a clean shutdown as a planned restart, otherwise as a
  crash

#### Scenario: A cold copy and the source database that kept working
- **WHEN** the directory of a stopped database is copied with `cp`, the source database kept working and issued version V
  of a number, and then the copy is started and changes the same number
- **THEN** the guarantee "version is never repeated" does not apply on such files (the price of D-13): a rename by V is
  accepted if and only if the current version of the number on the copy equals V; with a backup created by the command
  this is excluded within the bounds of D-17; the scenario is verified by a review of the mechanism, not by a test

#### Scenario: Mass conflicts are acceptable
- **WHEN** after a restore or a crash restart many clients get a "version conflict"
- **THEN** each gets a "version conflict" (not another kind of refusal); after re-reading the number with a search by number
  a retry of the action with the current version is performed

#### Scenario: Versions are not repeated through lost changes
- **WHEN** after a restore from a backup or a restart a new operation on a number is performed, and earlier versions of
  changes were issued that were not preserved in the database
- **THEN** the number version is equal to none issued earlier to this number, including the unpreserved ones:
  deterministically on the same installation (settings preserved), otherwise within the 2^-64 bound (D-17); within one
  generation — always
