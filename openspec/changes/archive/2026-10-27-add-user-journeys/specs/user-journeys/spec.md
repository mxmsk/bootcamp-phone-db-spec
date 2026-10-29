# user-journeys

## Purpose

The original task statement (the only original source of requirements):

1. It must be a good database
2. It must have a Visual User Interface and a programmatic API
3. It must store Names and Phone numbers
4. It must be possible to search Names by Phone number
5. It must be possible to search Phone numbers by Name
6. It must reliably store many records
7. Written from scratch in Rust

The capability fixes the user journeys (Critical User Journeys CUJ1–CUJ16, UI, programmatic API, operator) to which
all project requirements trace. A requirement with no trace to a CUJ or to an item of the original statement is extra scope.
A parameter that the owner's decisions leave open is marked "<not decided: Q-x>": Q-x is an open question in
`docs/open-questions.md`. "Decisions: ADR NNNN" are references to `docs/adr/NNNN-*.md`. "Open: Q-x" in a requirement text is a question
not expressed by an observable scenario. Primary source: `docs/source/brief.md`;
the owner's decisions after the brief — `docs/owner-decisions.md` (OD-x); verified facts — `facts.md` (F-x).

## ADDED Requirements

### Requirement: CUJ1: Adding a new number
The system SHALL allow the user to add a new phone number entered in the international format with
+<country-code>: the number is saved and is found by name, and the name is found by number.
Decisions: ADR 0002, 0003, 0004, 0005, 0006, 0007, 0008, 0019.

#### Scenario: Saving an unknown number
- **WHEN** the user enters in the form an unknown number "+<country-code>…" and a valid name and submits an add request
  (create only)
- **THEN** the record is created, the number has a version different from all versions previously issued for it (X-5)
- **AND** the UI reports a successful save <not decided: Q-61>

#### Scenario: The name is found by word-prefix search
- **WHEN** after saving, a search by the word prefix of the name is performed
- **THEN** the record with this number is reachable in the results (if there are more results than a page, by paging "next")

#### Scenario: The name is found by number
- **WHEN** after saving, a search by this number is performed (exact match)
- **THEN** the results contain a record with this name

#### Scenario: Date not passed
- **WHEN** the add request has no date (CUJ1 does not ask for a date; whether the form has an optional date field is
  <not decided: Q-62>)
- **THEN** the record is stored with the date 1876-03-07

#### Scenario: A number without a plus
- **WHEN** the number is entered without "+"
- **THEN** it is rejected with a hint and no record is created (the hint text in the UI is <not decided: Q-65>; how the hint is expressed
  for an API client is <not decided: Q-49>)

#### Scenario: A number of invalid shape
- **WHEN** after "+" there are fewer than 1 or more than 15 digits, or the first digit is 0 ([DEFAULT] D-1, ADR 0004), or there are
  characters other than digits and removable separators
- **THEN** the number is rejected and no record is created

#### Scenario: A number with separators
- **WHEN** the number is entered with spaces, hyphens, parentheses or dots (for example, "+7 (999) 123-45-67")
- **THEN** the record is created
- **AND** a search by the same number written differently (for example, "+7 999 123 45 67") finds the same record

#### Scenario: Country code not assigned
- **WHEN** the number has a valid shape ("+", 1–15 digits, first digit 1–9 — [DEFAULT] D-1, ADR 0004), but its country code is not assigned
- **THEN** the number is accepted: there is no check against a list of country codes ([DEFAULT] D-1, ADR 0004)

#### Scenario: The name is longer than allowed
- **WHEN** the name is longer than allowed (maximum — <not decided: Q-55>)
- **THEN** an error, no record is created, the name is not truncated

#### Scenario: An empty name
- **WHEN** the name is empty or consists only of spaces
- **THEN** an error and no record is created

#### Scenario: Name normalization
- **WHEN** the name is entered with spaces at the edges or in a non-normalized form
- **THEN** the record is stored with the name trimmed at the edges and brought to NFC

#### Scenario: Retry of an identical record
- **WHEN** an identical record is added again (for example, after a connection drop)
- **THEN** success, the data and the number version are unchanged

#### Scenario: The number turned out to be known already
- **WHEN** the number is already in the database
- **THEN** the flow moves to CUJ2 (how the UI determines this is <not decided: Q-28>)

#### Scenario: A change is impossible or the UI has an old token
- **WHEN** the database is read-only or there is a storage error, or the UI is open with an old token
- **THEN** CUJ16 (database state) or CUJ13 (old link) applies

#### Scenario: Write acknowledgement latency
- **WHEN** an add is performed on a database of 10^8 records
- **THEN** the acknowledgement is received no later than <not decided: Q-58>; the criterion applies to all write operations
  (CUJ1–CUJ3, CUJ6, CUJ9) and is repeated in X-1

### Requirement: CUJ2: Adding a known number with a new date
When saving a known number, the system SHALL show that such a number already exists and ask for the date from which the
number is assigned to the user; the number is saved and from then on a number query returns the list of all its
owners by date. Decisions: ADR 0002, 0003, 0004, 0005, 0006, 0007, 0019.

#### Scenario: Date prompt for a known number
- **WHEN** the user saves a number that is already in the database
- **THEN** the UI reports that the number already exists and asks for a date <under what conditions the UI determines this — not decided:
  Q-28>; the date input format is <not decided: Q-62>, the dialog content is <not decided: Q-37>

#### Scenario: A new date for a known number
- **WHEN** the user enters a valid date on which the number has no record
- **THEN** the record (number, date) is created, the number version changed (X-5)
- **AND** records with other dates are unchanged
- **AND** the UI reports success <not decided: Q-61>

#### Scenario: Number results after a new date
- **WHEN** after the add, a search by number is performed
- **THEN** the results contain all records of the number (old and new), sorted by date descending, without pagination
  (showing past, current and future owners is <not decided: Q-2>)

#### Scenario: A date earlier than allowed
- **WHEN** the date 1876-03-06 is entered
- **THEN** no record is created (the kind of refusal is <not decided: Q-37>, <not decided: Q-49>)

#### Scenario: A nonexistent calendar date
- **WHEN** a nonexistent calendar date is entered (for example, 30 February)
- **THEN** no record is created (the form of the refusal is <not decided: Q-37>, <not decided: Q-49>); values with a time or
  time zone: in the UI — <not decided: Q-62>, in the API — <not decided: Q-67>

#### Scenario: A future date
- **WHEN** a future date is entered (there is no product upper bound; the technical limit is <not decided: Q-64>)
- **THEN** the record is created

