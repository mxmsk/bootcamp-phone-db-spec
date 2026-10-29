# result-ordering

## Purpose
The observable order of name-search results: alphabetical under ru locale rules, then date DESC, then the number as a
string ascending; the order is total (suitable for a cursor) and does not change within v1, because a change of order is
not released in v1 (ICU4X pinning — ADR 0039). Name-comparison parameters: what follows from the ru rules — ADR 0037
([OURS]); what is visible and does not follow from them (case, punctuation, digits) — ADR 0038 (owner decision, OD-5).

## ADDED Requirements

### Requirement: Three keys of name-search order
The system SHALL order the records of name-search results by three keys in this order: the name alphabetically under the ru
locale rules; for equal names, the date in descending order; for equal name and date, the number ascending as a string.
There SHALL NOT be other keys or relevance scores. Trace: CUJ5; ADR 0009, 0036.

#### Scenario: Different names
- **GIVEN** the results contain records with the names «Борис», «Анна», «Виктор»
- **WHEN** a search by name is performed
- **THEN** the name order is: «Анна», «Борис», «Виктор»

#### Scenario: One name, different dates
- **GIVEN** the results contain two records «Анна» on the dates 2020-01-01 (number A) and 2024-01-01 (number B)
- **WHEN** a search by name is performed
- **THEN** first the record on 2024-01-01, then the one on 2020-01-01

#### Scenario: One name and date, different numbers
- **GIVEN** two records «Анна» on one date for the numbers «+2125550100» and «+1912555010»
- **WHEN** a search by name is performed
- **THEN** first «+1912555010», then «+2125550100»

#### Scenario: The name outweighs the date
- **GIVEN** a record «Борис» on 2030-01-01 and a record «Анна» on 2000-01-01
- **WHEN** a search by name is performed
- **THEN** «Анна» comes before «Борис», despite the earlier date

### Requirement: Whole-name comparison under ru locale rules
The system SHALL compare names under the sorting rules of the ru locale (CLDR, data version per ADR 0010) as whole stored
strings (in NFC), not by the word that produced the match; the sorting locale SHALL always be ru, regardless of the client,
the query language and the machine settings. Trace: CUJ5; ADR 0009, 0010, 0037.

#### Scenario: Cyrillic before Latin
- **GIVEN** the results contain the names «Ann» and «Яков»
- **WHEN** a search by name is performed
- **THEN** «Яков» comes before «Ann» (in ru, Cyrillic goes first, then the other scripts)

#### Scenario: Sorting by the whole name, not by the query word
- **GIVEN** the records «Bob Smith» and «Ann Smith»
- **WHEN** the query is «smith»
- **THEN** «Ann Smith» comes before «Bob Smith»: whole names are compared

#### Scenario: One order for all clients
- **WHEN** the same query is run by the UI and by API clients on machines with different languages and settings
- **THEN** the order of the results is the same

### Requirement: Placement of "ё" in the order
The system SHALL order "ё" under the ru rules as a variant of the letter "е" that goes right after it: with the other
letters equal, a name with "е" comes before a name with "ё", and the "е/ё" difference SHALL NOT outweigh a difference in
later letters. Trace: CUJ5; ADR 0037. Search "ё" = "е" (ADR 0008) does not affect the order.

#### Scenario: Е before Ё
- **GIVEN** the records «Ёлка» and «Елка»
- **WHEN** a search by name that finds both is performed
- **THEN** first «Елка», then «Ёлка»

#### Scenario: Ё does not outweigh the following letters
- **GIVEN** the records «Ежи», «Ёж», «Еж», «Её», «Жук»
- **WHEN** a search by name that finds all of them is performed
- **THEN** the order is: «Её», «Еж», «Ёж», «Ежи», «Жук» («Её» before «Еж»: the second letter "ё" as "е" goes before "ж";
  «Ёж» before «Ежи»: the comparison of «Еж» and «Ежи» is decided by length, not by "ё")

