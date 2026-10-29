# http-api

## Purpose

The capability defines the programming API v1 (HTTP + JSON): versioned paths, methods and resources of all operations, the representation of number, date, number version, cursor and page, the by-number results stream with a completeness marker, the error body, codes and statuses of all API refusals (including access, limits, time, server state and storage states), a single request check order, request size caps and the compatibility rule within v1. The access behavior (tokens, permissions, TLS, loopback, request limit) is set by `api-access`, the server behavior (connections, timeouts, startup, shutdown) by `server-runtime`; here only the codes, statuses and the place of their check in the common order.

## ADDED Requirements

### Requirement: API version in the path
The system SHALL serve API v1 only under the prefix `/v1/`; a request with a path without a version or with another version SHALL get 404 with the code `route_not_found`, without a redirect and without a default version. Trace: CUJ7, CUJ11; ADR 0013, 0060 (Q-51).

#### Scenario: Path without a version
- **WHEN** a client requests `GET /records/by-number?number=%2B79991234567`
- **THEN** response 404, code `route_not_found`; no redirect

#### Scenario: Another version
- **WHEN** a client requests a path under `/v2/`
- **THEN** response 404, code `route_not_found`

### Requirement: Resources and methods
The system SHALL provide exactly these operations: `POST /v1/records` (add), `POST /v1/records/rename` (rename), `POST /v1/records/delete` (delete), `GET /v1/records/by-number`, `GET /v1/records/by-words` (word-prefix), `GET /v1/records/by-name` (exact, whole-name). There SHALL NOT be administration through the API. Trace: CUJ7, CUJ8, CUJ9; item 1.2 of the original wording; ADR 0011, 0060.

#### Scenario: All UI operations and exact search
- **WHEN** a local program performs add, rename, delete, search by number, by words and by whole name
- **THEN** each operation is available at its own path and method from this requirement

#### Scenario: No administration
- **WHEN** a client calls a token issue, list or revoke path under `/v1/`
- **THEN** response 404, code `route_not_found`

#### Scenario: Wrong method
- **WHEN** a client sends `GET /v1/records/delete` or `POST /v1/records/by-number`
- **THEN** response 405, code `method_not_allowed`, an `Allow` header with the permitted method

#### Scenario: HEAD
- **WHEN** a client sends `HEAD` to a search path
- **THEN** a response with the same status and headers, without a body

### Requirement: Body format and encoding
The system SHALL accept a record body as a JSON object in UTF-8 with `Content-Type: application/json` (the parameter `charset=utf-8` is allowed) and respond with `application/json; charset=utf-8`. Another type or encoding — 415 `unsupported_media_type`; not JSON, not an object, invalid UTF-8, an unpaired surrogate in `\uXXXX`, a repeated object member name — 400 `invalid_request`. Trace: CUJ7, CUJ9; ADR 0021, 0060.

#### Scenario: Repeated member name
- **WHEN** the body contains `{"number":"+1","number":"+2","name":"A"}`
- **THEN** response 400, code `invalid_request`; no record is created

#### Scenario: Unpaired surrogate
- **WHEN** the name in the body contains `\ud800` without a pair
- **THEN** response 400, code `invalid_request`, before value checks

#### Scenario: Content type
- **WHEN** a record is sent with `Content-Type: text/plain`
- **THEN** response 415, code `unsupported_media_type`

### Requirement: Unknown fields and parameters
The system SHALL ignore unknown members of the request JSON object and unknown query-string parameters; a repeat of one known parameter SHALL give 400 `invalid_request`. Language headers and `Accept` SHALL NOT affect the response: the result order is always by the ru locale. Trace: CUJ11; ADR 0037 item 5, 0060 [DEFAULT].

#### Scenario: Extra field
- **WHEN** an add contains the extra field `"comment":"x"`
- **THEN** the record is processed as without it

#### Scenario: Extra parameter
- **WHEN** a search contains the parameter `debug=1`
- **THEN** it is ignored

