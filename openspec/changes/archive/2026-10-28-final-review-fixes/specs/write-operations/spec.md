## MODIFIED Requirements

### Requirement: Retrying an operation after an unknown outcome
The system SHALL, when a write operation whose result the client did not receive is retried, produce no second effect if the number's records have not changed: an add with a date — "unchanged"; an add without a date — "number known" (the client finds out the outcome by searching by number); a delete, and a rename that changed the name, with the old version — "version conflict"; a rename to the same name — "unchanged". If the records have changed, the ordinary rules apply: an add with a date — by whether the key is occupied, without a date — by whether the number has records. Trace: CUJ1, CUJ9, CUJ12, CUJ14; OD-6 (Q-160). Decisions: ADR 0026 (owner decision, OD-5, D-7).

#### Scenario: Retrying an add without a date
- **WHEN** an add without a date of a new number has been executed, the response is lost, and the client retries it
- **THEN** "number known" with the number version; there is one record, and search by number shows it

#### Scenario: Retrying an add
- **WHEN** an add with a date has been executed, the response is lost, the number's records have not changed, and the client retries the add
- **THEN** success "unchanged"; the database has one record

#### Scenario: Retrying an add after another client's delete
- **WHEN** an add has been executed, the response is lost, another client deleted this record, and the client retries the add
- **THEN** the record is created anew (outcome "changed"): an add carries no version and does not distinguish a "first attempt" from a new one
  (owner decision "add without a version"); a client that must not resurrect deleted data checks by searching by number

#### Scenario: Retrying an add after another client's name replacement
- **WHEN** an add of "Иван" has been executed, the response is lost, another client replaced this record's name with "Пётр", and the client
  retries the add of "Иван"
- **THEN** "name conflict" with the current name "Пётр" and the number version; the data is unchanged

#### Scenario: Retrying a delete
- **WHEN** a delete has been executed, the response is lost, and the client retries the delete with the old version
- **THEN** "version conflict" with the current version; the client re-reads the number by searching by number and sees that the record is gone

#### Scenario: Retrying a rename
- **WHEN** a rename has been executed, the response is lost, and the client retries it with the old version
- **THEN** "version conflict"; after re-reading the history, the client sees its own name and does not retry

#### Scenario: Retrying a rename to the same name
- **WHEN** a rename replaced the name with the same one (the number version did not change), the response is lost, and the client retries it with the old version
- **THEN** success "unchanged"; the number version does not change

#### Scenario: Retrying a rename after another client's delete
- **WHEN** a rename has been executed, the response is lost, another client deleted the record, and the client retries the rename with the old version
- **THEN** "version conflict" (the delete changed the version); the record is not resurrected
