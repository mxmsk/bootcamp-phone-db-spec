## MODIFIED Requirements

### Requirement: Error body
The system SHALL answer every error produced by the service with a JSON object `{code, field?, reason?, hint?, message}` plus members specific to the code; `code` and the status are the contract, `message` is a Russian human-readable text, outside the contract; `field` and `reason` are present for `invalid_input`. The client SHALL parse an error by `code`, and an unknown code by the status class. Trace: CUJ2, CUJ7–CUJ9, CUJ11, CUJ16; OD-6; ADR 0062 (Q-49, Q-50).

#### Scenario: Value error
- **WHEN** the number is invalid
- **THEN** the body contains `code` equal to `invalid_input`, `field` equal to `number`, `reason` equal to `invalid_format` and `message`

#### Scenario: Name conflict
- **WHEN** an add conflicts by name
- **THEN** the body contains `code` equal to `name_conflict`, `current_name`, `version`

#### Scenario: Number known
- **WHEN** an add without a date got the "number known" conflict
- **THEN** the body contains `code` equal to `number_known` and `version`; there is no `current_name`

#### Scenario: Unknown code
- **WHEN** the client received a code it does not know, with status 409
- **THEN** it handles the response as a failure of the "conflict" class

### Requirement: Client-failure codes and statuses
The system SHALL answer client failures with these codes and statuses: 400 — `invalid_request`, `invalid_input`, `invalid_cursor`; 401 — `unauthorized`; 403 — `forbidden`, `host_not_allowed`; 404 — `record_not_found`, `route_not_found`; 405 — `method_not_allowed`; 409 — `name_conflict`, `number_known`, `version_conflict`; 413 — `payload_too_large`; 415 — `unsupported_media_type`; 429 — `rate_limited`; 431 — `headers_too_large`. A single source of codes and statuses. A refusal produced by HTTP parsing before the service is called (unparseable HTTP/1.1, `Transfer-Encoding` together with `Content-Length`, 431) is status only: a body with `code` may be absent, and the client handles such a refusal by the status class. Trace: CUJ2, CUJ7–CUJ9, CUJ16; OD-6; ADR 0062 (items 4, 8).

#### Scenario: Invalid write input
- **WHEN** the name contains a control character
- **THEN** 400, `invalid_input`, field `name`, reason `control_character`

#### Scenario: Name reasons
- **WHEN** the name is empty, or 201 characters
- **THEN** reasons `empty` and `too_long` respectively, field `name`

#### Scenario: First error
- **WHEN** both the number and the name are invalid
- **THEN** the field `number` is reported (order: number, date, name, expected version)

#### Scenario: Invalid request
- **WHEN** the body does not parse as a JSON object, or a member or parameter name is repeated
- **THEN** 400 `invalid_request` with an error body

#### Scenario: Invalid request at the HTTP parsing level
- **WHEN** a request does not parse as HTTP/1.1, contains `Transfer-Encoding` together with `Content-Length`, or has no Host header
- **THEN** 400; a body with `code` is not guaranteed, the client relies on the status; on a message boundary error the connection is closed after the response

#### Scenario: Access refusals
- **WHEN** a request without a token; a "read" token calls an add; a network token exceeded its limit; a foreign Host arrived at the loopback listener
- **THEN** respectively 401 `unauthorized`, 403 `forbidden`, 429 `rate_limited`, 403 `host_not_allowed`; each has an error body per this capability

#### Scenario: One response for any unrecognized token
- **WHEN** an unknown, expired, revoked token or a token of another kind is presented
- **THEN** the code, status, headers and response body are the same (401 `unauthorized`), nothing in them reveals whether the token ever existed

### Requirement: Failure headers
The system SHALL accompany refusals with headers: 401 — `WWW-Authenticate: Bearer` (without an error code) and `Connection: close` (the same for all reasons); 405 — `Allow`; `server_starting` — `Retry-After: 5`; `rate_limited` — `Retry-After` in whole seconds, at least 1; `snapshot_revoked` — `Retry-After: 0`. Other refusals have no `Retry-After`. Closing the connection after a refusal is defined by `server-runtime`. Trace: CUJ7, CUJ8, CUJ14; ADR 0063 (D-42).