#### Scenario: Repeated parameter
- **WHEN** a search contains `query=a&query=b`
- **THEN** response 400, code `invalid_request`

#### Scenario: Client language
- **WHEN** a request contains `Accept-Language: en`
- **THEN** the result order is unchanged, by ru

### Requirement: Number representation
The system SHALL accept a number in a JSON string or the `number` parameter in any form valid for `records` (with separators), and in all responses return the number in canonical form ("+" and digits). An invalid shape is `invalid_input`, field `number`, reason `invalid_format`; for a number without "+" the response SHALL contain `hint: "missing_plus"`. Trace: CUJ1, CUJ4; ADR 0004, 0023, 0060 (Q-26).

#### Scenario: Input with separators
- **WHEN** the number «+7 (999) 123-45-67» is added
- **THEN** `number` in the response equals «+79991234567»

#### Scenario: Hint about the plus
- **WHEN** a search by number gets `number=79991234567`
- **THEN** response 400, `invalid_input`, field `number`, reason `invalid_format`, `hint` equals `missing_plus`

#### Scenario: Unencoded plus in the query string
- **WHEN** a client sends `?number=+79991234567`, where "+" is read as a space
- **THEN** after cleaning the number has no "+": the response is as in "Hint about the plus"

#### Scenario: Number is not a string
- **WHEN** the body contains `"number":79991234567` (a JSON number)
- **THEN** `invalid_input`, field `number`, reason `wrong_type`

### Requirement: Date representation
The system SHALL write the date in requests and responses strictly as `YYYY-MM-DD`: four ASCII digits of the year, two of the month, two of the day, without time, zone, sign or spaces. Any other syntax, including a year that is not four digits, is `invalid_input`, field `date`, reason `invalid_format`; then the reasons `no_such_day`, `out_of_range` from `records`. Trace: CUJ2, CUJ9; ADR 0024, 0060 (Q-67).

#### Scenario: Time and zone
- **WHEN** the date is `2024-05-01T00:00:00Z` or `2024-05-01 10:00`
- **THEN** `invalid_input`, field `date`, reason `invalid_format`

#### Scenario: Year not four digits
- **WHEN** the date is `10000-01-01`, `-0001-01-01` or `24-05-01`
- **THEN** `invalid_input`, field `date`, reason `invalid_format`; the value does not reach `records`

#### Scenario: Reasons from records
- **WHEN** the date is `2021-02-30`, `1800-01-01` or `9999-12-31`
- **THEN** the first is `no_such_day`, the second is `out_of_range`, the third is valid

### Requirement: Number version is an opaque string
The system SHALL represent the number version as a string of exactly 64 lowercase hexadecimal characters (32 bytes, ADR 0054 item 5); the client compares it only for equality. An expected version not of this form is `invalid_input`, field `expected_version`, reason `invalid_format` (without accessing the database); of this form but not equal to the current one — `version_conflict`. Trace: CUJ9, CUJ12, X-5; ADR 0006, 0025 item 7, 0060 (Q-32, Q-34).

#### Scenario: Invalid shape
- **WHEN** `expected_version` is «abc», «A…» in uppercase, a string of 63 characters or a number
- **THEN** `invalid_input`, field `expected_version`; the first three are `invalid_format`, the number is `wrong_type`; data does not change

#### Scenario: Valid shape, foreign version
- **WHEN** `expected_version` is 64 `0` characters not issued by the server
- **THEN** 409 `version_conflict` with the current version

#### Scenario: Version in results
- **WHEN** any search is performed
- **THEN** each record contains `version` of this form

### Requirement: Adding a record
The system SHALL, on `POST /v1/records` with `number`, `name` and an optional `date`, respond: 201 when a record is created, 200 on an identical retry; the body `{outcome, record:{number,date,name}, version}`, `outcome` is `changed` or `unchanged`. Without a date 1876-03-07 is applied, and the response contains it. The `expected_version` field is ignored, of any shape. Trace: CUJ1, CUJ2, CUJ9; ADR 0005, 0026, 0060.

