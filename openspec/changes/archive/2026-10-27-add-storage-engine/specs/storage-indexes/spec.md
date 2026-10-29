# storage-indexes Specification

## Purpose
Indexes of the dedicated engine: search by number, results by name in order with continuation from any position, word-prefix search
without full materialization of matches, and exact whole-name search; keys, the format label, rebuild, and size and
latency targets at 10^8 records. The semantics of data and searches was accepted in `records`, `number-search`, `name-search`, `result-ordering`.

## ADDED Requirements

### Requirement: Indexes are derived data of records
The system SHALL store records (number, date, name) and number versions as the sole source of truth; the order, word,
exact-search and word-dictionary indexes SHALL be a function of these records and SHALL allow a full recomputation. A record and all its
index keys SHALL change in one atomic operation. Trace: CUJ5, CUJ14; ADR 0045 items 1, 8; ADR 0039 item 3.

#### Scenario: Index equals a function of records
- **GIVEN** a database after any sequence of adds, renames and deletes
- **WHEN** the key set of each index is compared with the set recomputed from the records
- **THEN** the sets are equal

#### Scenario: Failure in the middle of an operation
- **GIVEN** an operation changes a record and its index keys
- **WHEN** the process is interrupted at any point before acknowledgment
- **THEN** after restart either all changes of the record and indexes are present, or none (CUJ14)

### Requirement: Search by number without scanning records
The system SHALL find the records of a number by the canonical string by reading a range whose page count does not
depend on the total number of records beyond the tree depth, and SHALL return them by date DESC without sorting. Trace: CUJ4, CUJ8;
ADR 0046 items 1–2; ADR 0035.

#### Scenario: A number that is the beginning of another
- **GIVEN** numbers "+7495" and "+74951234567" with records
- **WHEN** search by "+7495"
- **THEN** only the records of "+7495" are returned, dates descending

#### Scenario: Read count at 10^8 records
- **GIVEN** a database of 10^8 records, the inner tree levels in cache
- **WHEN** search for a number with ≤ 10 records
- **THEN** no more than 2 data pages are read (no more than 4 with an empty cache)

### Requirement: Number version from the same snapshot
The system SHALL read the records of a number and the header of its version from one consistent snapshot; a page of results by name SHALL
get the version of each number by a point read from the same snapshot as the page's records. Trace: CUJ6, CUJ9, X-5;
ADR 0046 items 3–4; ADR 0029.

#### Scenario: Two records of one number on a page
- **GIVEN** a results page with two records of one number, the number is changed in parallel
- **WHEN** the page is built
- **THEN** both records have one version, corresponding to a single state of the number

### Requirement: Number history stream without truncation
The system SHALL return up to 2,967,164 records of a number in one stream from the snapshot taken at the start of the search, with server memory
independent of the size of the results, without blocking writes to the number. Trace: CUJ4, CUJ9; ADR 0046 item 3; ADR 0035 item 6.

#### Scenario: A number on all valid dates
- **GIVEN** a number with records on each of the 2,967,164 days
- **WHEN** search by number
- **THEN** 2,967,164 records are returned by date DESC; server memory does not grow with the number of records

#### Scenario: Number change during a stream
- **GIVEN** a number stream has started
- **WHEN** another client changes the number
- **THEN** the stream contains the state as of the start, the client's write does not wait for the end of the stream

### Requirement: Bytewise order of the order key equals result order
The system SHALL encode the order key so that bytewise comparison of keys equals the order (name ru, date DESC, number
as a string) for any sort keys, including when one key is a prefix of another or contains the byte 0x00. Trace: CUJ5;
ADR 0045 items 4–5 (Q-109); ADR 0036.

#### Scenario: A sort key that is a prefix of another
- **GIVEN** two records, the sort key of the first is a proper prefix of the key of the second
- **WHEN** their order keys are compared
- **THEN** the first comes earlier; with equal sort keys, date DESC then number decide, independently of the bytes of the sort key

#### Scenario: A key with the byte 0x00
- **GIVEN** an artificial sort key with the byte 0x00
- **WHEN** order keys are compared
- **THEN** the order equals the order of tuples (sort key, date DESC, number)

#### Scenario: Consistency with the comparison function
- **WHEN** the build test runs the order corpus (ADR 0039 item 5 (b))
- **THEN** the byte order of the order key equals the result of name comparison for each adjacent pair; the presence of the byte 0x00 in the corpus
  sort keys is reported in the report

