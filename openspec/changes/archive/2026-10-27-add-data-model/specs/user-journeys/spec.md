## MODIFIED Requirements

### Requirement: CUJ1: Adding a new number
The system SHALL let the user add a new phone number entered in international format with +<country-code>: the number is
saved and found by name, and the name is found by number.
Decisions: ADR 0002, 0003, 0004, 0005, 0006, 0007, 0008, 0019, 0020–0024, 0025, 0027, 0028.

#### Scenario: Saving an unknown number
- **WHEN** the user enters in the form an unknown number "+<country-code>…" and a valid name and sends an add request
  (create only)
- **THEN** the record is created, the number has a version; its properties (difference from those issued earlier, bounds
  of the guarantee) are defined by X-5
- **AND** the UI reports a successful save <undecided: Q-61>

#### Scenario: The name is found by word-prefix search
- **WHEN** after saving, a search by a word prefix of the name is performed
- **THEN** the record with this number is reachable in the results (if there are more results than a page — by paging "next")

#### Scenario: The name is found by number
- **WHEN** after saving, a search by this number (exact match) is performed
- **THEN** the results contain the record with this name

#### Scenario: Date not passed
- **WHEN** the add request has no date (CUJ1 does not ask for a date; whether the form has an optional date field is
  <undecided: Q-62>)
- **THEN** the record is stored with the date 1876-03-07 (substitution only on add: on rename and delete the date is
  mandatory, owner decision OD-5, Q-90, ADR 0025 item 6)

#### Scenario: Number without a plus
- **WHEN** the number is entered without "+"
- **THEN** it is rejected with a hint and the record is not created (the hint text in the UI is <undecided: Q-65>; how the
  hint is expressed for an API client is <undecided: Q-49>)

#### Scenario: Number of invalid shape
- **WHEN** after "+" there are fewer than 1 or more than 15 digits, or the first digit is 0 (owner decision, OD-5, D-1, ADR
  0004), or there are characters other than digits and the separators that are removed
- **THEN** the number is rejected and the record is not created

#### Scenario: Number with separators
- **WHEN** the number is entered with spaces, hyphens, parentheses or periods (for example, «+7 (999) 123-45-67»)
- **THEN** the record is created
- **AND** a search by the same number written differently (for example, «+7 999 123 45 67») finds the same record

#### Scenario: Country code not assigned
- **WHEN** the number has a valid shape ("+", 1–15 digits, the first digit 1–9 — owner decision, OD-5, D-1, ADR 0004), but
  its country code is not assigned
- **THEN** the number is accepted: there is no check against a list of country codes (owner decision, OD-5, D-1, ADR 0004)

#### Scenario: Name longer than allowed
- **WHEN** the name is longer than allowed (the maximum is per `records`, "Text length and length cap" (owner decision, OD-5,
  D-3, ADR 0022))
- **THEN** an error, the record is not created, the name is not truncated

#### Scenario: Empty name
- **WHEN** the name is empty or consists only of spaces
- **THEN** an error and the record is not created

#### Scenario: Name normalization
- **WHEN** the name is entered with spaces at the edges or in a non-normalized form
- **THEN** the record is stored with the name trimmed at the edges and converted to NFC

#### Scenario: Retrying an identical record
- **WHEN** an identical record is added again (for example, after a connection drop)
- **THEN** success, the data and the number version are unchanged

#### Scenario: The number turned out to be already known
- **WHEN** the number is already in the database
- **THEN** the flow goes to CUJ2 (how the UI determines this is <undecided: Q-28>)

#### Scenario: A change is impossible or the UI has an old token
- **WHEN** the database is read-only or there is a storage error, or the UI is opened with an old token
- **THEN** CUJ16 (database state) or CUJ13 (old link) applies

#### Scenario: Write acknowledgement latency
- **WHEN** an add is performed on a database of 10^8 records
- **THEN** the acknowledgement is received no later than <undecided: Q-58>; the criterion applies to all write operations
  (CUJ1–CUJ3, CUJ6, CUJ9) and is repeated in X-1

### Requirement: CUJ2: Adding a known number with a new date
The system SHALL, when a known number is saved, show that such a number already exists and ask for the date from which the
number is assigned to the user; the number is saved and from then on a query by number returns the list of all its owners
by date. Decisions: ADR 0002, 0003, 0004, 0005, 0006, 0007, 0019, 0020–0025.

#### Scenario: Asking for a date for a known number
- **WHEN** the user saves a number that is already in the database
- **THEN** the UI reports that the number already exists and asks for a date <under what conditions the UI determines this
  is undecided: Q-28>; the date input format is <undecided: Q-62>, the content of the dialog is <undecided: Q-37>

#### Scenario: New date for a known number
- **WHEN** the user enters a valid date on which the number has no record
- **THEN** the record (number, date) is created, the number version changed (X-5)
- **AND** the records with other dates are unchanged
- **AND** the UI reports success <undecided: Q-61>

#### Scenario: Results by number after a new date
- **WHEN** after the add a search by number is performed
- **THEN** the results contain all records of the number (old and new), sorted by date in descending order, without
  pagination (showing past, current and future owners is <undecided: Q-2>)

