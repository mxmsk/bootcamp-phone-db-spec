## MODIFIED Requirements

### Requirement: CUJ1: Adding a new number
The system SHALL let a user add a new phone number entered in international format with
+<country code>: the number is saved and is found by name, and the name is found by number.
Decisions: ADR 0002, 0003, 0004, 0005, 0006, 0007, 0008, 0019, 0020–0024, 0025, 0027, 0028.

#### Scenario: Saving an unknown number
- **WHEN** the user enters an unknown number "+<country code>…" and a valid name in the form and submits an add
  request (create only)
- **THEN** the record is created, the number has a version; its properties (how it differs from previously issued ones, the limits of the guarantee) are defined by X-5
- **AND** the UI reports a successful save, «Сохранено: {имя}, {номер}, с {дата}» (D-83)

#### Scenario: The name is found by word-prefix search
- **WHEN** after saving, a word-prefix search by the name is performed
- **THEN** the record with this number is reachable in the results (if there are more results than a page, by paging "next")

#### Scenario: The name is found by number
- **WHEN** after saving, a search by this number is performed (exact match)
- **THEN** the results contain a record with this name

#### Scenario: Date not provided
- **WHEN** the add request has no date and the number has no records (CUJ1 does not ask for a date; whether the form has an optional date field —
  no, the field appears when the number is known, D-82)
- **THEN** the record is stored with the date 1876-03-07 (the substitution applies only on add: for rename and delete the date
  is mandatory, owner decision OD-5, Q-90, ADR 0025 item 6)

#### Scenario: Adding without a date for a known number
- **WHEN** the add request has no date and the number already has a record on any date
- **THEN** no record is created, the server responds with the "number known" conflict with the current number version (owner decision, OD-6, Q-160); the flow moves to CUJ2

#### Scenario: Number without a plus
- **WHEN** the number is entered without "+"
- **THEN** it is rejected with a hint and no record is created (the hint text in the UI — D-81, D-93; the hint's expression
  for an API client — the error body `{code, field, reason, hint, message}` (ADR 0062))

#### Scenario: Number of invalid shape
- **WHEN** after "+" there are fewer than 1 or more than 15 digits, or the first digit is 0 (owner decision, OD-5, D-1, ADR 0004), or there are
  characters other than digits and removable separators
- **THEN** the number is rejected and no record is created

#### Scenario: Number with separators
- **WHEN** the number is entered with spaces, hyphens, parentheses or dots (for example, "+7 (999) 123-45-67")
- **THEN** the record is created
- **AND** a search by the same number written differently (for example, "+7 999 123 45 67") finds the same record

#### Scenario: Country code not assigned
- **WHEN** the number has a valid shape ("+", 1–15 digits, first digit 1–9 — owner decision, OD-5, D-1, ADR 0004), but its country code is not assigned
- **THEN** the number is accepted: there is no check against a list of country codes (owner decision, OD-5, D-1, ADR 0004)

#### Scenario: Name longer than allowed
- **WHEN** the name is longer than allowed (the maximum is per `records`, "Text length and length cap" (owner decision, OD-5, D-3, ADR 0022))
- **THEN** an error, no record is created, the name is not truncated

#### Scenario: Empty name
- **WHEN** the name is empty or consists only of spaces
- **THEN** an error and no record is created

#### Scenario: Name normalization
- **WHEN** the name is entered with spaces at the edges or in a non-normalized form
- **THEN** the record is stored with the name trimmed at the edges and brought to NFC

#### Scenario: Retry of an identical record
- **WHEN** an identical record with a date is added again (for example, after a connection drop)
- **THEN** success, the data and the number version are unchanged

#### Scenario: Retry of an add without a date
- **WHEN** an add without a date completed, the response was lost, and the request was repeated
- **THEN** "number known" (409 `number_known`, `write-operations`, "Retrying an operation after an unknown outcome"): the record was created by the first attempt; after an unknown outcome the UI does not retry automatically, the user finds out the outcome with a number search (CUJ4)

#### Scenario: The number turned out to be already known
- **WHEN** the number is already in the database
- **THEN** the flow moves to CUJ2 (how the UI determines this — by the server response `409 number_known` (ADR 0090))

#### Scenario: Change impossible or UI with an old token
- **WHEN** the database is read-only or has a storage error, or the UI is open with an old token
- **THEN** CUJ16 (database state) or CUJ13 (old link) applies

#### Scenario: Write acknowledgement latency
- **WHEN** an add is performed on a database of 10^7 records
- **THEN** the acknowledgement is received within 100 ms (p99; D-27); the criterion applies to all write operations
  (CUJ1–CUJ3, CUJ6, CUJ9) and is repeated in X-1

### Requirement: CUJ2: Adding a known number with a new date
When saving a known number, the system SHALL show that the number already exists and ask for the date from which the
number is assigned to the user; the number is saved and from then on a search by number returns the list of all its
owners by date. Decisions: ADR 0002, 0003, 0004, 0005, 0006, 0007, 0019, 0020–0025.

#### Scenario: Date prompt for a known number
- **WHEN** the user saves a number that is already in the database
- **THEN** the server answers an add without a date with the "number known" conflict (owner decision, OD-6, Q-160), the UI reports that the number already exists and asks for a date; the date input format is YYYY-MM-DD (D-78), the dialog content — ADR 0091

#### Scenario: The CUJ1/CUJ2 race is closed by the server
- **WHEN** the number became known between opening the form and an add without a date
- **THEN** no record on 1876-03-07 is created: the server answers "number known", then CUJ2 (owner decision, OD-6, Q-160)

#### Scenario: New date for a known number
- **WHEN** the user enters a valid date on which the number has no record
- **THEN** the record (number, date) is created, the number version changed (X-5)
- **AND** records with other dates are unchanged
- **AND** the UI reports success «Сохранено: {имя}, {номер}, с {дата}» (D-83)

#### Scenario: Number search results after a new date
- **WHEN** after the add, a number search is performed
- **THEN** the results contain all the number's records (old and new), sorted by date descending, without pagination
  (past, current and future owners are not distinguished in display (D-87))

#### Scenario: Date earlier than allowed
- **WHEN** the date 1876-03-06 is entered
- **THEN** no record is created (the refusal kind — UI: ADR 0088 item 6; API: ADR 0062)

#### Scenario: Nonexistent calendar date
- **WHEN** a nonexistent calendar date is entered (for example, 30 February)
- **THEN** no record is created (the refusal form — UI: ADR 0088 item 6; API: ADR 0062); values with a time or
  time zone: in the UI — format YYYY-MM-DD, any other input is a refusal (D-78), in the API — anything else is `invalid_input`, `invalid_format` (ADR 0060 item 5)