#### Scenario: Creation without a date
- **WHEN** («+79991234567», «Ann») is added without `date`
- **THEN** 201, `outcome` equals `changed`, `record.date` equals `1876-03-07`, `version` is the new version

#### Scenario: Identical retry
- **WHEN** the same add is repeated
- **THEN** 200, `outcome` equals `unchanged`, `version` is the same

#### Scenario: Name conflict
- **WHEN** (number, date) already has the name «Bob» and «Ann» is added
- **THEN** 409 `name_conflict` with `current_name` «Bob» and `version`; the record is unchanged

#### Scenario: Version in an add
- **WHEN** an add contains `"expected_version":"мусор"`
- **THEN** the field is ignored, there is no error for it

### Requirement: Renaming
The system SHALL, on `POST /v1/records/rename` with `number`, `date`, `name`, `expected_version` (all required), respond 200 with the body `{outcome, record, version}` (`changed` or `unchanged`); a missing record with the current version — 404 `record_not_found`; a stale version — 409 `version_conflict` with the current `version`. Trace: CUJ3, CUJ9, CUJ12; ADR 0005, 0025, 0026, 0060.

#### Scenario: Rename
- **WHEN** a record's name is changed with the current version
- **THEN** 200, `outcome` equals `changed`, `version` is new

#### Scenario: Same record, same name
- **WHEN** the name equals the current one with the current version
- **THEN** 200, `outcome` equals `unchanged`, the version is the same

#### Scenario: No date
- **WHEN** a request without the `date` field (or with `null`)
- **THEN** `invalid_input`, field `date`, reason `missing`; there is no substitution

#### Scenario: No record
- **WHEN** the current version, there is no record for this date
- **THEN** 404 `record_not_found`

#### Scenario: Stale version and no record
- **WHEN** the version is stale and there is no record
- **THEN** 409 `version_conflict` (the version is checked before existence)

### Requirement: Deleting a record
The system SHALL, on `POST /v1/records/delete` with `number`, `date`, `expected_version` (all required), delete one record and respond 200 `{outcome:"changed", record:{number,date}, version}`; errors are as for rename: `record_not_found`, `version_conflict`, `invalid_input`. The DELETE method with a body SHALL NOT be used. Trace: CUJ6, CUJ9, CUJ12; ADR 0005, 0025, 0026, 0060.

#### Scenario: Deleting the last record
- **WHEN** the only record of a number is deleted
- **THEN** 200, `version` is the initial version of the current generation

#### Scenario: Retry after success
- **WHEN** the same delete is sent again with the earlier version
- **THEN** 409 `version_conflict` with the current version

#### Scenario: DELETE method
- **WHEN** a client sends `DELETE /v1/records`
- **THEN** 405, code `method_not_allowed`

### Requirement: Search by number: a stream with a completeness marker
The system SHALL, on `GET /v1/records/by-number?number=…`, respond 200 with a single JSON object with members strictly in the order `number`, `version`, `records`, `count`, `complete`; records go by date DESC, as a stream, without a cursor and without truncation; `count` is the number of records, `complete: true` is the last member. A response without them is considered incomplete. Trace: CUJ4, CUJ8; ADR 0007, 0035, 0061.

#### Scenario: Number without records
- **WHEN** the number is valid, there are no records
- **THEN** 200, `records` is empty, `count` equals 0, `complete` equals true, `version` is the initial one

#### Scenario: All records of a number
- **WHEN** the number has 3 records
- **THEN** `records` contains 3 records, each with `number`, `date`, `name`, `version` (the same as at the root), `count` equals 3

#### Scenario: Broken stream
- **WHEN** a storage error or snapshot revocation interrupted the output after sending began
- **THEN** status 200 does not change, and `count` and `complete` are absent, and the client considers the response incomplete

#### Scenario: Maximum number
- **WHEN** a number has 2,967,164 records
- **THEN** all are returned in one response without a cursor, the server does not accumulate the whole list in memory

