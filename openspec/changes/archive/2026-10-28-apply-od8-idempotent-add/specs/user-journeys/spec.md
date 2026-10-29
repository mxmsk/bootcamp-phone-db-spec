## MODIFIED Requirements

### Requirement: CUJ1: Adding a new number
The system SHALL let the user add a new phone number entered in international format with
+<country-code>: the number is saved and is found by name, and the name is found by number.
Decisions: ADR 0002, 0003, 0004, 0005, 0006, 0007, 0008, 0019, 0020–0024, 0025, 0027, 0028.

#### Scenario: Saving an unknown number
- **WHEN** the user enters an unknown number "+<country-code>…" and a valid name in the form and sends an add
  request (create only)
- **THEN** the record is created and the number has a version; its properties (how it differs from those issued earlier, the limits of the guarantee) are defined by X-5
- **AND** the UI reports a successful save «Сохранено: {имя}, {номер}, с {дата}» (Saved: {name}, {number}, since {date}) (D-83)

#### Scenario: The name is found by word-prefix search
- **WHEN** after saving, a search by word prefix of the name is run
- **THEN** the record with this number is reachable in the results (if there are more results than a page, by paging with "next")

#### Scenario: The name is found by number
- **WHEN** after saving, a search by this number (exact match) is run
- **THEN** the results contain the record with this name

#### Scenario: No date passed
- **WHEN** the add request has no date and the number has no records (CUJ1 does not ask for a date; whether the form has an optional date field —
  no, the field appears for a known number, D-82)
- **THEN** the record is stored with the date 1876-03-07 (substitution is only on add: on rename and delete the date
  is mandatory, owner decision OD-5, Q-90, ADR 0025 item 6)

#### Scenario: Adding without a date for a known number
- **WHEN** the add request has no date and the number already has a record on any date
- **THEN** no record is created, and the server answers with a "number known" conflict with the current number version (owner decision, OD-6, Q-160); the flow moves to CUJ2; the exception is a record on 1876-03-07 with the same name (OD-8, Q-180): "unchanged"

#### Scenario: A number without a plus
- **WHEN** the number is entered without "+"
- **THEN** it is rejected with a hint and no record is created (the hint text in the UI — D-81, D-93; the hint's expression
  for an API client — the error body `{code, field, reason, hint, message}` (ADR 0062))

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
- **WHEN** an identical record with a date is added again (for example, after a connection drop)
- **THEN** success, and the data and number version did not change

#### Scenario: A retry of an add without a date
- **WHEN** an add without a date was performed, the response was lost, and the request is repeated
- **THEN** success "unchanged" (200 `unchanged`, `write-operations`, "Retrying an operation after an unknown outcome"; OD-8, Q-180): the record was created by the first attempt, and the retry is safe; after an unknown outcome the UI does not retry the request automatically, and the user checks the outcome by number search (CUJ4) or submits the form again

#### Scenario: The number turned out to be known already
- **WHEN** the number is already in the database (except a record on 1876-03-07 with the same name — "unchanged", OD-8)
- **THEN** the flow moves to CUJ2 (how the UI determines this — by the server response `409 number_known` (ADR 0090))

#### Scenario: A change is impossible or the UI has an old token
- **WHEN** the database is available only for reading or there is a storage error, or the UI is open with an old token
- **THEN** CUJ16 (database state) or CUJ13 (old link) applies

#### Scenario: Write acknowledgement latency
- **WHEN** an add runs on a database of 10^7 records
- **THEN** the acknowledgement is received within 100 ms (p99; D-27); the criterion applies to all write operations
  (CUJ1–CUJ3, CUJ6, CUJ9) and is repeated in X-1

### Requirement: CUJ7: A local program works with the database
A local program on the same machine SHALL read the token from a file and perform the same operations as the UI, plus exact
whole-name search; after a server restart, its request with an old token SHALL get an "unauthorized" refusal,
and after re-reading the token file the retry succeeds (a rename and delete with an old version after a failure are a conflict,
OD-4, item 5; with the X-5 caveats: owner decision, OD-5, D-17, D-13). Decided: there is no TLS on loopback (D-49, Q-70).
Decisions: ADR 0005, 0006, 0007, 0008, 0011, 0012, 0013, 0019, 0027, 0028, 0031.

#### Scenario: The same operations as in the UI
- **WHEN** a local program with a valid token calls add, rename, delete, search by number,
  search by name words or exact whole-name search
- **THEN** the rules for number and name validation, conflicts and versions are the same as when called from the UI; there are no UI-level differences
  (date prompt, overwrite dialog, delete confirmation) in the API; on an add without a date the server substitutes 1876-03-07 if the number has no records (otherwise "unchanged" for a record on 1876-03-07 with the same name, OD-8, or "number known", OD-6, Q-160), and a rename and delete without a date are an error (date, "missing"; OD-5, Q-90)