#### Scenario: Future date
- **WHEN** a future date is entered (there is no product upper bound; the technical limit is 9999-12-31, `records`, "Valid date range", owner decision, OD-5, D-5, ADR 0024)
- **THEN** the record is created

#### Scenario: The same name on a different date
- **WHEN** the number has a record with name N on date D1 and a record with the same name N is added on another valid date D2
- **THEN** a separate record (number, D2) is created, the record (number, D1) is unchanged

#### Scenario: The same record on the same date
- **WHEN** on the entered date there is already a record with exactly the same name, or the name differs from the stored one only
  by edge whitespace or normalization form (NFC)
- **THEN** the operation succeeds, the data and the number version are unchanged

#### Scenario: Name differs only in case
- **WHEN** on the same date the name matches the stored one only case-insensitively ("Ann" against "ANN")
- **THEN** it is a conflict (move to CUJ3), not a silent success

#### Scenario: A date on which the number already has a different name
- **WHEN** on the entered date the number already has a record with a different name (including a different spelling)
- **THEN** the flow moves to CUJ3

#### Scenario: Date 1876-03-07
- **WHEN** the date 1876-03-07 is entered
- **THEN** it is the same record that would result from the server substituting the zero date (how an explicit date differs from a
  substituted one in the results — it is shown as an ordinary date, D-86)

#### Scenario: The user does not enter a date
- **WHEN** the user does not enter a date or declines to enter one
- **THEN** the behavior is defined by ADR 0091 and ADR 0090

#### Scenario: Invalid name or number
- **WHEN** the name is empty, only spaces, or longer than allowed (the maximum is per `records`, "Text length and length cap" (owner decision, OD-5, D-3, ADR 0022)), or the number has an invalid
  shape
- **THEN** no record is created and the data is unchanged

#### Scenario: Database state on an add with a date
- **WHEN** the database is read-only or has a storage error
- **THEN** CUJ16 applies

### Requirement: CUJ3: Adding a known number with an existing date
When adding a known number with a date that already exists, the system SHALL show an overwrite prompt: if yes —
the data changes, the old is completely erased; if no — the system waits for new input. Decisions: ADR 0002, 0003, 0005, 0006, 0020–0025.

#### Scenario: Conflict on add
- **WHEN** a record is added on (number, date) where there is already a different name
- **THEN** the server answers with a conflict carrying the current name and the number version, the record is unchanged
- **AND** the UI shows an overwrite prompt (content — ADR 0091)

#### Scenario: Overwrite confirmation
- **WHEN** the user answers "yes" and the number version sent by the UI is current (the version source — from the results row or the conflict response (ADR 0060 item 6))
- **THEN** the UI calls the explicit rename operation with the expected version and the record's date (the date is mandatory, OD-5, Q-90; overwrite is offered only for an add with a date, an add without a date gives "number known", OD-6, Q-160), the name on (number, date) is replaced by the entered one, the number version changed (X-5)
- **AND** the earlier record (number, date) with the earlier name is not returned by any search
- **AND** the UI reports success «Сохранено: {имя}, {номер}, с {дата}» (D-83)

#### Scenario: Declining the overwrite
- **WHEN** the user answers "no"
- **THEN** the data is unchanged and the UI waits for new input (what is kept in the form — ADR 0091)

#### Scenario: The number's other dates are unaffected
- **WHEN** the overwrite is performed
- **THEN** the number's records on other dates are unchanged

#### Scenario: Version is stale
- **WHEN** after "yes" the number version sent by the UI does not match the current one
- **THEN** the record is unchanged, CUJ12 applies

#### Scenario: The record vanished between the conflict and the confirmation
- **WHEN** the record (number, date) vanished between the conflict and the overwrite confirmation
- **THEN** it is a special case of a stale version (a delete changes the number version); the outcome is "version conflict" with the current
  version (the version is checked before record existence: ADR 0025 item 2, owner decision, OD-5, D-6); the UI reaction is as for stale data, `unchanged` is success (D-92)

#### Scenario: The same name on an existing date
- **WHEN** the entered name matches the stored one after whitespace trimming and NFC
- **THEN** a silent success and the overwrite prompt is not shown

#### Scenario: Typo correction only via overwrite
- **WHEN** the user corrects a typo in the name of an existing record
- **THEN** it is done by an add on the same (number, date) with the corrected name and an overwrite confirmation;
  there is no other correction path in the UI (how the UI lets the user enter the date of an existing record — by the server response `409 number_known` (ADR 0090))

#### Scenario: No "rename" button
- **WHEN** a row is shown in the results
- **THEN** it has no "rename" button

#### Scenario: Invalid new name
- **WHEN** the new name is empty, only spaces, or longer than allowed (the maximum is per `records`, "Text length and length cap" (owner decision, OD-5, D-3, ADR 0022))
- **THEN** an error and the record is unchanged

#### Scenario: Renaming a nonexistent record on the UI path
- **WHEN** the CUJ3 path is executed without a concurrent change
- **THEN** renaming a nonexistent record is unreachable; the scenario for the API is in CUJ9

#### Scenario: Database state on overwrite
- **WHEN** the database is read-only or has a storage error
- **THEN** CUJ16 applies

### Requirement: CUJ4: Search by number
The system SHALL let the user search by phone number: a search by number returns only an exact match and shows a list
of everyone the number was added for, sorted by date DESC. Decisions: ADR 0002, 0004, 0007, 0019.

#### Scenario: Full list of the number's records
- **WHEN** a full number with N records is entered
- **THEN** the results have exactly N records without pagination (row contents — trash can, name, number, date (D-84); the number's form in a row —
  the canonical form "+" and digits (ADR 0060 item 4); past, current and future owners are not distinguished in display (D-87))

#### Scenario: Sorting by date
- **WHEN** the results have several records
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

#### Scenario: Number of invalid shape
- **WHEN** a number of invalid shape is entered (without "+", letters, etc.)
- **THEN** a format error (the same rules as on save, ADR 0004) and no results

#### Scenario: Search by a number with separators
- **WHEN** the number is saved and the search is performed by the same number written with spaces, hyphens, parentheses or
  dots
- **THEN** this number's records are found

#### Scenario: Number search latency
- **WHEN** a number search is performed on a database of 10^7 records
- **THEN** latency is at most 10 ms (p99, a number with up to 100 records, D-23)

#### Scenario: Storage state on a number search
- **WHEN** the database is read-only or has a storage error
- **THEN** CUJ16 applies: search works while it can

### Requirement: CUJ5: Search by name
The system SHALL let the user search by name by the beginning of any word of the name (not by substring; all words of the query must
match; case-insensitive, "ё" = "е") and show the results as a paginated list, sorted alphabetically, then
by date DESC. Decisions: ADR 0003, 0008, 0009, 0010, 0019, 0020–0022, 0030–0034, 0036.

