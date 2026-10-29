## Purpose
The observable semantics of two searches by name: by word prefix (CUJ5) and exact whole-name search (CUJ7, CUJ8), and of
"next" pagination by cursor with no total count of results. The result order is defined by the `result-ordering`
capability (ADR 0009); here it is only used.

## ADDED Requirements

### Requirement: Name comparison key
The system SHALL compare names and query words by a comparison key obtained from the text as follows: convert to NFC,
apply full Unicode case folding (statuses C+F), convert to NFC again, replace "ё" (U+0451) with "е" (U+0435). The key is the
result of exactly these steps and is not normalized again. There are no other mappings ("й" = "и", accent stripping). The
key is computed for the whole name (exact search) or for each word separately. Trace: CUJ5, CUJ7; ADR 0008, 0030.

#### Scenario: Case and "ё"
- **GIVEN** the database has the records «Ёлка» and «Елка»
- **WHEN** a search «ЁЛКА» (or «елка», «Ёлка») is performed
- **THEN** both records are found

#### Scenario: The key does not strip accents and does not merge "й" with "и"
- **GIVEN** a record «José» and a record «Майя»
- **WHEN** the query «jose»; the query «маия»
- **THEN** in both cases the record is not in the results

#### Scenario: Full case folding
- **GIVEN** a record «STRASSE»
- **WHEN** the query «straße»
- **THEN** the record is found (ß folds to «ss»)

#### Scenario: Decomposed form
- **GIVEN** a record «Ёлка» (NFC)
- **WHEN** the query is entered in NFD form («Е» and U+0308)
- **THEN** the record is found

#### Scenario: The key is not recomputed
- **GIVEN** a record with the name «ё» + U+0308 (U+0451 U+0308)
- **WHEN** the query «ё» + U+0308 (the same text)
- **THEN** the record is found: the word key is «е» + U+0308 (two scalar values, not in NFC), the same for the name and the
  query
- **AND** the implementation does not renormalize the key («е» + U+0308 → «ё»)