#### Scenario: The same name on another date
- **WHEN** the number has a record with the name N on the date D1 and a record with the same name N is added on another valid date D2
- **THEN** a separate record (number, D2) is created, the record (number, D1) is unchanged

#### Scenario: The same record on the same date
- **WHEN** the entered date already has a record with exactly the same name, or the name differs from the stored one only by
  spaces at the edges or the normalization form (NFC)
- **THEN** the operation succeeds, the data and the number version are unchanged

#### Scenario: The name differs only in case
- **WHEN** on the same date the name matches the stored one only ignoring case ("Ann" against "ANN")
- **THEN** this is a conflict (moving to CUJ3), not a silent success

#### Scenario: A date on which the number already has another name
- **WHEN** on the entered date the number already has a record with another name (including another spelling)
- **THEN** the flow moves to CUJ3

#### Scenario: Date 1876-03-07
- **WHEN** the date 1876-03-07 is entered
- **THEN** this is the same record that would result from the server filling in the zero date (whether the results distinguish an explicit date from a
  filled-in one is <not decided: Q-1>)

#### Scenario: The user does not enter a date
- **WHEN** the user does not enter a date or declines to enter one
- **THEN** the behavior is <not decided: Q-37>, <not decided: Q-28>

#### Scenario: Invalid name or number
- **WHEN** the name is empty, only spaces or longer than allowed (maximum — <not decided: Q-55>), or the number has an invalid
  shape
- **THEN** no record is created and data is unchanged

#### Scenario: Database state on an add with a date
- **WHEN** the database is read-only or there is a storage error
- **THEN** CUJ16 applies

### Requirement: CUJ3: Adding a known number with an existing date
When adding a known number with a date that already exists, the system SHALL show an overwrite prompt: if yes —
the data changes, the old is fully erased; if no — the system waits for new input. Decisions: ADR 0002, 0003, 0005, 0006.

#### Scenario: Conflict on add
- **WHEN** a record is added on (number, date) where another name already exists
- **THEN** the server responds with a conflict carrying the current name and the number version, the record is unchanged
- **AND** the UI shows an overwrite prompt (the content is <not decided: Q-37>)

#### Scenario: Overwrite confirmation
- **WHEN** the user answers "yes" and the number version sent by the UI is current (the source of the version is <not decided: Q-34>)
- **THEN** the UI calls the explicit rename operation with the expected version, the name on (number, date) is replaced by the entered one,
  the number version changed (X-5)
- **AND** the earlier record (number, date) with the earlier name is not returned by any search
- **AND** the UI reports success <not decided: Q-61>

#### Scenario: Declining the overwrite
- **WHEN** the user answers "no"
- **THEN** data is unchanged and the UI waits for new input (what is kept in the form is <not decided: Q-37>)

#### Scenario: Other dates of the number are not affected
- **WHEN** the overwrite is performed
- **THEN** the number's records on other dates are unchanged

#### Scenario: The version is stale
- **WHEN** after "yes" the number version sent by the UI does not match the current one
- **THEN** the record is unchanged, CUJ12 applies

#### Scenario: The record vanished between the conflict and the confirmation
- **WHEN** the record (number, date) vanished between the conflict and the overwrite confirmation
- **THEN** this is a special case of a stale version (a delete changes the number version); the outcome is "not found" or a version
  conflict <not decided: Q-33>; the UI reaction is <not decided: Q-68>

#### Scenario: The same name on an existing date
- **WHEN** the entered name matches the stored one after trimming spaces and NFC
- **THEN** a silent success and the overwrite prompt is not shown

#### Scenario: A typo is corrected only through overwrite
- **WHEN** the user corrects a typo in the name of an existing record
- **THEN** this is done by an add on the same (number, date) with the corrected name and an overwrite confirmation;
  there is no other correction path in the UI (how the UI lets the user enter the date of an existing record is <not decided: Q-28>)

#### Scenario: No "change" button
- **WHEN** a row is shown in the results
- **THEN** it has no "change" button

#### Scenario: An invalid new name
- **WHEN** the new name is empty, only spaces or longer than allowed (maximum — <not decided: Q-55>)
- **THEN** an error and the record is unchanged

#### Scenario: Changing a nonexistent record in the UI path
- **WHEN** the CUJ3 path is executed without a concurrent change
- **THEN** changing a nonexistent record is unreachable; the scenario for the API is in CUJ9

#### Scenario: Database state on overwrite
- **WHEN** the database is read-only or there is a storage error
- **THEN** CUJ16 applies

### Requirement: CUJ4: Search by number
The system SHALL allow searching by phone number: a search by number returns only an exact match, and shows the list of
everyone the number was added for, sorted by date DESC. Decisions: ADR 0002, 0004, 0007, 0019.

#### Scenario: The full list of the number's records
- **WHEN** a full number with N records is entered
- **THEN** the results contain exactly N records without pagination (the row composition is <not decided: Q-29>; the form of the number in the row is
  <not decided: Q-26>; showing past, current and future owners is <not decided: Q-2>)

#### Scenario: Sorting by date
- **WHEN** the results contain several records
- **THEN** they are ordered by date descending

#### Scenario: Different names on different dates
- **WHEN** the number has records with different names on different dates
- **THEN** all are returned (a name may repeat on different dates)

#### Scenario: Exact match only
- **WHEN** a part of a number is entered
- **THEN** records of numbers that merely contain this part are not returned

#### Scenario: Number not found
- **WHEN** the number is not found
- **THEN** an empty list, the response is not an error

#### Scenario: A number of invalid format
- **WHEN** a number of invalid format is entered (without "+", letters, etc.)
- **THEN** a format error (the rules are the same as on save, ADR 0004) and no results

#### Scenario: Search by a number with separators
- **WHEN** the number is saved and the search is performed by the same number written with spaces, hyphens, parentheses or
  dots
- **THEN** the records of this number are found

#### Scenario: Number search latency
- **WHEN** a search by number is performed on a database of 10^8 records
- **THEN** the latency is no more than <not decided: Q-47>

#### Scenario: Storage state on a search by number
- **WHEN** the database is read-only or there is a storage error
- **THEN** CUJ16 applies: search works as long as it can

### Requirement: CUJ5: Search by name
The system SHALL allow searching by name by the start of any word of the name (not by substring; every query word must
match; case-insensitive, "ё" = "е") and show the results as a paginated list, sorted alphabetically, then
by date DESC. Decisions: ADR 0003, 0008, 0009, 0010, 0019.

