# records Specification

## Purpose
The capability defines the observable data model: a record (number, date) → name, its uniqueness, normalization and validity of the name,
number and date, a single length cap for the name and the query, and a single dictionary of value error kinds referenced by other
capabilities. It does not depend on the storage format, the API protocol or the UI form; decisions are in ADR 0002–0004, 0020–0025.

## Requirements

### Requirement: Record and its key
The system SHALL store a record as three values: number, date, name; the record key is the pair (canonical number, date), and
there are never two records with the same key; the name is not part of the key and is not unique. Trace: CUJ1, CUJ2, CUJ9; ADR 0002, 0020.

#### Scenario: One name for one number on different dates
- **GIVEN** there is a record (number, 2020-01-01) → «Ann»
- **WHEN** (the same number, 2021-01-01) → «Ann» is added
- **THEN** two separate records exist

#### Scenario: One name for different numbers
- **WHEN** different numbers are assigned the same name on the same date
- **THEN** all records exist; name uniqueness is not required

#### Scenario: Number written differently
- **GIVEN** there is a record («+7 999 123-45-67», 2020-01-01) → «Ann»
- **WHEN** («+79991234567», 2020-01-01) → «Bob» is added
- **THEN** the key is the same, this is a name conflict on (number, date), not a new record

### Requirement: Record identity and "the same name"
The system SHALL consider records identical if the key and the normalized name match; "the same name" is an exact match of
normalized names, taking into account case, "ё"/"е" and inner whitespace. Trace: CUJ1, CUJ2, CUJ3; ADR 0003, 0020, 0021.

#### Scenario: Difference only in the edges or the NFC form
- **GIVEN** the name «Ann Smith» is stored
- **WHEN** « Ann Smith » or a decomposed NFD name that equals the stored one after NFC is entered
- **THEN** the names are the same, the record is identical to the stored one

#### Scenario: Difference in case
- **GIVEN** «ANN» is stored
- **WHEN** «Ann» is entered
- **THEN** the names differ: this is not an identical record

#### Scenario: Difference in inner whitespace
- **GIVEN** «Ann Smith» is stored
- **WHEN** «Ann  Smith» (two spaces) is entered
- **THEN** the names differ

#### Scenario: "Ё" and "е"
- **WHEN** «Алёна» and «Алена» are compared
- **THEN** the names differ (search by name treats them as equal, ADR 0008)

### Requirement: Name and query normalization
The system SHALL normalize the name text (on add and rename) and the name-search query text with a single function: first
NFC, then removal of whitespace characters from both edges; inside the text characters are not changed, except by the effect of NFC. The order "first
trim, then NFC" (OD-3, Q-3) gives the same result on any input. Trace: CUJ1, CUJ2, CUJ3, CUJ5, CUJ7, CUJ9; ADR 0003, 0021.

#### Scenario: Trimming the edges
- **WHEN** the name is entered as «<U+00A0> Ann Smith<TAB>»
- **THEN** «Ann Smith» is stored

#### Scenario: Conversion to NFC
- **WHEN** the name is entered as «e» + U+0301
- **THEN** «é» (U+00E9) is stored

#### Scenario: Nothing is trimmed inside the text
- **WHEN** the name is «Ann<U+00A0><U+00A0>Smith»
- **THEN** the inner characters are kept as they are

#### Scenario: Normalization is idempotent
- **WHEN** a stored name is normalized again
- **THEN** the result equals the stored one

#### Scenario: The orders "NFC, trim" and "trim, NFC" are indistinguishable
- **WHEN** the text «<U+2000>e<U+0301><U+2001>», «<U+3000>Ann» or «Ann<U+2028>» is normalized in both orders
- **THEN** the results are equal: «é», «Ann», «Ann» (NFC maps U+2000 and U+2001 to U+2002 and U+2003, which are whitespace and get trimmed)

#### Scenario: The query is normalized the same way
- **WHEN** search by name receives the query « Ann » or «e»+U+0301
- **THEN** the query equals «Ann» and «é» respectively: the rule is the same as for the name

### Requirement: Whitespace character set
The system SHALL treat as whitespace characters the code points with the Unicode property White_Space (25): U+0009–000D, U+0020,
U+0085, U+00A0, U+1680, U+2000–200A, U+2028, U+2029, U+202F, U+205F, U+3000; all others are not. Trace: CUJ1, CUJ5; ADR 0021.