### Requirement: New members of the by-number stream
The system SHALL add new members to the by-number search response only between `version` and `records`; `count` and `complete` stay last, their order does not change within v1. Trace: CUJ11; ADR 0061, 0064.

#### Scenario: Client reads the end of the stream
- **WHEN** the server is upgraded within v1 and added a member after `version`
- **THEN** a client that checks `count` and `complete` at the end works unchanged

### Requirement: Word-prefix search and exact search
The system SHALL, on `GET /v1/records/by-words` and `GET /v1/records/by-name` with the parameters `query`, optional `page_size` and `cursor`, respond 200 `{records:[…], next_cursor}`; a record is `{number,date,name,version}`; `next_cursor` is a string or `null` on the last page; there is no total count. No matches — 200 with an empty `records` and `null`. Trace: CUJ5, CUJ7, CUJ8; ADR 0008, 0031, 0033, 0061.

#### Scenario: Page and cursor
- **WHEN** there are more matches than the page size
- **THEN** `records` contains a page, `next_cursor` is a string

#### Scenario: Last page
- **WHEN** there are no records after the page
- **THEN** `next_cursor` equals `null`; there is no total-count field in the response

#### Scenario: No matches
- **WHEN** nothing is found
- **THEN** 200, empty `records`, `next_cursor` equals `null`

#### Scenario: Exact search
- **WHEN** `by-name?query=ann` with the record «Ann Smith»
- **THEN** `records` is empty; there are no partial matches

### Requirement: Page size
The system SHALL accept `page_size` as a decimal integer from 1 to 100, default 50; a non-integer is `invalid_input`, field `page_size`, reason `invalid_format`, outside 1…100 — reason `out_of_range`; the check goes before the request checks. Trace: CUJ5, CUJ7; ADR 0033, 0061 [DEFAULT].

#### Scenario: Default
- **WHEN** `page_size` is not set, there are 120 matching records
- **THEN** the first page is 50 records

#### Scenario: Boundaries
- **WHEN** `page_size` equals 1, 100, 0, 101 or «abc»
- **THEN** 1 and 100 are valid; 0 and 101 are `out_of_range`; «abc» is `invalid_format`

#### Scenario: Check order
- **WHEN** `page_size=0` and an empty `query`
- **THEN** the error is for `page_size`

### Requirement: Name-search request errors
The system SHALL respond to a name-search request: too long — `invalid_input`, field `query`, reason `too_long`; empty — field `query`, reason `empty` (a missing or `null` parameter is empty); an invalid cursor — 400 `invalid_cursor`, field `cursor`. Order: `page_size`, `too_long`, `empty`, cursor. Trace: CUJ5, CUJ7; ADR 0034, 0061.

#### Scenario: Empty query
- **WHEN** `query` is empty, only spaces or missing altogether
- **THEN** 400, `invalid_input`, field `query`, reason `empty`; there are no empty results

#### Scenario: 300 hyphens
- **WHEN** `by-words?query=` of 300 «-»
- **THEN** reason `too_long`, not `empty`

#### Scenario: Empty cursor
- **WHEN** `cursor=` is passed with an empty value
- **THEN** 400 `invalid_cursor`; the absence of the parameter means the first page

#### Scenario: Cursor with an invalid query
- **WHEN** the query is empty and the cursor is corrupted
- **THEN** the error is for the query

### Requirement: Cursor encoding
The system SHALL issue the cursor as base64url without padding (RFC 4648 §5) of a binary structure: format 0x01, search kind (0x01 words, 0x02 exact), a 16-byte fingerprint, the position (name length u16, UTF-8 name, date u32 — days from 1876-03-07, number of digits of the number u8, the digits), CRC-32C (BE) of everything preceding. The string length is at most 1126 characters; a non-canonical encoding is invalid. Trace: CUJ5, CUJ14; ADR 0032 item 4, 0061.

#### Scenario: Length cap
- **WHEN** the position is a name of 200 four-byte characters, the date 9999-12-31, a number of 15 digits
- **THEN** the cursor is exactly 1126 characters; one character longer — `invalid_cursor`