#### Scenario: Word start, not a substring
- **GIVEN** the database has records with the names «Ann Smith», «John Annabel» and «Joann»
- **WHEN** the query is «ann»
- **THEN** the results contain «Ann Smith» and «John Annabel» and do not contain «Joann»

#### Scenario: Several query words
- **WHEN** the query consists of several words
- **THEN** the results contain only records where every query word matched; the order of the query words does not matter («ann smith» and
  «smith ann» find «Ann Smith»)

#### Scenario: One name word covers several query words
- **GIVEN** the record «Annabel»
- **WHEN** the query is «an ann»
- **THEN** the record is found

#### Scenario: Ё equals Е, case does not matter
- **GIVEN** the database has the records «Елка» and «Ёлка»
- **WHEN** the query is «ёлка» (and conversely «елка», and «ЕЛКА»)
- **THEN** both records are found

#### Scenario: Case does not matter
- **GIVEN** the database has the record «Ann»
- **WHEN** the query is «ann» (and «ANN»)
- **THEN** the record is found

#### Scenario: A hyphen separates words
- **GIVEN** the record «Anna-Maria»
- **WHEN** the query is «maria»
- **THEN** the record is found

#### Scenario: An apostrophe between letters does not separate words
- **GIVEN** the record «O'Brien»
- **WHEN** the query is «o'brien» or «o'bri»
- **THEN** the record is found
- **AND** for the query «brien» the record «O'Brien» is not in the results (word boundaries strictly per UAX #29; the specific functions are
  open question Q-76, docs/open-questions.md)

#### Scenario: A query in decomposed form
- **GIVEN** the record «José»
- **WHEN** the query «José» is entered in NFD form (the letter "e" and a separate accent U+0301)
- **THEN** the record is found: the query is brought to NFC