#### Scenario: Date earlier than allowed
- **WHEN** the date 1876-03-06 is entered
- **THEN** the record is not created (the kind of refusal is <undecided: Q-37>, <undecided: Q-49>)

#### Scenario: Nonexistent calendar date
- **WHEN** a nonexistent calendar date is entered (for example, 30 February)
- **THEN** the record is not created (the form of refusal is <undecided: Q-37>, <undecided: Q-49>); values with a time or
  a time zone: in the UI <undecided: Q-62>, in the API <undecided: Q-67>

#### Scenario: Future date
- **WHEN** a future date is entered (there is no product upper bound; the technical limit is 9999-12-31, `records`, "Valid
  date range", owner decision, OD-5, D-5, ADR 0024)
- **THEN** the record is created

#### Scenario: The same name on another date
- **WHEN** the number has a record with the name N on the date D1 and a record with the same name N is added on another
  valid date D2
- **THEN** a separate record (number, D2) is created, the record (number, D1) is unchanged

#### Scenario: The same record on the same date
- **WHEN** on the entered date there is already a record with exactly the same name, or the name differs from the saved one
  only in spaces at the edges or in normalization form (NFC)
- **THEN** the operation succeeds, the data and the number version are unchanged

#### Scenario: The name differs only in case
- **WHEN** on the same date the name matches the saved one only ignoring case («Ann» versus «ANN»)
- **THEN** this is a conflict (go to CUJ3), not a silent success

#### Scenario: A date on which the number already has a different name
- **WHEN** on the entered date the number already has a record with a different name (including a different spelling)
- **THEN** the flow goes to CUJ3

#### Scenario: Date 1876-03-07
- **WHEN** the date 1876-03-07 is entered
- **THEN** this is the same record that would result from the server substituting the zero date (the difference between an
  explicit and a substituted date in the results is <undecided: Q-1>)

#### Scenario: The user does not enter a date
- **WHEN** the user does not enter a date or declines to enter one
- **THEN** the behavior is <undecided: Q-37>, <undecided: Q-28>

#### Scenario: Invalid name or number
- **WHEN** the name is empty, only spaces or longer than allowed (the maximum is per `records`, "Text length and length cap"
  (owner decision, OD-5, D-3, ADR 0022)), or the number has an invalid format
- **THEN** the record is not created and the data is unchanged

#### Scenario: Database state when adding with a date
- **WHEN** the database is read-only or there is a storage error
- **THEN** CUJ16 applies

### Requirement: CUJ3: Adding a known number with an existing date
The system SHALL, when a known number is added with a date that already exists, show an overwrite prompt: if yes — the data
changes, the old is erased completely; if no — the system waits for new input. Decisions: ADR 0002, 0003, 0005, 0006,
0020–0025.

#### Scenario: Conflict on add
- **WHEN** a record is added on (number, date) where a different name already exists
- **THEN** the server responds with a conflict carrying the current name and the number version, the record is unchanged
- **AND** the UI shows an overwrite prompt (the content is <undecided: Q-37>)

#### Scenario: Overwrite confirmation
- **WHEN** the user answers "yes" and the number version sent by the UI is current (the source of the version is
  <undecided: Q-34>)
- **THEN** the UI calls the explicit rename operation with the expected version and the record date (the date is mandatory,
  OD-5, Q-90; if the add was without a date, this is 1876-03-07), the name on (number, date) is replaced with the entered
  one, the number version changed (X-5)
- **AND** the previous record (number, date) with the previous name is returned by no search
- **AND** the UI reports success <undecided: Q-61>

#### Scenario: Declining the overwrite
- **WHEN** the user answers "no"
- **THEN** the data is unchanged and the UI waits for new input (what is kept in the form is <undecided: Q-37>)

#### Scenario: Other dates of the number are not affected
- **WHEN** the overwrite is performed
- **THEN** the number's records on other dates are unchanged

#### Scenario: The version is stale
- **WHEN** after "yes" the number version sent by the UI does not match the current one
- **THEN** the record is unchanged, CUJ12 applies

#### Scenario: The record disappeared between the conflict and the confirmation
- **WHEN** the record (number, date) disappeared between the conflict and the overwrite confirmation
- **THEN** this is a special case of a stale version (a delete changes the number version); the outcome is a "version
  conflict" with the current version (the version is checked before the record's existence: ADR 0025 item 2, owner
  decision, OD-5, D-6); the UI reaction is <undecided: Q-68>

#### Scenario: The same name on an existing date
- **WHEN** the entered name matches the saved one after space trimming and NFC
- **THEN** a silent success and the overwrite prompt is not shown

#### Scenario: A typo is fixed only through an overwrite
- **WHEN** the user fixes a typo in the name of an existing record
- **THEN** this is done by an add on the same (number, date) with the corrected name and an overwrite confirmation; there is
  no other way to fix in the UI (how the UI lets the user enter the date of an existing record is <undecided: Q-28>)

