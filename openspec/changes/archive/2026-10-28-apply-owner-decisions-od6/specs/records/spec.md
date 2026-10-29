## RENAMED Requirements

- FROM: `### Requirement: Input size before normalization does not reject a valid name`
- TO: `### Requirement: Input size before normalization and request body cap`

## MODIFIED Requirements

### Requirement: Input size before normalization and request body cap
The system SHALL decide on the length of a name and a query only from the normalized text (the cap of 200 characters after edge trimming does not change), however many whitespace characters there are at the edges and however many input characters yield one character position after NFC (up to 4), provided the input fits within the request body cap (32 KiB, `http-api`). A body over the cap is rejected with the "request too large" refusal (413) before parsing; `records` does not see it. Trace: CUJ1, CUJ2, CUJ3, CUJ5; OD-6 (D-43); ADR 0021, 0022, 0064.

#### Scenario: Many spaces at the edges
- **WHEN** a name of 200 letters with 1 000 whitespace characters at each edge (the input fits within the 32 KiB body cap)
- **THEN** the name is valid; `records` does not refuse on input size

#### Scenario: Edges beyond the body cap
- **WHEN** a request body with a name of 200 letters and 20 000 whitespace characters at each edge is larger than 32 KiB
- **THEN** a "request too large" refusal (413) before parsing; `records` makes no decision on length, and the name is neither created nor rejected as too long

#### Scenario: A name of "ё" in decomposed form
- **WHEN** a name of 200 "ё" entered as "е" + U+0308 (400 input characters)
- **THEN** after NFC the length is 200 and the name is valid

### Requirement: Zero date when no date is given
The system SHALL substitute 1876-03-07 if no date is passed on add and the number has no records (otherwise — "number known", `write-operations`, OD-6, Q-160); an explicit 1876-03-07 is the same key, and the two cannot be told apart
after saving. On rename and delete the date is mandatory: its absence is a value error (date, "missing"),
and there is no substitution (owner decision, OD-5, Q-90: a narrowing of ADR 0002 item 4 for rename and delete; ADR 0025 item 6). Trace:
CUJ1, CUJ2, CUJ3, CUJ6, CUJ9; OD-6; ADR 0002 items 4–5, 0024, 0025 (item 6).

#### Scenario: No date passed on add
- **WHEN** a record is added without a date and the number has no records
- **THEN** the record is stored with the date 1876-03-07

#### Scenario: No date passed, number known
- **WHEN** a record is added without a date and the number has a record on any date
- **THEN** no record is created and there is no substitution: "number known" (`write-operations`)

#### Scenario: No date passed on rename or delete
- **WHEN** a rename or delete is sent without a date
- **THEN** a value error (date, "missing"); data is not changed, and the record on 1876-03-07 is not touched

#### Scenario: Explicit zero date on rename and delete
- **WHEN** a rename or delete is sent with an explicit date of 1876-03-07
- **THEN** the date is valid and the operation applies to the record (number, 1876-03-07)

#### Scenario: Explicit and substituted dates are one key
- **GIVEN** there is a record (number, substituted 1876-03-07) → "Ann"
- **WHEN** (the same number, explicit 1876-03-07) → "Ann" is added
- **THEN** an identical record, and data is not changed
