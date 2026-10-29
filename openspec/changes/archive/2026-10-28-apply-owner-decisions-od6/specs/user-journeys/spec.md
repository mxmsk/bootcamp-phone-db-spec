## REMOVED Requirements

### Requirement: X-1: Scale of 10^8 records
**Reason**: OD-6
**Migration**: the requirement is replaced by "X-1: Scale of 10^7 records" (values and scenarios recalculated per OD-6; the scenario names contained outdated numbers or wording).

## MODIFIED Requirements

### Requirement: CUJ1: Adding a new number
The system SHALL let the user add a new phone number entered in international format with
+<country-code>: the number is saved and is found by name, and the name is found by number.
Decisions: ADR 0002, 0003, 0004, 0005, 0006, 0007, 0008, 0019, 0020–0024, 0025, 0027, 0028.

#### Scenario: Saving an unknown number
- **WHEN** the user enters an unknown number "+<country-code>…" and a valid name in the form and sends an add
  request (create only)
- **THEN** the record is created and the number has a version; its properties (how it differs from those issued earlier, the limits of the guarantee) are defined by X-5
- **AND** the UI reports a successful save <undecided: Q-61>

#### Scenario: The name is found by word-prefix search
- **WHEN** after saving, a search by word prefix of the name is run
- **THEN** the record with this number is reachable in the results (if there are more results than a page, by paging with "next")

#### Scenario: The name is found by number
- **WHEN** after saving, a search by this number (exact match) is run
- **THEN** the results contain the record with this name

#### Scenario: No date passed
- **WHEN** the add request has no date and the number has no records (CUJ1 does not ask for a date; whether the form has an optional date field —
  <undecided: Q-62>)
- **THEN** the record is stored with the date 1876-03-07 (substitution is only on add: on rename and delete the date
  is mandatory, owner decision OD-5, Q-90, ADR 0025 item 6)

#### Scenario: Adding without a date for a known number
- **WHEN** the add request has no date and the number already has a record on any date
- **THEN** no record is created, and the server answers with a "number known" conflict with the current number version (owner decision, OD-6, Q-160); the flow moves to CUJ2

#### Scenario: A number without a plus
- **WHEN** the number is entered without "+"
- **THEN** it is rejected with a hint and no record is created (the hint text in the UI — <undecided: Q-65>; the hint's expression
  for an API client — <undecided: Q-49>)

#### Scenario: A number of invalid shape
- **WHEN** after "+" there are fewer than 1 or more than 15 digits, or the first digit is 0 (owner decision, OD-5, D-1, ADR 0004), or there are
  characters other than digits and removable separators
- **THEN** the number is rejected and no record is created

#### Scenario: A number with separators
- **WHEN** the number is entered with spaces, hyphens, parentheses or dots (for example, «+7 (999) 123-45-67»)
- **THEN** the record is created
- **AND** a search by the same number written differently (for example, «+7 999 123 45 67») finds the same record

#### Scenario: The country code is not assigned
- **WHEN** the number has a valid shape ("+", 1–15 digits, first digit 1–9 — owner decision, OD-5, D-1, ADR 0004), but its country code is not assigned
- **THEN** the number is accepted: there is no check against a list of country codes (owner decision, OD-5, D-1, ADR 0004)

#### Scenario: A name longer than allowed
- **WHEN** the name is longer than allowed (the maximum is per `records`, "Text length and length cap" (owner decision, OD-5, D-3, ADR 0022))
- **THEN** an error, no record is created, and the name is not truncated

#### Scenario: An empty name
- **WHEN** the name is empty or consists only of spaces
- **THEN** an error and no record is created

#### Scenario: Name normalization
- **WHEN** the name is entered with spaces at the edges or in a non-normalized form
- **THEN** the record is stored with the name trimmed at the edges and converted to NFC

#### Scenario: A retry of an identical record
- **WHEN** an identical record is added again (for example, after a connection drop)
- **THEN** success, and the data and number version did not change

#### Scenario: The number turned out to be known already
- **WHEN** the number is already in the database
- **THEN** the flow moves to CUJ2 (how the UI determines this — <undecided: Q-28>)

#### Scenario: A change is impossible or the UI has an old token
- **WHEN** the database is available only for reading or there is a storage error, or the UI is open with an old token
- **THEN** CUJ16 (database state) or CUJ13 (old link) applies