#### Scenario: An empty query
- **WHEN** the query is empty, only spaces, or a single punctuation mark (for example, «-», «'»): it contains no word
  at all
- **THEN** an error, no results (the whitespace character set is <not decided: Q-84>)

#### Scenario: Pagination
- **WHEN** there are more results than a page
- **THEN** a part is returned (the page size is <not decided: Q-30>) and the next one can be obtained by cursor
- **AND** the total number of results is not reported

#### Scenario: Result order
- **WHEN** the results contain different names
- **THEN** they go alphabetically under the ru locale rules (the comparison parameters are <not decided: Q-31>), records with one name
  — by date DESC

#### Scenario: Order with equal name and date
- **GIVEN** two records with the same name and date for different numbers
- **WHEN** a search by name is performed
- **THEN** they go by number ascending as a string: «+19…» before «+2…», numbers of one country next to each other
  (the cursor mechanics are open question Q-73, docs/open-questions.md)

#### Scenario: No matches
- **WHEN** there are no matches
- **THEN** the form of the empty results is <not decided: Q-29>

#### Scenario: The cursor is not accepted
- **WHEN** the cursor is not accepted (the data changed or the cursor is corrupted)
- **THEN** the behavior is <not decided: Q-30>, <not decided: Q-56>

#### Scenario: A server upgrade within v1 does not change search
- **WHEN** the server is upgraded within v1
- **THEN** the matches and the order of search on the same data and queries are unchanged: Unicode 18 is fixed for all of v1
  (the mechanics are open questions Q-74, Q-82, docs/open-questions.md)

#### Scenario: Name search latency
- **WHEN** a name search (including a multi-word one) is performed on 10^8 records
- **THEN** the latency is no more than <not decided: Q-47>; for a multi-word search — <not decided: Q-59>

#### Scenario: Storage state on a search by name
- **WHEN** the database is read-only or there is a storage error
- **THEN** CUJ16 applies

### Requirement: CUJ6: Deleting a record
The system SHALL allow deleting a record of a number: the user searches by number or name; in the found list,
on the left, a "delete" button with a trash icon is shown; confirmation is requested before deleting; after the delete
the list is re-queried and shown updated. Decisions: ADR 0005, 0006, 0007.

#### Scenario: A delete button in the row
- **WHEN** the results list (by number or by name) contains a row
- **THEN** on the left of it there is a "delete" button with a trash icon

#### Scenario: Confirmation request
- **WHEN** the user presses "delete"
- **THEN** confirmation is requested and until it is given the record is not deleted

#### Scenario: A confirmed delete
- **WHEN** the user confirms and the number version is current (the source of the version is <not decided: Q-34>)
- **THEN** exactly one record (number, date) is deleted, the number version changed (X-5), the number's other records are intact

#### Scenario: An updated list after the delete
- **WHEN** the delete is performed
- **THEN** the UI re-requests the data and shows the list without the deleted record (which request and page —
  <not decided: Q-35>; the form of an empty list — <not decided: Q-29>)

#### Scenario: Declining the confirmation
- **WHEN** the user declines in the confirmation dialog
- **THEN** data is unchanged (what the UI shows after the refusal is <not decided: Q-63>)

#### Scenario: A stale version on delete
- **WHEN** the number version in the delete request does not match the current one and the record (number, date) exists
- **THEN** the record is not deleted and the response carries the current version; CUJ12 applies (for a missing record —
  <not decided: Q-33>)

#### Scenario: Deleting a missing record
- **WHEN** a record that does not exist is deleted and the current number version is passed
- **THEN** "not found" (with a stale version — <not decided: Q-33>); the UI reaction is <not decided: Q-68>

#### Scenario: Undoing a completed delete
- **WHEN** undoing an already completed delete (undo) is needed
- **THEN** the CUJ6 text does not provide for it; whether it is needed is <not decided: Q-63>

#### Scenario: Database state on delete
- **WHEN** the database is read-only or there is a storage error
- **THEN** CUJ16 applies

### Requirement: CUJ7: A local program works with the database
A local program on the same machine SHALL read the token from a file and perform the same operations as the UI, plus the exact
whole-name search; after a server restart its request with the old token SHALL get an "unauthorized" refusal,
and after rereading the token file a retry succeeds (a change and delete with an old version after a crash is a conflict,
OD-4, item 5). Open: Q-70 (whether TLS is needed on loopback).
Decisions: ADR 0005, 0006, 0007, 0008, 0011, 0012, 0013, 0019.

#### Scenario: The same operations as in the UI
- **WHEN** a local program with a valid token calls add, rename, delete, search by number,
  search by words of the name or exact whole-name search
- **THEN** the rules of number and name validation, of conflicts and versions are the same as when called from the UI; there are no UI-level differences
  (date prompt, overwrite dialog, delete confirmation) in the API; if there is no date the server fills in 1876-03-07
- **AND** the local program's permissions are full; the location and access rights of the token file are <not decided: Q-38>

#### Scenario: Exact whole-name search
- **WHEN** the program calls the exact whole-name search
- **THEN** all records with this name are returned, case-insensitive, "ё" = "е"; the query is trimmed at the edges and
  brought to NFC, like a name on save (the order and pagination are <not decided: Q-30>)

#### Scenario: Exact search «елка»
- **GIVEN** the record «Ёлка»
- **WHEN** the exact whole-name search «елка» (or «ЁЛКА»)
- **THEN** the record is found

#### Scenario: An empty exact-search query
- **WHEN** the exact-search query is empty or only spaces
- **THEN** an error

#### Scenario: An old token after a restart
- **WHEN** the server is restarted and the program sends a request with the old token
- **THEN** an "unauthorized" refusal (a token not presented — the code and form are <not decided: Q-50>)

#### Scenario: An unknown token
- **WHEN** a request presents a token unknown to the server (wrong or arbitrary)
- **THEN** an "unauthorized" refusal and data is unchanged (the code and form are <not decided: Q-50>)

#### Scenario: Retry after rereading the token
- **WHEN** the server is restarted as planned, the program reread the token file and repeated the request (a search; a change or
  delete — with the number version read before the planned restart, which stays valid)
- **THEN** success

#### Scenario: Restart after a crash
- **WHEN** the server is restarted after a crash and the program repeats a change or delete with the number version read before
  the crash
- **THEN** a conflict and data is unchanged (for any numbers); the program rereads the number history and repeats with
  the current version — success

#### Scenario: Name or version conflict
- **WHEN** a name or version conflict occurs
- **THEN** the behavior is the same as in CUJ9

#### Scenario: The file token presented not from loopback
- **WHEN** the token from the file is presented on a request not from loopback (from another machine)
- **THEN** the request is rejected and data is unchanged (the refusal code is <not decided: Q-50>)

#### Scenario: The request limit does not apply
- **WHEN** a local program sends requests at any rate
- **THEN** there is no refusal by the per-token request limit

#### Scenario: API version in the path
- **WHEN** a client calls a v1 path
- **THEN** the request is served (a call without a version in the path is <not decided: Q-51>)

#### Scenario: The token file is unavailable
- **WHEN** the token file is unavailable or not yet updated after the start
- **THEN** the behavior is <not decided: Q-38>

#### Scenario: Connection load
- **WHEN** the number of connections, the request body size or the request time exceed the server's limits
- **THEN** the behavior is <not decided: Q-52> (ADR 0019)

#### Scenario: Storage state on a call from the API
- **WHEN** the database is read-only or there is a storage error
- **THEN** CUJ16 applies

### Requirement: CUJ8: A network service only searches
A network service on another machine SHALL connect over HTTPS with its own permanent token with the read permission and search by
number, by words of the name and by whole name; an attempt to add, change or delete a record SHALL get a "no
permission" refusal; requests over the limit SHALL get the refusal «слишком много запросов, повторите позже»; other clients do not
notice this. Decisions: ADR 0007, 0008, 0011, 0012, 0019.

#### Scenario: Three kinds of search with a read token
- **WHEN** a token with the "read" permission performs a search by number, by words of the name and by whole name
- **THEN** the HTTPS connection is served by the server process itself (TLS in the server); the results are the same as for the local
  program (CUJ7) on the same queries (the order and pagination of the exact whole-name
  search are <not decided: Q-30>; the latency of a multi-word search is <not decided: Q-59>)

#### Scenario: A change is forbidden to a read token
- **WHEN** a "read" token calls add, change or delete
- **THEN** a "no permission" refusal and data is unchanged

#### Scenario: Exceeding the limit
- **WHEN** a token sends requests more often than its limit (the value and model are <not decided: Q-46>)
- **THEN** the excess requests get the refusal «слишком много запросов, повторите позже»

#### Scenario: Other clients do not notice another's limit
- **WHEN** one service exceeded the limit
- **THEN** the requests of the other clients are executed without refusals due to another's limit and their latencies stay within
  the common targets (the targets are <not decided: Q-47>, <not decided: Q-58>, <not decided: Q-59>)

#### Scenario: A connection not over HTTPS
- **WHEN** a network client connects not over HTTPS, or a network token is presented not over HTTPS
- **THEN** the connection is not served and the token is not accepted (the form of the refusal is <not decided: Q-45>)

#### Scenario: The network listener is off by default
- **WHEN** the network listener is not enabled by an operator setting
- **THEN** the server accepts no connections from other machines

#### Scenario: The network listener is enabled without a certificate
- **WHEN** the network listener is enabled and there is no certificate
- **THEN** the server does not start and reports the reason (the message channel is <not decided: Q-66>; the certificate source is
  <not decided: Q-69>)

#### Scenario: A network token on loopback
- **WHEN** the network listener is enabled and a network token is presented from loopback over HTTPS
- **THEN** it is accepted (with the network listener off — <not decided: Q-45>)

#### Scenario: An unknown or revoked token
- **WHEN** the token is unknown or revoked
- **THEN** an unknown token gets "unauthorized"; a revoked one — a refusal (code <not decided: Q-50>; the form of the error body
  <not decided: Q-49>)

#### Scenario: The limit applies only to network tokens
- **WHEN** the limit is exhausted
- **THEN** a refusal as on exceeding the limit; the limit applies only to network tokens, not to local programs and the UI

#### Scenario: An empty query
- **WHEN** a query by words or by whole name is empty (by words — also a query of a single punctuation mark)
- **THEN** an error

#### Scenario: A number of invalid format
- **WHEN** a query by number contains a number of invalid format
- **THEN** a format error

#### Scenario: Number not found
- **WHEN** a query by number contains a number of valid format that is not in the database
- **THEN** an empty list, not an error

#### Scenario: The certificate is not accepted by the client
- **WHEN** the client does not accept the server's certificate
- **THEN** the behavior is <not decided: Q-69>

#### Scenario: Connection load
- **WHEN** the number of connections, the request body size or the request time exceed the server's limits
- **THEN** the behavior is <not decided: Q-52> (ADR 0019)

### Requirement: CUJ9: A network service changes data
A network service with the write permission SHALL add a number; if on this date the number already has another name, it gets
a conflict with the current name and the number version. It SHALL send a change and delete with the number version; on a stale
version it gets a conflict with the current version, rereads the number history and retries.
Decisions: ADR 0002, 0003, 0004, 0005, 0006, 0012, 0019.

#### Scenario: Adding a record
- **WHEN** a service with a "read and write" token adds a record on (number, date) with no record (the date is optional — by
  default 1876-03-07; the date format is <not decided: Q-67>)
- **THEN** the record is created

#### Scenario: Name conflict on add
- **WHEN** the (number, date) being added already has another name
- **THEN** a conflict, the response contains the current name and the number version, the record is unchanged

#### Scenario: Retry of adding an identical record
- **WHEN** a service repeats the add of an identical record (for example, after a connection drop)
- **THEN** success, data and version are unchanged

#### Scenario: A change without an expected version
- **WHEN** a change or delete is sent without an expected version
- **THEN** the operation is not performed (the form of the error is <not decided: Q-49>, <not decided: Q-50>)

#### Scenario: A stale version with an existing record
- **WHEN** the version in a change or delete is stale and the record (number, date) exists
- **THEN** a conflict, the response contains the current version, data is unchanged (the source of the current version is
  <not decided: Q-34>; the content of successful responses is <not decided: Q-32>)

#### Scenario: Retry with the current version
- **WHEN** a service repeated the change with the current version, the record exists and the new name differs from the current one (exact
  spelling)
- **THEN** success and the number version changed

#### Scenario: Changing a missing record
- **WHEN** a record that does not exist is changed and the current number version is passed
- **THEN** "not found"; with a stale version the outcome is determined by the check order <not decided: Q-33>

#### Scenario: Deleting a missing record
- **WHEN** a record that does not exist is deleted and the current number version is passed
- **THEN** "not found" (with a stale version — <not decided: Q-33>)

#### Scenario: Changing to the same name with a current version
- **WHEN** a rename is sent with the current number version and a name equal to the current one (exact spelling after
  trimming spaces and NFC)
- **THEN** a silent success with no changes, the number version does not change, as on a retry of an add (the response content is
  <not decided: Q-32>)

#### Scenario: Changing to the same name with a stale version
- **WHEN** a change to the same name is sent with a stale number version
- **THEN** a conflict with the current version and data is unchanged (the version check comes first)

#### Scenario: A read token and the limit
- **WHEN** a request is made by a "read" token, or a service exceeded the request limit
- **THEN** "no permission" (CUJ8) or a limit refusal (the value is <not decided: Q-46>)

#### Scenario: Retry after a connection drop
- **WHEN** a change or delete is repeated after a connection drop
- **THEN** the behavior is <not decided: Q-54>

#### Scenario: Invalid name, number, date
- **WHEN** the name is empty or only spaces, or longer than allowed (maximum — <not decided: Q-55>); or the number has an
  invalid format; or the date is not a calendar day or is earlier than 1876-03-07
- **THEN** an error, data is unchanged (the form of the date refusal is <not decided: Q-49>; values with a time or zone —
  <not decided: Q-67>)

### Requirement: CUJ10: The operator manages network client tokens
The operator SHALL manage network client access with a command on the server machine: issue a token to a new service with a client
name and permissions (read, or read and write; the token is shown once), show the list of issued tokens (without
secrets) and revoke a token. The server does not stop, revocation takes effect immediately, a restart does not change network tokens.
Open: Q-60 (the command channel to a running server), Q-46 (the per-token request limit).
Decisions: ADR 0011, 0012, 0016, 0018.

#### Scenario: Token issuance
- **WHEN** the network listener is enabled and the operator issues a token with a client name and permissions
- **THEN** the token is shown once
- **AND** it is accepted by the server with the given permissions without stopping or restarting the server

#### Scenario: The secret is not shown again
- **WHEN** after issuing a token the operator requests its secret again
- **THEN** the secret cannot be obtained

#### Scenario: Token list
- **WHEN** the operator requests the list
- **THEN** it lists the issued tokens and no secrets are shown (the set of fields is <not decided: Q-40>)

#### Scenario: Token revocation
- **WHEN** the operator revokes a token
- **THEN** the server is not stopped and the next request with this token is rejected (code — <not decided: Q-50>), including over
  a connection opened before the revocation
- **AND** a request in progress at the moment of revocation completes

#### Scenario: Server restart
- **WHEN** the server is restarted
- **THEN** previously issued unrevoked network tokens stay valid without reissue

#### Scenario: The client name is taken
- **WHEN** the operator issues a token with a client name that is taken by a valid token
- **THEN** the issuance is not performed

#### Scenario: A client name after revocation
- **WHEN** the token with a client name is revoked and the operator issues a token with this name
- **THEN** the issuance is performed, the new token is valid, the revoked token does not come back to life

#### Scenario: Revoking a nonexistent token
- **WHEN** the operator revokes a nonexistent or already revoked token
- **THEN** an error message and the token data is unchanged (how the token is specified is <not decided: Q-40>)

#### Scenario: Changing permissions
- **WHEN** the operator needs to change a client's permissions
- **THEN** this is done only by issuing a new token; the permissions of an issued token do not change

#### Scenario: Tokens and backup
- **WHEN** a backup is made, the database is restored from it and the server is started with the same settings next to which
  the tokens are stored
- **THEN** the backup contains only data, tokens are not part of it, the tokens are the same as before the restore, a revoked token
  stays revoked
- **AND** when moving to another machine the operator carries the tokens along with the settings, otherwise reissues them

#### Scenario: A network token before the network listener is enabled
- **WHEN** the network listener is not enabled by an operator setting
- **THEN** there are no connections from other machines; accepting a network token on loopback is <not decided: Q-45>

#### Scenario: A network token with the network listener enabled
- **WHEN** the network listener is enabled and a network token is presented over HTTPS
- **THEN** it is accepted from anywhere; over HTTP it is not accepted

#### Scenario: An operator command with the server stopped
- **WHEN** an operator command is executed and the server is stopped
- **THEN** the behavior is <not decided: Q-60>

### Requirement: CUJ11: Server upgrade, clients keep working
After a server upgrade within v1, programs and services SHALL work unchanged: new fields and
error codes may appear, but existing ones do not change and do not disappear. A move from v1 to v2 is out of scope (X-3).
Decisions: ADR 0010, 0013, 0018, 0019.

#### Scenario: An unchanged client after the upgrade
- **WHEN** the server is upgraded within v1
- **THEN** an unchanged client gets all the fields and error codes it had before the upgrade, with the earlier meaning; new
  fields and codes are allowed (whether a new code replaces an earlier one in the same situation is <not decided: Q-79>)

#### Scenario: Fields and codes are only added
- **WHEN** fields or error codes are added in v1
- **THEN** existing fields and codes in responses are unchanged and did not disappear (the error contract is <not decided: Q-49>,
  <not decided: Q-50>)

#### Scenario: A v1 path is served
- **WHEN** a client requests a v1 path
- **THEN** it is served after the upgrade (a call to a path without a version is <not decided: Q-51>)

#### Scenario: Data is available without operator actions
- **WHEN** the server is upgraded within v1
- **THEN** previously saved records are available without operator actions

#### Scenario: The new version opens earlier databases by itself
- **WHEN** a new version within v1 is started on a database created by an earlier v1 version (including on a backup made
  before the upgrade)
- **THEN** it opens the database by itself and there is no separate format migration command (how an ICU4X upgrade affects
  the start is open question Q-82, docs/open-questions.md)

#### Scenario: No rollback
- **WHEN** the operator needs to return to the data from before the upgrade
- **THEN** there is no separate rollback command; getting the data back is starting the new program on a backup made before the upgrade;
  the earlier program is not required to open a database after the new one

#### Scenario: Search does not change on upgrade
- **WHEN** the server is upgraded within v1
- **THEN** the matches and the order of search on the same data and queries are unchanged: Unicode 18 is fixed for all of v1
  (the mechanics are open question Q-74, docs/open-questions.md)

#### Scenario: Tokens and the UI after a restart during the upgrade
- **WHEN** the server is restarted during the upgrade
- **THEN** a local program rereads the token file (earlier number versions stay valid), a network
  service continues with the earlier token (CUJ10); the UI requires a new link (CUJ13)

### Requirement: CUJ12: The record was changed concurrently
If a number's record was changed concurrently (in another tab or through the API) and the user pressed "delete" or
confirmed an overwrite (CUJ3), the action SHALL NOT be performed; the UI SHALL report «данные устарели, список обновлён»,
reread the number history and offer to repeat the action on the current data; the name entered in the form is kept.
Open: Q-56 (visibility of intermediate states), Q-54 (retry after a connection drop), Q-63 (undoing a delete).
Decisions: ADR 0002, 0006, 0019.

#### Scenario: A stale delete
- **WHEN** the number version changed after the UI received the version, the target record exists and the user
  confirms the delete
- **THEN** the record is not deleted and the UI reports «данные устарели, список обновлён» (the exact wording is <not decided: Q-65>)

#### Scenario: A stale overwrite
- **WHEN** the version sent by the UI on the overwrite confirmation (the source of the version is <not decided: Q-34>) does not match the
  current one
- **THEN** the record is unchanged and the same message is shown

#### Scenario: Reread and the offer to retry
- **WHEN** a conflict is issued
- **THEN** the UI reread the number history, shows the current data and offers to repeat the action on the current
  data (the dialog content is <not decided: Q-37>; for an action from the by-name results what is reread is
  <not decided: Q-36>)

#### Scenario: The name in the form is kept
- **WHEN** a conflict is issued on an overwrite
- **THEN** the name entered in the form is kept

#### Scenario: Retry on the current data
- **WHEN** after the conflict the user repeats the action on the current data
- **THEN** the action is performed if the version matches

#### Scenario: Another date of the same number was changed
- **WHEN** only another date of the same number was changed
- **THEN** the action is still rejected as stale (the version belongs strictly to the number)

#### Scenario: The record was deleted concurrently
- **WHEN** the record the user acted on was deleted concurrently
- **THEN** the outcome ("not found" or a version conflict) is <not decided: Q-33>; what the user sees is <not decided: Q-68>

#### Scenario: The name already equals the entered one with a stale version
- **WHEN** the record exists, its name already equals the entered one, and the number version is stale
- **THEN** a conflict with the current version (the version check comes first, OD-4, item 7); what the user sees is
  <not decided: Q-68>

#### Scenario: The user does not repeat the action
- **WHEN** the user does not repeat the action
- **THEN** data does not change

### Requirement: CUJ13: First start and UI sign-in
The operator SHALL create an empty database with a separate command; the server does not start on a directory without a database and reports this.
On start the server SHALL print an address with an embedded token by which the user on the same machine immediately
works in the UI; after a restart the old link stops working, a new one from the server output is needed. Open: Q-70 (whether TLS is needed for the UI on loopback).
Decisions: ADR 0011, 0012, 0014, 0015.

#### Scenario: A directory without a database
- **WHEN** the server is started on a directory without a database
- **THEN** the server does not start and reports that there is no database (the message channel is <not decided: Q-66>)

#### Scenario: Creating the database and starting
- **WHEN** the operator creates a database with a separate command and starts the server
- **THEN** the server starts and prints an address with an embedded token

#### Scenario: Sign-in by link
- **WHEN** the user on the same machine opens the printed link
- **THEN** the UI is available immediately, with no credentials entered (whether the UI and local programs have one token, how the UI keeps
  the sign-in is <not decided: Q-38>)

#### Scenario: An old link after a restart
- **WHEN** the server is restarted
- **THEN** the old link does not work and the new one from the output works (what an open tab shows is <not decided: Q-39>)

#### Scenario: Access not from loopback
- **WHEN** the UI is accessed not through loopback
- **THEN** there is no access to the UI; the UI token is accepted only from loopback

#### Scenario: Creating the database again
- **WHEN** the operator runs the create-database command on a directory where a database already exists
- **THEN** a refusal and the existing database is unchanged (it is never overwritten)

#### Scenario: A second server process
- **WHEN** a second server process is started on a database that another process has already opened
- **THEN** the start is rejected with the message «база уже открыта другим процессом» (the database is already open by another process)

#### Scenario: The network is off by default
- **WHEN** the server is started without a network listener setting
- **THEN** the UI and local programs are available through loopback, the server accepts no connections from other machines; if
  the network listener is enabled and there is no certificate, the server does not start (the set of network listeners is <not decided: Q-45>)

#### Scenario: A damaged database at start
- **WHEN** at start the check shows that the database does not open in a consistent state
- **THEN** the server does not start (CUJ14); corruption of the rest of the data is detected on read — "storage error"

#### Scenario: A freshly created empty database
- **WHEN** a search is performed on a freshly created empty database
- **THEN** a search by number gives an empty list; the form of the empty results by name is <not decided: Q-29>

### Requirement: CUJ14: Data survived a failure
A save of a number acknowledged to the user SHALL survive a process crash or power loss: after the server restarts
the number is in place; a save that did not get an acknowledgement is either fully present or fully absent. If
at start the database is inconsistent, the server does not start and reports to the operator; other corruption is a "storage error" on read
(OD-4, item 3); data is not lost silently; the operator restores the database from a backup (CUJ15). Decisions: ADR 0006, 0013, 0014, 0016.

#### Scenario: An acknowledged save survives a failure
- **WHEN** a user or client received the acknowledgement of a save (an add), the process crashed or power was lost and
  the server is restarted (the moment of acknowledgement is <not decided: Q-57>)
- **THEN** the acknowledged record is present

#### Scenario: An unacknowledged save
- **WHEN** a save (an add) did not get an acknowledgement and there was a failure
- **THEN** after the restart the record is either fully present or fully absent

#### Scenario: Acknowledged rename and delete
- **WHEN** a rename (CUJ3) or delete (CUJ6) is acknowledged, there was a failure and the server is restarted
- **THEN** the acknowledged change is kept and the deleted record does not "come back to life"

#### Scenario: Unacknowledged rename and delete
- **WHEN** a rename or delete did not get an acknowledgement and there was a failure
- **THEN** after the restart the change is either fully present or fully absent (the record and the change of the number version
  are indivisible, X-5)

#### Scenario: Guarantee boundaries
- **WHEN** a process crash, an OS failure or a power loss occurred and the disk honestly performs a flush to stable storage
- **THEN** acknowledged adds and changes are present and acknowledged deleted records are absent
- **AND** a "lying" disk and media corruption are outside the guarantee, detection of a "lying" disk is not promised; a disk write error —
 "storage error" (CUJ16)

#### Scenario: Check at start
- **WHEN** at start the check (structure and latest changes) shows that the database does not open in a consistent state
- **THEN** the server does not start, the operator gets a message (the channel is <not decided: Q-66>) and records are not silently
  discarded (the scope of the check is <not decided: Q-43>; the start time on 10^8 records is <not decided: Q-42>)

#### Scenario: Corruption found on read
- **WHEN** corruption of the rest of the data (not found by the check at start) is detected when it is read or on a running
  server
- **THEN** the server moves to the "storage error" state: changes are rejected (CUJ16), search works as long as it can,
  the operator gets a message

#### Scenario: Restore for a damaged database
- **WHEN** the database is damaged and the operator starts the server on a backup
- **THEN** the server starts on the backup's data (including on a backup made before an upgrade: the new program opens
  such a database by itself; there is no rollback)