### Requirement: Keys of any length without truncation
The system SHALL store the sort key of a valid name up to 8,513 bytes and any mix of them, as well as word and comparison keys, without truncation and without refusal within the tree key cap of 65,535 bytes
(`storage-format`, "Key and value: limits, spill to overflow"); a larger key SHALL reject the operation, not be truncated. Trace: CUJ1, CUJ5; ADR 0045 item 6 (Q-48, Q-103); ADR 0022 item 5.

#### Scenario: A name of 200 U+FDFA characters
- **GIVEN** the valid name "U+FDFA × 200" (sort key 8,513 bytes)
- **WHEN** it is added and found by word search and exact search
- **THEN** the record is in the results; the order equals the comparison order; no key is truncated

#### Scenario: A key above the cap
- **WHEN** a test feeds the index layer a key of 65,536 bytes (unreachable for valid names: the additive bound is ≈ 20,400 bytes)
- **THEN** the operation is rejected entirely, data and version are unchanged, the key is not truncated, the "storage error" state did not occur

#### Scenario: Ordinary keys do not use overflow
- **GIVEN** names up to 100 characters of Cyrillic and Latin
- **WHEN** they are added
- **THEN** no index key is spilled to overflow pages

### Requirement: Name results in order with continuation from any position
The system SHALL return records by name in order key order and continue from the strictly next record after the cursor
position (name, date, number) by a single "first record after key" search and a forward read, without reading or repeating earlier
pages; the position of a deleted record is valid. Trace: CUJ5, CUJ12; ADR 0047 items 1–3; ADR 0032, 0036.

#### Scenario: Continuation does not depend on the page number
- **GIVEN** results of 10^6 records, a cursor from position 900,000
- **WHEN** a page is requested
- **THEN** the number of pages read is the same as for a cursor from position 50 (a descent plus a forward read)

#### Scenario: The position is deleted
- **GIVEN** a cursor on a record deleted after it was returned
- **WHEN** the next page is requested
- **THEN** the page starts with the first record after the position, nothing is skipped

#### Scenario: Names equal alphabetically
- **GIVEN** the records "Ann" (2024-01-01), "A­nn" (U+00AD, 2023-01-01), "Ann" (2022-01-01)
- **WHEN** a search that finds all three
- **THEN** the order is by dates descending

### Requirement: Exact whole-name search without materialization
The system SHALL find the records of exact search by a separate name comparison key index, return them in order key
order with a cursor, verify the record's comparison key against the query key (a hash collision does not give a false record) and fill the page
up to full. Trace: CUJ7, CUJ8; ADR 0047 item 4; ADR 0031.

#### Scenario: A frequent name
- **GIVEN** the name "Иван" in 10^6 records
- **WHEN** the first page and a page from the middle by cursor are requested
- **THEN** each contains 50 records and reads about 50 data records plus 1–2 index leaves; the range is not materialized

#### Scenario: A hash collision
- **GIVEN** two names with different comparison keys and an equal hash
- **WHEN** exact search for one of them
- **THEN** the other's record is absent from the results, the page is filled with the next matching ones

### Requirement: Word-prefix search: result independent of method
The system SHALL return the records for which every word of the query has a name word with the key prefix, in order key
order, and the result SHALL not depend on the chosen way of obtaining candidates or on whether the method was switched
mid-results. A query word that is a prefix of another query word does not change the result. Trace: CUJ5, CUJ8; ADR 0048
items 2–3; ADR 0008, 0030.

#### Scenario: All methods give one result
- **GIVEN** a set of queries (rare, frequent, short prefixes, 1–200 words) on one database
- **WHEN** each is run by each method (posting merge, order traversal, intersection) from the same positions
- **THEN** the results are bytewise equal to the definition (`name-search`, "Word-prefix search")

#### Scenario: A redundant prefix
- **GIVEN** the record "Annabel"
- **WHEN** query "an ann annab"
- **THEN** the record is found, as with "annab"

#### Scenario: One name word covers several query words
- **GIVEN** the record "Annabel"
- **WHEN** query "an ann"
- **THEN** the record is found

### Requirement: First page without full materialization of matches
The system SHALL prepare a word-search page holding in memory no more than 131,072 postings of one prefix plus the page,
and not accumulating the matches of the whole query; continuation SHALL be performed without server state and without reading earlier
pages. Trace: CUJ5, CUJ8; ADR 0048 item 4; ADR 0032.

#### Scenario: A frequent word
- **GIVEN** a word that 10^7 names begin with
- **WHEN** the first page
- **THEN** query memory does not depend on 10^7; a page of 50 records is gathered by scanning the order forward (a few hundred records expected)

#### Scenario: A cursor after restart
- **GIVEN** a cursor issued before a server restart
- **WHEN** the next page is requested
- **THEN** the page is obtained from the data as of the request, without the earlier state

