## MODIFIED Requirements

### Requirement: Word search always completes with a result, with no cap on word count
The system SHALL accept a word-search query with any number of words up to 200 and always complete a valid query with the result defined by `name-search` ("Word-prefix search"): a full
page or the end of the results; there is no search time limit whose expiry ends the search with an error, and no refusals by volume; the exceptions are snapshot revocation (`snapshot_revoked`) by the retention cap and the lifetime ceiling (`storage-snapshots`) and the request timeout of area 3. Planning SHALL cost at most ≈ 2 × tree depth reads per
distinct non-redundant prefix. Trace: CUJ5, CUJ8, X-1; ADR 0048 item 6 (Q-106); ADR 0049 item 3 [DEFAULT]; OD-6 (accepted, scale 10^7).

#### Scenario: 200-word query
- **GIVEN** a database of names of 200 Han characters, a query of 200 distinct characters
- **WHEN** the search is performed
- **THEN** it completes within the class target, and a record whose name contains all the characters is found

#### Scenario: A heavy query completes with a result
- **GIVEN** a query of several very frequent, mutually exclusive prefixes
- **WHEN** the search is performed
- **THEN** it completes without a search error (snapshot revocation under `storage-snapshots` and the request timeout are exceptions): with a page of records if there are any, or with empty results; a page with fewer records than the page size is not returned when records exist; the work is bounded by a walk of the order index (≈ 1.4 s at 10^7, Q-114; recomputed proportionally from ≈ 14 s at 10^8), the request timeout, if the API introduces one, belongs to area 3

#### Scenario: A heavy query does not hinder writes
- **GIVEN** a heavy query is running
- **WHEN** write operations arrive
- **THEN** they are acknowledged within the `storage-durability` targets, and the query continues on its own snapshot