#### Scenario: A version read before the failure
- **WHEN** a client sends a change or delete with a number version read before the failure, and the server is restarted after the
  failure
- **THEN** a conflict and data is unchanged; the client rereads; a conflict is allowed for any number (the mechanics are
  open question Q-78, docs/open-questions.md)

#### Scenario: A planned restart
- **WHEN** the server is restarted as planned (without a failure) and a client sends a change or delete with a number version
  read before the stop
- **THEN** the version is valid: the action is performed if the version matches

### Requirement: CUJ15: Backup and restore
The operator SHALL make a database backup with a command on the server machine while the server is running: users and programs
keep working without interruption, the backup corresponds to one consistent point. Restore is starting the server
on the backup. Decisions: ADR 0006, 0012, 0013, 0016, 0018.

#### Scenario: Backup with the server running
- **WHEN** the operator starts a backup (the parameters are <not decided: Q-41>; who performs the backup and the command channel are
  <not decided: Q-60>)
- **THEN** the server is not stopped and UI and API requests are processed

#### Scenario: No refusals, latency growth within bounds
- **WHEN** a backup is in progress
- **THEN** requests are processed without refusals and the latency growth is within <not decided: Q-58>, <not decided: Q-59>

#### Scenario: A consistent cut
- **WHEN** the backup is ready
- **THEN** it matches the database state at some point: every change is either fully present or fully
  absent, there are no partial changes (which exact point the backup fixes is open question Q-83,
  docs/open-questions.md; the notions of "point" and "acknowledged" are Q-56, Q-57)

