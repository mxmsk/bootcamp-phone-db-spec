# number-search Specification

## Purpose
The observable semantics of searching by phone number: exact match only on the canonical form of the number, all records
of the number by date DESC without pagination (at most 2 967 164 records), the number version once — as a result field (OD-6),
an empty list with a version for a number not found, a value error (number, "invalid shape") for an invalid format. The canonical form, separator cleaning and format validation are defined by the `records` capability;
here they are only applied.

## Requirements

### Requirement: Search by number: exact match only
For a number, the system SHALL return the records only of the number whose canonical form equals the canonical form of
the query. Numbers that merely contain the query or start with it SHALL NOT appear in the results. Trace: CUJ4, CUJ8; ADR 0007.

#### Scenario: Full number
- **GIVEN** the number «+14155550100» has records
- **WHEN** a search by «+14155550100» is performed
- **THEN** the records of this number are returned

#### Scenario: Part of a number finds nothing
- **GIVEN** the database contains the numbers «+14155550100» and «+14155550199»
- **WHEN** a search by «+1415555» or «+4155550100» is performed
- **THEN** the results are empty: no number matched fully

#### Scenario: A number that is the beginning of another
- **GIVEN** the database contains the numbers «+7495» and «+74951234567»
- **WHEN** a search by «+7495» is performed
- **THEN** only the records of the number «+7495» are returned

### Requirement: Same cleaning and canonical form as on save
The system SHALL bring the query number to canonical form using the same cleaning and the same format rules as when
saving a record (the `records` capability), so that a number written in the query with separators finds the same records as
one written without them. Separate normalization rules for search SHALL NOT exist. Trace: CUJ4, CUJ1; ADR 0004, 0007.

#### Scenario: Query with separators
- **GIVEN** the number is saved as «+14155550100»
- **WHEN** a search by «+1 (415) 555-01.00» is performed
- **THEN** the records of this number are returned, as for a search by «+14155550100»

#### Scenario: Number saved with separators
- **GIVEN** the number was entered on save as «+1 415 555 0100»
- **WHEN** a search by «+14155550100» is performed
- **THEN** the records of this number are returned

#### Scenario: Whitespace characters at the edges of the query
- **WHEN** the query contains whitespace characters before and after the number (the character set is per `records`, "Whitespace character set")
- **THEN** they are handled as on save (removed as separators), and the result is the same as for the query
  without them

### Requirement: Results: all records of the number, date DESC, no pagination
For a found number, the system SHALL return all its records (number, date, name) as one complete list ordered by date
descending (from later to earlier, including future dates and the zero date 1876-03-07). Search by number SHALL NOT have
pagination or a cursor. Trace: CUJ4, CUJ2; ADR 0007.

#### Scenario: All records of a number
- **GIVEN** a number has N records on N different dates, names may repeat
- **WHEN** a search by number is performed
- **THEN** exactly N records are returned, none is skipped, regardless of N

#### Scenario: Order by date DESC
- **GIVEN** a number has records on the dates 2020-05-01, 2031-01-01 (future), 1876-03-07 and 2023-12-31
- **WHEN** a search by number is performed
- **THEN** the order of dates is: 2031-01-01, 2023-12-31, 2020-05-01, 1876-03-07

#### Scenario: The order is unambiguous
- **WHEN** a number has several records
- **THEN** the order is fully determined by the date: the date is unique per number (the `records` capability), there are no equal dates, and there are no two
  valid result orders

#### Scenario: One name on different dates
- **GIVEN** the name «Анна» owns a number on two different dates
- **WHEN** a search by number is performed
- **THEN** both records are returned as separate rows, each with its own date

#### Scenario: No pages
- **WHEN** a number has a very large number of records
- **THEN** the results remain a complete list without a cursor and without a "there is a next page" marker

#### Scenario: A write during the results does not change their content
- **GIVEN** results by number with a very large number of records are being delivered to the client in chunks
- **WHEN** other clients change the number during the delivery
- **THEN** the results contain the records and the number version as of the start of the search (a consistent pair), not a mix of states; other clients' writes are not held up by the results (storage and API: ADR 0035 item 6)

#### Scenario: Cut-off results are distinguishable from complete ones
- **WHEN** the results by number are interrupted by a storage error after sending began
- **THEN** the client can establish that the response is incomplete (there is no final integrity marker) and does not take it for the complete history

### Requirement: Upper bound on the number of records per number
The number of records of one number SHALL be bounded by the number of valid dates, 2 967 164 (1876-03-07…9999-12-31, the date is
unique per number; `records`, ADR 0024); there SHALL be no other limit on the number of records of a number. Search by number SHALL
return all records of the number up to this bound without truncation; storage and the API SHALL ensure this (ADR 0035 item 6). Trace: CUJ4, CUJ2; ADR 0007, 0024.