### Requirement: Role of case, punctuation and digits in name comparison
When all letters match, the system SHALL distinguish names by case, lowercase before uppercase («анна» before «Анна»);
treat spaces and punctuation as characters that go before letters and digits («Ann Zed» before «Anna»); compare digits as
characters, without numeric ordering («Ann 10» before «Ann 2»). These are owner decisions (OD-5, D-11, D-12, D-15; ADR
0038). Trace: CUJ5; ADR 0038.

#### Scenario: Case distinguishes only names equal in letters
- **GIVEN** the records «Анна», «анна» and «Антон»
- **WHEN** a search by name that finds all of them is performed
- **THEN** the order is: «анна», «Анна», «Антон» (case does not change the order of names that differ in letters)

#### Scenario: Space and hyphen
- **GIVEN** the records «Ann-Smith», «Ann Smith» and «AnnSmith»
- **WHEN** a search by name that finds all of them is performed
- **THEN** the order is: «Ann Smith», «Ann-Smith», «AnnSmith»

#### Scenario: A name with a space before a longer word
- **GIVEN** the records «Anna» and «Ann Zed»
- **WHEN** a search by name that finds both is performed
- **THEN** «Ann Zed» comes before «Anna»

#### Scenario: Digits are compared as characters
- **GIVEN** the records «Ann 2» and «Ann 10»
- **WHEN** a search by name that finds both is performed
- **THEN** «Ann 10» comes before «Ann 2» (there is no numeric ordering)

### Requirement: Names equal under alphabetic comparison are ordered by date
The system SHALL consider names that are indistinguishable under ru alphabetic comparison (for example, differing only in
an ignorable character) equal by the first order key and order the records of such names by date in descending order, then
by number, like records of one name; there SHALL NOT be a separate key by spelling (code points) between the alphabet and
the date. Trace: CUJ5; ADR 0009, 0036.

#### Scenario: Distinguishable only by an invisible character
- **GIVEN** the valid names «Ann» and «A­nn» (with a soft hyphen U+00AD, not distinguished by ru alphabetic
  comparison); records: «Ann» on 2024-01-01, «A­nn» on 2023-01-01, «Ann» on 2022-01-01
- **WHEN** a search by name that finds all three records is performed
- **THEN** the record order is: «Ann» 2024-01-01, «A­nn» 2023-01-01, «Ann» 2022-01-01 (the dates descend; records
  are not grouped by spelling)

#### Scenario: Names equal alphabetically with one date
- **GIVEN** the records «Ann» and «A­nn» on one date for the numbers «+2125550100» and «+1912555010»
- **WHEN** a search by name is performed
- **THEN** the record of the number «+1912555010» comes first (decided by the third key, not by the spelling of the name)

### Requirement: Number in the third key is compared as a string
The system SHALL compare the numbers in the third key as canonical strings (the `records` capability) by code points, left
to right; a shorter string that is the beginning of a longer one comes first. Numeric comparison of numbers SHALL NOT be
applied. Trace: CUJ5; ADR 0009, 0036.

#### Scenario: "+19…" before "+2…"
- **GIVEN** two records of one name on one date for the numbers «+19001234» and «+2001234»
- **WHEN** a search by name is performed
- **THEN** the record of the number «+19001234» comes first

#### Scenario: A short number before one that continues it
- **GIVEN** two records of one name on one date for the numbers «+7495» and «+74951234567»
- **WHEN** a search by name is performed
- **THEN** the record of the number «+7495» comes first

#### Scenario: Numbers of one country are adjacent
- **GIVEN** records of one name on one date for the numbers «+7495», «+4915», «+7916» and «+4930»
- **WHEN** a search by name is performed
- **THEN** the order is: «+4915», «+4930», «+7495», «+7916»

### Requirement: Order is total and independent of query and page
The system SHALL order any two distinct records of the results unambiguously, by the same order in all results, including
word search and exact whole-name search: the place of a record relative to another does not depend on the query text, the
set of records found or the page size. A cursor is a position in this order (ADR 0032). Trace: CUJ5, CUJ7; ADR 0009, 0031,
0036.

#### Scenario: Distinct records are not equal
- **WHEN** two distinct records (different number or date) are compared
- **THEN** one is strictly before the other; there are no records "in the same place" (a record is identified by the pair
  "number, date")

