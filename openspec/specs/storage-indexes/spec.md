# storage-indexes Specification

## Purpose
Indexes of the own engine: search by number, name results in order with continuation from any position, word-prefix search
without full materialization of matches and exact whole-name search; keys, the format label, rebuild, and size and
latency targets at 10^7 records. The semantics of data and searches is accepted in `records`, `number-search`, `name-search`, `result-ordering`.

## Requirements

### Requirement: Indexes are derived data of records
The system SHALL store records (number, date, name) and number versions as the only source of truth; the order, word,
exact-search and word dictionary indexes SHALL be a function of these records and SHALL allow a full recomputation. A record and all its
index keys SHALL change in one atomic operation. Trace: CUJ5, CUJ14; ADR 0045 items 1, 8; ADR 0039 item 3.

#### Scenario: Index equals a function of the records
- **GIVEN** a database after any sequence of adds, renames and deletes
- **WHEN** the key set of each index is compared with the set recomputed from the records
- **THEN** the sets are equal

#### Scenario: Failure in the middle of an operation
- **GIVEN** an operation changes a record and its index keys
- **WHEN** the process is interrupted at any point before acknowledgment
- **THEN** after a restart either all changes of the record and indexes are present, or none (CUJ14)

### Requirement: Number version from the same snapshot
The system SHALL read the records of a number and the header of its version from one consistent snapshot; a name results page SHALL
obtain the version of each number by a point read from the same snapshot as the records of the page. Trace: CUJ6, CUJ9, X-5;
ADR 0046 items 3–4; ADR 0029.

#### Scenario: Two records of one number on a page
- **GIVEN** a results page with two records of one number, the number is modified concurrently
- **WHEN** the page is built
- **THEN** both records have one version, corresponding to one state of the number

### Requirement: Number history stream without truncation
The system SHALL deliver up to 2 967 164 records of a number as a single stream from a snapshot started when the search starts, with server memory
independent of the size of the results, without blocking writes to the number. Trace: CUJ4, CUJ9; ADR 0046 item 3; ADR 0035 item 6.

#### Scenario: Number on all valid dates
- **GIVEN** a number with records for each of the 2 967 164 days
- **WHEN** search by number
- **THEN** 2 967 164 records are returned in date DESC order; server memory does not grow with the number of records

#### Scenario: Number change during the stream
- **GIVEN** a number stream has started
- **WHEN** another client changes the number
- **THEN** the stream contains the state at the moment of start, the client's write does not wait for the end of the stream

### Requirement: Bytewise order of the order key equals result order
The system SHALL encode the order key so that bytewise comparison of keys equals the order (name ru, date DESC, number
as a string) for any sort keys, including when one key is a prefix of another or contains the byte 0x00. Trace: CUJ5;
ADR 0045 items 4–5 (Q-109); ADR 0036.

#### Scenario: Sort key is a prefix of another
- **GIVEN** two records, the sort key of the first is a proper prefix of the key of the second
- **WHEN** their order keys are compared
- **THEN** the first comes earlier; with equal sort keys, date DESC decides, then the number, independently of the bytes of the sort key

#### Scenario: Key with byte 0x00
- **GIVEN** an artificial sort key with the byte 0x00
- **WHEN** the order keys are compared
- **THEN** the order equals the order of tuples (sort key, date DESC, number)

#### Scenario: Consistency with the comparison function
- **WHEN** the build test runs the order corpus (ADR 0039 item 5 (b))
- **THEN** the byte order of the order key equals the result of name comparison for every adjacent pair; the presence of the byte 0x00 in the corpus
  sort keys is reported in the report

### Requirement: Keys of any length without truncation
The system SHALL store the sort key of a valid name of up to 8 513 bytes and any mix of it, as well as the word and comparison keys, without truncation and without refusal within the tree key limit of 65 535 bytes
(`storage-format`, "Key and value: limits, spill to overflow"); a larger key SHALL reject the operation, not be truncated. Trace: CUJ1, CUJ5; ADR 0045 item 6 (Q-48, Q-103); ADR 0022 item 5.

#### Scenario: Name of 200 U+FDFA characters
- **GIVEN** the valid name "U+FDFA × 200" (sort key 8 513 bytes)
- **WHEN** it is added and found by word search and by exact search
- **THEN** the record is in the results; the order equals the comparison order; no key is truncated

#### Scenario: Key above the limit
- **WHEN** a test passes a key of 65 536 bytes to the index layer (unreachable for valid names: the additive bound is about 20 400 bytes)
- **THEN** the operation is rejected as a whole, data and version are unchanged, the key is not truncated, the "storage error" state does not occur

#### Scenario: Ordinary keys do not use overflow
- **GIVEN** names of up to 100 Cyrillic and Latin characters
- **WHEN** they are added
- **THEN** no index key is spilled to overflow pages