#### Scenario: No "rename" button
- **WHEN** a row is shown in the results
- **THEN** it has no "rename" button

#### Scenario: Invalid new name
- **WHEN** the new name is empty, only spaces or longer than allowed (the maximum is per `records`, "Text length and length
  cap" (owner decision, OD-5, D-3, ADR 0022))
- **THEN** an error and the record is unchanged

#### Scenario: Renaming a nonexistent record in the UI path
- **WHEN** the CUJ3 path is executed without a concurrent change
- **THEN** renaming a nonexistent record is unreachable; the scenario for the API is in CUJ9

#### Scenario: Database state on overwrite
- **WHEN** the database is read-only or there is a storage error
- **THEN** CUJ16 applies

### Requirement: CUJ5: Search by name
The system SHALL let the user search by name by the beginning of any word of the name (not by substring; all query words
must match; case-insensitive, "ё" = "е") and show the results as a paginated list, sorted alphabetically, then by date
DESC. Decisions: ADR 0003, 0008, 0009, 0010, 0019, 0020–0022, 0030–0034, 0036.

#### Scenario: Start of a word, not a substring
- **GIVEN** the database has records with the names «Ann Smith», «John Annabel» and «Joann»
- **WHEN** the query «ann»
- **THEN** the results contain «Ann Smith» and «John Annabel» and do not contain «Joann»

#### Scenario: Several query words
- **WHEN** the query consists of several words
- **THEN** the results contain only records in which every query word matched; the order of the query words does not matter
  («ann smith» and «smith ann» find «Ann Smith»)

#### Scenario: One word of the name covers several query words
- **GIVEN** a record «Annabel»
- **WHEN** the query «an ann»
- **THEN** the record is found

#### Scenario: Ё equals Е, case does not matter
- **GIVEN** the database has the records «Елка» and «Ёлка»
- **WHEN** the query «ёлка» (and conversely «елка», and «ЕЛКА»)
- **THEN** both records are found

#### Scenario: Case does not matter
- **GIVEN** the database has a record «Ann»
- **WHEN** the query «ann» (and «ANN»)
- **THEN** the record is found

#### Scenario: A hyphen splits words
- **GIVEN** a record «Anna-Maria»
- **WHEN** the query «maria»
- **THEN** the record is found

#### Scenario: An apostrophe between letters does not split words
- **GIVEN** a record «O'Brien»
- **WHEN** the query «o'brien» or «o'bri»
- **THEN** the record is found
- **AND** for the query «brien» the record «O'Brien» is not in the results (word boundaries strictly per UAX #29; the
  specific functions are open question Q-76, docs/open-questions.md)

#### Scenario: A query in decomposed form
- **GIVEN** a record «José»
- **WHEN** the query «José» is entered in NFD form (the letter «e» and a separate accent U+0301)
- **THEN** the record is found: the query is converted to NFC

