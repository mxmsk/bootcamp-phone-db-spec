# records

## Purpose

The capability defines the observable data model: a record (number, date) → name, its uniqueness, the normalization and
validity of the name, number and date, a single length cap for the name and the query, and a single dictionary of value
error kinds that other capabilities refer to. It does not depend on the storage format, the API protocol or the UI form;
decisions: ADR 0002–0004, 0020–0025.

## ADDED Requirements

### Requirement: Record and its key
The system SHALL store a record as three values: number, date, name; the record key is the pair (canonical number, date),
and there are no two records with one key; the name is not part of the key and is not unique. Trace: CUJ1, CUJ2, CUJ9; ADR
0002, 0020.

#### Scenario: One name on one number on different dates
- **GIVEN** there is a record (number, 2020-01-01) → «Ann»
- **WHEN** (the same number, 2021-01-01) → «Ann» is added
- **THEN** two separate records exist

#### Scenario: One name on different numbers
- **WHEN** one name is assigned to different numbers on one date
- **THEN** all records exist; name uniqueness is not required

#### Scenario: The number is written differently
- **GIVEN** there is a record («+7 999 123-45-67», 2020-01-01) → «Ann»
- **WHEN** («+79991234567», 2020-01-01) → «Bob» is added
- **THEN** the key is the same, this is a name conflict on (number, date), not a new record

### Requirement: Record identity and "the same name"
The system SHALL consider records identical if the key and the normalized name match; "the same name" is an exact match of
normalized names, taking into account case, "ё"/"е" and inner whitespace. Trace: CUJ1, CUJ2, CUJ3; ADR 0003, 0020, 0021.

#### Scenario: Difference only at the edges or in NFC form
- **GIVEN** the name «Ann Smith» is saved
- **WHEN** « Ann Smith » or a decomposed NFD name equal to the saved one after NFC is entered
- **THEN** the names are the same, the record is identical to the saved one

#### Scenario: Difference in case
- **GIVEN** «ANN» is saved
- **WHEN** «Ann» is entered
- **THEN** the names are different: this is not an identical record

#### Scenario: Difference in inner whitespace
- **GIVEN** «Ann Smith» is saved
- **WHEN** «Ann  Smith» (two spaces) is entered
- **THEN** the names are different

#### Scenario: "Ё" and "е"
- **WHEN** «Алёна» and «Алена» are compared
- **THEN** the names are different (search by name treats them as the same, ADR 0008)

### Requirement: Name and query normalization
The system SHALL normalize the name text (on add and rename) and the name-search query text with one function: first NFC,
then removal of whitespace characters from both edges; inside the text the characters do not change except for the effect
of NFC. The order "trim first, then NFC" (OD-3, Q-3) gives the same result on any input. Trace: CUJ1, CUJ2, CUJ3, CUJ5,
CUJ7, CUJ9; ADR 0003, 0021.

#### Scenario: Edge trimming
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
- **THEN** the results are equal: «é», «Ann», «Ann» (NFC maps U+2000 and U+2001 to U+2002 and U+2003, which are
  whitespace and are trimmed)

#### Scenario: The query is normalized the same way
- **WHEN** search by name receives the query « Ann » or «e»+U+0301
- **THEN** the query equals «Ann» and «é» respectively: the rule is the same as for the name

### Requirement: Whitespace character set
The system SHALL consider as whitespace characters the code points with the Unicode property White_Space (25): U+0009–000D,
U+0020, U+0085, U+00A0, U+1680, U+2000–200A, U+2028, U+2029, U+202F, U+205F, U+3000; others are not. Trace: CUJ1, CUJ5; ADR
0021.

#### Scenario: Exotic spaces are trimmed
- **WHEN** the name starts with U+3000 or ends with U+2028
- **THEN** these characters are removed on normalization

#### Scenario: Non-whitespace characters
- **WHEN** the name consists of U+200B (ZERO WIDTH SPACE) or U+FEFF
- **THEN** the name is not empty and is not trimmed as whitespace

