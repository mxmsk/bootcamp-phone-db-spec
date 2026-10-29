## MODIFIED Requirements

### Requirement: Body size cap
The system SHALL reject a request with a body over 32 768 bytes (32 KiB) with a 413 `payload_too_large` refusal: by `Content-Length` before the body is read, and without it by the count of bytes read; the remainder is not read. The measure is on the wire, without decompression (`Content-Encoding` — 415). A valid name of 200 characters in the longest JSON form (`\uXXXX`, about 11 600 bytes of body) SHALL fit; edges of thousands of whitespace characters may not fit and then give 413. An owner decision (OD-6, D-43). Trace: CUJ1, CUJ3; OD-6; ADR 0022, 0064, 0071 (Q-101, Q-102).

#### Scenario: A whole name in decomposed form
- **WHEN** a name of 200 Hangul syllables entered in decomposed form (up to 600 Unicode scalar values), each as `\uXXXX`
- **THEN** the request is accepted and processed per `records`

#### Scenario: The worst allowed case
- **WHEN** 800 Unicode scalar values outside the BMP, each as a `\uXXXX` pair (9 600 bytes), with edges of 1 500 spaces U+3000 on each side, written as `　` (18 000 bytes), the other members about 2 000 bytes
- **THEN** the request is accepted (body about 29 600 bytes, less than 32 768)

#### Scenario: Edges beyond the cap
- **WHEN** a name of 200 characters with 20 000 spaces U+3000 on each side, written as `　` (240 000 bytes)
- **THEN** 413 `payload_too_large` before parsing, not `too_long`; no record is created

#### Scenario: A body over the cap
- **WHEN** a request with a valid token declares `Content-Length` of 300 KiB, or a body without a length exceeded 32 KiB
- **THEN** 413 `payload_too_large`, not `too_long` and not the creation of a record; in the first case the body was not read

#### Scenario: The cap boundary
- **WHEN** a body of exactly 32 768 bytes, then 32 769 bytes
- **THEN** the first is accepted for parsing, the second gets 413 `payload_too_large`

### Requirement: Adding a record
The system SHALL respond to `POST /v1/records` with `number`, `name` and an optional `date` as follows: 201 on creating a record, 200 on an identical retry; the body is `{outcome, record:{number,date,name}, version}`, `outcome` being `changed` or `unchanged`. Without a date, 1876-03-07 is applied (and the response contains it) only if the number has no records; otherwise 409 `number_known` with `version` (the current number version), and no record is created (OD-6, Q-160). The `expected_version` field is ignored, of any kind. Trace: CUJ1, CUJ2, CUJ9; OD-6; ADR 0005, 0026, 0060.

#### Scenario: Creation without a date
- **WHEN** ("+79991234567", "Ann") is added without `date`
- **THEN** 201, `outcome` is `changed`, `record.date` is `1876-03-07`, and `version` is a new version

#### Scenario: Number known
- **WHEN** the number has a record on 2020-01-01 and ("+79991234567", "Ann") is added without `date`
- **THEN** 409 `number_known` with the number's `version`; no record is created and the version is not changed

#### Scenario: An identical retry
- **WHEN** an add with `date` is repeated
- **THEN** 200, `outcome` is `unchanged`, and `version` is the same

#### Scenario: A retry without a date
- **WHEN** an add without `date` that created a record is repeated
- **THEN** 409 `number_known`; there is one record

#### Scenario: Name conflict
- **WHEN** (number, date) already has the name "Bob" and "Ann" is added with that date
- **THEN** 409 `name_conflict` with `current_name` "Bob" and `version`; the record is not changed

#### Scenario: A version in an add
- **WHEN** an add contains `"expected_version":"garbage"`
- **THEN** the field is ignored and there is no error for it

### Requirement: Error body
The system SHALL answer every error with a JSON object `{code, field?, reason?, hint?, message}` plus members specific to the code; `code` and status are the contract, `message` is human-readable Russian text, outside the contract; `field` and `reason` are present for `invalid_input`. A client SHALL parse an error by `code`, and an unknown code by status class. Trace: CUJ2, CUJ7–CUJ9, CUJ11, CUJ16; OD-6; ADR 0062 (Q-49, Q-50).

#### Scenario: A value error
- **WHEN** the number is invalid
- **THEN** the body contains `code` equal to `invalid_input`, `field` equal to `number`, `reason` equal to `invalid_format` and `message`

#### Scenario: Name conflict
- **WHEN** an add conflicts by name
- **THEN** the body contains `code` equal to `name_conflict`, `current_name`, `version`

#### Scenario: Number known
- **WHEN** an add without a date got a "number known" conflict
- **THEN** the body contains `code` equal to `number_known` and `version`; there is no `current_name`

#### Scenario: An unknown code
- **WHEN** a client received a code it does not know, with status 409
- **THEN** it treats the response as a failure of the "conflict" class