#### Scenario: Empty query
- **WHEN** the query is empty, consists only of spaces or of a single punctuation mark (for example, «-», «'»): there is
  not a single word in it
- **THEN** an error, no results (the whitespace character set is `records`, "Whitespace character set", ADR 0021 item 1; the
  reason is (query, "empty"), ADR 0034)

#### Scenario: Pagination
- **WHEN** there are more results than a page
- **THEN** a part is returned (the page size is 50 by default, owner decision, OD-5, D-10, ADR 0033) and the next one can be
  obtained by cursor
- **AND** the total number of results is not reported

#### Scenario: Result order
- **WHEN** the results contain different names
- **THEN** they go alphabetically under ru locale rules (the comparison parameters are ADR 0037, 0038; owner decision, OD-5,
  D-11, D-12, D-15, Q-93), records with one name — by date DESC

#### Scenario: Order for equal name and date
- **GIVEN** two records with the same name and date on different numbers
- **WHEN** a search by name is performed
- **THEN** they go by number ascending as a string: «+19…» before «+2…», numbers of one country are adjacent (the total
  order is ADR 0036, `result-ordering`; the cursor is ADR 0032)

#### Scenario: No matches
- **WHEN** there are no matches
- **THEN** the form of the empty results is <undecided: Q-29>

#### Scenario: The cursor is not accepted
- **WHEN** the cursor is corrupted (does not decode, the checksum does not match) or belongs to another query or search kind
- **THEN** a "cursor invalid" error (ADR 0032 item 4, ADR 0034); a data change between pages does not spoil the cursor: the
  next page is the records strictly after the cursor position by the data at the moment of the request (ADR 0032 items 1–3;
  consistency between pages is not promised — owner decision, OD-5, D-8, Q-56)

#### Scenario: A server update within v1 does not change search
- **WHEN** the server is updated within v1
- **THEN** the matches and the search order on the same data and queries did not change: Unicode 18 is fixed for all of v1
  (the mechanics are open questions Q-74, Q-82, docs/open-questions.md)

#### Scenario: Name-search latency
- **WHEN** a search by name (including a multi-word one) is performed on 10^8 records
- **THEN** the latency is no more than <undecided: Q-47>; for a multi-word search — <undecided: Q-59>

#### Scenario: Storage state during name search
- **WHEN** the database is read-only or there is a storage error
- **THEN** CUJ16 applies

### Requirement: CUJ6: Deleting a record
The system SHALL let the user delete a record of a number: the user searches by number or name; in the found list, on the
left, a "delete" button with a trash-can icon is shown; before deletion a confirmation is requested; after deletion the
list is re-queried and shown updated. Decisions: ADR 0005, 0006, 0007, 0025.

#### Scenario: Delete button in the row
- **WHEN** the results list (by number or by name) has a row
- **THEN** on its left there is a "delete" button with a trash-can icon

#### Scenario: Confirmation request
- **WHEN** the user presses "delete"
- **THEN** a confirmation is requested and until it is confirmed the record is not deleted

#### Scenario: Confirmed delete
- **WHEN** the user confirms and the number version is current (the source of the version is <undecided: Q-34>)
- **THEN** exactly one record (number, date) is deleted, the number version changed (X-5), the other records of the number
  are intact; the UI passes the date of the results row explicitly (the date is mandatory in a delete, OD-5, Q-90)

#### Scenario: Updated list after the delete
- **WHEN** the delete is performed
- **THEN** the UI re-queries the data and shows the list without the deleted record (which query and page — <undecided:
  Q-35>; the form of an empty list — <undecided: Q-29>)

#### Scenario: Declining the confirmation
- **WHEN** the user declines in the confirmation dialog
- **THEN** the data is unchanged (what the UI shows after the decline is <undecided: Q-63>)

#### Scenario: Stale version on delete
- **WHEN** the number version in the delete request does not match the current one and the record (number, date) exists
- **THEN** the record is not deleted and the response contains the current version; CUJ12 applies (for a missing record with
  a stale version — "version conflict", ADR 0025 item 2, owner decision, OD-5, D-6)

#### Scenario: Deleting a missing record
- **WHEN** a record that does not exist is deleted and the current number version is passed
- **THEN** "not found" (with a stale version — "version conflict" with the current version, ADR 0025 item 2, owner
  decision, OD-5, D-6); the UI reaction is <undecided: Q-68>

#### Scenario: Undoing a performed delete
- **WHEN** an undo of an already performed delete is needed
- **THEN** the CUJ6 text does not provide for it; whether it is needed is <undecided: Q-63>

#### Scenario: Database state on delete
- **WHEN** the database is read-only or there is a storage error
- **THEN** CUJ16 applies

### Requirement: CUJ7: A local program works with the database
A local program on the same machine SHALL read the token from a file and perform the same operations as the UI, plus exact
whole-name search; after a server restart its request with the old token SHALL receive an "unauthorized" refusal, and
after re-reading the token file a retry succeeds (a rename and delete with an old version after a failure are a conflict,
OD-4, item 5; with the caveats of X-5: owner decision, OD-5, D-17, D-13). Open: Q-70 (whether TLS is needed on loopback).
Decisions: ADR 0005, 0006, 0007, 0008, 0011, 0012, 0013, 0019, 0027, 0028, 0031.

#### Scenario: The same operations as in the UI
- **WHEN** a local program with a valid token calls add, rename, delete, search by number, word search by name or exact
  whole-name search
- **THEN** the rules of number and name validation, conflicts and versions are the same as when called from the UI; there
  are no UI-level differences (date prompt, overwrite dialog, delete confirmation) in the API; on an add without a date the
  server substitutes 1876-03-07, while a rename and delete without a date are an error (date, "missing"; OD-5, Q-90)
- **AND** the local program's permissions are full; the location and access rights of the token file are <undecided: Q-38>

#### Scenario: Exact whole-name search
- **WHEN** the program calls exact whole-name search
- **THEN** all records with this name are returned, ignoring case, "ё" = "е"; the query is trimmed at the edges and converted
  to NFC, like a name on save (order and pagination are as for word search, ADR 0031, owner decision, OD-5, D-9)

#### Scenario: Exact search «елка»
- **GIVEN** a record «Ёлка»
- **WHEN** an exact whole-name search «елка» (or «ЁЛКА»)
- **THEN** the record is found

#### Scenario: Empty exact-search query
- **WHEN** the exact-search query is empty or consists only of spaces
- **THEN** an error

#### Scenario: Old token after a restart
- **WHEN** the server is restarted and the program sends a request with the old token
- **THEN** an "unauthorized" refusal (the token is not presented — the code and form are <undecided: Q-50>)

#### Scenario: Unknown token
- **WHEN** a request presents a token unknown to the server (wrong or arbitrary)
- **THEN** an "unauthorized" refusal and the data is unchanged (the code and form are <undecided: Q-50>)

#### Scenario: Retry after re-reading the token
- **WHEN** the server was restarted in a planned way, the program re-read the token file and repeated the request (a search;
  a rename or delete — with the number version read before the planned restart, which remains valid)
- **THEN** success

#### Scenario: Restart after a failure
- **WHEN** the server is restarted after a failure and the program repeats a rename or delete with a number version read
  before the failure
- **THEN** a conflict and the data is unchanged (for any numbers; the bounds of the guarantee are X-5, D-17, D-13); the
  program re-reads the number history and repeats with the current version — success

#### Scenario: Name or version conflict
- **WHEN** a name or version conflict occurs
- **THEN** the behavior is the same as in CUJ9

#### Scenario: A file token presented not from loopback
- **WHEN** the token from the file is presented from other than loopback (from another machine)
- **THEN** the request is rejected and the data is unchanged (the refusal code is <undecided: Q-50>)

#### Scenario: The request limit does not apply
- **WHEN** the local program sends requests at any rate
- **THEN** there is no refusal by the per-token request limit

#### Scenario: API version in the path
- **WHEN** a client calls a v1 path
- **THEN** the request is served (a call without a version in the path is <undecided: Q-51>)

#### Scenario: The token file is unavailable
- **WHEN** the token file is unavailable or not yet updated after start
- **THEN** the behavior is <undecided: Q-38>

#### Scenario: Connection load
- **WHEN** the number of connections, the request body size or the request time exceed the server limits
- **THEN** the behavior is <undecided: Q-52> (ADR 0019)

#### Scenario: Storage state on an API call
- **WHEN** the database is read-only or there is a storage error
- **THEN** CUJ16 applies

### Requirement: CUJ8: A network service only searches
A network service on another machine SHALL connect over HTTPS with its own permanent token with the read permission and
search by number, by name words and by whole name; an attempt to add, rename or delete a record SHALL receive a "no
permission" refusal; requests over the limit SHALL receive the refusal «слишком много запросов, повторите позже» (too many
requests, try again later), other clients do not notice this. Decisions: ADR 0007, 0008, 0011, 0012, 0019, 0031.