#### Scenario: Retry hint at startup
- **WHEN** a request arrives during startup
- **THEN** 503 `server_starting` with `Retry-After: 5`

#### Scenario: 401 hint
- **WHEN** a request got 401
- **THEN** `WWW-Authenticate: Bearer` is present, without the `error` parameter

#### Scenario: No hint
- **WHEN** a write got `read_only` or `storage_error`
- **THEN** there is no `Retry-After` in the response

### Requirement: Snapshot revocation, server start and shutdown
The system SHALL answer 503 `snapshot_revoked` if the snapshot was revoked before the response began (after the stream began — a break without a marker); from listener opening until working mode — 503 `server_starting` to any request on any path, before route and token checks; after the stop signal — 503 `server_stopping` on an open connection and for a write not yet taken by the writer. These refusals are safe to retry. Trace: CUJ5, CUJ7, CUJ14; ADR 0035 item 6, 0056, 0058, 0063, 0074.

#### Scenario: Revocation before the start
- **WHEN** the snapshot is revoked before the first byte
- **THEN** 503 `snapshot_revoked`; retry is allowed

#### Scenario: Revocation mid-stream
- **WHEN** the snapshot is revoked after some records were sent
- **THEN** the connection is closed, no marker

#### Scenario: Rebuild
- **WHEN** an index rebuild is in progress
- **THEN** 503 `server_starting` and `Retry-After`

#### Scenario: Tokens of this start not created yet
- **WHEN** during startup a request arrives without a token, with any token or on an unknown path
- **THEN** 503 `server_starting`, not 401 and not 404

#### Scenario: Shutdown
- **WHEN** after the stop signal a request arrives on an open connection
- **THEN** 503 `server_stopping`, the operation is not executed

#### Scenario: Write not taken by the writer
- **WHEN** the stop signal arrived while the write body was still being read or the operation was waiting in the writer queue
- **THEN** 503 `server_stopping`, the operation is not executed; retry is safe

### Requirement: Header size cap
The system SHALL reject a request whose request line (method and target) together with headers exceeds 16,384 bytes (16 KiB) with the refusal 431 `headers_too_large` (when HTTP parsing refuses — status only, without a body with `code`); there are no separate caps for the request target and the number of headers (the hyper default `max_headers` cap is lifted). A valid search request in percent-encoding together with a cursor and ordinary headers SHALL fit. Trace: CUJ5; ADR 0064, 0071 (D-43).

#### Scenario: Long search query
- **WHEN** a search query of 200 characters, each decomposed into 4 scalar values (3,200 bytes, 9,600 characters in percent-encoding), with a cursor of 1,126 characters and ordinary headers
- **THEN** the request is accepted

#### Scenario: Search query edges larger than the cap
- **WHEN** a word-prefix search query with 2,000 U+3000 characters at the edges (≈ 18,000 bytes in percent-encoding) with valid word length
- **THEN** 431 `headers_too_large` before parsing, although the size is under the 32 KiB body

#### Scenario: Headers too large
- **WHEN** the path and query string together with headers take 16,385 bytes
- **THEN** 431 `headers_too_large`, not 414 and not 413; the request is not executed

## ADDED Requirements

### Requirement: Absent member and `null`
The system SHALL treat a JSON body member with the value `null` as equivalent to an absent one. Absent member: `number` — `invalid_input`, `invalid_format`; `name` — `empty`; `date` on add — not an error (the substitution and `number_known` — "Adding a record"), on rename and delete — `missing`; `expected_version` on rename and delete — `missing`, on add it is ignored. A wrong member type is `wrong_type`. Trace: CUJ1, CUJ9; ADR 0062 item 3.

#### Scenario: Add with `"date": null`
- **WHEN** an add contains `"date":null` for a new number
- **THEN** same as without `date`: 201 with the date 1876-03-07; for a known number — 409 `number_known`

#### Scenario: No number
- **WHEN** an add contains `"number":null` or has no `number`
- **THEN** 400 `invalid_input`, field `number`, reason `invalid_format`

#### Scenario: No name
- **WHEN** an add has no `name` or has `"name":null`
- **THEN** 400 `invalid_input`, field `name`, reason `empty`

#### Scenario: No expected version
- **WHEN** a rename contains `"expected_version":null`
- **THEN** 400 `invalid_input`, field `expected_version`, reason `missing`
