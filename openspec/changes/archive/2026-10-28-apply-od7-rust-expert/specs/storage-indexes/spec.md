## MODIFIED Requirements

### Requirement: Word-prefix search: result independent of method
The system SHALL return records for which, for every word of the query, there is a word of the name with the key prefix, in order-key order, and the result SHALL NOT depend on the method of obtaining candidates (order scan, postings intersection; postings merge — per profile) or on switching the method in the middle of the results. A query word that is a prefix of another does not change the result. Trace: CUJ5, CUJ8; ADR 0048 items 2–3 (OD-7); ADR 0008, 0030.

#### Scenario: All methods give one result
- **GIVEN** a set of queries (rare, frequent, short prefixes, 1–200 words) on one database
- **WHEN** each is executed by every applied method (order scan, intersection, postings merge if introduced) from the same positions
- **THEN** the results are byte-for-byte equal to the definition (`name-search`, "Word-prefix search")

#### Scenario: A redundant prefix
- **GIVEN** the record "Annabel"
- **WHEN** the query "an ann annab"
- **THEN** the record is found, as with "annab"

#### Scenario: One name word covers several query words
- **GIVEN** the record "Annabel"
- **WHEN** the query "an ann"
- **THEN** the record is found

### Requirement: Word search always completes with a result, with no cap on word count
The system SHALL accept a word-search query with any number of words up to 200 and always complete a valid query with a result: a full page or the end of the results; there is no search deadline and no refusals by volume; the exceptions are `snapshot_revoked` and the request timeout. Planning SHALL cost about 2 × tree depth plus at most 4 dictionary leaves per non-redundant prefix: the dictionary is a tree without aggregates, and counting words and postings is a scan that stops at a threshold. Trace: CUJ5, CUJ8, X-1; ADR 0048 item 6, 0045 item 3 (OD-7).

#### Scenario: A 200-word query
- **GIVEN** a database of names of 200 Han ideographs, a query of 200 distinct ideographs
- **WHEN** the search is executed
- **THEN** it completes within the class target, and the record whose name contains all the ideographs is found

#### Scenario: A heavy query completes with a result
- **GIVEN** a query of several very frequent, mutually exclusive prefixes
- **WHEN** the search is executed
- **THEN** it completes without a search error (snapshot revocation per `storage-snapshots` and the request timeout are the exceptions): with a page of records if there are any, or with empty results; a page with fewer records than the page size is not returned when records exist; the work is bounded by a scan of the order index (about 1.4 s at 10^7, Q-114; scaled proportionally from about 14 s at 10^8), and the request timeout, if the API introduces one, belongs to area 3

#### Scenario: A heavy query does not interfere with writes
- **GIVEN** a heavy query is executing
- **WHEN** write operations arrive
- **THEN** they are acknowledged within the `storage-durability` targets, and the query continues on its own snapshot
