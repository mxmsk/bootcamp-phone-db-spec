## MODIFIED Requirements

### Requirement: Client-failure codes and statuses
The system SHALL respond to client failures with: 400 — `invalid_request`, `invalid_input`, `invalid_cursor`; 401 — `unauthorized`; 403 — `forbidden`, `host_not_allowed`; 404 — `record_not_found`, `route_not_found`; 405 — `method_not_allowed`; 409 — `name_conflict`, `number_known`, `version_conflict`; 413 — `payload_too_large`; 415 — `unsupported_media_type`; 429 — `rate_limited`; 431 — `headers_too_large`. HTTP parsing failures (an unparseable request, 431) carry no `code`. Trace: CUJ2, CUJ7–CUJ9, CUJ16; ADR 0062.

#### Scenario: Invalid record input
- **WHEN** the name contains a control character
- **THEN** 400, `invalid_input`, field `name`, reason `control_character`

#### Scenario: Name reasons
- **WHEN** the name is empty, or 201 characters
- **THEN** the reasons are `empty` and `too_long` respectively, field `name`

#### Scenario: First error
- **WHEN** both the number and the name are invalid
- **THEN** the field `number` is reported (order: number, date, name, expected version)

#### Scenario: Invalid request
- **WHEN** the body does not parse as a JSON object, or a member or parameter name is repeated
- **THEN** 400 `invalid_request` with an error body

#### Scenario: Invalid request at the HTTP parsing level
- **WHEN** the request does not parse as HTTP/1.1 or has no Host header
- **THEN** 400; a body with `code` is not guaranteed, the client goes by the status; the connection is closed after the response

#### Scenario: Access denials
- **WHEN** a request has no token; a "read" token calls add; a network token exceeded its limit; a foreign Host arrived at the loopback listener
- **THEN** respectively 401 `unauthorized`, 403 `forbidden`, 429 `rate_limited`, 403 `host_not_allowed`; each has an error body per this capability

#### Scenario: One response for any unrecognized token
- **WHEN** an unknown, stale, revoked token or a token of another kind is presented
- **THEN** the code, status, headers and body of the response are the same (401 `unauthorized`), and they do not reveal whether the token existed

### Requirement: Server-failure codes and statuses
The system SHALL respond to server failures with the codes and statuses: 500 — `storage_error`, `internal_error`; 503 — `read_only`, `server_starting`, `server_stopping`, `snapshot_revoked`, `request_timeout` (not executed); 504 — `write_outcome_unknown` (the operation may have been executed). A connection closed on the read or idle deadline, on pool overflow, at shutdown without accepted headers, or cut mid-stream gets no code. Trace: CUJ8, CUJ9, CUJ16; ADR 0062 item 8, 0063; ADR 0072 (OD-7).

#### Scenario: Not executed and outcome unknown
- **WHEN** a read did not finish in time, or a write was not accepted by the writer in time, or the writer accepted a write and the deadline expired without an outcome
- **THEN** in the first two cases 503 `request_timeout` (data not changed), in the third 504 `write_outcome_unknown`; the client tells "not executed" from "outcome unknown" by the code and the status

#### Scenario: Server failures
- **WHEN** the server is shutting down and the request headers were accepted; the server is starting
- **THEN** respectively 503 `server_stopping`, 503 `server_starting`

#### Scenario: Closing without a response
- **WHEN** a client did not send the headers within the request read deadline, or the connection pool is full and there is no one to evict
- **THEN** the connection is closed, there is no response with a code

### Requirement: Request check order
The system SHALL check a request in one order for all capabilities and respond at the first step that fires: (1) the connection limit (closing without a response); (2) header parsing (400, 431); (3) server start and shutdown; (4) the Host of the loopback listener; (5) route and method (404, 405); (6) token, then request limit, then permissions (401, 429, 403), by headers only. A refusal at steps 1–6 SHALL NOT lead to reading the body or touching storage. Trace: CUJ7, CUJ8, CUJ14; ADR 0063 item 7.

#### Scenario: No token, large body
- **WHEN** a request without a token declares a 300 KiB body
- **THEN** 401 `unauthorized`, not 413; the body is not read

#### Scenario: "Read" token and a large body
- **WHEN** a "read" token sends an add with a body above the cap
- **THEN** 403 `forbidden`, not 413

#### Scenario: Limit before permissions
- **WHEN** a network "read" token over its limit tries to write
- **THEN** 429 `rate_limited`, not 403

#### Scenario: Unknown path without a token
- **WHEN** a request without a token goes to an unknown path in working mode
- **THEN** 404 `route_not_found`, not 401

#### Scenario: Foreign Host before the token
- **WHEN** a request with a foreign Host and no token arrives at the loopback listener
- **THEN** 403 `host_not_allowed`

#### Scenario: Start before everything except header parsing
- **WHEN** during startup a correct request to any path arrives, and also a request that does not parse as HTTP/1.1
- **THEN** the first gets 503 `server_starting`, the second 400

#### Scenario: UI shell
- **WHEN** a request for a UI shell resource (area 5) arrives at the loopback listener in working mode
- **THEN** it is served without a token after step 4, no database data is served

### Requirement: Snapshot revocation, server start and shutdown
The system SHALL respond 503 `snapshot_revoked` if the snapshot is revoked before the response begins (after the stream begins — a cut with no marker); from the opening of the listeners to working mode — 503 `server_starting` to any request; after the shutdown signal — 503 `server_stopping` to a request with accepted headers and to a write not yet taken by the writer, and idle connections are closed without a response. These refusals are safe to retry. Trace: CUJ5, CUJ7, CUJ14; ADR 0035 item 6, 0056, 0058, 0063, 0074 (OD-7).

#### Scenario: Revocation before the start
- **WHEN** the snapshot is revoked before the first byte
- **THEN** 503 `snapshot_revoked`; a retry is allowed

#### Scenario: Revocation mid-stream
- **WHEN** the snapshot is revoked after part of the records has been sent
- **THEN** the connection is closed, there is no marker

#### Scenario: Rebuild
- **WHEN** an index rebuild is in progress
- **THEN** 503 `server_starting` and `Retry-After`

#### Scenario: Tokens of this start are not yet created
- **WHEN** during startup a request arrives without a token, with any token, or to an unknown path
- **THEN** 503 `server_starting`, not 401 and not 404

#### Scenario: Shutdown
- **WHEN** after the shutdown signal a request with already accepted headers is being processed
- **THEN** 503 `server_stopping`, the operation is not executed

#### Scenario: Write not taken by the writer
- **WHEN** the shutdown signal arrived while the write body was still being read or the operation was waiting in the writer queue
- **THEN** 503 `server_stopping`, the operation is not executed; a retry is safe