#### Scenario: Non-canonical encoding
- **WHEN** a cursor with the same bytes but nonzero unused bits of the last character, or with "=" at the end, is passed
- **THEN** `invalid_cursor`

#### Scenario: Corruption
- **WHEN** a position byte in the decoded cursor is changed
- **THEN** `invalid_cursor` (the CRC does not match)

#### Scenario: A cursor is a position, not a page
- **WHEN** a cursor issued with `page_size=10` is passed with `page_size=40`
- **THEN** it is accepted and gives 40 records after the position

### Requirement: Query fingerprint and reference vectors
The system SHALL compute the fingerprint as the first 16 bytes of SHA-256 of "phonedb-cursor-v1", the byte 0x00, the search kind, a u32 number of keys and the keys (in ascending order of UTF-8 bytes, without repeats, each with a u32 length); the vectors of ADR 0061 are fixed before the first release and pass a golden test in every v1 release. Trace: CUJ5, CUJ11; ADR 0032 item 3, 0061.

#### Scenario: One fingerprint
- **WHEN** the queries «ann smith», «SMITH  Ann», «ann ann smith» (by words)
- **THEN** the fingerprint 5646b76be8265f28547d98fa34ebc340 for all three

#### Scenario: Exact search
- **WHEN** the exact query «ann smith»
- **THEN** the fingerprint 541085902fbeb42537b129ecefb02cc5; a cursor from a word search with the same text is rejected

#### Scenario: Reference cursor
- **WHEN** the position («Ann Smith», 2024-05-01, +79991234567), word search «ann smith»
- **THEN** the cursor equals `AQFWRrdr6CZfKFR9mPo068NAAAlBbm4gU21pdGgAANNfCzc5OTkxMjM0NTY3HHIJVg`

### Requirement: Error body
The system SHALL respond to every error with a JSON object `{code, field?, reason?, hint?, message}` plus members specific to the code; `code` and the status are the contract, `message` is Russian text for a human, outside the contract; `field` and `reason` are present for `invalid_input`. A client SHALL parse an error by `code`, an unknown code by the status class. Trace: CUJ7–CUJ9, CUJ11, CUJ16; ADR 0062 (Q-49, Q-50).

#### Scenario: Value error
- **WHEN** the number is invalid
- **THEN** the body contains `code` equal to `invalid_input`, `field` equal to `number`, `reason` equal to `invalid_format` and `message`

#### Scenario: Name conflict
- **WHEN** an add conflicts by name
- **THEN** the body contains `code` equal to `name_conflict`, `current_name`, `version`

#### Scenario: Unknown code
- **WHEN** a client received a code it does not know, with status 409
- **THEN** it handles the response as a failure of the "conflict" class

### Requirement: Client-failure codes and statuses
The system SHALL respond with client failures having these codes and statuses: 400 — `invalid_request`, `invalid_input`, `invalid_cursor`; 401 — `unauthorized`; 403 — `forbidden`, `host_not_allowed`; 404 — `record_not_found`, `route_not_found`; 405 — `method_not_allowed`; 409 — `name_conflict`, `version_conflict`; 413 — `payload_too_large`; 415 — `unsupported_media_type`; 429 — `rate_limited`; 431 — `headers_too_large`. The single source of codes and statuses. Trace: CUJ7–CUJ9, CUJ16; ADR 0062 (items 4, 8).

#### Scenario: Invalid record input
- **WHEN** the name contains a control character
- **THEN** 400, `invalid_input`, field `name`, reason `control_character`

#### Scenario: Name reasons
- **WHEN** the name is empty, or 201 characters
- **THEN** reasons `empty` and `too_long` respectively, field `name`

#### Scenario: First error
- **WHEN** both the number and the name are invalid
- **THEN** the field `number` is reported (order: number, date, name, expected version)