#### Scenario: Word start, not a substring
- **GIVEN** the database has records with the names "Ann Smith", "John Annabel" and "Joann"
- **WHEN** the query "ann"
- **THEN** the results include "Ann Smith" and "John Annabel" and do not include "Joann"

#### Scenario: Several query words
- **WHEN** the query consists of several words
- **THEN** the results have only records where every query word matched; the order of query words does not matter ("ann smith" and
  "smith ann" find "Ann Smith")

#### Scenario: One name word covers several query words
- **GIVEN** the record "Annabel"
- **WHEN** the query "an ann"
- **THEN** the record is found

#### Scenario: Ё equals Е, case does not matter
- **GIVEN** the database has the records "Елка" and "Ёлка"
- **WHEN** the query "ёлка" (and conversely "елка", and "ЕЛКА")
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
  open question Q-76, docs/open-questions.md)

#### Scenario: Query in decomposed form
- **GIVEN** the record "José"
- **WHEN** the query "José" is entered in NFD form (the letter "e" and a separate accent U+0301)
- **THEN** the record is found: the query is brought to NFC

#### Scenario: Empty query
- **WHEN** the query is empty, only spaces, or only punctuation (for example, "-", "'"): it has no
  words
- **THEN** an error, no results (the whitespace character set — `records`, "Whitespace character set", ADR 0021 item 1; the reason — (query, "empty"), ADR 0034)

#### Scenario: Pagination
- **WHEN** there are more results than a page
- **THEN** a part is returned (page size — 50 by default, owner decision, OD-5, D-10, ADR 0033) and the next one can be obtained by cursor
- **AND** the total number of results is not reported

#### Scenario: Result order
- **WHEN** the results have different names
- **THEN** they go alphabetically under ru locale rules (comparison parameters — ADR 0037, 0038; owner decision, OD-5, D-11, D-12, D-15, Q-93), records with one name
  — by date DESC

#### Scenario: Order for equal name and date
- **GIVEN** two records with the same name and date for different numbers
- **WHEN** a name search is performed
- **THEN** they go by number ascending as a string: "+19…" before "+2…", numbers of one country adjacent
  (total order — ADR 0036, `result-ordering`; cursor — ADR 0032)

#### Scenario: No matches
- **WHEN** there are no matches
- **THEN** the empty results are shown as a neutral text, not an error (D-85)

#### Scenario: Cursor is not accepted
- **WHEN** the cursor is corrupted (does not decode, the checksum does not match) or belongs to a different query or search kind
- **THEN** the error "cursor invalid" (ADR 0032 item 4, ADR 0034); a data change between pages does not break the cursor:
  the next page is the records strictly after the cursor position in the data as of the request (ADR 0032 items 1–3;
  consistency between pages is not promised — owner decision, OD-5, D-8, Q-56)

#### Scenario: A server upgrade within v1 does not change search
- **WHEN** the server is upgraded within v1
- **THEN** the matches and order for the same data and queries are unchanged: Unicode 18 is fixed for all of v1
  (the mechanics — open questions Q-74, Q-82, docs/open-questions.md)

#### Scenario: Name search latency
- **WHEN** a name search (including multi-word) is performed on 10^7 records
- **THEN** latency is at most the D-23 targets by class; for multi-word search, heavy intersections — at most 2 s

#### Scenario: Storage state on a name search
- **WHEN** the database is read-only or has a storage error
- **THEN** CUJ16 applies

### Requirement: CUJ6: Deleting a record
The system SHALL let the user delete a record about a number: the user searches by number or name; in the found list
a "delete" button with a trash can icon is shown on the left; confirmation is requested before deleting; after deleting the
list is re-queried and shown updated. Decisions: ADR 0005, 0006, 0007, 0025.

#### Scenario: Delete button in a row
- **WHEN** the results list (by number or by name) has a row
- **THEN** on the left of it there is a "delete" button with a trash can icon

#### Scenario: Confirmation request
- **WHEN** the user presses "delete"
- **THEN** confirmation is requested and until it is given the record is not deleted

#### Scenario: Confirmed delete
- **WHEN** the user confirms and the number version is current (the version source — from the results row or the conflict response (ADR 0060 item 6))
- **THEN** exactly one record (number, date) is deleted, the number version changed (X-5), the number's other records are intact; the UI passes
  the date of the results row explicitly (the date is mandatory in a delete, OD-5, Q-90)

#### Scenario: Updated list after a delete
- **WHEN** the delete is performed
- **THEN** the UI re-queries the data and shows the list without the deleted record (which query and page —
  the same query, as many pages as are shown (D-90); the empty list view — a neutral text, not an error (D-85))

#### Scenario: Declining confirmation
- **WHEN** the user declines in the confirmation dialog
- **THEN** the data is unchanged (what the UI shows after declining — the dialog closes, nothing is deleted (D-89))

#### Scenario: Stale version on delete
- **WHEN** the number version in the delete request does not match the current one and the record (number, date) exists
- **THEN** the record is not deleted and the response contains the current version; CUJ12 applies (for a missing record with
  a stale version — "version conflict", ADR 0025 item 2, owner decision, OD-5, D-6)

#### Scenario: Deleting a missing record
- **WHEN** a record that does not exist is deleted and the current number version is passed
- **THEN** "not found" (with a stale version — "version conflict" with the current version, ADR 0025 item 2, owner decision, OD-5, D-6); the UI reaction is as for stale data, `unchanged` is success (D-92)

#### Scenario: Undoing a completed delete
- **WHEN** undoing an already completed delete (undo) is needed
- **THEN** it is not provided for by the CUJ6 text; whether it is needed is not introduced, undo is declining in the dialog (D-89)

#### Scenario: Database state on a delete
- **WHEN** the database is read-only or has a storage error
- **THEN** CUJ16 applies

### Requirement: CUJ7: A local program works with the database
A local program on the same machine SHALL read the token from a file and perform the same operations as the UI, plus exact
whole-name search; after a server restart its request with the old token SHALL get the refusal "unauthorized",
and after re-reading the token file the retry succeeds (a rename and delete with an old version after a crash is a conflict,
OD-4, item 5; with X-5 caveats: owner decision, OD-5, D-17, D-13). Decided: there is no TLS on loopback (D-49, Q-70).
Decisions: ADR 0005, 0006, 0007, 0008, 0011, 0012, 0013, 0019, 0027, 0028, 0031.

#### Scenario: The same operations as in the UI
- **WHEN** a local program with a valid token calls add, rename, delete, number search,
  word-prefix name search or exact whole-name search
- **THEN** the number and name validation rules, conflicts and versions are the same as when called from the UI; there are no UI-level differences
  (date prompt, overwrite dialog, delete confirmation) in the API; on an add without a date the server substitutes 1876-03-07 if the number has no records (otherwise "number known", OD-6, Q-160), and a rename and delete without a date is an error (date, "missing"; OD-5, Q-90)