#### Scenario: Write acknowledgement latency
- **WHEN** an add runs on a database of 10^7 records
- **THEN** the acknowledgement is received no later than <undecided: Q-58>; the criterion applies to all write operations
  (CUJ1–CUJ3, CUJ6, CUJ9) and is repeated in X-1

### Requirement: CUJ2: Adding a known number with a new date
When a known number is saved, the system SHALL show that such a number already exists and ask for the date from which the
number is assigned to the user; the number is saved and from that moment a search by number returns the list of all its
owners by date. Decisions: ADR 0002, 0003, 0004, 0005, 0006, 0007, 0019, 0020–0025.

#### Scenario: A date prompt for a known number
- **WHEN** the user saves a number that is already in the database
- **THEN** the server answers an add without a date with a "number known" conflict (owner decision, OD-6, Q-160), the UI reports that the number already exists and asks for a date; the date input format is <undecided: Q-62>, the dialog content is <undecided: Q-37>

#### Scenario: The CUJ1/CUJ2 race is closed by the server
- **WHEN** the number became known between opening the form and an add without a date
- **THEN** no record on 1876-03-07 is created: the server answers "number known", then CUJ2 (owner decision, OD-6, Q-160)

#### Scenario: A new date for a known number
- **WHEN** the user enters a valid date on which the number has no record
- **THEN** the record (number, date) is created and the number version changed (X-5)
- **AND** records with other dates are not changed
- **AND** the UI reports success <undecided: Q-61>

#### Scenario: Results by number after a new date
- **WHEN** after the add, a search by number is run
- **THEN** the results contain all the number's records (old and new), sorted by date descending, without pagination
  (showing past, current and future owners — <undecided: Q-2>)

#### Scenario: A date earlier than allowed
- **WHEN** the date 1876-03-06 is entered
- **THEN** no record is created (the kind of refusal — <undecided: Q-37>, <undecided: Q-49>)

#### Scenario: A nonexistent calendar date
- **WHEN** a nonexistent calendar date is entered (for example, 30 February)
- **THEN** no record is created (the shape of the refusal — <undecided: Q-37>, <undecided: Q-49>); values with a time or
  a time zone: in the UI — <undecided: Q-62>, in the API — <undecided: Q-67>

#### Scenario: A future date
- **WHEN** a future date is entered (there is no product upper bound; the technical limit is 9999-12-31, `records`, "Valid date range", owner decision, OD-5, D-5, ADR 0024)
- **THEN** the record is created

#### Scenario: The same name on a different date
- **WHEN** the number has a record with name N on date D1 and a record with the same name N is added on another valid date D2
- **THEN** a separate record (number, D2) is created, and the record (number, D1) is not changed

#### Scenario: The same record on the same date
- **WHEN** the entered date already has a record with exactly the same name, or the name differs from the stored one only in
  edge spaces or normalization form (NFC)
- **THEN** the operation succeeds, and the data and number version did not change

#### Scenario: The name differs only in case
- **WHEN** on the same date the name matches the stored one only case-insensitively ("Ann" versus "ANN")
- **THEN** it is a conflict (moving to CUJ3), not a silent success

#### Scenario: A date on which the number already has a different name
- **WHEN** on the entered date the number already has a record with a different name (including a different spelling)
- **THEN** the flow moves to CUJ3

#### Scenario: The date 1876-03-07
- **WHEN** the date 1876-03-07 is entered
- **THEN** it is the same record as the server would produce by substituting the zero date (how an explicit date differs from
  a substituted one in the results — <undecided: Q-1>)

#### Scenario: The user does not enter a date
- **WHEN** the user does not enter a date or declines to enter one
- **THEN** the behavior is <undecided: Q-37>, <undecided: Q-28>

#### Scenario: Invalid name or number
- **WHEN** the name is empty, only spaces, or longer than allowed (the maximum is per `records`, "Text length and length cap" (owner decision, OD-5, D-3, ADR 0022)), or the number has an invalid
  shape
- **THEN** no record is created and the data is not changed

#### Scenario: Database state on an add with a date
- **WHEN** the database is available only for reading or there is a storage error
- **THEN** CUJ16 applies

### Requirement: CUJ3: Adding a known number with an existing date
When a known number is added with a date that already exists, the system SHALL show an overwrite prompt: if yes —
the data changes and the old is erased entirely; if no — the system waits for new input. Decisions: ADR 0002, 0003, 0005, 0006, 0020–0025.