### Requirement: Client-failure codes and statuses
The system SHALL answer with client failures with these codes and statuses: 400 — `invalid_request`, `invalid_input`, `invalid_cursor`; 401 — `unauthorized`; 403 — `forbidden`, `host_not_allowed`; 404 — `record_not_found`, `route_not_found`; 405 — `method_not_allowed`; 409 — `name_conflict`, `number_known`, `version_conflict`; 413 — `payload_too_large`; 415 — `unsupported_media_type`; 429 — `rate_limited`; 431 — `headers_too_large`. A single source of codes and statuses. Trace: CUJ2, CUJ7–CUJ9, CUJ16; OD-6; ADR 0062 (items 4, 8).

#### Scenario: Invalid record input
- **WHEN** the name contains a control character
- **THEN** 400, `invalid_input`, field `name`, reason `control_character`

#### Scenario: Name reasons
- **WHEN** the name is empty, or 201 characters
- **THEN** reasons `empty` and `too_long` respectively, field `name`

#### Scenario: The first error
- **WHEN** both the number and the name are invalid
- **THEN** the field `number` is reported (order: number, date, name, expected version)

#### Scenario: An invalid request
- **WHEN** a request cannot be parsed as HTTP/1.1 or as a JSON object, contains `Transfer-Encoding` together with `Content-Length`, has no Host header, or repeats a member or parameter name
- **THEN** 400 `invalid_request`; on a message-boundary error the connection is closed after the response

#### Scenario: Access refusals
- **WHEN** a request without a token; a `read` token calls an add; a network token exceeded the limit; a foreign Host arrived at the loopback listener
- **THEN** respectively 401 `unauthorized`, 403 `forbidden`, 429 `rate_limited`, 403 `host_not_allowed`; each has an error body per this capability

#### Scenario: One response for any unrecognized token
- **WHEN** an unknown, stale, revoked token or a token of another kind is presented
- **THEN** the code, status, headers and body of the response are the same (401 `unauthorized`), and they cannot reveal whether the token ever existed

### Requirement: Number version is an opaque string
The system SHALL represent the number version as a string of exactly 64 lowercase hexadecimal characters (32 bytes, ADR 0054 item 5); a client compares it only for equality. An expected version not of this kind — `invalid_input`, field `expected_version`, reason `invalid_format` (without touching the database); of this kind but not equal to the current — `version_conflict`. Trace: CUJ9, CUJ12, X-5; OD-6; ADR 0006, 0025 item 7, 0060 (Q-32, Q-34).

#### Scenario: A wrong kind
- **WHEN** `expected_version` is "abc", "A…" in uppercase, a string of 63 characters, or a number
- **THEN** `invalid_input`, field `expected_version`; the first three — `invalid_format`, the number — `wrong_type`; data does not change

#### Scenario: The right kind, a foreign version
- **WHEN** `expected_version` is 64 characters `0`, not issued by the server
- **THEN** 409 `version_conflict` with the current version

#### Scenario: A version in the results
- **WHEN** a search by number is run
- **THEN** a `version` of this kind is present once at the response root, and the records do not contain it (OD-6, Q-128)

#### Scenario: A version in name results
- **WHEN** a word search or an exact name search is run
- **THEN** each record contains the `version` of its number of this kind

### Requirement: Search by number: a stream with a completeness marker
The system SHALL respond to `GET /v1/records/by-number?number=…` with 200 and one JSON object with members strictly in the order `number`, `version`, `records`, `count`, `complete`; records come by date DESC, as a stream, without a cursor and without truncation; `version` is the number version once per number, and the records do not have it (OD-6, Q-128); `count` is the number of records, and `complete: true` is the last member. A response without them is considered incomplete. Trace: CUJ4, CUJ8; OD-6; ADR 0007, 0035, 0061.

#### Scenario: A number with no records
- **WHEN** the number is valid and there are no records
- **THEN** 200, `records` is empty, `count` equals 0, `complete` equals true, and `version` is the initial one

#### Scenario: All records of a number
- **WHEN** the number has 3 records
- **THEN** `records` contains 3 records, each with `number`, `date`, `name` and no `version` (the version is at the root), and `count` equals 3

#### Scenario: A cut stream
- **WHEN** a storage error or a snapshot revocation interrupted the results after sending began
- **THEN** status 200 does not change, and `count` and `complete` are absent, and the client considers the response incomplete

#### Scenario: The maximum number
- **WHEN** the number has 2 967 164 records
- **THEN** all are returned in one response without a cursor, and the server does not accumulate the whole list in memory

### Requirement: Retry after a connection drop
The system SHALL NOT introduce idempotency keys: retrying an add with a date and a rename to the same name is safe while the number's records are unchanged, retrying an add without a date after success gives `number_known`, and retrying a rename or delete with the earlier version after success gives `version_conflict`; the client finds out the outcome by re-reading the number. Trace: CUJ1, CUJ9, CUJ12; OD-6; ADR 0026 items 5–6, 0064 (Q-54).

#### Scenario: A delete response is lost
- **WHEN** the response to a delete is lost and the client repeats the request
- **THEN** 409 `version_conflict`; search by number shows the record is absent