### Requirement: Name results in order with continuation from any position
The system SHALL return records by name in order key order and continue from the strictly next record after the cursor
position (name, date, number) with a single "first record after the key" search and a forward read, without reading or repeating earlier
pages; the position of a deleted record is valid. Trace: CUJ5, CUJ12; ADR 0047 items 1–3; ADR 0032, 0036.

#### Scenario: Continuation does not depend on the page number
- **GIVEN** results of 10^6 records, a cursor at position 900 000
- **WHEN** a page is requested
- **THEN** the number of pages read is the same as with a cursor at position 50 (descent plus forward read)

#### Scenario: Position deleted
- **GIVEN** a cursor on a record deleted after the results were returned
- **WHEN** the next page is requested
- **THEN** the page starts with the first record after the position, nothing is skipped

#### Scenario: Alphabetically equal names
- **GIVEN** records "Ann" (2024-01-01), "A­nn" (U+00AD, 2023-01-01), "Ann" (2022-01-01)
- **WHEN** a search that finds all three
- **THEN** the order is by date descending

### Requirement: Exact whole-name search without materialization
The system SHALL find exact-search records through a separate index of the name comparison key, return them in order key
order with a cursor, check the record's comparison key against the query key (a hash collision does not yield a false record) and fill the page
until it is full. Trace: CUJ7, CUJ8; ADR 0047 item 4; ADR 0031.

#### Scenario: Frequent name
- **GIVEN** the name "Иван" in 10^6 records
- **WHEN** the first page and a page from the middle by cursor are requested
- **THEN** each contains 50 records and reads about 50 data records plus 1–2 index leaves; the range is not materialized

#### Scenario: Hash collision
- **GIVEN** two names with different comparison keys and an equal hash
- **WHEN** exact search for one of them
- **THEN** the record of the other is absent from the results, the page is filled with the following matching ones

### Requirement: Word-prefix search: result independent of method
The system SHALL return records for which every query word has a name word with that prefix of the key, in order key order, and the result SHALL NOT depend on the method of obtaining candidates (order traversal, posting intersection; posting merge — by profile) or on switching the method in the middle of the results. A query word that is a prefix of another does not change the result. Trace: CUJ5, CUJ8; ADR 0048 items 2–3 (OD-7); ADR 0008, 0030.

#### Scenario: All methods give one result
- **GIVEN** a set of queries (rare, frequent, short prefixes, 1–200 words) on one database
- **WHEN** each is executed by each applicable method (order traversal, intersection, posting merge, if introduced) from the same positions
- **THEN** the results are bytewise equal to the definition (`name-search`, "Word-prefix search")

#### Scenario: Redundant prefix
- **GIVEN** the record "Annabel"
- **WHEN** query "an ann annab"
- **THEN** the record is found, just as for "annab"

#### Scenario: One name word covers several query words
- **GIVEN** the record "Annabel"
- **WHEN** query "an ann"
- **THEN** the record is found

### Requirement: First page without full materialization of matches
The system SHALL prepare a word search page keeping in memory at most 131 072 postings of one prefix plus the page,
and without accumulating the matches of the whole query; continuation SHALL be performed without server state and without reading earlier
pages. Trace: CUJ5, CUJ8; ADR 0048 item 4; ADR 0032.

#### Scenario: Frequent word
- **GIVEN** a word that begins 10^6 names
- **WHEN** the first page
- **THEN** query memory does not depend on 10^6; a page of 50 records is collected by scanning the order forward (a few hundred records expected)

#### Scenario: Cursor after a restart
- **GIVEN** a cursor issued before a server restart
- **WHEN** the next page is requested
- **THEN** the page is obtained from the data as of the request, without the earlier state

### Requirement: Word search always completes with a result, with no cap on word count
The system SHALL accept a word search query with any number of words up to 200 and always complete a valid query with a result: a full page or the end of the results; there is no search deadline and no refusals by volume; the exceptions are `snapshot_revoked` and request timeout. Planning SHALL cost ≈ 2 × tree depth plus ≤ 4 dictionary leaves per non-redundant prefix: the dictionary is a tree without aggregates, counting words and postings is a scan that stops at a threshold. Trace: CUJ5, CUJ8, X-1; ADR 0048 item 6, 0045 item 3 (OD-7).

#### Scenario: 200-word query
- **GIVEN** a database of names of 200 Han ideographs, a query of 200 distinct characters
- **WHEN** the search is executed
- **THEN** it completes within the class target, and the record whose name contains all the characters is found