#### Scenario: A conflict on add
- **WHEN** a record is added on (number, date) where a different name already exists
- **THEN** the server answers with a conflict with the current name and the number version, and the record is not changed
- **AND** the UI shows the overwrite prompt (content — <undecided: Q-37>)

#### Scenario: Overwrite confirmation
- **WHEN** the user answers "yes" and the number version sent by the UI is current (the source of the version — <undecided: Q-34>)
- **THEN** the UI calls the explicit rename operation with the expected version and the record's date (the date is mandatory, OD-5, Q-90; overwrite is offered only for an add with a date, an add without a date gives "number known", OD-6, Q-160), the name on (number, date) is replaced with the entered one, and the number version changed (X-5)
- **AND** the earlier record (number, date) with the earlier name is not returned by any search
- **AND** the UI reports success <undecided: Q-61>

#### Scenario: Declining the overwrite
- **WHEN** the user answers "no"
- **THEN** the data is not changed and the UI waits for new input (what is kept in the form — <undecided: Q-37>)

#### Scenario: The number's other dates are not affected
- **WHEN** the overwrite is performed
- **THEN** the number's records on other dates are not changed

#### Scenario: The version is stale
- **WHEN** after "yes" the number version sent by the UI does not match the current one
- **THEN** the record is not changed, and CUJ12 applies

