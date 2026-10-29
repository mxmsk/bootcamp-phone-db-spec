## MODIFIED Requirements

### Requirement: Input size before normalization and request body cap
The system SHALL decide on the length of a name and a query only from the normalized text (the 200-character cap after edge trimming is unchanged), however many whitespace characters are at the edges and however many input characters yield one character cell after NFC (up to 4), provided the input fits within the protocol cap: for a write body — 32 KiB (413), for a search query — 16 KiB of query string with headers (431), `http-api`. Larger input is rejected before parsing; `records` does not see it. Trace: CUJ1, CUJ2, CUJ3, CUJ5; OD-6 (D-43); ADR 0021, 0022, 0064.

#### Scenario: Many whitespace characters at the edges
- **WHEN** a name of 200 letters with 1,000 whitespace characters on each edge (the input fits within the 32 KiB body cap)
- **THEN** the name is valid; `records` does not refuse on input size

#### Scenario: Edges larger than the body cap
- **WHEN** a request body with a name of 200 letters and 20,000 whitespace characters on each edge is larger than 32 KiB
- **THEN** refusal "request too large" (413) before parsing; `records` makes no length decision, the name is neither created nor rejected as too long

#### Scenario: Search query edges larger than the header cap
- **WHEN** a word-prefix search query has 2,000 U+3000 characters at the edges (≈ 18,000 bytes in percent-encoding)
- **THEN** refusal 431 before parsing (`http-api`, "Header size cap"); `records` makes no length decision

#### Scenario: A name of "ё" in decomposed form
- **WHEN** a name of 200 "ё" entered as "е" + U+0308 (400 input characters)
- **THEN** after NFC the length is 200, the name is valid

### Requirement: Value error kind dictionary
The system SHALL name a value error with a (field, reason) pair from the dictionary: number — "invalid shape" (with the "missing
plus" marker); date — "missing" (rename and delete only), "nonexistent day", "out of range"; name — "empty", "control character", "too long"; name-search
query — "too long", "empty". For value validations, other capabilities do not introduce other reason names; protocol-level refusals (syntax, JSON type, absent member) are expressed by `http-api` with its own reasons `invalid_format`, `wrong_type`, `missing`. Trace: CUJ1, CUJ2, CUJ3, CUJ5,
CUJ9; ADR 0020.

#### Scenario: Reason from the dictionary
- **WHEN** a value validation rejects a number, date, name or query
- **THEN** the error contains the field and exactly one reason from that field's dictionary; the representation in the protocol is an API area matter (Q-49)

#### Scenario: Protocol-level refusal
- **WHEN** `http-api` rejects a date not in `YYYY-MM-DD` or a number that is not a string
- **THEN** this is not a dictionary violation: the reasons `invalid_format` and `wrong_type` belong to the protocol, the value never reaches `records`

#### Scenario: A number format error is not a separate kind
- **WHEN** a number search or a write receives a number that failed the shape check
- **THEN** the error is (number, "invalid shape"); "number format error" is a synonym, there is no other name

#### Scenario: Query reasons
- **WHEN** a name-search query is empty (no words or no text) or longer than the cap
- **THEN** the error is (query, "empty") or (query, "too long") respectively; the "empty" rule for a query is defined by `name-search`

#### Scenario: A control character in a query gives no reason
- **WHEN** a name-search query contains a Cc control character, for example "Ann<TAB>Smith"
- **THEN** a query has no "control character" reason; such a query is not rejected (ADR 0034 item 5)

#### Scenario: The marker only for digits without a plus
- **WHEN** the number is "+7abc", "+", "7+999", "+0123" or "  - () . " (empty after cleaning)
- **THEN** (number, "invalid shape") without the "missing plus" marker