#### Scenario: Source of the White_Space and Cc properties
- **WHEN** the implementation determines whether a character is whitespace (White_Space) and whether it is a control
  character (Cc, "Name validity")
- **THEN** it takes the properties from the `icu_properties` 2.4.0 data (Unicode 18), which on all 1 112 064 scalar values
  equal UCD 18.0.0 (PropList, UnicodeData; F-47)

### Requirement: Name validity
The system SHALL reject a name that is empty after normalization (value error: name, "empty") or contains control
characters of category Cc (name, "control character"); other names within the length cap are valid. Trace: CUJ1, CUJ2,
CUJ3, CUJ9; ADR 0003, 0021 (item 5 — owner decision, OD-5, D-2).

#### Scenario: Empty name
- **WHEN** the name is empty or consists only of whitespace characters
- **THEN** a value error (name, "empty"), the record is not created or changed

#### Scenario: Control character inside
- **WHEN** the name is «Ann<TAB>Smith» or contains U+0000
- **THEN** a value error (name, "control character")

#### Scenario: Control character at the edge
- **WHEN** the name is «<LF>Ann»
- **THEN** LF is trimmed as whitespace, the name «Ann» is valid

#### Scenario: Different scripts
- **WHEN** the name is «O'Brien», «Иван Петров-Водкин», «李雷» or «Ann 😀»
- **THEN** the name is valid

### Requirement: Text length and length cap
The system SHALL measure the length of the name and of the name-search query by the number of Unicode scalar values of the
normalized text (after NFC and edge trimming; before case folding, "ё" replacement and splitting into words) and reject
text longer than 200, without truncation: for the name — (name, "too long"), for the query — (query, "too long"). The
unit is not a byte and not a grapheme. This is the only source of the cap. Trace: CUJ1, CUJ2, CUJ3, CUJ5, CUJ7, CUJ9; ADR
0003, 0022 (owner decision, OD-5, D-3).

#### Scenario: Exactly at the cap
- **WHEN** the name consists of 200 letters «Я»
- **THEN** the name is valid

#### Scenario: Over the cap
- **WHEN** the name consists of 201 letters
- **THEN** an error (name, "too long"), the record is not created or changed, the name is not truncated

#### Scenario: Length is counted after NFC
- **WHEN** 201 scalar values are entered that became 200 after NFC
- **THEN** the name is valid

#### Scenario: The edge is not counted
- **WHEN** the name is 200 letters, with 50 whitespace characters on each edge
- **THEN** the name is valid: the length of the normalized text is 200

#### Scenario: The unit is not bytes
- **WHEN** the name is 200 Cyrillic letters (400 bytes in UTF-8)
- **THEN** the name is valid

#### Scenario: The query is counted the same way
- **WHEN** a name-search query of 200 letters, with spaces at the edges, or of 201 letters
- **THEN** the first is valid; the second is an error (query, "too long"), no results

#### Scenario: Folding does not lengthen
- **WHEN** the name or query is 200 characters «ß» (the comparison key is longer than the text)
- **THEN** the length is 200, the text is valid: the text is counted, not the comparison key

### Requirement: Input size before normalization does not reject a valid name
The system SHALL NOT reject, because of the input size before normalization, a name or query that is valid under the
length cap: the length decision is made only on the normalized text, however many whitespace characters there are at the
edges and however many input characters give one character cell after NFC (up to 4). The specific byte cap on input and
the kind of refusal when it is exceeded belong to the API area (a requirement on other areas). Trace: CUJ1, CUJ2, CUJ3,
CUJ5; ADR 0021, 0022.

#### Scenario: Many spaces at the edges
- **WHEN** the name is 200 letters with 20 000 whitespace characters on each edge (the input fits the protocol cap)
- **THEN** the name is valid; `records` does not refuse on input size

#### Scenario: A name of "ё" in decomposed form
- **WHEN** the name is 200 "ё" entered as «е» + U+0308 (400 input characters)
- **THEN** after NFC the length is 200, the name is valid