### Requirement: Words of the name and of the query
The system SHALL convert the name (query) to NFC, split it into segments by default UAX #29 (WB3–WB999; Unicode 18; without
dictionaries or locale) and consider a word a segment containing a character outside the categories Z*, P*, Cc, Cf; the
comparison key is computed for each word separately. The predicate is an owner decision (OD-5, D-19, "Clarification of
D-19"). Trace: CUJ5; ADR 0008 item 4, ADR 0030 item 3.

#### Scenario: Word predicate by categories
- **WHEN** a segment contains at least one character whose General_Category is not Z*, not P*, not Cc and not Cf
- **THEN** it is a word: letters, numbers, M* marks, S* symbols (Sm, Sc, Sk, So, including emoji), Co, Cn
- **AND** not words are segments of only spaces (Z*), punctuation (P*: "_" and U+203F — Pc, hyphen — Pd, apostrophe —
  Po), control characters (Cc) and format characters (Cf: U+200B)

#### Scenario: First the boundaries on the NFC text, then the word key
- **GIVEN** a record «Ёлка Иван»
- **WHEN** the query «Е» + U+0308 + «Л ИВ» (NFD, upper case)
- **THEN** the record is found: the query words «ЁЛ» and «ИВ» are obtained from the NFC text, the keys are «ел» and «ив»

#### Scenario: Hyphen and space separate
- **GIVEN** a record «Anna-Maria»
- **WHEN** the query «maria»
- **THEN** the record is found

#### Scenario: Digits and letters in one word
- **GIVEN** a record «Agent 007b» and a record «R2-D2»
- **WHEN** the query «007»; the query «d2»
- **THEN** the first record is found; the second is found (words «R2» and «D2»)

#### Scenario: A name without words is not found by word search
- **GIVEN** records with the names «-» and «_»
- **WHEN** the query «a»
- **THEN** there are no records in the results (a name without words — only spaces and punctuation — is found only by exact
  search)

#### Scenario: Underscore and arc are not words
- **WHEN** the word query «_» (category Pc), or «‿» (U+203F, Pc)
- **THEN** an error (query, "empty"): the query has no words, it is punctuation (ADR 0030 item 3)

#### Scenario: Emoji is a word
- **GIVEN** a record «Ann 😀»
- **WHEN** the word query «😀»
- **THEN** there is no "empty" error, the record is found: the segment «😀» (So) is a word

#### Scenario: Sm and Sc symbols are words
- **GIVEN** records «C++» and «Ann $»
- **WHEN** the word query «+», then the query «$»
- **THEN** there is no "empty" error: «+» (Sm) and «$» (Sc) are words; the query «+» finds «C++» (words «C» and «+», «+»),
  the query «$» finds «Ann $»

#### Scenario: A one-character name is found by word search
- **GIVEN** a record «😀» (a name of only an emoji)
- **WHEN** the query «😀»
- **THEN** the record is found; unlike a name made of punctuation («-», «_»), which is not found by word search

#### Scenario: An invisible format character is not a word
- **WHEN** a word query of a single U+200B (Cf) or U+200B with spaces
- **THEN** an error (query, "empty"): there are no words (Cf gives no word)
#### Scenario: Narrow no-break space inside a word
- **GIVEN** a record «Ann» + U+202F + «Smith»
- **WHEN** the query «ann»; the query «smith»
- **THEN** the first finds the record, the second does not: U+202F (ExtendNumLet) between letters does not split the word,
  as in UAX #29; at the beginning and the end of the name it is trimmed as whitespace (`records`)

#### Scenario: Tibetan syllable
- **GIVEN** a record «བཀྲ་ཤིས»
- **WHEN** the query «བཀྲ»
- **THEN** the record is found, there is no "empty" error: the syllable contains letters; U+0F0B (tsheg) between syllables
  does not form a word

### Requirement: Characters inside a word: UAX #29 defaults
The system SHALL NOT split a word on characters that Unicode 18 (UAX #29, default rules) leaves inside a word: the
apostrophe U+0027, U+2018, U+2019, the period, the colon and U+00B7 between letters, the comma and period between digits,
"_". The hyphen U+002D splits a word. The list is illustrative; the UAX #29 rules are normative (the implementation is
checked against WordBreakTest-18.0.0: 1944 of 1944). Trace: CUJ5; ADR 0030.

#### Scenario: Apostrophe between letters
- **GIVEN** a record «O'Brien»
- **WHEN** the query «o'brien», or «o'bri», or «o'»
- **THEN** the record is found
- **AND** for the query «brien» there is no record

#### Scenario: A colon between letters does not split
- **GIVEN** a record «Smith:John»
- **WHEN** the query «john»; the query «smith:j»
- **THEN** the first finds no record; the second finds it

#### Scenario: Underscore and middle dot do not split
- **GIVEN** records «john_smith» and «Ann·Bob» (U+00B7)
- **WHEN** the query «smith»; the query «bob»; the query «john_s»; the query «ann·b»
- **THEN** the first two find no records; the last two find the corresponding records

#### Scenario: A period between letters does not split
- **GIVEN** a record «Smith.John»
- **WHEN** the query «john»
- **THEN** the record is not in the results; the query «smith.j» finds it

#### Scenario: Different apostrophes are not identified
- **GIVEN** a record «O'Brien» (U+0027)
- **WHEN** the query «o’brien» (U+2019)
- **THEN** the record is not in the results

### Requirement: Han, kana, Thai, Lao, Myanmar scripts: as in UAX #29
Owner decision (OD-5, D-14): the system SHALL apply to these scripts the same default UAX #29 rules, without dictionaries:
each Han character and each hiragana is a separate word; continuous katakana is one word; in Thai, Lao and Myanmar scripts
a word is a character with the characters Word_Break=Extend that follow it. There is no rule for a change of script: a
boundary exists only where a rule gives it. Trace: CUJ5; ADR 0008 item 4, ADR 0030 item 4.

#### Scenario: Han characters are separate words
- **GIVEN** a record «李雷»
- **WHEN** the query «李»; the query «雷»; the query «李雷»; the query «雷李»
- **THEN** all four find the record (the query words «李» and «雷» are word prefixes of the name; word order does not
  matter)

#### Scenario: A space splits words
- **GIVEN** a record «李雷 韩梅梅»
- **WHEN** the query «梅»
- **THEN** the record is found

#### Scenario: Hiragana and kanji run together: by character
- **GIVEN** a record «こんにちは世界»
- **WHEN** the query «世界»; the query «は»
- **THEN** in both cases the record is found

#### Scenario: Katakana is one word
- **GIVEN** a record «東京タワー»
- **WHEN** the query «タワ»; the query «ワー»; the query «京»
- **THEN** the first and the third find the record; «ワー» does not: it is not a prefix of the word «タワー»

#### Scenario: Thai script: a letter with marks
- **GIVEN** a record «Somchai สมชาย» and a record «สวัสดี» (U+0E2A U+0E27 U+0E31 U+0E2A U+0E14 U+0E35)
- **WHEN** the query «ชาย»; the query «วั» (U+0E27 U+0E31); the query «somchai»
- **THEN** the first finds the first record (words «ช», «า», «ย»); the second — the second; the third — the first

#### Scenario: Lao and Myanmar scripts
- **GIVEN** records «ສະບາຍດີ» and «မြန်မာ»
- **WHEN** the query «ບາ»; the query «မာ»
- **THEN** each finds its own record

#### Scenario: Change of script with no space
- **GIVEN** records «李雷Smith» and «Annиван»
- **WHEN** the query «smith»; the query «иван»
- **THEN** the first finds the record (there is a boundary between «雷» and «Smith»); the second does not: Latin and Cyrillic
  (ALetter) are one word «annиван»

### Requirement: Word-prefix search
The system SHALL perform a search by the words of the name as follows: a record is found if for every query word there is a
word of the name whose key starts with the key of this query word (comparison by code points of the keys). The order of the
query words does not matter; one word of the name may cover several query words; the query words need not match different
words of the name. A match inside a word (a substring) is not a match. Trace: CUJ5, CUJ8; ADR 0008 items 1–3.

#### Scenario: Start of a word, not a substring
- **GIVEN** records «Ann Smith», «John Annabel», «Joann»
- **WHEN** the query «ann»
- **THEN** the results contain «Ann Smith» and «John Annabel»; «Joann» is not there

#### Scenario: All query words (AND), order does not matter
- **GIVEN** a record «Ann Smith»
- **WHEN** the query «smith ann»; the query «ann jones»
- **THEN** in the first case the record is found, in the second it is not

#### Scenario: One word of the name covers several query words
- **GIVEN** a record «Annabel»
- **WHEN** the query «an ann»
- **THEN** the record is found

#### Scenario: A repeated word in the query
- **GIVEN** a record «Ann Smith»
- **WHEN** the query «ann ann»
- **THEN** the record is found

#### Scenario: The query is normalized at the edges
- **GIVEN** a record «Ann Smith»
- **WHEN** the query «  ann  » or «ann   smith» (several spaces between words)
- **THEN** the record is found: the query is normalized like a name (`records`, "Name and query normalization": NFC, then
  edge trimming), the spaces between words are word boundaries

### Requirement: Empty word-search query is an error
The system SHALL respond with a value error (query, "empty") — "empty query" — to a word-search query that has no words at
all after normalization (`records`, "Name and query normalization"): empty, of whitespace characters, of punctuation, "_",
of control characters or invisible format characters (Cf). A query of symbols or emoji is not empty: these are words (owner
decision, OD-5, D-19). There are no results in this case. Trace: CUJ5, CUJ8; ADR 0008 item 6.

#### Scenario: Punctuation only
- **WHEN** the query «- '»
- **THEN** the "empty query" error

#### Scenario: Empty string
- **WHEN** the query is empty or consists of spaces
- **THEN** the "empty query" error

### Requirement: Too-long query is an error
The system SHALL respond with a value error (query, "too long") to a query of either search whose length, per the
requirement "Text length and length cap" of the `records` capability, exceeds the cap: the normalized query text (NFC, edge
trimming), before the comparison key, the cap is 200 scalar values; the query is not truncated. The number of words is not
limited separately (for scripts without spaces — up to 200 words: each character is a word). Trace: CUJ5, CUJ7, CUJ8; ADR
0034, 0022.

#### Scenario: A query longer than the cap
- **WHEN** the query is longer than the cap (after NFC and trimming)
- **THEN** an error (query, "too long"), no results

#### Scenario: A query exactly at the cap
- **WHEN** the query consists of exactly 200 scalar values after normalization
- **THEN** the search is performed, there is no error

### Requirement: Control characters in the query are ordinary characters
The system SHALL NOT check name validity (`records`, "Name validity") for the query: a control character (Cc) in the query
causes no error. In word search it gives no word (Cc is not a word), that is, it splits words; in exact search it is
compared as is, and there is no name equal to such a query. Edge whitespace control characters are trimmed. Trace: CUJ5,
CUJ7; ADR 0034, 0021.

#### Scenario: Tab between words
- **GIVEN** a record «Ann Smith»
- **WHEN** the word query «Ann» + TAB + «Smith»
- **THEN** the record is found, no error

#### Scenario: A control character in exact search
- **GIVEN** a record «Ann Smith»
- **WHEN** the exact search «Ann» + TAB + «Smith»; the exact search «Ann» + U+0000
- **THEN** the results are empty, no error

#### Scenario: A query of a single control character
- **WHEN** the word query U+0000 (not whitespace, not trimmed)
- **THEN** an error (query, "empty"): there are no words

### Requirement: Request check order
The system SHALL check the search query only by the values themselves and report the first violation in the order: "too
long" (query, length per `records`), then "empty" (for word search — no words; for exact — empty after normalization), then
"cursor invalid"; the cursor is checked only for a valid query. Trace: CUJ5, CUJ7, CUJ8; ADR 0034.

#### Scenario: A long query without words
- **WHEN** the word query of 300 hyphens
- **THEN** an error (query, "too long"), not "empty"

#### Scenario: An empty query and a bad cursor
- **WHEN** the query is empty, and the cursor does not decode
- **THEN** an error (query, "empty")

#### Scenario: A too-long query and a bad cursor
- **WHEN** the query is longer than the cap, and the cursor belongs to another query
- **THEN** an error (query, "too long")

### Requirement: Exact whole-name search
Owner decision (OD-5, D-9): the system SHALL, on an exact-search query, return all records whose name comparison key equals
the comparison key of the normalized query (`records`, "Name and query normalization": NFC and edge trimming). Words are
not separated, inner spaces are compared as they are («Ann  Smith» with two spaces is not equal to «Ann Smith»). There are
no partial matches. Trace: CUJ7, CUJ8; ADR 0008 item 8, ADR 0031.

#### Scenario: Case, "ё", trimming
- **GIVEN** records «Ёлка» and «ЕЛКА»
- **WHEN** the exact search « елка »
- **THEN** both records are found

#### Scenario: Only the whole name
- **GIVEN** a record «Ann Smith»
- **WHEN** the exact search «ann»
- **THEN** the results are empty

#### Scenario: Inner spaces are significant
- **GIVEN** a record «Ann  Smith» (two spaces)
- **WHEN** the exact search «Ann Smith»
- **THEN** the results are empty

#### Scenario: Punctuation in the name
- **GIVEN** a record with the name «-»
- **WHEN** the exact search «-»
- **THEN** the record is found

### Requirement: Empty exact-search query is an error
Owner decision (OD-5, D-9): the system SHALL respond with a value error (query, "empty") — "empty query" — to an
exact-search query that is empty after normalization (`records`, "Name and query normalization"): empty or only whitespace
characters. A query of punctuation, "_" or a single control character (not whitespace) is not empty. Trace: CUJ7, CUJ8; ADR
0008 item 8, ADR 0031.

#### Scenario: Spaces
- **WHEN** the exact search «   »
- **THEN** the "empty query" error

### Requirement: Result order of both searches
The system SHALL return the records of both searches in the order defined by the `result-ordering` capability (ADR 0009):
name under ru locale rules, then date DESC, then number; the order is total. Pages follow one another in this order. Trace:
CUJ5, CUJ7, CUJ8; ADR 0009, 0031.

#### Scenario: One order for two searches
- **GIVEN** records «ANN» and «Ann» (one name by the key), on different numbers and dates
- **WHEN** the exact search «ann» and the word search «ann» are performed
- **THEN** the records in both responses go in the same order

### Requirement: "Next" pagination by cursor
The system SHALL return the results of both searches (for exact — owner decision, OD-5, D-9) in pages, without reporting the
total number of results. If after a page, in the result order at the moment of the response, there are more records, the
response SHALL contain a cursor for the next page; if not — no cursor. A cursor is not shown on an empty page, and a page
without a cursor is the last. Trace: CUJ5, CUJ7, CUJ8; ADR 0008 item 7, ADR 0032.

#### Scenario: Next page
- **GIVEN** there are more results than the page size
- **WHEN** a client requests the next page by the cursor from the previous response
- **THEN** it receives the records that come in the result order right after the last record of the previous page
- **AND** the total number of results is reported nowhere

#### Scenario: Last page
- **GIVEN** there are exactly as many results as fit on a page
- **WHEN** this page is requested
- **THEN** there is no cursor in the response

### Requirement: Cursor is a position, not a snapshot
The system SHALL understand a cursor as a position in the result order (the last record returned) and compute the next page
from the data at the moment this page is requested; there is no snapshot between pages. A matching record that did not
change during the traversal SHALL appear in the traversal exactly once; a changed or added one is shown if by the data at
the moment of the request it matches and stands after the cursor (the previous copy on an already returned page is
allowed); the result order is not violated. Trace: CUJ5, CUJ12; ADR 0032.

#### Scenario: Add between pages
- **GIVEN** a client received the first page and a cursor
- **WHEN** before the second page is requested a matching record is added that in the result order stands after the cursor
- **THEN** the second page contains it
- **AND** a record added before the cursor does not appear on the second page

#### Scenario: Delete between pages
- **GIVEN** a client received the first page and a cursor
- **WHEN** a record that stands after the cursor, or the very record the cursor stopped on, is deleted
- **THEN** the second page does not contain the deleted record, the others are not skipped or repeated

#### Scenario: A name change between pages
- **GIVEN** a record is already shown on the first page
- **WHEN** its name is changed so that it now stands after the cursor and still matches the query
- **THEN** the following pages contain it exactly once, with the new name (by the data at the moment of the page request)
- **AND** in the traversal as a whole it occurs twice: on the first page with the previous name and later with the new one;
  the other matching records are not skipped or repeated

### Requirement: Cursor contents and "the same query"
The system SHALL issue a cursor as an opaque value with a position (the last record returned), the search kind, the query
fingerprint and a checksum. "The same query" is the same search kind and the same fingerprint of the normalized text
(`records`): for word search — equal sets of word keys, for exact — an equal query comparison key. The cursor does not
depend on the page size. Trace: CUJ5, CUJ7, CUJ8; ADR 0032 items 3, 6.

#### Scenario: Equivalent queries
- **GIVEN** a cursor from the query «ann smith»
- **WHEN** it is passed with the query «SMITH  Ann», or «ann ann smith»
- **THEN** the cursor is accepted

#### Scenario: NFC and NFD forms of one query
- **GIVEN** a cursor from the query «Ёлка» (NFC)
- **WHEN** it is passed with the same query in NFD form («Е» and U+0308 and «лка»)
- **THEN** the cursor is accepted: the fingerprint is computed from the normalized text; for word search the order,
  repeats, case, "ё" and the number of spaces between words do not affect the fingerprint

#### Scenario: Exact search: by the key of the whole name
- **GIVEN** a cursor from the exact search «Ann Smith»
- **WHEN** it is passed with the exact query « ann smith »
- **THEN** the cursor is accepted
- **AND** with the query «ann  smith» (two spaces) — a "cursor invalid" error

#### Scenario: Page size does not matter
- **GIVEN** a cursor obtained with a page size of 50
- **WHEN** the next page is requested with a different size (only if the interface lets the client set it)
- **THEN** the cursor is accepted and the page starts right after the cursor position

### Requirement: Cursor lifetime and validity
The system SHALL accept a cursor that passed the format and integrity checks and belongs to the same search and the same
query, regardless of data changes, time, restart, an update within v1 and a restore from a backup. Any other cursor SHALL
be rejected with a "cursor invalid" error: it does not decode, the checksum does not match, another search kind or another
query. A correct cursor composed by the client is no different from an issued one. Trace: CUJ5, CUJ11, CUJ14; ADR 0032,
0034.

#### Scenario: A data change does not spoil the cursor
- **GIVEN** a client holds a cursor
- **WHEN** the data changed, including deletion of the record the cursor stopped on
- **THEN** the cursor is accepted

#### Scenario: A cursor from another query
- **GIVEN** a cursor from the query «ann»
- **WHEN** it is passed with the query «smith», or to exact search
- **THEN** a "cursor invalid" error

#### Scenario: A cursor with a bad format
- **WHEN** a cursor is passed that is empty, truncated, with an appended tail or not decodable
- **THEN** a "cursor invalid" error

#### Scenario: A cursor with broken integrity
- **GIVEN** an issued cursor
- **WHEN** a byte of the decoded contents is changed so that the checksum does not match
- **THEN** a "cursor invalid" error

#### Scenario: A correctly composed cursor is accepted
- **GIVEN** a client composed by the format a value that passed all checks and belongs to the same query
- **WHEN** it is passed to a search
- **THEN** the results begin right after the position it encodes, in the result order
- **AND** there is no error: there is no signing secret, the cursor opens no one else's data

#### Scenario: Server restart
- **GIVEN** a client holds a cursor
- **WHEN** the server is restarted or updated within v1
- **THEN** the cursor is accepted and gives the next page by the data at the moment of the request

### Requirement: Valid cursor position and shape
The system SHALL consider a cursor invalid if the position in it (name, date, number) does not satisfy the `records` value
rules: the name is valid, normalized and not longer than the cap; the number is in canonical form; the date exists and is in
range; and also if the cursor is longer than the encoding of the longest valid position or its contents have another string
representation. Trace: CUJ5, CUJ14; ADR 0032 item 4.

#### Scenario: A position that violates the value rules
- **GIVEN** a value with a correct checksum, search kind and query fingerprint
- **WHEN** the position contains an empty name, a name with U+0000, a name not in NFC, a name longer than 200, the date
  2021-02-30 or a number not in canonical form
- **THEN** a "cursor invalid" error

#### Scenario: One content, one representation
- **GIVEN** an issued cursor
- **WHEN** another string representation of it with the same decoded contents is passed (for example, with nonzero unused
  bits)
- **THEN** a "cursor invalid" error

### Requirement: Query fingerprint is stable within v1
The system SHALL compute the query fingerprint with a deterministic function of the set of word keys (word search) or the key
(exact search) and the search kind, whose result does not change within v1 across restart, a change of platform, compiler
and library versions (a random per-process hash is not suitable). Trace: CUJ5, CUJ7, CUJ11, CUJ14; ADR 0032 items 3, 4.

#### Scenario: A cursor survives a release
- **GIVEN** a cursor issued by a v1 build before an update within v1
- **WHEN** it is passed with the same query to the updated build or to another platform
- **THEN** the cursor is accepted

#### Scenario: Reference vectors of the first release
- **GIVEN** normative vectors "query → set of word keys → fingerprint → cursor", fixed at the first v1 release (the
  fingerprint algorithm and the encoding are chosen by the API area, ADR 0032 item 3)
- **WHEN** the product test computes the fingerprint and encodes the cursor for each query of the vector
- **THEN** the result is byte-for-byte equal to the vector; a change of even one vector within v1 is not released
- **AND** the vector includes queries with repeats and different word order («ann smith», «SMITH  Ann», «ann ann smith» —
  one fingerprint), exact search and «ё»+U+0308

### Requirement: Number version in name-search result records
The system SHALL include in every record of the results of both name searches the version of its number per the requirement
"Number version in search results" of the `write-operations` capability: the version belongs to the whole number and is
consistent with the records of this number on the same page. Each result record contains the number, date and name: the date
is needed for rename and delete (owner decision, OD-5, Q-90; ADR 0025 item 6). Trace: CUJ6, CUJ12, X-5; ADR 0029, 0026.

#### Scenario: Version in word-search results and in exact search
- **GIVEN** a number with three records, one of which is found
- **WHEN** a word search and an exact search are performed
- **THEN** the record contains the version of the whole number; to delete from the results (CUJ6) it need not be re-read

#### Scenario: A result record contains the date
- **GIVEN** a record (number, 1876-03-07, «Ann») found by a word search or an exact search
- **WHEN** the results are formed
- **THEN** the record contains the number, the date 1876-03-07 and the name, and a delete from the results (CUJ6) passes
  this date without re-reading

#### Scenario: Two records of one number on a page
- **GIVEN** a page contains two records of one number
- **WHEN** the results are formed
- **THEN** they have the same version, corresponding to the state of the number from which they were taken

### Requirement: Page size
The system SHALL return a page of the size the client set (if the interface lets it be set), otherwise of 50 records (owner
decision, OD-5, D-10); a page is full if there are more records after it, and only the last may be shorter. Only a page after
which there is nothing may be empty. The valid size and the maximum belong to the interface area. Trace: CUJ5, CUJ7, CUJ8;
ADR 0033.

#### Scenario: Full page by default
- **GIVEN** 120 matching records, no size set
- **WHEN** a client traverses the results by cursor
- **THEN** pages of 50, 50 and 20 records; on the last there is no cursor

#### Scenario: A set size
- **GIVEN** 120 matching records, the interface lets a size be set, 40 is set
- **WHEN** a client traverses the results by cursor
- **THEN** pages of 40, 40 and 40 records; on the last there is no cursor

### Requirement: No matches is not an error
The system SHALL respond to a word search and an exact search with no matches with successful empty results without a
cursor, not with an error. How empty results look to a human is a UI question (Q-29). Trace: CUJ5, CUJ7, CUJ8; ADR 0034.

#### Scenario: Nothing found
- **WHEN** no name matches the query
- **THEN** success, no records, no cursor

### Requirement: Search does not depend on the server version within v1
The system SHALL give, for the same data and query, the same matches and the same order before and after a server update
within v1: the comparison key and word boundaries are determined by Unicode 18 and the ICU4X tables and code pinned for all
of v1. A change of matches or order is not released in v1; otherwise only by the owner revisiting OD-4 item 2 (ADR 0039
item 8). Trace: CUJ5, CUJ7, CUJ8, CUJ11; ADR 0010, 0030, 0039.

#### Scenario: The ICU4X code is not updated
- **WHEN** a server update within v1 is released
- **THEN** the code and the Word_Break, Extended_Pictographic, case and normalization tables are the same; a change of
  matches or order is not released in v1 (ADR 0039 item 8); dependency fixes are allowed only without changing results (the
  requirement "Dependency fixes in v1 do not change order or search" of the `result-ordering` capability, ADR 0039 item 6)

#### Scenario: Server update
- **GIVEN** a set of records and queries recorded before a server update within v1
- **WHEN** the same queries are run after the update
- **THEN** the matches and the order are the same

#### Scenario: The same result for different clients
- **WHEN** a local program and a network service with the read permission make one query
- **THEN** the results are identical; the searches only read data

### Requirement: Word-boundary and key references run through product code
The system SHALL have a test that runs the scenarios of the reference `out-uax29-golden.txt` and WordBreakTest-18.0.0 through
the product's own functions — word segmentation, the comparison key and the "word" predicate; the `research/name-search`
programs serve as a prototype that produces the reference and replace no product check. A discrepancy of the product output
from the reference blocks the release. Trace: CUJ5, CUJ7, CUJ11; ADR 0030 item 3, ADR 0039 item 5.

#### Scenario: The reference passes through product functions
- **WHEN** the test feeds the product functions the reference lines (Han, kana, Thai, apostrophes, "_", U+202F, etc.) and
  the 1944 lines of WordBreakTest-18.0.0
- **THEN** the boundaries and the "word" flag are byte-for-byte equal to the reference

#### Scenario: A change of the rules or of the key steps is noticed
- **GIVEN** a product in which the order of the key steps is changed (for example, "ё"→"е" before folding) or the boundary
  rule
- **WHEN** the test is run
- **THEN** the test fails, although `research/name-search` did not change