#### Scenario: Starting the server on a backup
- **WHEN** the server is started on a backup
- **THEN** searches return the data of the backup point

#### Scenario: Versions after restore
- **WHEN** the server is started on a backup and a client sends a change or delete with a number version read before
  the restore
- **THEN** a conflict and data is unchanged; mass conflicts right after the restore are allowed, for any numbers
  (the mechanics are open question Q-78, docs/open-questions.md)

#### Scenario: Tokens and the backup
- **WHEN** a backup is made
- **THEN** tokens are not part of the backup; a server started on the backup with the same settings next to which the
  tokens are stored accepts the same tokens, and a revoked one is still rejected; when moving to another machine the operator
  carries the tokens along with the settings, otherwise reissues them

#### Scenario: A backup of an earlier v1 version
- **WHEN** the backup was made by an earlier v1 version and the server on the backup is a new version
- **THEN** the new program opens such a database by itself, without a command; the earlier program is not required to open a database
  after the new one ("no rollback")

#### Scenario: Backup with the target space occupied or a write error
- **WHEN** the target place is occupied, there is no space or a backup write error
- **THEN** the behavior is <not decided: Q-41>

#### Scenario: Backup from a read-only database or one with a storage error
- **WHEN** the source database is read-only or there is a storage error
- **THEN** the behavior is <not decided: Q-71>