#### Scenario: Three kinds of search with a read token
- **WHEN** a token with the "read" permission performs a search by number, by name words and by whole name
- **THEN** the HTTPS connection is served by the server process itself (TLS in the server); the results are the same as for
  a local program (CUJ7) for the same queries (order and pagination of the whole-name search — ADR 0031, owner decision, OD-5,
  D-9; latency of a multi-word search — <undecided: Q-59>)

#### Scenario: A change is forbidden to a read token
- **WHEN** a "read" token calls add, rename or delete
- **THEN** a "no permission" refusal and the data did not change

#### Scenario: Limit exceeded
- **WHEN** a token sends requests more often than its limit (the value and model are <undecided: Q-46>)
- **THEN** the extra requests receive the refusal «слишком много запросов, повторите позже»

#### Scenario: Other clients do not notice another's limit
- **WHEN** one service exceeded the limit
- **THEN** the requests of other clients are served without refusals because of another's limit and their latencies stay
  within the general targets (the targets are <undecided: Q-47>, <undecided: Q-58>, <undecided: Q-59>)

#### Scenario: A connection not over HTTPS
- **WHEN** a network client connects not over HTTPS, or a network token is presented not over HTTPS
- **THEN** the connection is not served and the token is not accepted (the form of the refusal is <undecided: Q-45>)

#### Scenario: The network listener is off by default
- **WHEN** the network listener is not enabled by an operator setting
- **THEN** the server does not accept connections from other machines

#### Scenario: Network listener enabled without a certificate
- **WHEN** the network listener is enabled and there is no certificate
- **THEN** the server does not start and reports the reason (the reporting channel is <undecided: Q-66>; the certificate
  source is <undecided: Q-69>)

#### Scenario: A network token on loopback
- **WHEN** the network listener is enabled and a network token is presented from loopback over HTTPS
- **THEN** it is accepted (with the network listener off — <undecided: Q-45>)

#### Scenario: Unknown or revoked token
- **WHEN** the token is unknown or revoked
- **THEN** an unknown token gets "unauthorized"; a revoked one — a refusal (code <undecided: Q-50>; the form of the error
  body <undecided: Q-49>)

#### Scenario: The limit applies only to network tokens
- **WHEN** the limit is exhausted
- **THEN** a refusal as when the limit is exceeded; the limit applies only to network tokens, not to local programs and the UI

#### Scenario: Empty query
- **WHEN** a word query or a whole-name query is empty (for a word query — also a query of a single punctuation mark)
- **THEN** an error

#### Scenario: Number of invalid format
- **WHEN** a query by number contains a number of invalid format
- **THEN** a format error

#### Scenario: Number not found
- **WHEN** a query by number contains a number of valid format that is not in the database
- **THEN** an empty list, not an error

#### Scenario: The certificate is not accepted by the client
- **WHEN** the client does not accept the server certificate
- **THEN** the behavior is <undecided: Q-69>

#### Scenario: Connection load
- **WHEN** the number of connections, the request body size or the request time exceed the server limits
- **THEN** the behavior is <undecided: Q-52> (ADR 0019)

### Requirement: CUJ9: A network service changes data
A network service with the write permission SHALL add a number; if the number already has a different name on this date, it
gets a conflict with the current name and the number version. It SHALL send rename and delete with the number version; on a
stale version it gets a conflict with the current version, re-reads the number history and repeats.
Decisions: ADR 0002, 0003, 0004, 0005, 0006, 0012, 0019, 0022, 0025.