#### Scenario: The relative order does not depend on the query
- **GIVEN** the records X and Y are found by both the query «ann» and the query «smith»
- **WHEN** both searches are performed
- **THEN** X and Y go in the same relative order in both results

#### Scenario: Pages together give the total order
- **GIVEN** the data did not change between page requests
- **WHEN** a client reads all pages in sequence by cursor
- **THEN** the union of the pages is the same list in the same order as a single result, without repeats or gaps

### Requirement: Order does not change within v1
The system SHALL return the same results in the same order for the same data and queries on any server version within v1.
A change of order is not released in v1 (ADR 0039 item 8); the system SHALL NOT promise an unchanged order under an
arbitrary library update beyond this. Stored sorting data SHALL NOT determine the order otherwise than the comparison
function of this specification. Trace: CUJ5, CUJ11; ADR 0010, 0039 items 2, 8 (Q-74).

#### Scenario: The same order on another v1 version
- **GIVEN** a database snapshot and a set of queries run on one v1 server version
- **WHEN** the same queries are repeated on the same data on another v1 server version
- **THEN** the order of records in each result is byte-for-byte the same

#### Scenario: The order from the examples of this specification
- **WHEN** any v1 server version compares the names from the scenarios of this capability
- **THEN** the result equals the one stated in the scenarios

#### Scenario: A cursor issued on another v1 version
- **GIVEN** a client received a cursor from a server of one v1 version
- **WHEN** a server of another v1 version receives a next-page request
- **THEN** the cursor position means the same point of the total order (ADR 0032)

### Requirement: Dependency fixes in v1 do not change order or search
Within v1 the system SHALL release fixes to the Unicode libraries it uses (for example, because of a vulnerability) only as
a backport into the pinned version without changing the data or the comparison, verified by the references and the order
corpus (ADR 0039 items 5, 6 (b)). A change of the `icu_*` version in v1 SHALL NOT be released (owner decision, OD-5, D-18,
ADR 0039 item 6 (c)); a change of order or matches is not released in v1 (ADR 0039 item 8). Trace: CUJ5, CUJ11; ADR 0010,
0039.

#### Scenario: A backport with no change of behavior
- **GIVEN** a Unicode library fix closing a vulnerability was released and was carried over into the pinned version without
  changing the data
- **WHEN** the reference sets and the order corpus after it give a byte-for-byte identical result
- **THEN** the fix may be released in v1; the order and matches for clients are the same

#### Scenario: A fix changes the order
- **WHEN** after the fix any reference set or the order corpus gives a different result, or the fix cannot be separated
  from a change of comparison
- **THEN** no release is made in v1: the fix goes to v2 with a migration, or the owner revisits OD-4 item 2

#### Scenario: A library version change is not released without a revision
- **GIVEN** a new `icu_*` version was released, and all references match with it
- **WHEN** the release in v1 is decided, and the owner rejected the update admission by match (D-18, alternative 1) (OD-5)
- **THEN** the version change is not released in v1; only a backport into the pinned version is allowed

### Requirement: Name-comparison parameters and reference checks by the product
The system SHALL compare names with `icu_collator` `=2.4.0`, ru, `case_level` Off, `max_variable` default, without
Identical ([OURS], ADR 0037 item 4, ADR 0039 item 4), and under the owner decisions (OD-5): D-11 `strength` Tertiary, D-12
NonIgnorable, D-15 numbers as characters, `case_first` default for ru (ADR 0038). The references and the order corpus SHALL
be checked by the product code, including the order by the sort key bytes (ADR 0039 item 9 (d)). Trace: CUJ5, CUJ11; ADR
0037, 0038, 0039.

#### Scenario: A parameter change is caught by a product test
- **GIVEN** a collator parameter is changed in the product (for example, `alternate_handling` to Shifted)
- **WHEN** the build test with the ADR 0039 item 5 references is run
- **THEN** the test fails, the release is blocked (the `research/` programs by themselves do not notice such a change)

#### Scenario: The sort key and the comparison are consistent
- **WHEN** the build test runs the order corpus over adjacent pairs
- **THEN** the order by the sort key bytes matches the result of the comparison function for each pair