#### Scenario: A number with records on all valid dates
- **GIVEN** a number has a record on each of the 2 967 164 valid days
- **WHEN** a search by number is performed
- **THEN** 2 967 164 records are returned, by date DESC, none is skipped, without a cursor

#### Scenario: There cannot be more records than dates
- **WHEN** a record is added for a number on a date on which it already has a record
- **THEN** per `write-operations` ("Name conflict and add idempotency"): "name conflict" (a different name) or success
  "unchanged" (the same name); no new record is created, and the number of records of the number does not exceed 2 967 164

### Requirement: Number not found: empty list
For a query with a number of valid format that has no records at all, the system SHALL answer with success, an empty list of
records and the number version (the initial version of the current generation), not with an error. The same SHALL be the result after deleting
the last record of a number. The version is a result field, not a record's (an empty list has no records); the source is "Number version in
search results" and "Initial version and a number with no records" of the `write-operations` capability. Trace: CUJ4, CUJ6, X-5; ADR 0007, 0027.

#### Scenario: The number is not in the database
- **WHEN** a search by a number of valid format that is not in the database
- **THEN** the result is success, an empty list of records and the initial version of the current generation, not "not found"

#### Scenario: The last record was deleted
- **GIVEN** a number has one record
- **WHEN** the record is deleted and then a search by this number is performed
- **THEN** the result is an empty list of records and the initial version of the current generation (a number with no records is indistinguishable from an
  unknown one, `write-operations`)

#### Scenario: Version of an empty number
- **WHEN** a search by a number of valid format with no records
- **THEN** the version in the result is the initial version of the current generation, accepted as current by an action addressed to a
  nonexistent record

### Requirement: Malformed number: value error
For a query with a number that failed the record format validation (the `records` capability), the system SHALL return a value
error (number, "invalid shape" — the `records` dictionary) and no results; an empty list SHALL NOT
be returned in this case. The error SHALL differ from the "number not found" result. Trace: CUJ4, CUJ8; ADR 0004, 0007.

#### Scenario: Number without "+"
- **WHEN** a search by «14155550100»
- **THEN** a value error (number, "invalid shape") with the "missing plus" marker (a hint, ADR 0004 item 4; `records`,
  "Valid number shape"), no results

#### Scenario: Letters and empty input
- **WHEN** a search by «+1415abc0100», by an empty string or by a string of only separators
- **THEN** a value error (number, "invalid shape"), no results

#### Scenario: Number too long
- **WHEN** a search by «+» with 16 digits
- **THEN** a value error (number, "invalid shape"); the cap of 1–15 digits is set by `records`, "Valid number shape" (ADR 0004)

#### Scenario: The error is distinguishable from empty results
- **WHEN** the response to an invalid format and the response to a not-found number of valid format are compared
- **THEN** the first is an error, the second is a successful empty list

### Requirement: Search by number: a read that reflects committed changes
Search by number SHALL be a read-only operation: it creates and changes nothing. The results SHALL reflect all
changes acknowledged to the client before the search began. Access permissions are area 3 (here only a consequence: reading does not require
the write permission). Trace: CUJ6, CUJ8, CUJ9; ADR 0035 item 5.

#### Scenario: Client with the read permission
- **WHEN** a client with the "read" permission searches by number
- **THEN** it gets the same result as a client with the write permission for the same query and the same data

#### Scenario: Search does not create a number
- **WHEN** a search is performed by a number of valid format that is not in the database
- **THEN** the data and the number versions are unchanged

#### Scenario: List after a delete
- **GIVEN** a number has records on the dates D1 and D2
- **WHEN** the record (number, D1) is deleted with acknowledgment, then a search by number is performed again
- **THEN** only the record on D2 is returned (CUJ6: after a delete the list is re-queried)

#### Scenario: List after an add and a rename
- **WHEN** a record is added or a name is changed, and the change is acknowledged to the client
- **THEN** the next search by number shows the new record or the new name; the old name of this record is not returned

### Requirement: Number version in a by-number result: once per number
The system SHALL return the number version as a field of the by-number search result, always, including for empty results, once per number; there is no version in the records of the results (OD-6, Q-128). The version is consistent with the number's records in this result (the requirement "Number version in
search results" of the `write-operations` capability). Trace: CUJ6, CUJ9, X-5; OD-6; ADR 0006, 0029, 0035 item 4.

#### Scenario: Version once per number
- **GIVEN** a number has records on three dates
- **WHEN** a search by number is performed
- **THEN** the result contains the number version once; the three records contain no versions

#### Scenario: Version is consistent with the history
- **GIVEN** a record was deleted with acknowledgment, the number received a new version V
- **WHEN** a search by number is performed
- **THEN** the result contains version V; the remaining records belong to state V and do not contain the record deleted before V (the results and the version are a
  consistent pair)

#### Scenario: A version from the results is accepted by a change
- **WHEN** a client changes or deletes a record, supplying the version from the by-number search result, and the number has not changed since
- **THEN** the operation accepts the version as current