#### Scenario: The server is unavailable to the backup command
- **WHEN** the server is stopped or unavailable to the backup command
- **THEN** the behavior is <not decided: Q-60>

### Requirement: CUJ16: The server cannot perform a change
If the database is read-only or there is a storage error, the user SHALL learn of it when their action did not
go through («Сохранение не выполнено: база доступна только для чтения» or «ошибка хранилища, обратитесь к оператору»);
search keeps working as long as it can. The UI does not show the server state in advance. Decisions: ADR 0011, 0014, 0019.

#### Scenario: A read-only database
- **WHEN** the database is read-only and the user performs a change
- **THEN** the UI shows «Сохранение не выполнено: база доступна только для чтения» and data is unchanged
  (the exact wording and the text for a failed delete are <not decided: Q-65>)

#### Scenario: A storage error on a change
- **WHEN** there is a storage error and the user performs a change
- **THEN** the UI shows «ошибка хранилища, обратитесь к оператору» and data is unchanged (the exact wording is
  <not decided: Q-65>)

#### Scenario: No warnings in advance
- **WHEN** the database is in the "read-only" or "storage error" state, the user performed no actions or all their actions succeeded
- **THEN** there is no warning about the server state

#### Scenario: Search in read-only mode
- **WHEN** the database is read-only
- **THEN** all searches are performed

#### Scenario: Search on a storage error
- **WHEN** there is a storage error
- **THEN** searches for which the readable data suffices are performed; for the rest there are no guarantees (what counts as "suffices" is
  <not decided: Q-71>, <not decided: Q-43>)