- **AND** the local program's permissions are full; the location and access rights of the token file — the file `local.token`, permissions 0600 (ADR 0065 items 1–2)

#### Scenario: Exact whole-name search
- **WHEN** the program calls exact whole-name search
- **THEN** all records with this name are returned, case-insensitive, "ё" = "е"; the query is trimmed at the edges and
  brought to NFC, like a name on save (order and pagination — as for word search, ADR 0031, owner decision, OD-5, D-9)

#### Scenario: Exact search "елка"
- **GIVEN** the record "Ёлка"
- **WHEN** exact whole-name search "елка" (or "ЁЛКА")
- **THEN** the record is found

#### Scenario: Empty exact-search query
- **WHEN** the exact-search query is empty or consists only of spaces
- **THEN** an error

#### Scenario: Old token after a restart
- **WHEN** the server is restarted and the program sends a request with the old token
- **THEN** refusal "unauthorized" (token not presented — code and shape 401 `unauthorized` (ADR 0062 item 2))

#### Scenario: Unknown token
- **WHEN** a request presents a token unknown to the server (invalid or arbitrary)
- **THEN** refusal "unauthorized" and the data is unchanged (code and shape — 401 `unauthorized` (ADR 0062 item 2))

#### Scenario: Retry after re-reading the token
- **WHEN** the server is restarted on a plan, the program re-read the token file and repeated the request (a search; a rename or
  delete — with a number version read before the planned restart, which remains valid)
- **THEN** success

#### Scenario: Restart after a crash
- **WHEN** the server is restarted after a crash and the program repeats a rename or delete with a number version read before
  the crash
- **THEN** a conflict and the data is unchanged (for any numbers; the limits of the guarantee — X-5, D-17, D-13); the program
  re-reads the number's history and repeats with the current version — success

#### Scenario: Name or version conflict
- **WHEN** a name or version conflict occurs
- **THEN** the behavior is the same as in CUJ9

#### Scenario: File token presented not from loopback
- **WHEN** the token from the file is presented from other than loopback (from another machine)
- **THEN** the request is rejected and the data is unchanged (the refusal code — 401 `unauthorized` (ADR 0062 item 2))

#### Scenario: Request limit does not apply
- **WHEN** a local program sends requests at any rate
- **THEN** there is no per-token request limit refusal

#### Scenario: API version in the path
- **WHEN** a client calls a v1 path
- **THEN** the request is served (a call without a version in the path — 404 `route_not_found` (ADR 0060 item 3))

#### Scenario: Token file unavailable
- **WHEN** the token file is unavailable or not yet updated after startup
- **THEN** the behavior is defined by ADR 0065 items 1–2

#### Scenario: Connection load
- **WHEN** the number of connections, request body size or request time exceed the server limits
- **THEN** the behavior is defined by ADR 0071 (server limits: D-43, D-52, D-53)

#### Scenario: Storage state on an API call
- **WHEN** the database is read-only or has a storage error
- **THEN** CUJ16 applies

### Requirement: CUJ8: A network service only searches
A network service on another machine SHALL connect over HTTPS with its own permanent token with the read permission and search by
number, by name words and by whole name; an attempt to add, rename or delete a record SHALL get the refusal "no
permissions"; requests over the limit SHALL get the refusal «слишком много запросов, повторите позже» (too many requests, retry later); other clients do not
notice it. Decisions: ADR 0007, 0008, 0011, 0012, 0019, 0031.

#### Scenario: Three kinds of search with a read token
- **WHEN** a token with the "read" permission performs a search by number, by name words and by whole name
- **THEN** the HTTPS connection is served by the server process itself (TLS in the server); the results are the same as for a local
  program (CUJ7) with the same queries (order and pagination of exact whole-name
  search — ADR 0031, owner decision, OD-5, D-9; multi-word search latency — D-23, D-25)

#### Scenario: Changes are forbidden to a read token
- **WHEN** a "read" token calls add, rename or delete
- **THEN** refusal "no permissions" and the data is unchanged

#### Scenario: Limit exceeded
- **WHEN** a token sends requests faster than its limit (the value and model — 100 requests per second, burst 100 (D-51))
- **THEN** the excess requests get the refusal «слишком много запросов, повторите позже»

#### Scenario: Other clients do not notice another's limit
- **WHEN** one service exceeded the limit
- **THEN** other clients' requests are served without refusals due to another's limit and their latencies stay within
  the general targets (targets — D-23 for search, D-27 for writes and D-25 for search (at most 2 × the D-23 target))

#### Scenario: Connection not over HTTPS
- **WHEN** a network client connects not over HTTPS, or a network token is presented not over HTTPS
- **THEN** the connection is not served and the token is not accepted (the refusal shape — D-47: only TLS on the network port)

#### Scenario: Network listener is off by default
- **WHEN** the network listener is not enabled by an operator setting
- **THEN** the server does not accept connections from other machines