#### Scenario: Heavy query completes with a result
- **GIVEN** a query of several very frequent, mutually exclusive prefixes
- **WHEN** the search is executed
- **THEN** it completes without a search error (snapshot revocation per `storage-snapshots` and request timeout are exceptions): with a page of records, if there are any, or with empty results; a page with fewer records than the page size is not returned if records exist; the work is bounded by a traversal of the order index (≈ 1.4 s at 10^7, Q-114; scaled proportionally from ≈ 14 s at 10^8), the request timeout, if the API introduces one, belongs to area 3

#### Scenario: Heavy query does not interfere with writes
- **GIVEN** a heavy query is running
- **WHEN** write operations arrive
- **THEN** they are acknowledged within the `storage-durability` targets, and the query continues on its own snapshot

### Requirement: Index format label and rebuild at start
The system SHALL store a label at the index root (key and value format version, `icu_collator` version, word table version,
hash identifier) and, if it does not match the server's label, rebuild the order, word, exact-search and dictionary indexes from the
records before accepting requests; the result order after a rebuild with the same comparison function SHALL be unchanged. Trace: CUJ5, CUJ11,
CUJ14; ADR 0045 items 7–8; ADR 0039 item 3 (Q-82).

#### Scenario: Change of the key record format
- **GIVEN** a database with indexes of the old key format in our code, the comparison function unchanged
- **WHEN** the server starts with a new label
- **THEN** the indexes are rebuilt before requests are accepted; results on the query corpus before and after are bytewise equal

#### Scenario: Failure during rebuild
- **GIVEN** a rebuild is in progress
- **WHEN** the process is interrupted before the atomic swap of the roots
- **THEN** the next start sees the previous indexes and label and starts the rebuild again

#### Scenario: Rebuild time [DEFAULT]
- **GIVEN** a database of 10^7 records and a mismatched label
- **WHEN** the server rebuilds the order, word, exact-search and dictionary indexes before accepting requests
- **THEN** the rebuild takes at most 6 minutes (estimate 1–3 minutes; scaled proportionally from 60 and 10–30 minutes at 10^8; D-26 accepted by the owner, OD-6)

#### Scenario: Rebuild is not a way to change the order
- **GIVEN** a server with a different `icu_collator` version
- **WHEN** after a rebuild at least one reference of ADR 0039 item 5 gives a different result
- **THEN** the version is not accepted (the build fails, ADR 0039 item 6 (d)), the order of the database is not changed

### Requirement: Index size and latency targets at 10^7 records
The system SHALL meet the ADR 0049 targets on the reference corpus of 10^7 records (accepted by the owner, OD-6: the latency targets are the same numbers as for 10^8, but at 10^7):
file size — per the requirement "File size at 10^7 records" of `storage-format` (D-21); p99 of search by number up to 100 records ≤ 10 ms; pages of exact search and of search with a word of ≤ 10^4 postings
≤ 20 ms; pages of a single prefix of any frequency and of queries up to C ≤ 10^7 — ≤ 50 ms; heavy intersections ≤ 2 s. Trace: X-1,
CUJ4, CUJ5, CUJ8; ADR 0049 (Q-47, Q-59, Q-105); OD-6.

#### Scenario: Size on the reference corpus
- **GIVEN** 10^7 records of profile P0 (`storage-format`: name 22 characters, 2.5 words, 1.25 records per number)
- **WHEN** the database is filled with random inserts
- **THEN** the length of `data` is at most 8 × 10^9 bytes (D-21); the ADR 0049 item 1 calculation of the indexes (≈ 3.9 GB for their share with prefix compression) is
  an estimate, not a threshold

#### Scenario: Query classes
- **WHEN** on the same database the query classes of ADR 0049 item 2 are measured on the assumptions bench
- **THEN** the p99 of each class is no higher than its target; measurement results are per class, not a single overall p99

#### Scenario: Worst size per record
- **GIVEN** a valid name of 200 Han ideographs (200 words)
- **WHEN** the record is added
- **THEN** the record's indexes occupy about 180 KiB; out of space yields "read-only" (CUJ16; `storage-durability`, ""Read-only" state"), not truncation

### Requirement: Search by number without scanning records: read count
The system SHALL find the records of a number by the canonical string with a range read whose number of pages read does not
depend on the total number of records beyond the tree depth, and return them in date DESC order without sorting. Trace: CUJ4, CUJ8;
ADR 0046 items 1–2; ADR 0035.

#### Scenario: Number that is the beginning of another
- **GIVEN** numbers "+7495" and "+74951234567" with records
- **WHEN** search by "+7495"
- **THEN** only the records of "+7495" are returned, dates descending

#### Scenario: Read count at 10^7 records
- **GIVEN** a database of 10^7 records, the internal tree levels in the cache
- **WHEN** search for a number with ≤ 10 records
- **THEN** at most 2 data pages are read (at most 4 with an empty cache; at 10^7 3 tree levels are expected, to be confirmed by a prototype)