### Requirement: Word search always completes with a result, with no cap on word count
The system SHALL accept a word-search query with any number of words up to 200 and always complete a valid query with a result defined by `name-search` ("Word-prefix search"): a full
page or the end of results; there is no search timeout whose expiry ends it with an error, and no refusals by volume. Planning SHALL cost no more than ≈ 2 × tree depth reads per
distinct non-redundant prefix. Trace: CUJ5, CUJ8, X-1; ADR 0048 item 6 (Q-106); ADR 0049 item 3 [DEFAULT].

#### Scenario: A 200-word query
- **GIVEN** a database of names of 200 Han characters, a query of 200 distinct characters
- **WHEN** the search is run
- **THEN** it completes within the class target, the record whose name contains all the characters is found

#### Scenario: A heavy query completes with a result
- **GIVEN** a query of several very frequent, mutually exclusive prefixes
- **WHEN** the search is run
- **THEN** it completes without an error: with a page of records if there are any, or with empty results; a page with fewer records than the page size is not returned if records exist; the work is bounded by a traversal of the order index (≈ 14 s at 10^8, Q-114), a request timeout, if the API introduces one, belongs to area 3

#### Scenario: A heavy query does not interfere with writes
- **GIVEN** a heavy query is running
- **WHEN** write operations arrive
- **THEN** they are acknowledged within the `storage-durability` targets, while the query continues on its own snapshot

### Requirement: Index format label and rebuild at start
The system SHALL store in the root of the indexes a label (the version of the key and value format, the `icu_collator` version, the word table version,
the hash identifier) and, on a mismatch with the server's label, rebuild the order, word, exact-search and dictionary indexes from the
records before accepting requests; the result order after a rebuild with the same comparison function SHALL be unchanged. Trace: CUJ5, CUJ11,
CUJ14; ADR 0045 items 7–8; ADR 0039 item 3 (Q-82).

#### Scenario: A change of the key record format
- **GIVEN** a database with indexes of the old key format in our code, the comparison function unchanged
- **WHEN** a server with the new label starts
- **THEN** the indexes are rebuilt before accepting requests; the results on the query corpus before and after are bytewise equal

#### Scenario: Failure during a rebuild
- **GIVEN** a rebuild is in progress
- **WHEN** the process is interrupted before the atomic swap of roots
- **THEN** the next start sees the earlier indexes and label and starts the rebuild anew

#### Scenario: Rebuild time [DEFAULT]
- **GIVEN** a database of 10^8 records and a mismatched label
- **WHEN** the server rebuilds the order, word, exact-search and dictionary indexes before accepting requests
- **THEN** the rebuild takes no more than 60 minutes (an estimate of 10–30 minutes; a default awaiting the owner, D-26)

#### Scenario: A rebuild is not a way to change the order
- **GIVEN** a server with a different `icu_collator` version
- **WHEN** after the rebuild at least one reference test of ADR 0039 item 5 gives a different result
- **THEN** the version is not accepted (the build fails, ADR 0039 item 6 (d)), the database order does not change

### Requirement: Index size and latency targets at 10^8 records
The system SHALL meet on the reference corpus of 10^8 records the ADR 0049 targets (defaults, until the owner decides):
file size — per the `storage-format` requirement "File size at 10^8 records" (D-21); p99 of search by number up to 100 records ≤ 10 ms; pages of exact search and of search with a word ≤ 10^4 postings
≤ 20 ms; pages of a single prefix of any frequency and of queries up to C ≤ 10^7 — ≤ 50 ms; heavy intersections ≤ 2 s. Trace: X-1,
CUJ4, CUJ5, CUJ8; ADR 0049 [DEFAULT] (Q-47, Q-59, Q-105).

#### Scenario: Size on the reference corpus
- **GIVEN** 10^8 records of profile P0 (`storage-format`: a 22-character name, 2.5 words, 1.25 records per number)
- **WHEN** the database is filled by random inserts
- **THEN** the length of `data` is no more than 64 × 10^9 bytes (D-21); the index calculation of ADR 0049 item 1 (≈ 39 GB for their share with prefix compression) is
  an estimate, not a threshold

#### Scenario: Query classes
- **WHEN** the ADR 0049 item 2 query classes are measured on the same database on the assumptions bench
- **THEN** the p99 of each class is no higher than its target; results are measured separately per class, not as one common p99

#### Scenario: Worst size per record
- **GIVEN** a valid name of 200 Han characters (200 words)
- **WHEN** the record is added
- **THEN** the record's indexes take about 180 KiB; out of space is "read-only" (CUJ16; the `storage-durability` requirement "Read-only" state), not truncation