#### Scenario: Exotic spaces are trimmed
- **WHEN** the name starts with U+3000 or ends with U+2028
- **THEN** these characters are removed during normalization

#### Scenario: Non-whitespace characters
- **WHEN** the name consists of U+200B (ZERO WIDTH SPACE) or U+FEFF
- **THEN** the name is not empty and is not trimmed as whitespace

#### Scenario: Source of the White_Space and Cc properties
- **WHEN** the implementation determines whether a character is whitespace (White_Space) and whether it is a control character (Cc, "Name validity")
- **THEN** it takes the properties from the `icu_properties` 2.4.0 data (Unicode 18), which on all 1 112 064 scalar
  values equal UCD 18.0.0 (PropList, UnicodeData; F-47)

### Requirement: Name validity
The system SHALL reject a name that is empty after normalization (value error: name, "empty") or contains control characters
of category Cc (name, "control character"); all other names within the length cap are valid. Trace: CUJ1, CUJ2, CUJ3, CUJ9; ADR 0003, 0021 (item 5 is an owner decision, OD-5, D-2).

#### Scenario: Empty name
- **WHEN** the name is empty or consists only of whitespace characters
- **THEN** value error (name, "empty"), the record is not created or changed

#### Scenario: Control character inside
- **WHEN** the name is «Ann<TAB>Smith» or contains U+0000
- **THEN** value error (name, "control character")

#### Scenario: Control character at the edge
- **WHEN** the name is «<LF>Ann»
- **THEN** LF is trimmed as whitespace, the name «Ann» is valid

#### Scenario: Different scripts
- **WHEN** the name is «O'Brien», «Иван Петров-Водкин», «李雷» or «Ann 😀»
- **THEN** the name is valid

### Requirement: Text length and length cap
The system SHALL measure the length of the name and of the name-search query by the number of Unicode scalar values of the normalized text
(after NFC and edge trimming; before case folding, "ё" replacement and splitting into words) and reject text longer than 200, without
truncation: name — (name, "too long"), query — (query, "too long"). The unit is neither a byte nor a grapheme. This is the only
source of the cap. Trace: CUJ1, CUJ2, CUJ3, CUJ5, CUJ7, CUJ9; ADR 0003, 0022 (owner decision, OD-5, D-3).

#### Scenario: Exactly at the cap
- **WHEN** the name consists of 200 letters «Я»
- **THEN** the name is valid

#### Scenario: Over the cap
- **WHEN** the name consists of 201 letters
- **THEN** error (name, "too long"), the record is not created or changed, the name is not truncated

#### Scenario: Length is counted after NFC
- **WHEN** 201 scalar values are entered that become 200 after NFC
- **THEN** the name is valid

#### Scenario: The edge is not counted
- **WHEN** the name has 200 letters, with 50 whitespace characters on each edge
- **THEN** the name is valid: the length of the normalized text is 200

#### Scenario: The unit is not bytes
- **WHEN** the name has 200 Cyrillic letters (400 bytes in UTF-8)
- **THEN** the name is valid

#### Scenario: The query is counted the same way
- **WHEN** a name-search query has 200 letters, with spaces at the edges, or 201 letters
- **THEN** the first is valid; the second is an error (query, "too long"), there are no results

#### Scenario: Folding does not lengthen
- **WHEN** the name or query has 200 characters «ß» (the comparison key is longer than the text)
- **THEN** the length is 200, the text is valid: the text is counted, not the comparison key

### Requirement: Input size before normalization and request body cap
The system SHALL decide on the length of the name and the query only by the normalized text (the cap of 200 characters after edge trimming does not change), no matter how many whitespace characters are at the edges and how many input characters give one character cell after NFC (up to 4), if the input fits within the protocol limit: for the write body — 32 KiB (413), for the search query — 16 KiB of query string with headers (431), `http-api`. Larger input is rejected before parsing; `records` does not see it. Trace: CUJ1, CUJ2, CUJ3, CUJ5; OD-6 (D-43); ADR 0021, 0022, 0064.

#### Scenario: Many spaces at the edges
- **WHEN** the name has 200 letters with 1 000 whitespace characters on each edge (the input fits within the 32 KiB body cap)
- **THEN** the name is valid; `records` does not refuse on input size

#### Scenario: Edges larger than the body cap
- **WHEN** a request body with a name of 200 letters and 20 000 whitespace characters on each edge is larger than 32 KiB
- **THEN** refusal "request too large" (413) before parsing; `records` makes no length decision, the name is neither created nor rejected as too long