#### Scenario: Invalid request
- **WHEN** a request does not parse as HTTP/1.1 or as a JSON object, contains `Transfer-Encoding` together with `Content-Length`, has no Host header, repeats a member or parameter name
- **THEN** 400 `invalid_request`; on a message-boundary error the connection is closed after the response

#### Scenario: Access refusals
- **WHEN** a request without a token; a "read" token calls add; a network token exceeded the limit; a foreign Host came to the loopback listener
- **THEN** respectively 401 `unauthorized`, 403 `forbidden`, 429 `rate_limited`, 403 `host_not_allowed`; each has an error body per this capability

#### Scenario: One response for any unrecognized token
- **WHEN** an unknown, stale, revoked token or a token of another kind is presented
- **THEN** the code, status, headers and response body are the same (401 `unauthorized`), one cannot tell from them whether the token existed

### Requirement: Server-failure codes and statuses
The system SHALL respond with server failures having these codes and statuses: 500 — `storage_error`, `internal_error`; 503 — `read_only`, `server_starting`, `server_stopping`, `snapshot_revoked`, `request_timeout` (not executed), `too_many_connections`; 504 — `write_outcome_unknown` (the operation may have executed). A connection closed by the request-read or idle timeout or dropped in the middle of a stream gets no code. Trace: CUJ8, CUJ9, CUJ16; ADR 0062 item 8, 0063.

#### Scenario: Not executed and outcome unknown
- **WHEN** a read did not fit into the deadline, or a write was not accepted by the writer in time, or the writer accepted the write and the deadline expired without an outcome
- **THEN** in the first two cases 503 `request_timeout` (data not changed), in the third 504 `write_outcome_unknown`; the client distinguishes "not executed" from "outcome unknown" by the code and by the status

#### Scenario: Server refusals
- **WHEN** the connection pool is full; the server is stopping; the server is starting
- **THEN** respectively 503 `too_many_connections`, 503 `server_stopping`, 503 `server_starting`

#### Scenario: Close without a response
- **WHEN** a client did not send the headers within the request-read timeout
- **THEN** the connection is closed, there is no response with a code

### Requirement: Code and status stay fixed for a given situation
Within v1, for a situation in which a v1 release responded with a code and status, the system SHALL respond with the same code and status; new codes are introduced only for new situations. Trace: CUJ11; ADR 0013, 0062 [DEFAULT] (Q-79).

#### Scenario: Upgrade within v1
- **WHEN** the server is upgraded, a client repeats a request that earlier gave 409 `version_conflict`
- **THEN** the response is 409 `version_conflict`, not a new code

### Requirement: Storage states in responses
In the "read-only" state the system SHALL respond to any write operation with 503 `read_only`, and in "storage error" with 500 `storage_error`, before any value checks; all searches work in "read-only"; in "storage error" a search that needs the corrupted pages is 500 `storage_error`, the others work; "storage error" takes priority. Trace: CUJ16; ADR 0053, 0063.

#### Scenario: Write in read-only
- **WHEN** the database is read-only, an add with an invalid number or a stale version
- **THEN** 503 `read_only`, not an input error or a conflict

#### Scenario: Search in read-only
- **WHEN** the state is "read-only"
- **THEN** searches respond as usual

#### Scenario: Storage error
- **WHEN** the state is "storage error"
- **THEN** a write is 500 `storage_error`; a search without corrupted pages responds, otherwise 500 `storage_error`

#### Scenario: Both states
- **WHEN** the database is in "read-only", then a "storage error" occurred
- **THEN** a write gets `storage_error`

#### Scenario: Diagnostics are not disclosed
- **WHEN** `storage_error` or `read_only` is returned
- **THEN** the body contains no `errno`, file, offset or page (they are only for the operator)

### Requirement: Failure headers
The system SHALL accompany refusals with headers: 401 — `WWW-Authenticate: Bearer` (without an error code; the same for all reasons); 405 — `Allow`; `server_starting` — `Retry-After: 5`; `rate_limited` — `Retry-After` in whole seconds, at least 1; `snapshot_revoked` — `Retry-After: 0`. Other refusals have no `Retry-After`. Closing the connection after a refusal is determined by `server-runtime`. Trace: CUJ7, CUJ8, CUJ14; ADR 0063 (D-42).