#### Scenario: A change refusal for an API client
- **WHEN** an API client performs a change in these states
- **THEN** it gets a refusal and data is unchanged (the code and error body are <not decided: Q-44>, <not decided: Q-49>,
  <not decided: Q-50>)

#### Scenario: Damage on a running server
- **WHEN** damage is detected on a running server
- **THEN** the server moves to the "storage error" state, changes are rejected, search works as long as it can,
  the operator gets a message

#### Scenario: A disk write error
- **WHEN** a disk write error occurs during a change
- **THEN** the "storage error" state and the change is not performed; there is no partial change (X-5)

#### Scenario: A search failed because of a storage error
- **WHEN** there is a storage error and a search cannot be performed
- **THEN** the user gets a search refusal (the form in the UI is <not decided: Q-65>), an API client — <not decided: Q-44>

#### Scenario: Leaving the state
- **WHEN** the server leaves the "read-only" or "storage error" state (by itself or after a restart)
- **THEN** the behavior is <not decided: Q-71>

### Requirement: X-1: Scale of 10^8 records
The system SHALL perform the operations CUJ1–CUJ16 without refusals caused by volume on a database of 10^8 records. Trace: item 6
of the original statement ("reliably store many records"); the owner decision "Scale and reliability: 10^8 records".
Decisions: ADR 0017.

#### Scenario: Operations on 10^8 records
- **WHEN** the database contains 10^8 records
- **THEN** the operations CUJ1–CUJ16 are performed without refusals caused by volume (there is no numeric threshold until the targets are decided:
  the size and the search latency are <not decided: Q-47>, the start time is <not decided: Q-42>)

#### Scenario: Write operation latency
- **WHEN** any write operation is performed on a database of 10^8 records
- **THEN** the acknowledgement is received no later than <not decided: Q-58>

#### Scenario: Filling the database to verify the scale
- **WHEN** a database of 10^8 records is needed for verification
- **THEN** the filling method is <not decided: Q-53>; there is no batch load as a product operation (X-3)

### Requirement: X-2: One server process owns the database
The system SHALL ensure that the database is owned by one server process, and the UI and API are its clients. Trace: the owner
decision "Interfaces: one server process owns the database"; item 2 of the original statement. Decisions: ADR 0011, 0015.

#### Scenario: A second process does not get access
- **WHEN** a second server process tries to open the same database while the first one owns it
- **THEN** the start of the second process is rejected with the message «база уже открыта другим процессом»

#### Scenario: Creating over an existing database
- **WHEN** the create-database command is run on a directory with a database
- **THEN** a refusal and the existing database is never overwritten

#### Scenario: The UI and API are clients of one process
- **WHEN** a record is acknowledged through the UI (or through the programmatic API)
- **THEN** it is found by a search through the programmatic API (or through the UI), because both reach the database through the single
  server process

### Requirement: X-3: Out of scope
The system SHALL NOT contain batch load and import, a "who changed what" log, administration through the API,
a v1 to v2 migration policy, and an operator-commanded "read-only" mode (OD-3, Q-17). This is a scope limit: the trace is the owner
decision "Out of scope" (`docs/owner-decisions.md`, OD-3, Q-17); it limits items 1–2 of the original statement and
CUJ1, CUJ10, CUJ11, CUJ15, CUJ16. Decisions: ADR 0014, 0018.

#### Scenario: Out-of-scope functions are absent
- **WHEN** an operation of batch load or import, a change log, or administration through the API is looked for
- **THEN** it is not in the product or the v1 API (tokens and backup are only by an operator command: CUJ10, CUJ15)

#### Scenario: The v1 to v2 policy is not described
- **WHEN** compatibility is described (CUJ11)
- **THEN** the v1 to v2 migration policy is not described

#### Scenario: Read-only mode by operator command
- **WHEN** the operator wants to put the database in "read-only" mode
- **THEN** there is no such mode: the "read-only" state arises only from external causes

### Requirement: X-4: Implementation constraints
The system SHALL be written in Rust, the DB itself by hand (the storage engine, indexes, durability, operation logic and the
server), and ready-made storages and search engines are forbidden. Trace: item 7 of the original statement; the owner decisions
"Implementation". Decisions: ADR 0010, 0017.

#### Scenario: Verified libraries with a rationale
- **WHEN** an external crate is added to the project for the rest (Unicode, HTTP, JSON, random numbers, CRC, base64, etc.)
- **THEN** it has a rationale (Decision Record) and a cargo deny check; Fact Grounding is mandatory

#### Scenario: The Unicode stack and HTTP/JSON
- **WHEN** the Unicode, HTTP and JSON libraries are chosen
- **THEN** the whole Unicode stack is ICU4X of one version, Unicode 18, fixed for all of v1; HTTP and JSON are hyper and serde_json

#### Scenario: A ready-made storage
- **WHEN** a ready-made storage or search engine is proposed
- **THEN** it is forbidden (verified by review and decision records, not by a behavior test)

### Requirement: X-5: Number version
The system SHALL ensure that any change to a number's records yields a version different from all those previously received by a client
for this number, and that the write and the version change are indivisible. Trace: the owner decision "Operations: versioning belongs to the number"; CUJ12;
OD-3, Q-77 (restore and failure, `docs/owner-decisions.md`). Open: Q-72, Q-32, Q-34, Q-78. Decisions: ADR 0006.

#### Scenario: The version is never repeated
- **WHEN** a number that had records has all records deleted, the number is added again and a client sends a change or delete with a version
  received before the delete
- **THEN** the action is not performed and data is unchanged; if the target record (number, date) exists — a conflict, otherwise the
  kind of refusal is <not decided: Q-33>

#### Scenario: A number without records
- **WHEN** the number has no records (it is at the initial version) and a client sends an action with this version on a missing
  record
- **THEN** the action is not performed and data is unchanged; the kind of refusal is <not decided: Q-33>

#### Scenario: The record and the version are indivisible
- **WHEN** a failure occurs between the write and the change of the number version
- **THEN** there is no partial result: there is no state where the record is changed and the version is not

#### Scenario: After a restore or failure
- **WHEN** a number is restored from a backup or the server is restarted after a failure and a client sends a change or delete with a
  version read before the restore or before the failure
- **THEN** a conflict and data is unchanged (for any numbers)

#### Scenario: After a planned restart
- **WHEN** the server is stopped and started as planned (without a failure) and a client sends a change or delete with a version
  read before the stop
- **THEN** the version is valid: the action is performed if the version matches