#### Scenario: The record vanished between the conflict and the confirmation
- **WHEN** the record (number, date) vanished between the conflict and the overwrite confirmation
- **THEN** this is a special case of a stale version (a delete changes the number version); the outcome is a "version conflict" with the current
  version (the version is checked before the record's existence: ADR 0025 item 2, owner decision, OD-5, D-6); the UI's reaction is <undecided: Q-68>

#### Scenario: The same name on an existing date
- **WHEN** the entered name matches the stored one after space trimming and NFC
- **THEN** a silent success, and the overwrite prompt is not shown

#### Scenario: A typo is fixed only through overwrite
- **WHEN** the user fixes a typo in the name of an existing record
- **THEN** this is done by an add on the same (number, date) with the corrected name and an overwrite confirmation;
  there is no other way to fix it in the UI (how the UI lets the user enter the date of an existing record — <undecided: Q-28>)

#### Scenario: No "edit" button
- **WHEN** a row is shown in the results
- **THEN** it has no "edit" button

#### Scenario: An invalid new name
- **WHEN** the new name is empty, only spaces, or longer than allowed (the maximum is per `records`, "Text length and length cap" (owner decision, OD-5, D-3, ADR 0022))
- **THEN** an error and the record is not changed

#### Scenario: Renaming a nonexistent record on the UI path
- **WHEN** the CUJ3 path is executed without a concurrent change
- **THEN** renaming a nonexistent record is unreachable; the scenario for the API is in CUJ9

#### Scenario: Database state on overwrite
- **WHEN** the database is available only for reading or there is a storage error
- **THEN** CUJ16 applies

### Requirement: CUJ4: Search by number
The system SHALL let the user search by phone number: a number returns only an exact match, and shows the list
of everyone the number was added for, sorted by date DESC. Decisions: ADR 0002, 0004, 0007, 0019.

#### Scenario: The full list of a number's records
- **WHEN** a full number with N records is entered
- **THEN** the results have exactly N records without pagination (the row contents — <undecided: Q-29>; the number's form in a row —
  <undecided: Q-26>; showing past, current and future owners — <undecided: Q-2>)

#### Scenario: Sorting by date
- **WHEN** the results have several records
- **THEN** they are ordered by date descending

#### Scenario: Different names on different dates
- **WHEN** a number has records with different names on different dates
- **THEN** all are returned (a name may repeat on different dates)

#### Scenario: Exact match only
- **WHEN** part of a number is entered
- **THEN** records of numbers that merely contain this part are not returned

#### Scenario: Number not found
- **WHEN** the number is not found
- **THEN** an empty list, and the response is not an error

#### Scenario: A number of invalid shape
- **WHEN** a number of invalid shape is entered (without "+", letters, etc.)
- **THEN** a format error (the same rules as on save, ADR 0004) and no results

#### Scenario: Search by a number with separators
- **WHEN** a number is saved and the search is run for the same number written with spaces, hyphens, parentheses or
  dots
- **THEN** the records of this number are found

#### Scenario: Search-by-number latency
- **WHEN** a search by number runs on a database of 10^7 records
- **THEN** the latency is no more than <undecided: Q-47>

#### Scenario: Storage state on search by number
- **WHEN** the database is available only for reading or there is a storage error
- **THEN** CUJ16 applies: search works as long as it can

### Requirement: CUJ5: Search by name
The system SHALL let the user search by name by the prefix of any word of the name (not by substring; all words of the query must
match; case-insensitive, "ё" = "е") and show the results as a paginated list sorted alphabetically, then
by date DESC. Decisions: ADR 0003, 0008, 0009, 0010, 0019, 0020–0022, 0030–0034, 0036.

#### Scenario: A word prefix, not a substring
- **GIVEN** the database has records with the names "Ann Smith", "John Annabel" and "Joann"
- **WHEN** the query "ann"
- **THEN** the results contain "Ann Smith" and "John Annabel" and not "Joann"

#### Scenario: Several query words
- **WHEN** the query consists of several words
- **THEN** the results contain only records where every query word matched; the order of query words does not matter ("ann smith" and
  "smith ann" find "Ann Smith")

#### Scenario: One name word covers several query words
- **GIVEN** the record "Annabel"
- **WHEN** the query "an ann"
- **THEN** the record is found

#### Scenario: Ё equals Е, case does not matter
- **GIVEN** the database has the records «Елка» and «Ёлка»
- **WHEN** the query «ёлка» (and conversely «елка», and «ЕЛКА»)
- **THEN** both records are found

#### Scenario: Case does not matter
- **GIVEN** the database has the record "Ann"
- **WHEN** the query "ann" (and "ANN")
- **THEN** the record is found

#### Scenario: A hyphen separates words
- **GIVEN** the record "Anna-Maria"
- **WHEN** the query "maria"
- **THEN** the record is found

#### Scenario: An apostrophe between letters does not separate words
- **GIVEN** the record "O'Brien"
- **WHEN** the query "o'brien" or "o'bri"
- **THEN** the record is found
- **AND** for the query "brien" the record "O'Brien" is not in the results (word boundaries strictly per UAX #29; the specific functions are
  the open question Q-76, docs/open-questions.md)

#### Scenario: A query in decomposed form
- **GIVEN** the record "José"
- **WHEN** the query "José" is entered in NFD form (the letter "e" and a separate accent U+0301)
- **THEN** the record is found: the query is converted to NFC

#### Scenario: An empty query
- **WHEN** the query is empty, consists only of spaces or only of punctuation (for example, "-", "'"): it has no
  words
- **THEN** an error, no results (the whitespace character set — `records`, "Whitespace character set", ADR 0021 item 1; the reason — (query, "empty"), ADR 0034)

#### Scenario: Pagination
- **WHEN** there are more results than a page
- **THEN** a part is returned (page size — default 50, owner decision, OD-5, D-10, ADR 0033) and the next can be obtained by cursor
- **AND** the total number of results is not reported

#### Scenario: Result order
- **WHEN** the results have different names
- **THEN** they go alphabetically under ru locale rules (comparison parameters — ADR 0037, 0038; owner decision, OD-5, D-11, D-12, D-15, Q-93), records with one name
  by date DESC

#### Scenario: Order at equal name and date
- **GIVEN** two records with the same name and date for different numbers
- **WHEN** a name search is run
- **THEN** they go by number ascending as a string: "+19…" before "+2…", numbers of one country adjacent
  (the total order — ADR 0036, `result-ordering`; the cursor — ADR 0032)

#### Scenario: No matches
- **WHEN** there are no matches
- **THEN** the form of empty results is <undecided: Q-29>

#### Scenario: The cursor is not accepted
- **WHEN** the cursor is corrupted (does not decode, the checksum does not match) or belongs to a different query or kind of search
- **THEN** an "invalid cursor" error (ADR 0032 item 4, ADR 0034); a data change between pages does not break the cursor:
  the next page is the records strictly after the cursor position by the data as of the request (ADR 0032 items 1–3;
  consistency between pages is not promised — owner decision, OD-5, D-8, Q-56)

#### Scenario: A server upgrade within v1 does not change search
- **WHEN** the server is upgraded within v1
- **THEN** the matches and the search order for the same data and queries did not change: Unicode 18 is fixed for all of v1
  (the mechanics — open questions Q-74, Q-82, docs/open-questions.md)

#### Scenario: Name-search latency
- **WHEN** a name search (including a multi-word one) runs on 10^7 records
- **THEN** the latency is no more than <undecided: Q-47>; for a multi-word search — <undecided: Q-59>

#### Scenario: Storage state on name search
- **WHEN** the database is available only for reading or there is a storage error
- **THEN** CUJ16 applies

### Requirement: CUJ7: A local program works with the database
A local program on the same machine SHALL read the token from a file and perform the same operations as the UI, plus exact
whole-name search; after a server restart, its request with an old token SHALL get an "unauthorized" refusal,
and after re-reading the token file the retry succeeds (a rename and delete with an old version after a failure are a conflict,
OD-4, item 5; with the X-5 caveats: owner decision, OD-5, D-17, D-13). Open: Q-70 (whether TLS is needed on loopback).
Decisions: ADR 0005, 0006, 0007, 0008, 0011, 0012, 0013, 0019, 0027, 0028, 0031.

#### Scenario: The same operations as in the UI
- **WHEN** a local program with a valid token calls add, rename, delete, search by number,
  search by name words or exact whole-name search
- **THEN** the rules for number and name validation, conflicts and versions are the same as when called from the UI; there are no UI-level differences
  (date prompt, overwrite dialog, delete confirmation) in the API; on an add without a date the server substitutes 1876-03-07 if the number has no records (otherwise "number known", OD-6, Q-160), and a rename and delete without a date are an error (date, "missing"; OD-5, Q-90)
- **AND** the local program's permissions are full; the location and access rights of the token file — <undecided: Q-38>

#### Scenario: Exact whole-name search
- **WHEN** the program calls exact whole-name search
- **THEN** all records with this name are returned, case-insensitively, "ё" = "е"; the query is trimmed at the edges and
  converted to NFC, like a name when saved (order and pagination — as for word search, ADR 0031, owner decision, OD-5, D-9)

#### Scenario: Exact search for "елка"
- **GIVEN** the record «Ёлка»
- **WHEN** exact whole-name search for «елка» (or «ЁЛКА»)
- **THEN** the record is found

#### Scenario: An empty exact-search query
- **WHEN** the exact-search query is empty or consists only of spaces
- **THEN** an error

#### Scenario: An old token after a restart
- **WHEN** the server was restarted and the program sends a request with an old token
- **THEN** an "unauthorized" refusal (token not presented — the code and shape <undecided: Q-50>)

#### Scenario: An unknown token
- **WHEN** a request presents a token unknown to the server (invalid or arbitrary)
- **THEN** an "unauthorized" refusal and the data is not changed (the code and shape — <undecided: Q-50>)

#### Scenario: A retry after re-reading the token
- **WHEN** the server was restarted in a planned way, the program re-read the token file and repeated the request (a search; a rename or
  delete — with the number version read before the planned restart, which remains valid)
- **THEN** success

#### Scenario: A restart after a failure
- **WHEN** the server was restarted after a failure and the program repeats a rename or delete with a number version read before
  the failure
- **THEN** a conflict and the data is not changed (for any numbers; the limits of the guarantee — X-5, D-17, D-13); the program
  re-reads the number history and retries with the current version — success

#### Scenario: A name or version conflict
- **WHEN** a name or version conflict occurs
- **THEN** the behavior is the same as in CUJ9

#### Scenario: A file token presented not from loopback
- **WHEN** a token from the file is presented from a non-loopback address (from another machine)
- **THEN** the request is rejected and the data is not changed (the refusal code — <undecided: Q-50>)

#### Scenario: The request limit does not apply
- **WHEN** a local program sends requests at any frequency
- **THEN** there is no per-token request-limit refusal

#### Scenario: API version in the path
- **WHEN** a client requests a v1 path
- **THEN** the request is served (a request without a version in the path — <undecided: Q-51>)

#### Scenario: The token file is unavailable
- **WHEN** the token file is unavailable or not yet updated after start
- **THEN** the behavior is <undecided: Q-38>

#### Scenario: Connection load
- **WHEN** the number of connections, the request body size or the request time exceed the server limits
- **THEN** the behavior is <undecided: Q-52> (ADR 0019)

#### Scenario: Storage state when called from the API
- **WHEN** the database is available only for reading or there is a storage error
- **THEN** CUJ16 applies

### Requirement: CUJ9: A network service changes data
A network service with write permission SHALL add a number; if the number already has a different name on that date, it gets a
conflict with the current name and the number version. It SHALL send a rename and delete with the number version; on a stale
version it gets a conflict with the current version, re-reads the number history and retries.
Decisions: ADR 0002, 0003, 0004, 0005, 0006, 0012, 0019, 0022, 0025.

#### Scenario: Adding a record
- **WHEN** a service with a `read_write` token adds a record on (number, date) with no record (on add the date is optional — by
  default 1876-03-07 if the number has no records, otherwise "number known", OD-6, Q-160; the date format is <undecided: Q-67>)
- **THEN** the record is created

#### Scenario: A name conflict on add
- **WHEN** the (number, date) being added already has a different name
- **THEN** a conflict, the response contains the current name and the number version, and the record is not changed

#### Scenario: A retry of an add of an identical record
- **WHEN** the service repeats an add of an identical record (for example, after a connection drop)
- **THEN** success, and the data and version are not changed

#### Scenario: A rename or delete without a date
- **WHEN** a rename or delete is sent without a date
- **THEN** a value error (date, "missing"), the data is not changed; there is no 1876-03-07 substitution (owner decision, OD-5,
  Q-90; ADR 0025 item 6); the shape of the refusal — <undecided: Q-49>

#### Scenario: A rename without an expected version
- **WHEN** a rename or delete is sent without an expected version
- **THEN** the operation is not performed (the error shape — <undecided: Q-49>, <undecided: Q-50>)

#### Scenario: A stale version with an existing record
- **WHEN** the version in a rename or delete is stale and the record (number, date) exists
- **THEN** a conflict, the response contains the current version, and the data is not changed (the source of the current version —
  <undecided: Q-34>; the contents of successful responses — <undecided: Q-32>)

#### Scenario: A retry with the current version
- **WHEN** the service repeated a rename with the current version, the record exists and the new name differs from the current one (exact
  spelling)
- **THEN** success and the number version changed

#### Scenario: Renaming a missing record
- **WHEN** a record that does not exist is renamed and the current number version is passed
- **THEN** "not found"; with a stale version — "version conflict" (check order — ADR 0025, owner decision, OD-5, D-6)

#### Scenario: Deleting a missing record
- **WHEN** a record that does not exist is deleted and the current number version is passed
- **THEN** "not found" (with a stale version — "version conflict", ADR 0025 item 2, owner decision, OD-5, D-6)

#### Scenario: A rename to the same name with the current version
- **WHEN** a rename is sent with the current number version and a name equal to the current one (exact spelling after
  space trimming and NFC)
- **THEN** a silent success without changes, the number version does not change, as on a retry of an add (the response contents —
  <undecided: Q-32>)

#### Scenario: A rename to the same name with a stale version
- **WHEN** a rename to the same name is sent with a stale number version
- **THEN** a conflict with the current version and the data is not changed (the version check comes first)

#### Scenario: A read token and the limit
- **WHEN** the request is made by a `read` token, or the service exceeded the request limit
- **THEN** "no permissions" (CUJ8) or a limit refusal (the value — <undecided: Q-46>)

#### Scenario: A retry after a connection drop
- **WHEN** a rename or delete is repeated after a connection drop
- **THEN** a retry with the earlier version is a "version conflict" (ADR 0026, owner decision, OD-5, D-7; `write-operations`, "Retrying an
  operation after an unknown outcome"); the representation in the API — <undecided: Q-54>, <undecided: Q-32>

#### Scenario: Invalid name, number, date
- **WHEN** the name is empty or only spaces, or longer than allowed (the maximum is per `records`, "Text length and length cap" (owner decision, OD-5, D-3, ADR 0022)); or the number
  has an invalid shape; or the date is not a calendar day, is earlier than 1876-03-07 or (in a rename and delete) is not passed
- **THEN** an error, the data is not changed (the shape of the date refusal — <undecided: Q-49>; values with a time or zone —
  <undecided: Q-67>)

### Requirement: CUJ14: Data survived a failure
A save of a number acknowledged to the user SHALL survive a process crash or power loss: after the server restarts,
the number is in place; a save that did not receive acknowledgement is either entirely present or entirely absent. If
the database is inconsistent at start, the server does not start and reports to the operator; other corruption is a "storage error" on read
(OD-4, item 3); data is not silently lost; the operator restores the database from a backup (CUJ15). Decisions: ADR 0006, 0013, 0014, 0016, 0027, 0028 (X-5 caveats: owner decision, OD-5, D-17, D-13).

#### Scenario: An acknowledged save survives a failure
- **WHEN** the user or client received an acknowledgement of a save (add), the process crashed or power was lost, and
  the server was restarted (the moment of acknowledgement — <undecided: Q-57>)
- **THEN** the acknowledged record is present

#### Scenario: An unacknowledged save
- **WHEN** a save (add) did not receive acknowledgement and there was a failure
- **THEN** after the restart the record is either entirely present or entirely absent

#### Scenario: An acknowledged rename and delete
- **WHEN** a rename (CUJ3) or delete (CUJ6) was acknowledged, there was a failure and the server was restarted
- **THEN** the acknowledged change is preserved and a deleted record does not "come back to life"

#### Scenario: An unacknowledged rename and delete
- **WHEN** a rename or delete did not receive acknowledgement and there was a failure
- **THEN** after the restart the change is either entirely present or entirely absent (the record and the number version change
  are indivisible, X-5)

#### Scenario: The limits of the guarantee
- **WHEN** a process crash, an OS failure or a power loss occurred and the disk honestly performs flush to stable storage
- **THEN** acknowledged adds and changes are present and acknowledged-deleted records are absent
- **AND** a "lying" disk and medium corruption are outside the guarantee, detecting a "lying" disk is not promised; a disk write error is
  a "storage error" (CUJ16)

#### Scenario: A check at start
- **WHEN** at start the check (structure and the latest changes) shows that the database does not open consistently
- **THEN** the server does not start, the operator gets a message (the channel — <undecided: Q-66>) and records are not silently
  discarded (the scope of the check — <undecided: Q-43>; the start time at 10^7 records — <undecided: Q-42>)

#### Scenario: Corruption found on read
- **WHEN** corruption of the rest of the data (not found by the check at start) is detected when it is read or on a running
  server
- **THEN** the server goes to the "storage error" state: changes are rejected (CUJ16), search works as long as it can,
  and a message is issued to the operator

#### Scenario: Restore on a damaged database
- **WHEN** the database is damaged and the operator starts the server on a backup
- **THEN** the server starts on the backup's data (including a backup made before an upgrade: the new program opens
  such a database itself; there is no rollback)

#### Scenario: A version read before the failure
- **WHEN** a client sends a rename or delete with a number version read before the failure, and the server was restarted after the
  failure
- **THEN** a conflict and the data is not changed; the client re-reads; a conflict is acceptable for any number (the mechanics —
  the open question Q-78, docs/open-questions.md); the limits of the guarantee between version generations — per X-5 (owner decision, OD-5, D-17;
  the exception — files copied not by the command, D-13)

#### Scenario: A planned restart
- **WHEN** the server was restarted in a planned way (clean shutdown: defined in `write-operations`, "Clean and crash
  shutdown", ADR 0028 item 10) and a client sends a rename or delete with a number version read before the shutdown
- **THEN** the version is valid: the action is performed if the version matches

## ADDED Requirements

### Requirement: X-1: Scale of 10^7 records
The system SHALL perform operations CUJ1–CUJ16 without refusals caused by volume on a database of 10^7 records. Trace: item 6
of the original wording ("reliably store many records"); owner decision OD-6 (scale of 10^7 instead of the 10^8 of brief §4;
the earlier "Scale and reliability: 10^8 records" is replaced).
Decisions: ADR 0017.

#### Scenario: Operations at 10^7 records
- **WHEN** the database contains 10^7 records
- **THEN** operations CUJ1–CUJ16 are performed without refusals caused by volume (there is no numeric threshold until the targets are decided:
  size and search latency — <undecided: Q-47>, start time — <undecided: Q-42>)

#### Scenario: Write operation latency
- **WHEN** any write operation runs on a database of 10^7 records
- **THEN** the acknowledgement is received no later than <undecided: Q-58>

#### Scenario: Filling a database to check the scale
- **WHEN** a database of 10^7 records is needed for checking
- **THEN** the way to fill it is <undecided: Q-53>; there is no bulk load as a product operation (X-3)