#### Scenario: Retry hint at startup
- **WHEN** a request arrives during startup
- **THEN** 503 `server_starting` with `Retry-After: 5`

#### Scenario: 401 hint
- **WHEN** a request got 401
- **THEN** there is `WWW-Authenticate: Bearer`, without the `error` parameter

#### Scenario: No hint
- **WHEN** a write got `read_only` or `storage_error`
- **THEN** there is no `Retry-After` in the response

### Requirement: Snapshot revocation, server start and shutdown
The system SHALL respond 503 `snapshot_revoked` if the snapshot is revoked before the response begins (after the stream begins — a drop without the marker); from opening the listeners to working mode — 503 `server_starting` to any request on any path, before the route and token check; after the shutdown signal on an open connection — 503 `server_stopping`. These refusals are safe to retry. Trace: CUJ5, CUJ7, CUJ14; ADR 0035 item 6, 0056, 0058, 0063, 0074.

#### Scenario: Revocation before the start
- **WHEN** the snapshot is revoked before the first byte
- **THEN** 503 `snapshot_revoked`; a retry is allowed

#### Scenario: Revocation in the middle of a stream
- **WHEN** the snapshot is revoked after part of the records were sent
- **THEN** the connection is closed, there is no marker

#### Scenario: Rebuild
- **WHEN** an index rebuild is in progress
- **THEN** 503 `server_starting` and `Retry-After`

#### Scenario: Tokens of this run are not created yet
- **WHEN** during startup a request arrives without a token, with any token or on an unknown path
- **THEN** 503 `server_starting`, not 401 and not 404

#### Scenario: Shutdown
- **WHEN** after the shutdown signal a request arrives on an open connection
- **THEN** 503 `server_stopping`, the operation is not executed

### Requirement: Request check order
The system SHALL check a request in a single order for all capabilities and respond at the first step that triggers: (1) the connection limit; (2) header parsing (400, 431); (3) server startup and shutdown; (4) the loopback listener's Host; (5) route and method (404, 405); (6) token, then the request limit, then permissions (401, 429, 403), by headers only. A refusal at steps 1–6 SHALL NOT lead to reading the body or accessing storage. Trace: CUJ7, CUJ8, CUJ14; ADR 0063 item 7.

#### Scenario: No token, large body
- **WHEN** a request without a token declares a body of 300 KiB
- **THEN** 401 `unauthorized`, not 413; the body is not read

#### Scenario: "Read" token and a large body
- **WHEN** a "read" token sends an add with a body above the cap
- **THEN** 403 `forbidden`, not 413

#### Scenario: Limit before permissions
- **WHEN** a network "read" token over the limit tries to write
- **THEN** 429 `rate_limited`, not 403

#### Scenario: Unknown path without a token
- **WHEN** a request without a token to an unknown path in working mode
- **THEN** 404 `route_not_found`, not 401

#### Scenario: Foreign Host before the token
- **WHEN** a request with a foreign Host and without a token comes to the loopback listener
- **THEN** 403 `host_not_allowed`

#### Scenario: Startup before everything except header parsing
- **WHEN** during startup a correct request to any path arrives, as well as a request with a `Transfer-Encoding` and `Content-Length` conflict
- **THEN** the first gets 503 `server_starting`, the second 400 `invalid_request`

#### Scenario: UI shell
- **WHEN** a request for a UI shell resource (area 5) comes to the loopback listener in working mode
- **THEN** it is served without a token after step 4, database data is not served

### Requirement: Request check order after access
After the access checks the system SHALL check: (7) the body — content type (415), size cap (413), JSON parsing (400 `invalid_request`); (8) for a write — the database state (503 `read_only`, 500 `storage_error`); (9) values in field order (`invalid_input`, `invalid_cursor`); (10) the operation (409, 404 `record_not_found`, `request_timeout`, `write_outcome_unknown`, `snapshot_revoked`, `internal_error`). Trace: CUJ8, CUJ16; ADR 0025 item 4, 0063 item 7.