#### Scenario: Network listener enabled without a certificate
- **WHEN** the network listener is enabled and there is no certificate
- **THEN** the server does not start and reports the reason (the message channel — stderr, `key=value` (D-59); the certificate source —
  the operator's PEM files (D-50))

#### Scenario: Network token on loopback
- **WHEN** the network listener is enabled and a network token is presented from loopback over HTTPS
- **THEN** it is accepted (with the network listener off — D-47: only TLS on the network port)

#### Scenario: Unknown or revoked token
- **WHEN** the token is unknown or revoked
- **THEN** an unknown token gets "unauthorized"; a revocation is a refusal (code 401 `unauthorized` (ADR 0062 item 2); the error body shape
  is the error body `{code, field, reason, hint, message}` (ADR 0062))

#### Scenario: The limit applies only to network tokens
- **WHEN** the limit is exhausted
- **THEN** a refusal as on exceeding the limit; the limit applies only to network tokens, not to local programs and the UI

#### Scenario: Empty query
- **WHEN** a word or whole-name query is empty (for word search — also a query of only punctuation)
- **THEN** an error

#### Scenario: Number of invalid shape
- **WHEN** a number query contains a number of invalid shape
- **THEN** a format error

#### Scenario: Number not found
- **WHEN** a number query contains a number of valid shape that is not in the database
- **THEN** an empty list, not an error

#### Scenario: Certificate not accepted by the client
- **WHEN** the client does not accept the server certificate
- **THEN** the behavior is defined by D-50: the certificate and key are the operator's PEM files

#### Scenario: Connection load
- **WHEN** the number of connections, request body size or request time exceed the server limits
- **THEN** the behavior is defined by ADR 0071 (server limits: D-43, D-52, D-53)

### Requirement: CUJ9: A network service changes data
A network service with the write permission SHALL add a number; if the number already has a different name on that date, it gets a
conflict with the current name and the number version. It SHALL send a rename and delete with the number version; with a stale
version it gets a conflict with the current version, re-reads the number's history and repeats.
Decisions: ADR 0002, 0003, 0004, 0005, 0006, 0012, 0019, 0022, 0025.

#### Scenario: Adding a record
- **WHEN** a service with a "read and write" token adds a record on (number, date) with no record (on an add the date is optional — by
  default 1876-03-07 if the number has no records, otherwise "number known", OD-6, Q-160; the date format is strictly `YYYY-MM-DD` (ADR 0060 item 5))
- **THEN** the record is created

#### Scenario: Name conflict on add
- **WHEN** the (number, date) being added already has a different name
- **THEN** a conflict, the response contains the current name and the number version, the record is unchanged

#### Scenario: Retry of an add of an identical record
- **WHEN** a service repeats an add of an identical record with a date (for example, after a connection drop)
- **THEN** success, the data and version are unchanged

#### Scenario: Retry of an add without a date
- **WHEN** a service repeats an add without a date whose first attempt created the record
- **THEN** "number known" with the current number version; the service finds out the outcome with a number search

#### Scenario: Rename or delete without a date
- **WHEN** a rename or delete is sent without a date
- **THEN** a value error (date, "missing"), the data is unchanged; there is no 1876-03-07 substitution (owner decision, OD-5,
  Q-90; ADR 0025 item 6); the refusal shape — the error body `{code, field, reason, hint, message}` (ADR 0062)

#### Scenario: Rename without an expected version
- **WHEN** a rename or delete is sent without an expected version
- **THEN** the operation is not performed (the error shape — ADR 0062 items 1, 2)

#### Scenario: Stale version with an existing record
- **WHEN** the version in a rename or delete is stale and the record (number, date) exists
- **THEN** a conflict, the response contains the current version, the data is unchanged (the source of the current version —
  from the results row or the conflict response (ADR 0060 item 6); the version in responses is a string of 64 lowercase hexadecimal characters (ADR 0060 item 6))

#### Scenario: Retry with the current version
- **WHEN** a service repeated a rename with the current version, the record exists and the new name differs from the current (exact
  spelling)
- **THEN** success and the number version changed

#### Scenario: Renaming a missing record
- **WHEN** a record that does not exist is renamed and the current number version is passed
- **THEN** "not found"; with a stale version — "version conflict" (check order — ADR 0025, owner decision, OD-5, D-6)

#### Scenario: Deleting a missing record
- **WHEN** a record that does not exist is deleted and the current number version is passed
- **THEN** "not found" (with a stale version — "version conflict", ADR 0025 item 2, owner decision, OD-5, D-6)

#### Scenario: Rename to the same name with the current version
- **WHEN** a rename is sent with the current number version and a name equal to the current one (exact spelling after
  whitespace trimming and NFC)
- **THEN** a silent success with no change, the number version does not change, as on a retry of an add (the version in the response is
  a string of 64 lowercase hexadecimal characters (ADR 0060 item 6))

#### Scenario: Rename to the same name with a stale version
- **WHEN** a rename to the same name is sent with a stale number version
- **THEN** a conflict with the current version and the data is unchanged (the version check is first)

#### Scenario: Read token and limit
- **WHEN** a request is made by a "read" token, or a service exceeded the request limit
- **THEN** "no permissions" (CUJ8) or a limit refusal (the value — 100 requests per second, burst 100 (D-51))

#### Scenario: Retry after a connection drop
- **WHEN** a rename or delete is repeated after a connection drop
- **THEN** a retry with the old version is a "version conflict" (ADR 0026, owner decision, OD-5, D-7; `write-operations`, "Retrying an
  operation after an unknown outcome"); the API representation — there are no idempotency keys, the outcome is found out by re-reading the number (ADR 0064 item 5); the version is a string of 64 characters (ADR 0060 item 6)

#### Scenario: Invalid name, number, date
- **WHEN** the name is empty or only spaces, or longer than allowed (the maximum is per `records`, "Text length and length cap" (owner decision, OD-5, D-3, ADR 0022)); or the number has an
  invalid shape; or the date is not a calendar day, earlier than 1876-03-07 or (in a rename and delete) not passed
- **THEN** an error, the data is unchanged (the date refusal shape — the error body `{code, field, reason, hint, message}` (ADR 0062); values with a time or zone —
  strictly `YYYY-MM-DD` (ADR 0060 item 5))

### Requirement: CUJ10: The operator manages network client tokens
The operator SHALL manage network clients' access with a command on the server machine: issue a token to a new service with a client
name and permissions (read, or read and write; the token is shown once), show the list of issued tokens (without
secrets) and revoke a token. The server does not stop, revocation takes effect immediately, a restart does not change network tokens.
Decided: the control channel is a Unix socket (ADR 0080, Q-60); the request limit per network token — D-51 (Q-46).
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
- **THEN** it lists the issued tokens and secrets are not shown (the field set — identifier — client name (ADR 0082))

#### Scenario: Token revocation
- **WHEN** the operator revokes a token
- **THEN** the server is not stopped and the next request with this token is rejected (code — 401 `unauthorized` (ADR 0062 item 2)), including over a
  connection opened before the revocation
- **AND** a request executing at the moment of revocation completes

#### Scenario: Server restart
- **WHEN** the server is restarted
- **THEN** previously issued unrevoked network tokens remain valid without reissuing

#### Scenario: Client name is taken
- **WHEN** the operator issues a token with a client name that is taken by an active token
- **THEN** the issuance is not performed

#### Scenario: Client name after revocation
- **WHEN** the token with a client name is revoked and the operator issues a token with that name
- **THEN** the issuance is performed, the new token is valid, the revoked token does not come back to life

#### Scenario: Revoking a nonexistent token
- **WHEN** the operator revokes a nonexistent or already revoked token
- **THEN** an error message and the token data is unchanged (how the token is specified — identifier — client name (ADR 0082))

#### Scenario: Permission change
- **WHEN** the operator needs to change a client's permissions
- **THEN** it is done only by issuing a new token; the permissions of an issued token do not change

#### Scenario: Tokens and a backup
- **WHEN** a backup was made, the database was restored from it and the server was started with the same settings, next to which
  the tokens are stored
- **THEN** the backup contains only data, tokens are not part of it, the tokens are the same as before the restore, a revoked token
  stays revoked
- **AND** when moving to another machine the operator moves the tokens together with the settings, or otherwise reissues them

#### Scenario: Network token before the network listener is enabled
- **WHEN** the network listener is not enabled by an operator setting
- **THEN** there are no connections from other machines; a network token on loopback is not accepted (ADR 0066 item 1)

#### Scenario: Network token with the network listener enabled
- **WHEN** the network listener is enabled and a network token is presented over HTTPS
- **THEN** it is accepted from anywhere; it is not accepted over HTTP

#### Scenario: Operator command with the server stopped
- **WHEN** an operator command runs and the server is stopped
- **THEN** the behavior is defined by ADR 0080 (control channel, D-62…D-64)

### Requirement: CUJ11: Server upgrade, clients keep working
After a server upgrade within v1, programs and services SHALL work unchanged: new fields and
error codes may appear, but existing ones do not change or disappear. A v1 → v2 transition is out of scope (X-3).
Decisions: ADR 0010, 0013, 0018, 0019.

#### Scenario: Unchanged client after an upgrade
- **WHEN** the server is upgraded within v1
- **THEN** an unchanged client gets all the fields and error codes that existed before the upgrade, with the same meaning; new
  fields and codes are allowed (whether a new code replaces an earlier one in the same situation — a new code does not replace an earlier one within v1 (D-39))

#### Scenario: Fields and codes are only added
- **WHEN** fields or error codes are added in v1
- **THEN** existing fields and codes in responses are unchanged and have not disappeared (the error contract — the error body `{code, field, reason, hint, message}` (ADR 0062),
  401 `unauthorized` (ADR 0062 item 2))

#### Scenario: The v1 path is served
- **WHEN** a client requests a v1 path
- **THEN** it is served after the upgrade (a call to a path without a version — 404 `route_not_found` (ADR 0060 item 3))

#### Scenario: Data available without operator actions
- **WHEN** the server is upgraded within v1
- **THEN** previously saved records are available without operator actions

#### Scenario: A new version opens earlier databases itself
- **WHEN** a new version within v1 is started on a database created by an earlier v1 version (including on a backup made
  before the upgrade)
- **THEN** it opens the database itself and there is no separate format-migration command (how an ICU4X upgrade affects
  startup — open question Q-82, docs/open-questions.md)

#### Scenario: No rollback
- **WHEN** the operator needs to return to the data before the upgrade
- **THEN** there is no separate rollback command; returning the data is starting the new program on a backup made before the upgrade;
  the earlier program is not required to open a database after the new one

#### Scenario: Search does not change on upgrade
- **WHEN** the server is upgraded within v1
- **THEN** the matches and order for the same data and queries are unchanged: Unicode 18 is fixed for all of v1
  (the mechanics — open question Q-74, docs/open-questions.md)

#### Scenario: Tokens and UI after a restart on upgrade
- **WHEN** the server is restarted on upgrade
- **THEN** a local program re-reads the token file (earlier number versions remain valid), a network
  service continues with its earlier token (CUJ10); the UI needs a new link (CUJ13)

### Requirement: CUJ12: The record was changed concurrently
If a number's record was changed concurrently (in another tab or via the API) and the user pressed "delete" or
confirmed an overwrite (CUJ3), the action SHALL NOT be performed; the UI SHALL report «данные устарели, список обновлён»
(data is stale, the list is updated), re-read the number's history and offer to repeat the action on current data; the name entered in the form is kept.
Decided: visibility of intermediate states — ADR 0055 (Q-56); retry after a connection drop in the API — there are no idempotency keys, ADR 0064 item 5 (Q-54); undoing a delete — declining in the dialog, D-89 (Q-63).
Decisions: ADR 0002, 0006, 0019, 0025.

#### Scenario: Stale delete
- **WHEN** the number version changed after the UI got the version, the target record exists and the user
  confirms the delete
- **THEN** the record is not deleted and the UI reports «данные устарели, список обновлён» (the exact wording — D-81, D-93)

#### Scenario: Stale overwrite
- **WHEN** the version sent by the UI on an overwrite confirmation (the version source — from the results row or the conflict response (ADR 0060 item 6)) does not match the
  current one
- **THEN** the record is unchanged and the same message is shown

#### Scenario: Re-read and offer to retry
- **WHEN** a conflict is returned
- **THEN** the UI re-read the number's history, shows the current data and offers to repeat the action on the current
  data (dialog content — ADR 0091; for an action from name-search results, what is re-read —
  the number's history in the dialog, this number's rows are updated (D-91))

#### Scenario: The name in the form is kept
- **WHEN** a conflict is returned on an overwrite
- **THEN** the name entered in the form is kept

#### Scenario: Retry on current data
- **WHEN** after a conflict the user repeats the action on the current data
- **THEN** the action is performed if the version matches

#### Scenario: Another date of the same number was changed
- **WHEN** only another date of the same number was changed
- **THEN** the action is still rejected as stale (the version belongs strictly to the number)

#### Scenario: Record deleted concurrently
- **WHEN** the record the user acted on was deleted concurrently
- **THEN** the outcome is "version conflict" with the current version (ADR 0025 item 2, owner decision, OD-5, D-6); what the user sees — as for stale data, `unchanged` is success (D-92)

#### Scenario: Name already equal to the entered one with a stale version
- **WHEN** the record exists, its name is already equal to the entered one, and the number version is stale
- **THEN** a conflict with the current version (the version check is first, OD-4, item 7); what the user sees —
  as for stale data, `unchanged` is success (D-92)

#### Scenario: The user does not repeat the action
- **WHEN** the user does not repeat the action
- **THEN** the data does not change

### Requirement: CUJ13: First start and UI sign-in
The operator SHALL create an empty database with a separate command; a server on a directory without a database does not start and says so.
At startup the server SHALL print an address with an embedded token, by which a user on the same machine immediately
works in the UI; after a restart the old link stops working, a new one from the server output is needed. Decided: there is no TLS for the UI on loopback (D-49, Q-70).
Decisions: ADR 0011, 0012, 0014, 0015.

#### Scenario: Directory without a database
- **WHEN** the server is started on a directory without a database
- **THEN** the server does not start and reports that there is no database (the message channel — stderr, `key=value` (D-59))

#### Scenario: Creating the database and starting
- **WHEN** the operator creates a database with a separate command and starts the server
- **THEN** the server starts and prints an address with an embedded token

#### Scenario: Sign-in by link
- **WHEN** a user on the same machine opens the printed link
- **THEN** the UI is available at once, without entering credentials (whether the UI and local programs have one token, how the UI keeps
  the sign-in — a token in the tab, ADR 0086, D-77)

#### Scenario: Old link after a restart
- **WHEN** the server is restarted
- **THEN** the old link does not work and the new one from the output works (what an open tab shows — the token is removed, the form and results stay, sign-in by the new link (ADR 0088 item 2))

#### Scenario: Access not from loopback
- **WHEN** the UI is accessed not via loopback
- **THEN** there is no access to the UI; the UI token is accepted only from loopback

#### Scenario: Repeated database creation
- **WHEN** the operator runs the database creation command on a directory where a database already exists
- **THEN** a refusal and the existing database is unchanged (it is never overwritten)

#### Scenario: Second server process
- **WHEN** a second server process is started on a database already opened by another process
- **THEN** the start is rejected with the message «база уже открыта другим процессом» (the database is already open in another process)

#### Scenario: Network off by default
- **WHEN** the server is started without a network listener setting
- **THEN** the UI and local programs are available via loopback, the server does not accept connections from other machines; if
  the network listener is enabled and there is no certificate — the server does not start (the set of network listeners — D-47: only TLS on the network port)

#### Scenario: Damaged database at startup
- **WHEN** at startup the check shows that the database does not open consistent
- **THEN** the server does not start (CUJ14); corruption of other data is detected on read — "storage error"

#### Scenario: Freshly created empty database
- **WHEN** a search is performed on a just-created empty database
- **THEN** a number search gives an empty list; the empty name-search results view is a neutral text, not an error (D-85)

### Requirement: CUJ14: Data survived a failure
A save of a number acknowledged to the user SHALL survive a process crash or power loss: after a server restart
the number is in place; a save that did not get an acknowledgement is either fully present or fully absent. If
at startup the database is inconsistent, the server does not start and reports to the operator; other corruption is a "storage error" on read
(OD-4, item 3); data is not lost silently; the operator restores the database from a backup (CUJ15). Decisions: ADR 0006, 0013, 0014, 0016, 0027, 0028 (X-5 caveats: owner decision, OD-5, D-17, D-13).

#### Scenario: An acknowledged save survives a crash
- **WHEN** a user or client got a save (add) acknowledgement, the process crashed or power was lost and the
  server is restarted (the acknowledgement moment is after the WAL entry is flushed to stable storage (ADR 0050))
- **THEN** the acknowledged record is present

#### Scenario: Unacknowledged save
- **WHEN** a save (add) did not get an acknowledgement and there was a crash
- **THEN** after the restart the record is either fully present or fully absent

#### Scenario: Acknowledged rename and delete
- **WHEN** a rename (CUJ3) or delete (CUJ6) was acknowledged, there was a crash and the server is restarted
- **THEN** the acknowledged change is preserved and the deleted record does not "come back to life"

#### Scenario: Unacknowledged rename and delete
- **WHEN** a rename or delete did not get an acknowledgement and there was a crash
- **THEN** after the restart the change is either fully present or fully absent (the record and the number version change
  are indivisible, X-5)

#### Scenario: Limits of the guarantee
- **WHEN** a process crash, an OS failure or a power loss occurred and the disk honestly performs a flush to stable storage
- **THEN** acknowledged adds and renames are present and acknowledged deleted records are absent
- **AND** a "lying" disk and media corruption are outside the guarantee, detecting a "lying" disk is not promised; a disk write error is a
  "storage error" (CUJ16)

#### Scenario: Check at startup
- **WHEN** at startup the check (structure and recent changes) shows that the database does not open consistent
- **THEN** the server does not start, the operator gets a message (channel — stderr, `key=value` (D-59)) and records are not dropped
  silently (the check volume — ADR 0053 item 1; start time on 10^7 records — ≤ 5 s after a clean and ≤ 30 s after a crash shutdown (D-28))

#### Scenario: Corruption found on read
- **WHEN** corruption of other data (not found by the startup check) is detected on reading it or on a running
  server
- **THEN** the server moves to the "storage error" state: changes are rejected (CUJ16), search works while it can,
  the operator is given a message

#### Scenario: Restore on a damaged database
- **WHEN** the database is damaged and the operator starts the server on a backup
- **THEN** the server starts on the backup's data (including a backup made before an upgrade: the new program opens
  such a database itself; there is no rollback)

#### Scenario: Version read before a crash
- **WHEN** a client sends a rename or delete with a number version read before the crash, and the server was restarted after the
  crash
- **THEN** a conflict and the data is unchanged; the client re-reads; a conflict is allowed for any number (the mechanics —
  open question Q-78, docs/open-questions.md); the limits of the guarantee between version generations — per X-5 (owner decision, OD-5, D-17;
  exception — files copied not by the command, D-13)

#### Scenario: Planned restart
- **WHEN** the server is restarted on a plan (clean shutdown: defined in `write-operations`, "Clean and crash
  shutdown", ADR 0028 item 10) and a client sends a rename or delete with a number version read before the shutdown
- **THEN** the version is valid: the action is performed if the version matches

### Requirement: CUJ15: Backup and restore
The operator SHALL make a database backup with a command on the server machine while the server is running: users and programs
keep working without interruption, the backup corresponds to one consistent moment. Restore is starting the server
on a backup. Decisions: ADR 0006, 0012, 0013, 0016, 0018, 0027, 0028 (X-5 caveats: owner decision, OD-5, D-17, D-13).

#### Scenario: Backup with the server running
- **WHEN** the operator starts a backup (parameters — ADR 0057 items 4–7, ADR 0083; who performs the backup and the command channel —
  a Unix socket (ADR 0080))
- **THEN** the server is not stopped and UI and API requests are processed

#### Scenario: No refusals, latency growth within limits
- **WHEN** a backup is in progress
- **THEN** requests are processed without refusals and latency growth is within D-27 for writes and D-25 for search (at most 2 × the D-23 target)

#### Scenario: Consistent cut
- **WHEN** the backup is ready
- **THEN** it matches the database state at some moment: every change is either fully present or fully
  absent, there are no partial changes (which exact moment the backup fixes — open question Q-83,
  docs/open-questions.md; the concepts "moment" and "acknowledged" — Q-56, Q-57)

#### Scenario: Starting the server on a backup
- **WHEN** the server is started on a backup
- **THEN** searches return the data of the backup moment

#### Scenario: Versions after a restore
- **WHEN** the server is started on a backup and a client sends a rename or delete with a number version read before
  the restore
- **THEN** a conflict and the data is unchanged; mass conflicts right after a restore are allowed, for any numbers
  (the mechanics — open question Q-78, docs/open-questions.md); the limits of the guarantee between version generations — per X-5
  (owner decision, OD-5, D-17; exception — files copied not by the command, D-13)

#### Scenario: Tokens and a backup
- **WHEN** a backup was made
- **THEN** tokens are not part of the backup; a server started on a backup with the same settings, next to which the
  tokens are stored, accepts the same tokens, and a revoked one is still rejected; when moving to another machine the operator
  moves the tokens together with the settings, or otherwise reissues them

#### Scenario: Backup of an earlier v1 version
- **WHEN** the backup was made by an earlier v1 version and the server on the backup is a new version
- **THEN** the new program opens such a database itself, without a command; the earlier program is not required to open a database after the new one
  ("no rollback")

#### Scenario: Backup with the target occupied or a write error
- **WHEN** the target place is occupied, there is no space or a backup write error
- **THEN** the behavior is defined by ADR 0057 items 4–7 and ADR 0083

#### Scenario: Backup from a read-only database or one with a storage error
- **WHEN** the source database is read-only or has a storage error
- **THEN** the behavior is defined by ADR 0053 items 5–6 and D-29, D-34

#### Scenario: Server unavailable for the backup command
- **WHEN** the server is stopped or unavailable for the backup command
- **THEN** the behavior is defined by ADR 0080 (control channel, D-62…D-64)

### Requirement: CUJ16: The server cannot perform a change
If the database is read-only or has a storage error, the user SHALL learn of it when their action did not
go through («Сохранение не выполнено: база доступна только для чтения» (save not performed: the database is read-only) or «ошибка хранилища, обратитесь к оператору» (storage error, contact the operator));
search keeps working while it can. The UI does not show the server state in advance. Decisions: ADR 0011, 0014, 0019.

#### Scenario: Read-only database
- **WHEN** the database is read-only and the user performs a change
- **THEN** the UI shows «Сохранение не выполнено: база доступна только для чтения» and the data is unchanged
  (the exact wording and the text for a failed delete — D-81, D-93)

#### Scenario: Storage error on a change
- **WHEN** there is a storage error and the user performs a change
- **THEN** the UI shows «ошибка хранилища, обратитесь к оператору» and the data is unchanged (the exact wording —
  D-81, D-93)

#### Scenario: No warnings in advance
- **WHEN** the database is in the "read-only" or "storage error" state, the user took no actions or all their actions succeeded
- **THEN** there is no server state warning

#### Scenario: Search in read-only mode
- **WHEN** the database is read-only
- **THEN** all searches are performed

#### Scenario: Search with a storage error
- **WHEN** there is a storage error
- **THEN** searches for which the readable data is enough are performed; for the rest there are no guarantees (what counts as "enough" —
  ADR 0053 items 5–6, ADR 0053 item 1)

#### Scenario: Change refusal for an API client
- **WHEN** an API client performs a change in these states
- **THEN** it gets a refusal and the data is unchanged (code and error body — 503 `read_only` and 500 `storage_error` (ADR 0063), error body and codes — ADR 0062)

#### Scenario: Damage on a running server
- **WHEN** damage is detected on a running server
- **THEN** the server moves to the "storage error" state, changes are rejected, search works while it can,
  the operator is given a message

#### Scenario: Disk write error
- **WHEN** a disk write error occurs during a change
- **THEN** the state is "storage error" and the change is not performed; there is no partial change (X-5)

#### Scenario: Search not performed because of a storage error
- **WHEN** there is a storage error and the search cannot be performed
- **THEN** the user gets a search refusal (the UI form — D-81, D-93), an API client — 503 `read_only`, 500 `storage_error` (ADR 0063)

#### Scenario: Leaving the state
- **WHEN** the server leaves the "read-only" or "storage error" state (by itself or after a restart)
- **THEN** the behavior is defined by ADR 0053 items 5–6 and D-29, D-34

### Requirement: X-5: Number version
The system SHALL give any change to a number's records after which records exist a version different from all previously
received for that number: deterministically within a generation (between crash restarts and starts from a backup)
and between generations on an installation with preserved server settings; without them — with a collision probability of at most 2^-64
per event (owner decision, OD-5, D-17); the exception is a directory copied in another way (D-13). A number without records has the initial version (one per generation); the record and version are indivisible. Trace: the owner decision "Operations:
versioning is per number"; CUJ12; OD-3, Q-77 (`docs/owner-decisions.md`). Decided: Q-32, Q-34 — ADR 0060 item 6; Q-78 — ADR 0051 item 7. Decisions: ADR 0006,
0025, 0027 (item 4 refines item 6(a) of ADR 0006), 0028 (items 7, 9, 10).

#### Scenario: Version is never repeated
- **WHEN** for a number that had records, all records were deleted, the number was added again and a client sends a rename or delete with a version
  received before the deletion
- **THEN** the action is not performed and the data is unchanged; the refusal kind is "version conflict" (the version is checked before
  record existence, ADR 0025)

#### Scenario: Number without records
- **WHEN** a number has no records (it is at the initial version) and a client sends an action with that version for a missing
  record
- **THEN** the action is not performed and the data is unchanged; the refusal kind is "not found" (ADR 0025)

#### Scenario: A number returning to the no-records state
- **WHEN** a number's last record is deleted
- **THEN** the number version is the initial version of the current generation, one for all numbers without records; the guarantee "different from
  all previously received" applies to a number with records

#### Scenario: Directory copied not by the command
- **WHEN** the directory of a stopped database was copied manually, the source database continued to work, then the copy was started
- **THEN** the guarantee "different from all previously received" does not hold on the copy: a change by a version issued by the source
  database is accepted if and only if the current version on the copy equals it; the exception is an owner decision
  (OD-5, D-13), a backup created by the command does not have it; the scenario is verified by a review of the mechanism, not by a test

#### Scenario: Record and version are indivisible
- **WHEN** a crash occurs between the record write and the number version change
- **THEN** there is no partial result: there is no state where the record is changed and the version is not

#### Scenario: After a restore or a crash
- **WHEN** a number is restored from a backup or the server is restarted after a crash and a client sends a rename or delete with a
  version read before the restore or before the crash
- **THEN** a conflict and the data is unchanged (for any numbers; the limits of the guarantee between generations — D-17: deterministic
  on an installation with preserved settings, otherwise with probability at least 1 − 2^-64 per event)

#### Scenario: After a planned restart
- **WHEN** the server is stopped cleanly (definition — `write-operations`, "Clean and crash shutdown", ADR 0028 item 10) and
  started, and a client sends a rename or delete with a version read before the shutdown
- **THEN** the version is valid: the action is performed if the version matches

### Requirement: X-1: Scale of 10^7 records
The system SHALL perform operations CUJ1–CUJ16 without refusals caused by volume on a database of 10^7 records. Trace: item 6
of the original wording ("reliably store many records"); owner decision OD-6 (scale 10^7 instead of the brief §4's 10^8;
the earlier "Scale and reliability: 10^8 records" is replaced).
Decisions: ADR 0017.

#### Scenario: Operations on 10^7 records
- **WHEN** the database contains 10^7 records
- **THEN** operations CUJ1–CUJ16 are performed without refusals caused by volume (thresholds: file ≤ 8 GB (D-21), search latencies by class (D-23), start ≤ 5 s after a clean and ≤ 30 s after a crash shutdown (D-28))

#### Scenario: Write operation latency
- **WHEN** any write operation is performed on a database of 10^7 records
- **THEN** the acknowledgement is received within 100 ms (p99; D-27)

#### Scenario: Filling a database for a scale check
- **WHEN** a database of 10^7 records is needed for a check
- **THEN** the way to fill it is the separate program `npdb-fill` (D-61); there is no bulk load as a product operation (X-3)