#### Scenario: Search query edges larger than the header cap
- **WHEN** a word search query has 2 000 U+3000 characters at the edges (in percent-encoding ≈ 18 000 bytes)
- **THEN** refusal 431 before parsing (`http-api`, "Header size cap"); `records` makes no length decision

#### Scenario: Name of "ё" in decomposed form
- **WHEN** the name has 200 "ё" entered as «е» + U+0308 (400 input characters)
- **THEN** after NFC the length is 200, the name is valid

### Requirement: Valid number shape
The system SHALL accept a number if, after removing separators, what remains is "+" and from 1 to 15 ASCII digits, the first of which is
1–9 (owner decision, OD-5, D-1, ADR 0004 item 6); otherwise reject with a value error (number, "invalid shape"). Trace: CUJ1, CUJ4, CUJ9; ADR 0004, 0023.

#### Scenario: Length boundary
- **WHEN** after "+" there is 1 digit «1», or 15 digits
- **THEN** the number is valid

#### Scenario: Too long or empty
- **WHEN** after "+" there are 16 digits, or no digits («+», «+ -»)
- **THEN** value error (number, "invalid shape")

#### Scenario: Missing plus
- **WHEN** the number is «79991234567» or «0079991234567»
- **THEN** error (number, "invalid shape") with the "missing plus" marker: after cleaning there is no "+" and at least one digit remains, there are no other characters, all digits are ASCII (a hint, ADR 0004 item 4)

#### Scenario: Plus not in first position
- **WHEN** the number is «7+999…» or «++7999…»
- **THEN** value error (number, "invalid shape")

#### Scenario: Non-ASCII digits
- **WHEN** the number contains an Arabic-Indic digit, a fullwidth digit or «＋» (U+FF0B)
- **THEN** value error (number, "invalid shape")

#### Scenario: Stray characters
- **WHEN** the number contains a letter, «x», «/» or «#»
- **THEN** value error (number, "invalid shape")

### Requirement: Number separator cleaning
The system SHALL, before validation, remove from the number at any position whitespace characters, "(", ")", ".", and the hyphens
U+002D, U+2010, U+2011 (the hyphen set is an owner decision, OD-5, D-4); no other characters are removed. Trace: CUJ1, CUJ4; ADR 0004, 0021, 0023 (item 5 is an owner decision, OD-5).

#### Scenario: All separators
- **WHEN** the number is «+7 (999) 123-45-67», «+7.999.123.45.67» or «+7<U+00A0>999<U+2011>123<TAB>45-67»
- **THEN** after cleaning each input equals «+79991234567»; the number is valid