#### Scenario: Adding a record
- **WHEN** a service with a "read and write" token adds a record on (number, date) with no record (the date is optional on
  add — 1876-03-07 by default; the date format is <undecided: Q-67>)
- **THEN** the record is created

#### Scenario: Name conflict on add
- **WHEN** the (number, date) being added already has a different name
- **THEN** a conflict, the response contains the current name and the number version, the record is unchanged

#### Scenario: Retrying an add of an identical record
- **WHEN** a service repeats the add of an identical record (for example, after a connection drop)
- **THEN** success, the data and the version are unchanged

#### Scenario: Rename or delete without a date
- **WHEN** a rename or delete is sent without a date
- **THEN** a value error (date, "missing"), the data is unchanged; there is no substitution of 1876-03-07 (owner decision,
  OD-5, Q-90; ADR 0025 item 6); the form of the refusal is <undecided: Q-49>

#### Scenario: Rename without an expected version
- **WHEN** a rename or delete is sent without an expected version
- **THEN** the operation is not performed (the form of the error is <undecided: Q-49>, <undecided: Q-50>)

#### Scenario: Stale version with an existing record
- **WHEN** the version in a rename or delete is stale and the record (number, date) exists
- **THEN** a conflict, the response contains the current version, the data is unchanged (the source of the current version
  is <undecided: Q-34>; the content of successful responses is <undecided: Q-32>)

#### Scenario: Retry with the current version
- **WHEN** a service repeated a rename with the current version, the record exists and the new name differs from the current
  one (exact spelling)
- **THEN** success and the number version changed

#### Scenario: Renaming a missing record
- **WHEN** a record that does not exist is renamed and the current number version is passed
- **THEN** "not found"; with a stale version — "version conflict" (check order — ADR 0025, owner decision, OD-5, D-6)

#### Scenario: Deleting a missing record
- **WHEN** a record that does not exist is deleted and the current number version is passed
- **THEN** "not found" (with a stale version — "version conflict", ADR 0025 item 2, owner decision, OD-5, D-6)

#### Scenario: A rename to the same name with the current version
- **WHEN** a rename is sent with the current number version and a name equal to the current one (exact spelling after space
  trimming and NFC)
- **THEN** a silent success with no change, the number version does not change, as on a retried add (the content of the
  response is <undecided: Q-32>)

#### Scenario: A rename to the same name with a stale version
- **WHEN** a rename to the same name is sent with a stale number version
- **THEN** a conflict with the current version and the data is unchanged (the version check is first)

#### Scenario: Read token and the limit
- **WHEN** the request is made by a "read" token, or the service exceeded the request limit
- **THEN** "no permission" (CUJ8) or a limit refusal (the value is <undecided: Q-46>)

#### Scenario: Retry after a connection drop
- **WHEN** a rename or delete is repeated after a connection drop
- **THEN** a retry with the previous version is a "version conflict" (ADR 0026, owner decision, OD-5, D-7;
  `write-operations`, "Retrying an operation after an unknown outcome"); the representation in the API is <undecided: Q-54>,
  <undecided: Q-32>