- **AND** the local program's permissions are full; the location and access rights of the token file — the file `local.token`, mode 0600 (ADR 0065 items 1–2)

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
- **THEN** an "unauthorized" refusal (token not presented — the code and shape of 401 `unauthorized` (ADR 0062 item 2))

#### Scenario: An unknown token
- **WHEN** a request presents a token unknown to the server (invalid or arbitrary)
- **THEN** an "unauthorized" refusal and the data is not changed (the code and shape — 401 `unauthorized` (ADR 0062 item 2))

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
- **THEN** the request is rejected and the data is not changed (the refusal code — 401 `unauthorized` (ADR 0062 item 2))

#### Scenario: The request limit does not apply
- **WHEN** a local program sends requests at any frequency
- **THEN** there is no per-token request-limit refusal

#### Scenario: API version in the path
- **WHEN** a client requests a v1 path
- **THEN** the request is served (a request without a version in the path — 404 `route_not_found` (ADR 0060 item 3))

#### Scenario: The token file is unavailable
- **WHEN** the token file is unavailable or not yet updated after start
- **THEN** the behavior is defined by ADR 0065 items 1–2

#### Scenario: Connection load
- **WHEN** the number of connections, the request body size or the request time exceed the server limits
- **THEN** the behavior is defined by ADR 0071 (server limits: D-43, D-52, D-53)

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
  default 1876-03-07 if the number has no records, otherwise "unchanged" for a record on 1876-03-07 with the same name (OD-8, Q-180) or "number known", OD-6, Q-160; the date format is strictly `YYYY-MM-DD` (ADR 0060 item 5))
- **THEN** the record is created

#### Scenario: A name conflict on add
- **WHEN** the (number, date) being added already has a different name
- **THEN** a conflict, the response contains the current name and the number version, and the record is not changed

#### Scenario: A retry of an add of an identical record
- **WHEN** the service repeats an add of an identical record with a date (for example, after a connection drop)
- **THEN** success, and the data and version are not changed

#### Scenario: A retry of an add without a date
- **WHEN** the service repeats an add without a date whose first attempt created a record
- **THEN** success "unchanged" (200 `unchanged`) with the current number version (OD-8, Q-180); there is one record, and the retry is safe

#### Scenario: A rename or delete without a date
- **WHEN** a rename or delete is sent without a date
- **THEN** a value error (date, "missing"), the data is not changed; there is no 1876-03-07 substitution (owner decision, OD-5,
  Q-90; ADR 0025 item 6); the shape of the refusal — the error body `{code, field, reason, hint, message}` (ADR 0062)

#### Scenario: A rename without an expected version
- **WHEN** a rename or delete is sent without an expected version
- **THEN** the operation is not performed (the error shape — ADR 0062 items 1, 2)

#### Scenario: A stale version with an existing record
- **WHEN** the version in a rename or delete is stale and the record (number, date) exists
- **THEN** a conflict, the response contains the current version, and the data is not changed (the source of the current version —
  a result row or the conflict response (ADR 0060 item 6); the version in responses is a string of 64 lowercase hexadecimal characters (ADR 0060 item 6))

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
- **THEN** a silent success without changes, the number version does not change, as on a retry of an add (the version in the response is a
  string of 64 lowercase hexadecimal characters (ADR 0060 item 6))

#### Scenario: A rename to the same name with a stale version
- **WHEN** a rename to the same name is sent with a stale number version
- **THEN** a conflict with the current version and the data is not changed (the version check comes first)

#### Scenario: A read token and the limit
- **WHEN** the request is made by a `read` token, or the service exceeded the request limit
- **THEN** "no permissions" (CUJ8) or a limit refusal (the value — 100 requests per second, burst 100 (D-51))

#### Scenario: A retry after a connection drop
- **WHEN** a rename or delete is repeated after a connection drop
- **THEN** a retry with the earlier version is a "version conflict" (ADR 0026, owner decision, OD-5, D-7; `write-operations`, "Retrying an
  operation after an unknown outcome"); the representation in the API — there are no idempotency keys, and the outcome is found out by re-reading the number (ADR 0064 item 5); the version is a string of 64 characters (ADR 0060 item 6)

#### Scenario: Invalid name, number, date
- **WHEN** the name is empty or only spaces, or longer than allowed (the maximum is per `records`, "Text length and length cap" (owner decision, OD-5, D-3, ADR 0022)); or the number
  has an invalid shape; or the date is not a calendar day, is earlier than 1876-03-07 or (in a rename and delete) is not passed
- **THEN** an error, the data is not changed (the shape of the date refusal — the error body `{code, field, reason, hint, message}` (ADR 0062); values with a time or zone —
  strictly `YYYY-MM-DD` (ADR 0060 item 5))