### Requirement: Valid number shape
The system SHALL accept a number if after separator removal what remains is "+" and from 1 to 15 ASCII digits, the first of
which is 1–9 (owner decision, OD-5, D-1, ADR 0004 item 6); otherwise reject with a value error (number, "invalid shape").
Trace: CUJ1, CUJ4, CUJ9; ADR 0004, 0023.

#### Scenario: Length boundary
- **WHEN** after "+" there is 1 digit «1», or 15 digits
- **THEN** the number is valid

#### Scenario: Too long or empty
- **WHEN** after "+" there are 16 digits, or no digits («+», «+ -»)
- **THEN** a value error (number, "invalid shape")

#### Scenario: Missing plus
- **WHEN** the number is «79991234567» or «0079991234567»
- **THEN** an error (number, "invalid shape") with the "missing plus" marker: after cleaning there is no "+" and at least
  one digit remains, there are no other characters, all digits are ASCII (a hint, ADR 0004 item 4)

#### Scenario: Plus not in the first position
- **WHEN** the number is «7+999…» or «++7999…»
- **THEN** a value error (number, "invalid shape")

#### Scenario: Non-ASCII digits
- **WHEN** the number contains an Arabic-Indic digit, a fullwidth digit or «＋» (U+FF0B)
- **THEN** a value error (number, "invalid shape")

#### Scenario: Extraneous characters
- **WHEN** the number contains a letter, «x», «/» or «#»
- **THEN** a value error (number, "invalid shape")

### Requirement: Number separator cleaning
Before validation the system SHALL remove from the number, in any position, whitespace characters, "(", ")", ".", and the
hyphens U+002D, U+2010, U+2011 (the hyphen set is an owner decision, OD-5, D-4); no other characters are removed. Trace:
CUJ1, CUJ4; ADR 0004, 0021, 0023 (item 5 — owner decision, OD-5).

#### Scenario: All separators
- **WHEN** the number is «+7 (999) 123-45-67», «+7.999.123.45.67» or «+7<U+00A0>999<U+2011>123<TAB>45-67»
- **THEN** after cleaning each of the inputs equals «+79991234567»; the number is valid