#### Scenario: Forbidden and invalid input
- **WHEN** a "read" token sends an add with an invalid number
- **THEN** 403 `forbidden`, not an input error

#### Scenario: Invalid JSON in read-only
- **WHEN** the database is "read-only", the body is not JSON
- **THEN** 400 `invalid_request`

#### Scenario: State before values
- **WHEN** the database is "read-only", an add with an invalid number
- **THEN** 503 `read_only`, not `invalid_input`

### Requirement: Body size cap
The system SHALL reject a request with a body larger than 262,144 bytes (256 KiB) with the refusal 413 `payload_too_large`: by `Content-Length` before reading the body, and without it by the count of what was read; the remainder is not read. The measure is on the wire, without decompression (`Content-Encoding` — 415). A valid name of 200 characters in the longest JSON form (`\uXXXX`) with edges of up to 20,000 whitespace characters each SHALL fit. Trace: CUJ1, CUJ3; ADR 0022, 0064, 0071 (Q-101, Q-102, D-43).

#### Scenario: Whole name in decomposed form
- **WHEN** a name of 200 Hangul syllables entered in decomposed form (up to 600 scalar values), each as `\uXXXX`
- **THEN** the request is accepted and processed per `records`

#### Scenario: Worst valid case
- **WHEN** 800 scalar values outside the BMP, each as a `\uXXXX` pair (9,600 bytes), with edges of 20,000 U+3000 spaces on each side, written as `　` (240,000 bytes)
- **THEN** the request is accepted (the body is below 262,144 bytes)

#### Scenario: Body above the cap
- **WHEN** a request with a valid token declares `Content-Length` of 300 KiB, or a body without a length exceeded 256 KiB
- **THEN** 413 `payload_too_large`, not `too_long` and not a record creation; in the first case the body was not read

### Requirement: Header size cap
The system SHALL reject a request whose request line (method and target) together with the headers is larger than 16,384 bytes (16 KiB) with the refusal 431 `headers_too_large`; there is no separate cap or refusal for the request target. A valid search request in percent-encoding together with the cursor and ordinary headers SHALL fit. Trace: CUJ5; ADR 0064, 0071 (D-43).

#### Scenario: Long search request
- **WHEN** a search request of 200 characters, each decomposed into 4 scalar values (3,200 bytes, 9,600 characters in percent-encoding), with a cursor of 1,126 characters and ordinary headers
- **THEN** the request is accepted

#### Scenario: Headers too large
- **WHEN** the path and query string together with the headers take 16,385 bytes
- **THEN** 431 `headers_too_large`, not 414 and not 413; the request is not executed

### Requirement: Compatibility within v1
Within v1 the system SHALL only add: response members, error codes and reasons, optional request members, parameters and paths; existing members, their types and meaning, codes, statuses and paths do not change and do not disappear; clients SHALL ignore unknown members. Trace: CUJ11; ADR 0013, 0064.

#### Scenario: Server upgrade
- **WHEN** the server is upgraded within v1, a response member is added
- **THEN** a client written before the upgrade works unchanged

#### Scenario: New code for a new situation
- **WHEN** a situation arose that did not exist before, with a new code for it
- **THEN** the client classifies it by the status class

### Requirement: Retry after a connection drop
The system SHALL NOT introduce idempotency keys: a retry of an add and of a rename to the same name with the number's records unchanged is safe, a retry of a rename or delete with the earlier version after a success gives `version_conflict`; the client finds out the outcome by re-reading the number. Trace: CUJ9, CUJ12; ADR 0026 items 5–6, 0064 (Q-54).

#### Scenario: Delete response lost
- **WHEN** the response to a delete is lost, the client repeats the request
- **THEN** 409 `version_conflict`; a search by number shows the absence of the record