#### Scenario: Unbalanced parentheses
- **WHEN** the number is «+7 (999 123-45-67»
- **THEN** the number is valid, parenthesis balance is not checked

#### Scenario: A dash is not a separator
- **WHEN** the number contains U+2013 (EN DASH), U+2212 (MINUS SIGN)
- **THEN** value error (number, "invalid shape")

#### Scenario: Only separators
- **WHEN** the number is «  - () . » or an empty string (after cleaning the string is empty)
- **THEN** value error (number, "invalid shape") without the "missing plus" marker: there are no digits

### Requirement: Canonical form and number comparison
The system SHALL consider two numbers the same number if, after cleaning, their strings of "+" and digits match character by character; there are no other
equivalences (prefixes, leading digits, country). Trace: CUJ1, CUJ2, CUJ4; ADR 0004, 0023 (Q-75).

#### Scenario: Different spellings, one number
- **WHEN** «+7 (999) 123-45-67» and «+79991234567»
- **THEN** this is one number and one key on the same date

#### Scenario: A prefix is not the number
- **WHEN** «+7999123» and «+79991234»
- **THEN** these are different numbers

#### Scenario: National prefix is not converted
- **WHEN** «+79991234567» and «+89991234567»
- **THEN** these are different numbers

#### Scenario: Country code is not checked
- **WHEN** the number «+999123» has a valid shape with an unassigned code
- **THEN** the number is accepted (owner decision, OD-5, D-1, ADR 0004 item 6)

### Requirement: Uniform number rules across all operations
The system SHALL apply the number rules identically on add, rename, delete and search by number; an invalid
number gives a number value error, not "not found" or empty results. Trace: CUJ1, CUJ4, CUJ9; ADR 0004 item 5, 0007, 0023.

#### Scenario: Search by a number of invalid shape
- **WHEN** a search by the number «79991234567» is performed
- **THEN** value error (number), no results

#### Scenario: Search by a different spelling
- **GIVEN** «+79991234567» is stored
- **WHEN** search by «+7 (999) 123-45-67»
- **THEN** the records of this number are found

#### Scenario: Delete with an invalid number
- **WHEN** delete with a number without "+"
- **THEN** value error (number), not "not found"

### Requirement: Date as a calendar day
The system SHALL accept a date as a day of the proleptic Gregorian calendar (year, month, day) without time of day and
time zone; a nonexistent day is invalid (value error: date, "nonexistent day"). The year on input is a signed 32-bit
integer; outside this type the input does not reach `records` (refusal by the protocol layer, Q-67). The date notation in the protocol is
the API area. Trace: CUJ2, CUJ9; ADR 0002, 0024.

#### Scenario: Year on input
- **WHEN** the year lies within −2 147 483 648…2 147 483 647, including outside the valid date range
- **THEN** the existence of the day is determined by the proleptic Gregorian calendar: year 0 exists, leap years are determined
  by the remainder of division (−4 and −400 are leap, −100 is not)

#### Scenario: Nonexistent day
- **WHEN** the date is 2021-02-30, 2021-02-29, 2021-13-01 or 2021-04-31
- **THEN** value error (date, "nonexistent day"), the record is not created

#### Scenario: Leap day
- **WHEN** the date is 2000-02-29 or 2024-02-29
- **THEN** the date is valid

#### Scenario: Year is not a leap year
- **WHEN** the date is 2023-02-29 or 2100-02-29 (a year divisible by 100 but not by 400)
- **THEN** value error (date, "nonexistent day")

#### Scenario: One day for everyone
- **WHEN** a record with the date 2030-05-17 is read from any time zone
- **THEN** the date is the same, 2030-05-17

### Requirement: Valid date range
The system SHALL accept dates from 1876-03-07 to 9999-12-31 inclusive (owner decision, OD-5, D-5 — the upper limit); reject a date out of
range with a value error (date, "out of range"); validity does not depend on the current time. Trace: CUJ2, CUJ9; ADR 0002, 0024.

#### Scenario: Lower bound
- **WHEN** the date is 1876-03-06
- **THEN** error (date, "out of range"), the record is not created

#### Scenario: Bound is inclusive
- **WHEN** the date is 1876-03-07
- **THEN** the date is valid

#### Scenario: Future date
- **WHEN** the date is 2999-01-01 or 9999-12-31
- **THEN** the date is valid regardless of the server's current date

#### Scenario: Above the technical limit
- **WHEN** the date is the triple (10000, 1, 1)
- **THEN** error (date, "out of range"); if the protocol cannot write a five-digit year, such input is rejected by the protocol layer before `records` (Q-67)

#### Scenario: Year 0000 exists in the calendar but is out of range
- **WHEN** the date is the triple (0, 2, 29) or (10000, 2, 29); both years are leap under the proleptic Gregorian calendar
- **THEN** error (date, "out of range"), not "nonexistent day"

#### Scenario: Negative year and year type bounds
- **WHEN** the date is the triple (−1, 1, 1), (−400, 2, 29) or (2147483647, 1, 1)
- **THEN** error (date, "out of range"): the year is representable on input, the day exists

#### Scenario: Negative year, day does not exist
- **WHEN** the date is the triple (−1, 2, 29) or (−100, 2, 29)
- **THEN** error (date, "nonexistent day"): year −1 is not divisible by 4, year −100 is divisible by 100 but not by 400

#### Scenario: Invalid by both calendar and range
- **WHEN** the date is (1875, 2, 30) or (10000, 2, 30)
- **THEN** error (date, "nonexistent day"): the calendar check goes before the range check

### Requirement: Zero date when no date is given
The system SHALL substitute 1876-03-07 if the date is not passed on add and the number has no records (otherwise — "unchanged" when writing to 1876-03-07 with the same name or "number known", `write-operations`, OD-8); an explicit 1876-03-07 is the same key, and they cannot be told apart after saving. On rename and delete the date is mandatory: its absence is a value error (date, "missing"), there is no substitution (OD-5, Q-90; ADR 0025 item 6). Trace: CUJ1–CUJ3, CUJ6, CUJ9; OD-6, OD-8; ADR 0002.

#### Scenario: Date not passed on add
- **WHEN** a record is added without a date, the number has no records
- **THEN** the record is stored with the date 1876-03-07

#### Scenario: Date not passed, number known
- **WHEN** a record is added without a date, the number has a record on any date
- **THEN** the record is not created, there is no substitution: "number known" or, if there is a record on 1876-03-07 with the same name, "unchanged" (`write-operations`)

#### Scenario: Date not passed on rename or delete
- **WHEN** a rename or delete is sent without a date
- **THEN** value error (date, "missing"); data is not changed, the record on 1876-03-07 is not affected

#### Scenario: Explicit zero date on rename and delete
- **WHEN** a rename or delete is sent with the explicit date 1876-03-07
- **THEN** the date is valid and the operation applies to the record (number, 1876-03-07)

#### Scenario: Explicit and substituted dates are one key
- **GIVEN** there is a record (number, substituted 1876-03-07) → «Ann»
- **WHEN** (the same number, explicit 1876-03-07) → «Ann» is added
- **THEN** an identical record, data is not changed

### Requirement: Value error kind dictionary
The system SHALL name a value error with a pair (field, reason) from the dictionary: number — "invalid shape" (with the "missing
plus" marker); date — "missing" (only rename and delete), "nonexistent day", "out of range"; name — "empty", "control character", "too long"; name-search
query — "too long", "empty". For value checks other capabilities do not introduce other reason names; protocol-level refusals (syntax, JSON type, absent member) are expressed by `http-api` with its own reasons `invalid_format`, `wrong_type`, `missing`. Trace: CUJ1, CUJ2, CUJ3, CUJ5,
CUJ9; ADR 0020.

#### Scenario: Reason from the dictionary
- **WHEN** a value check rejects a number, date, name or query
- **THEN** the error contains the field and exactly one reason from the dictionary of that field; the representation in the protocol is the API area (Q-49)

#### Scenario: Protocol-level refusal
- **WHEN** `http-api` rejects a date not in `YYYY-MM-DD` form or a number that is not a string
- **THEN** this is not a violation of the dictionary: the reasons `invalid_format` and `wrong_type` belong to the protocol, the value does not reach `records`

#### Scenario: Number format error is not a separate kind
- **WHEN** a search by number or a write receives a number that failed the shape check
- **THEN** the error is (number, "invalid shape"); "number format error" is a synonym, there is no other name

#### Scenario: Query reasons
- **WHEN** a name-search query is empty (no words or no text) or longer than the cap
- **THEN** error (query, "empty") or (query, "too long") respectively; the "empty" rule for the query is set by `name-search`

#### Scenario: A control character in the query gives no reason
- **WHEN** a name-search query contains a Cc control character, for example «Ann<TAB>Smith»
- **THEN** the query has no "control character" reason; such a query is not rejected (ADR 0034 item 5)

#### Scenario: The marker applies only to digits without a plus
- **WHEN** the number is «+7abc», «+», «7+999», «+0123» or «  - () . » (empty after cleaning)
- **THEN** (number, "invalid shape") without the "missing plus" marker

### Requirement: Deterministic value validation
The system SHALL validate the number, date and name only by the values themselves, without consulting the database state or the clock; with several
violations the first is reported in the order: number, date, name; within the date — "missing" (rename and delete), "nonexistent day", then "out of range"; within
the name — "empty", "control character", "too long"; within the query — "too long", then "empty" by the `name-search` rule (ADR 0034 item 4). Trace: CUJ1, CUJ2, CUJ3, CUJ9; ADR 0020.

#### Scenario: Same input, same result
- **WHEN** the same value is validated before and after a database change
- **THEN** the validation result is the same

#### Scenario: Several invalid fields
- **WHEN** both the number and the name are invalid
- **THEN** the number error is reported

#### Scenario: Value validation does not read the database
- **WHEN** the name is invalid and there is no record for the key
- **THEN** value error (name), not "not found"

#### Scenario: The name violates several rules
- **WHEN** the name is «Ann<TAB>Smith», padded to 201 characters
- **THEN** error (name, "control character"): it goes before "too long"

#### Scenario: Empty name made of spaces
- **WHEN** the name consists of 300 whitespace characters
- **THEN** error (name, "empty"); the length of the normalized text is 0, the length check is not reached
