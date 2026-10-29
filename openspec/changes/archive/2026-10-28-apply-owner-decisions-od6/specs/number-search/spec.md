## REMOVED Requirements

### Requirement: Number version in the by-number search result
**Reason**: OD-6
**Migration**: the requirement is replaced by "Number version in a by-number result: once per number" (values and scenarios recalculated per OD-6; the scenario names contained outdated numbers or wording).

## ADDED Requirements

### Requirement: Number version in a by-number result: once per number
The system SHALL return the number version as a field of the by-number search result, always, including for empty results, once per number; the result records have no version (OD-6, Q-128). The version is consistent with the number's records in this result (the requirement "Number version in
search results" of the `write-operations` capability). Trace: CUJ6, CUJ9, X-5; OD-6; ADR 0006, 0029, 0035 item 4.

#### Scenario: One version per number
- **GIVEN** the number has records on three dates
- **WHEN** a search by number is run
- **THEN** the result contains the number version once; the three records contain no versions

#### Scenario: The version is consistent with the history
- **GIVEN** a record was deleted with acknowledgement, and the number got a new version V
- **WHEN** a search by number is run
- **THEN** the result contains version V; the remaining records belong to state V and do not include the record deleted before V (the results and the version are a
  consistent pair)

#### Scenario: A version from the results is accepted by a change
- **WHEN** a client renames or deletes a record using the version from a by-number search result, and the number has not changed since
- **THEN** the operation accepts the version as current