#### Scenario: Invalid name, number, date
- **WHEN** the name is empty or only spaces, or longer than allowed (the maximum is per `records`, "Text length and length
  cap" (owner decision, OD-5, D-3, ADR 0022)); or the number has an invalid format; or the date is not a calendar day,
  earlier than 1876-03-07 or (in a rename and delete) not passed
- **THEN** an error, the data is unchanged (the form of the refusal for the date is <undecided: Q-49>; values with a time or
  zone — <undecided: Q-67>)

### Requirement: CUJ12: The record was changed concurrently
If a number's record was changed concurrently (in another tab or via the API), and the user pressed "delete" or confirmed
an overwrite (CUJ3), the action SHALL NOT be performed; the UI SHALL report «данные устарели, список обновлён» (data is
stale, the list was refreshed), re-read the number history and offer to repeat the action on the current data; the name
entered in the form is kept.
Open: Q-56 (visibility of intermediate states), Q-54 (representation of a retry after a connection drop in the API; the
meaning is ADR 0026), Q-63 (undoing a delete).
Decisions: ADR 0002, 0006, 0019, 0025.

#### Scenario: Stale delete
- **WHEN** the number version changed after the UI received the version, the target record exists and the user confirms the
  delete
- **THEN** the record is not deleted and the UI reports «данные устарели, список обновлён» (the exact wording is
  <undecided: Q-65>)

#### Scenario: Stale overwrite
- **WHEN** the version sent by the UI on the overwrite confirmation (the source of the version is <undecided: Q-34>) does
  not match the current one
- **THEN** the record is unchanged and the same message is shown

#### Scenario: Re-reading and an offer to retry
- **WHEN** a conflict is issued
- **THEN** the UI re-read the number history, shows the current data and offers to repeat the action on the current data
  (the content of the dialog is <undecided: Q-37>; for an action from the by-name results, what is re-read is <undecided:
  Q-36>)

#### Scenario: The name in the form is kept
- **WHEN** a conflict is issued on an overwrite
- **THEN** the name entered in the form is kept

#### Scenario: Retry on the current data
- **WHEN** after a conflict the user repeats the action on the current data
- **THEN** the action is performed if the version matches

#### Scenario: Another date of the same number was changed
- **WHEN** only another date of the same number was changed
- **THEN** the action is still rejected as stale (the version belongs strictly to the number)

#### Scenario: The record was deleted concurrently
- **WHEN** the record the user acted on was deleted concurrently
- **THEN** the outcome is a "version conflict" with the current version (ADR 0025 item 2, owner decision, OD-5, D-6); what
  the user sees is <undecided: Q-68>

#### Scenario: The name already equals the entered one with a stale version
- **WHEN** the record exists, its name already equals the entered one, and the number version is stale
- **THEN** a conflict with the current version (the version check is first, OD-4, item 7); what the user sees is
  <undecided: Q-68>

#### Scenario: The user does not repeat the action
- **WHEN** the user does not repeat the action
- **THEN** the data does not change

### Requirement: CUJ14: Data survived a failure
A save of a number acknowledged to the user SHALL survive a process crash or power loss: after the server restarts the
number is in place; a save that was not acknowledged is either fully present or fully absent. If at start the database is
inconsistent, the server does not start and reports to the operator; other corruption is a "storage error" on read (OD-4,
item 3); data is not silently lost; the operator restores the database from a backup (CUJ15). Decisions: ADR 0006, 0013,
0014, 0016, 0027, 0028 (caveats of X-5: owner decision, OD-5, D-17, D-13).

#### Scenario: An acknowledged save survives a failure
- **WHEN** a user or client received an acknowledgement of a save (an add), the process crashed or power was lost and the
  server is restarted (the moment of acknowledgement is <undecided: Q-57>)
- **THEN** the acknowledged record is present

#### Scenario: An unacknowledged save
- **WHEN** a save (an add) did not receive an acknowledgement and there was a failure
- **THEN** after restart the record is either fully present or fully absent

#### Scenario: Acknowledged rename and delete
- **WHEN** a rename (CUJ3) or delete (CUJ6) was acknowledged, there was a failure and the server is restarted
- **THEN** the acknowledged change is kept and a deleted record does not "come back to life"

#### Scenario: Unacknowledged rename and delete
- **WHEN** a rename or delete did not receive an acknowledgement and there was a failure
- **THEN** after restart the change is either fully present or fully absent (the record and the number version change are
  indivisible, X-5)

#### Scenario: Bounds of the guarantee
- **WHEN** a process crash, an OS failure or a power loss occurred and the disk honestly performs a flush to stable storage
- **THEN** acknowledged adds and renames are present and records acknowledged as deleted are absent
- **AND** a "lying" disk and medium corruption are outside the guarantee, detecting a "lying" disk is not promised; a disk
  write error is a "storage error" (CUJ16)

#### Scenario: Check at start
- **WHEN** at start the check (structure and the latest changes) shows that the database does not open consistent
- **THEN** the server does not start, the operator receives a message (the channel is <undecided: Q-66>) and records are not
  silently discarded (the scope of the check is <undecided: Q-43>; the start time on 10^8 records is <undecided: Q-42>)

#### Scenario: Corruption found on read
- **WHEN** corruption of the rest of the data (not found by the start check) is detected on reading it or on a running server
- **THEN** the server goes into the "storage error" state: changes are rejected (CUJ16), search works while it can, a message
  is issued to the operator

#### Scenario: Restore with a damaged database
- **WHEN** the database is damaged and the operator starts the server from a backup
- **THEN** the server starts on the backup data (including on a backup made before an update: the new program opens such a
  database itself; there is no rollback)

#### Scenario: A version read before a failure
- **WHEN** a client sends a rename or delete with a number version read before a failure, and the server was restarted
  after the failure
- **THEN** a conflict and the data is unchanged; the client re-reads; a conflict is acceptable for any number (the mechanics
  are open question Q-78, docs/open-questions.md); the bounds of the guarantee between version generations are per X-5
  (owner decision, OD-5, D-17; the exception is files copied not by the command, D-13)

#### Scenario: Planned restart
- **WHEN** the server is restarted in a planned way (a clean shutdown: defined in `write-operations`, "Clean and crash
  shutdown", ADR 0028 item 10) and a client sends a rename or delete with a number version read before the shutdown
- **THEN** the version is valid: the action is performed if the version matches

### Requirement: CUJ15: Backup and restore
The operator SHALL make a database backup with a command on the server machine while the server is running: users and
programs keep working without interruption, the backup corresponds to one consistent moment. Restore is starting the server
on the backup. Decisions: ADR 0006, 0012, 0013, 0016, 0018, 0027, 0028 (caveats of X-5: owner decision, OD-5, D-17, D-13).

#### Scenario: Copying while the server is running
- **WHEN** the operator starts the copy (parameters are <undecided: Q-41>; who performs the backup and the command channel
  are <undecided: Q-60>)
- **THEN** the server is not stopped and UI and API requests are handled

#### Scenario: No refusals, latency growth within bounds
- **WHEN** copying is in progress
- **THEN** requests are handled without refusals and the latency growth is within <undecided: Q-58>, <undecided: Q-59>

#### Scenario: A consistent cut
- **WHEN** the backup is ready
- **THEN** it matches the database state at some moment: every change is either fully present or fully absent, there are no
  partial changes (which exact moment the backup fixes is open question Q-83, docs/open-questions.md; the notions "moment"
  and "acknowledged" are Q-56, Q-57)

#### Scenario: Starting the server on a backup
- **WHEN** the server is started on a backup
- **THEN** searches return the data of the backup moment

#### Scenario: Versions after a restore
- **WHEN** the server is started on a backup and a client sends a rename or delete with a number version read before the
  restore
- **THEN** a conflict and the data is unchanged; mass conflicts right after a restore are acceptable, for any numbers (the
  mechanics are open question Q-78, docs/open-questions.md); the bounds of the guarantee between version generations are per
  X-5 (owner decision, OD-5, D-17; the exception is files copied not by the command, D-13)

#### Scenario: Tokens and the backup
- **WHEN** a backup is made
- **THEN** tokens are not included in the backup; a server started on the backup with the same settings next to which the
  tokens are kept accepts the same tokens, and a revoked one is still rejected; when moving to another machine the operator
  carries the tokens over together with the settings, otherwise reissues them

#### Scenario: A backup of an earlier v1 version
- **WHEN** the backup was made by an earlier v1 version and the server on the backup is a newer version
- **THEN** the new program opens such a database itself, without a command; the earlier program is not obliged to open a
  database after the new one ("no rollback")

#### Scenario: Copying with the target space occupied or a write error
- **WHEN** the target place is occupied, there is no space or a backup write error
- **THEN** the behavior is <undecided: Q-41>

#### Scenario: Copying from a read-only database or one with a storage error
- **WHEN** the source database is read-only or there is a storage error
- **THEN** the behavior is <undecided: Q-71>

#### Scenario: The server is unavailable to the backup command
- **WHEN** the server is stopped or unavailable to the backup command
- **THEN** the behavior is <undecided: Q-60>

### Requirement: X-5: Number version
The system SHALL give any change to a number's records after which records exist a version different from all previously
received for this number: deterministically within a generation (between crash restarts and starts from a backup) and
between generations on an installation with preserved server settings; without them — with a coincidence probability no
higher than 2^-64 per event (owner decision, OD-5, D-17); the exception is a directory copied otherwise (D-13). A number
with no records has the initial version (one per generation); the record and the version are indivisible. Trace: owner
decision "Operations: versioning belongs to the number"; CUJ12; OD-3, Q-77 (`docs/owner-decisions.md`). Open: Q-32, Q-34,
Q-78. Decisions: ADR 0006, 0025, 0027 (item 4 refines item 6(a) of ADR 0006), 0028 (items 7, 9, 10).

#### Scenario: The version is not repeated
- **WHEN** a number that had records has all its records deleted, the number is added again and a client sends a rename or
  delete with a version received before the delete
- **THEN** the action is not performed and the data is unchanged; the kind of refusal is "version conflict" (the version is
  checked before the record's existence, ADR 0025)

#### Scenario: A number with no records
- **WHEN** a number has no records (it is at the initial version) and a client sends an action with this version for a
  missing record
- **THEN** the action is not performed and the data is unchanged; the kind of refusal is "not found" (ADR 0025)

#### Scenario: A number returns to the no-records state
- **WHEN** the last record of a number is deleted
- **THEN** the number version is the initial version of the current generation, one for all numbers with no records; the
  guarantee "different from all previously received" applies to a number with records

#### Scenario: A directory copied not by the command
- **WHEN** the directory of a stopped database is copied by hand, the source database kept working, then the copy is started
- **THEN** the guarantee "different from all previously received" does not apply on the copy: a change by a version issued by
  the source database is accepted if and only if the current version on the copy equals it; the exception is an owner
  decision (OD-5, D-13), a backup created by the command does not have it; the scenario is verified by a review of the
  mechanism, not by a test

#### Scenario: Record and version are indivisible
- **WHEN** a failure happens between the write and the number version change
- **THEN** there is no partial result: there is no state where the record is changed and the version is not

#### Scenario: After a restore or a failure
- **WHEN** a number is restored from a backup or the server is restarted after a failure, and a client sends a rename or
  delete with a version read before the restore or before the failure
- **THEN** a conflict and the data is unchanged (for any numbers; the bounds of the guarantee between generations — D-17:
  deterministically on an installation with preserved settings, otherwise with probability not lower than 1 − 2^-64 per
  event)

#### Scenario: After a planned restart
- **WHEN** the server is stopped cleanly (definition — `write-operations`, "Clean and crash shutdown", ADR 0028 item 10) and
  started, and a client sends a rename or delete with a version read before the shutdown
- **THEN** the version is valid: the action is performed if the version matches
