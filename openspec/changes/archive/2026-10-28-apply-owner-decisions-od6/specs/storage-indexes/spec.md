## RENAMED Requirements

- FROM: `### Requirement: Index size and latency targets at 10^8 records`
- TO: `### Requirement: Index size and latency targets at 10^7 records`

## REMOVED Requirements

### Requirement: Search by number without scanning records
**Reason**: OD-6
**Migration**: the requirement is replaced by "Search by number without scanning records: read count" (values and scenarios recalculated per OD-6; the scenario names contained outdated numbers or wording).

## MODIFIED Requirements

### Requirement: First page without full materialization of matches
The system SHALL prepare a word-search page keeping in memory at most 131 072 postings of one prefix plus the page,
and not accumulating the matches of the whole query; continuation SHALL be performed without server state and without reading earlier
pages. Trace: CUJ5, CUJ8; ADR 0048 item 4; ADR 0032.

#### Scenario: A frequent word
- **GIVEN** a word that 10^6 names begin with
- **WHEN** the first page
- **THEN** query memory does not depend on 10^6; a page of 50 records is gathered by scanning the order forward (a few hundred records expected)

#### Scenario: A cursor after a restart
- **GIVEN** a cursor issued before a server restart
- **WHEN** the next page is requested
- **THEN** the page is obtained from the data as of the request, without the earlier state

### Requirement: Word search always completes with a result, with no cap on word count
The system SHALL accept a word-search query with any number of words up to 200 and always complete a valid query with the result defined by `name-search` ("Word-prefix search"): a full
page or the end of the results; there is no search deadline whose expiry ends it with an error, and no refusals by volume. Planning SHALL cost at most about 2 × tree depth reads per
distinct non-redundant prefix. Trace: CUJ5, CUJ8, X-1; ADR 0048 item 6 (Q-106); ADR 0049 item 3 [DEFAULT]; OD-6 (accepted, scale 10^7).

#### Scenario: A 200-word query
- **GIVEN** a database of names of 200 Han ideographs, a query of 200 distinct ideographs
- **WHEN** the search is executed
- **THEN** it completes within the class target, and the record whose name contains all the ideographs is found

#### Scenario: A heavy query completes with a result
- **GIVEN** a query of several very frequent, mutually exclusive prefixes
- **WHEN** the search is executed
- **THEN** it completes without an error: with a page of records if there are any, or with empty results; a page with fewer records than the page size is not returned when records exist; the work is bounded by a scan of the order index (about 1.4 s at 10^7, Q-114; scaled proportionally from about 14 s at 10^8), and the request timeout, if the API introduces one, belongs to area 3

#### Scenario: A heavy query does not interfere with writes
- **GIVEN** a heavy query is executing
- **WHEN** write operations arrive
- **THEN** they are acknowledged within the `storage-durability` targets, and the query continues on its own snapshot

### Requirement: Index format label and rebuild at start
The system SHALL store in the index root a label (the key and value format version, the `icu_collator` version, the word table version,
the hash identifier) and, on a mismatch with the server's label, rebuild the order, word, exact-search and dictionary indexes from
the records before accepting requests; the result order after a rebuild with the same comparison function SHALL be unchanged. Trace: CUJ5, CUJ11,
CUJ14; ADR 0045 items 7–8; ADR 0039 item 3 (Q-82).

#### Scenario: A change of the key record format
- **GIVEN** a database with indexes of the old key format in our code, the comparison function unchanged
- **WHEN** a server with the new label starts
- **THEN** the indexes are rebuilt before requests are accepted; the results on the query corpus before and after are byte-for-byte equal

#### Scenario: A failure during the rebuild
- **GIVEN** a rebuild is in progress
- **WHEN** the process is interrupted before the atomic swap of the roots
- **THEN** the next start sees the earlier indexes and label and begins the rebuild anew

#### Scenario: Rebuild time [DEFAULT]
- **GIVEN** a database of 10^7 records and a mismatched label
- **WHEN** the server rebuilds the order, word, exact-search and dictionary indexes before accepting requests
- **THEN** the rebuild takes at most 6 minutes (estimate 1–3 minutes; scaled proportionally from 60 and 10–30 minutes at 10^8; D-26 accepted by the owner, OD-6)

#### Scenario: A rebuild is not a way to change the order
- **GIVEN** a server with a different `icu_collator` version
- **WHEN** after a rebuild at least one reference of ADR 0039 item 5 gives a different result
- **THEN** the version is not accepted (the build fails, ADR 0039 item 6 (d)), and the database order is not changed

### Requirement: Index size and latency targets at 10^7 records
The system SHALL meet the targets of ADR 0049 on the reference corpus of 10^7 records (accepted by the owner, OD-6: the latency targets are the same numbers as for 10^8, but at 10^7):
file size — per the requirement "File size at 10^7 records" of `storage-format` (D-21); p99 of search by number up to 100 records ≤ 10 ms; pages of exact search and of a search with a word of ≤ 10^4 postings
≤ 20 ms; pages of a single prefix of any frequency and of queries up to C ≤ 10^7 — ≤ 50 ms; heavy intersections ≤ 2 s. Trace: X-1,
CUJ4, CUJ5, CUJ8; ADR 0049 (Q-47, Q-59, Q-105); OD-6.

#### Scenario: Size on the reference corpus
- **GIVEN** 10^7 records of profile P0 (`storage-format`: name of 22 characters, 2.5 words, 1.25 records per number)
- **WHEN** the database is filled with random inserts
- **THEN** the length of `data` is at most 8 × 10^9 bytes (D-21); the index calculation of ADR 0049 item 1 (about 3.9 GB as their share with prefix compression) is
  an estimate, not a threshold

#### Scenario: Query classes
- **WHEN** on the same database the query classes of ADR 0049 item 2 are measured on the assumptions rig
- **THEN** the p99 of each class is no higher than its target; the measurement results are per class, not one overall p99

#### Scenario: Worst size per write
- **GIVEN** a valid name of 200 Han ideographs (200 words)
- **WHEN** a record is added
- **THEN** the record's indexes take on the order of 180 KiB; running out of space leads to "read-only" (CUJ16; `storage-durability`, "\"Read-only\" state"), not truncation

## ADDED Requirements

### Requirement: Search by number without scanning records: read count
The system SHALL find a number's records by the canonical string by reading a range whose number of pages read does not
depend on the total number of records beyond the tree depth, and return them by date DESC without sorting. Trace: CUJ4, CUJ8;
ADR 0046 items 1–2; ADR 0035.

#### Scenario: A number that is the beginning of another
- **GIVEN** the numbers "+7495" and "+74951234567" with records
- **WHEN** a search for "+7495"
- **THEN** only the records of "+7495" are returned, dates descending

#### Scenario: Read count at 10^7 records
- **GIVEN** a database of 10^7 records, the inner tree levels in cache
- **WHEN** a search for a number with ≤ 10 records
- **THEN** at most 2 data pages are read (at most 4 with an empty cache; 3 tree levels are expected at 10^7, to be confirmed with a prototype)