#### Scenario: Unpaired parentheses
- **WHEN** the number is «+7 (999 123-45-67»
- **THEN** the number is valid, parenthesis pairing is not checked

#### Scenario: A dash is not a separator
- **WHEN** the number contains U+2013 (EN DASH), U+2212 (MINUS SIGN)
- **THEN** a value error (number, "invalid shape")

#### Scenario: Only separators
- **WHEN** the number is «  - () . » or an empty string (after cleaning the string is empty)
- **THEN** a value error (number, "invalid shape") without the "missing plus" marker: there are no digits

### Requirement: Canonical form and number comparison
The system SHALL consider two numbers one if after cleaning their strings of "+" and digits match character by character;
there are no other equivalences (prefixes, leading digits, country). Trace: CUJ1, CUJ2, CUJ4; ADR 0004, 0023 (Q-75).

#### Scenario: Different spellings, one number
- **WHEN** «+7 (999) 123-45-67» and «+79991234567»
- **THEN** this is one number and one key on the same date

#### Scenario: A prefix is not the number
- **WHEN** «+7999123» and «+79991234»
- **THEN** these are different numbers

#### Scenario: The national prefix is not converted
- **WHEN** «+79991234567» and «+89991234567»
- **THEN** these are different numbers

#### Scenario: The country code is not checked
- **WHEN** the number «+999123» of valid shape with an unassigned code
- **THEN** the number is accepted (owner decision, OD-5, D-1, ADR 0004 item 6)

### Requirement: Uniform number rules across all operations
The system SHALL apply the number rules identically on add, rename, delete and search by number; an invalid number yields a
number value error, not "not found" or empty results. Trace: CUJ1, CUJ4, CUJ9; ADR 0004 item 5, 0007, 0023.

#### Scenario: Search by a malformed number
- **WHEN** a search by the number «79991234567» is performed
- **THEN** a value error (number), no results

#### Scenario: Search by a different spelling
- **GIVEN** «+79991234567» is saved
- **WHEN** a search by «+7 (999) 123-45-67» is performed
- **THEN** the records of this number are found

#### Scenario: Delete with an invalid number
- **WHEN** a delete with a number without "+"
- **THEN** a value error (number), not "not found"

### Requirement: Date as a calendar day
The system SHALL accept a date as a day of the proleptic Gregorian calendar (year, month, day) without a time of day and a
time zone; a nonexistent day is invalid (value error: date, "nonexistent day"). The year on input is a signed 32-bit
integer; outside this type the input does not reach `records` (a protocol-layer refusal, Q-67). The way a date is written
in the protocol belongs to the API area. Trace: CUJ2, CUJ9; ADR 0002, 0024.

#### Scenario: Year on input
- **WHEN** the year is in −2 147 483 648…2 147 483 647, including outside the valid date range
- **THEN** the existence of the day is determined by the proleptic Gregorian calendar: year 0 exists, leap-year status is
  by remainder of division (−4 and −400 are leap, −100 is not)

#### Scenario: Nonexistent day
- **WHEN** the date is 2021-02-30, 2021-02-29, 2021-13-01 or 2021-04-31
- **THEN** a value error (date, "nonexistent day"), the record is not created

#### Scenario: Leap day
- **WHEN** the date is 2000-02-29 or 2024-02-29
- **THEN** the date is valid

#### Scenario: Not a leap year
- **WHEN** the date is 2023-02-29 or 2100-02-29 (a year divisible by 100 but not by 400)
- **THEN** a value error (date, "nonexistent day")

#### Scenario: One day for everyone
- **WHEN** a record with the date 2030-05-17 is read from any time zone
- **THEN** the date is the same, 2030-05-17

### Requirement: Valid date range
The system SHALL accept dates from 1876-03-07 to 9999-12-31 inclusive (owner decision, OD-5, D-5 — the upper limit); reject
a date out of range with a value error (date, "out of range"); validity does not depend on the current time. Trace: CUJ2,
CUJ9; ADR 0002, 0024.

#### Scenario: Lower bound
- **WHEN** the date is 1876-03-06
- **THEN** an error (date, "out of range"), the record is not created

#### Scenario: The bound is inclusive
- **WHEN** the date is 1876-03-07
- **THEN** the date is valid

#### Scenario: Future date
- **WHEN** the date is 2999-01-01 or 9999-12-31
- **THEN** the date is valid regardless of the server's current date

#### Scenario: Above the technical limit
- **WHEN** the date is the triple (10000, 1, 1)
- **THEN** an error (date, "out of range"); if the protocol cannot write a five-digit year, such input is rejected by the
  protocol layer before `records` (Q-67)

#### Scenario: Year 0000 exists in the calendar but is out of range
- **WHEN** the date is the triple (0, 2, 29) or (10000, 2, 29); both years are leap in the proleptic Gregorian calendar
- **THEN** an error (date, "out of range"), not "nonexistent day"

#### Scenario: Negative year and the bounds of the year type
- **WHEN** the date is the triple (−1, 1, 1), (−400, 2, 29) or (2147483647, 1, 1)
- **THEN** an error (date, "out of range"): the year is representable on input, the day exists

#### Scenario: Negative year, the day does not exist
- **WHEN** the date is the triple (−1, 2, 29) or (−100, 2, 29)
- **THEN** an error (date, "nonexistent day"): year −1 is not divisible by 4, year −100 is divisible by 100 but not by 400

#### Scenario: Invalid by both calendar and range
- **WHEN** the date is (1875, 2, 30) or (10000, 2, 30)
- **THEN** an error (date, "nonexistent day"): the calendar check goes before the range check

### Requirement: Zero date when no date is given
The system SHALL substitute 1876-03-07 if no date is passed on add; an explicit 1876-03-07 is the same key, and after
saving they cannot be told apart. On rename and delete the date is mandatory: its absence is a value error (date,
"missing"), with no substitution (owner decision, OD-5, Q-90: a narrowing of ADR 0002 item 4 for rename and delete; ADR
0025 item 6). Trace: CUJ1, CUJ2, CUJ3, CUJ6, CUJ9; ADR 0002 items 4–5, 0024, 0025 (item 6).

#### Scenario: No date passed on add
- **WHEN** a record is added without a date
- **THEN** the record is stored with the date 1876-03-07

#### Scenario: No date passed on rename or delete
- **WHEN** a rename or delete is sent without a date
- **THEN** a value error (date, "missing"); the data is unchanged, the record on 1876-03-07 is not touched

#### Scenario: Explicit zero date on rename and delete
- **WHEN** a rename or delete is sent with the explicit date 1876-03-07
- **THEN** the date is valid and the operation applies to the record (number, 1876-03-07)

#### Scenario: Explicit and substituted dates are one key
- **GIVEN** there is a record (number, substituted 1876-03-07) → «Ann»
- **WHEN** (the same number, explicit 1876-03-07) → «Ann» is added
- **THEN** an identical record, the data is unchanged

### Requirement: Value error kind dictionary
The system SHALL name a value error with a pair (field, reason) from the dictionary: number — "invalid shape" (with the
"missing plus" marker); date — "missing" (rename and delete only), "nonexistent day", "out of range"; name — "empty",
"control character", "too long"; name-search query — "too long", "empty". Other capabilities do not introduce other reason
names for the fields number, date, name, query. Trace: CUJ1, CUJ2, CUJ3, CUJ5, CUJ9; ADR 0020.

#### Scenario: A reason from the dictionary
- **WHEN** value validation rejects a number, date, name or query
- **THEN** the error contains the field and exactly one reason from that field's dictionary; the protocol representation
  belongs to the API area (Q-49)

#### Scenario: A number format error is not a separate kind
- **WHEN** a search by number or a write receives a number that failed the shape check
- **THEN** the error is (number, "invalid shape"); "number format error" is a synonym, there is no other name

#### Scenario: Query reasons
- **WHEN** a name-search query is empty (no words or no text) or longer than the cap
- **THEN** an error (query, "empty") or (query, "too long") respectively; the "empty" rule for the query is defined by
  `name-search`

#### Scenario: A control character in the query gives no reason
- **WHEN** a name-search query contains a Cc control character, for example «Ann<TAB>Smith»
- **THEN** the query has no "control character" reason; such a query is not rejected (ADR 0034 item 5)

#### Scenario: The marker is only for digits without a plus
- **WHEN** the number is «+7abc», «+», «7+999», «+0123» or «  - () . » (empty after cleaning)
- **THEN** (number, "invalid shape") without the "missing plus" marker

### Requirement: Deterministic value validation
The system SHALL validate the number, date and name only by the values themselves, without consulting the database state or
the clock; with several violations the first one is reported in the order: number, date, name; within the date —
"missing" (rename and delete), "nonexistent day", then "out of range"; within the name — "empty", "control character",
"too long"; within the query — "too long", then "empty" per the `name-search` rule (ADR 0034 item 4). Trace: CUJ1, CUJ2,
CUJ3, CUJ9; ADR 0020.

#### Scenario: Same input, same result
- **WHEN** the same value is validated before and after a database change
- **THEN** the validation result is the same

#### Scenario: Several invalid fields
- **WHEN** both the number and the name are invalid
- **THEN** the number error is reported

#### Scenario: Value validation does not read the database
- **WHEN** the name is invalid and there is no record for the key
- **THEN** a value error (name), not "not found"

#### Scenario: A name violates several rules
- **WHEN** the name is «Ann<TAB>Smith», padded to 201 characters
- **THEN** an error (name, "control character"): it goes before "too long"

#### Scenario: Empty name made of spaces
- **WHEN** the name is 300 whitespace characters
- **THEN** an error (name, "empty"); the length of the normalized text is 0, the length check is not reached
