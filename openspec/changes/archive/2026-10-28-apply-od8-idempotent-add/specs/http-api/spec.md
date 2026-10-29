## MODIFIED Requirements

### Requirement: Adding a record
The system SHALL respond to `POST /v1/records` with `number`, `name` and an optional `date` as follows: 201 on creation, 200 on an identical retry; the body is `{outcome, record:{number,date,name}, version}`, `outcome` being `changed` or `unchanged`. Without a date: no records — 201 with the date 1876-03-07; a record on 1876-03-07 with the same name exists — 200 `unchanged` (OD-8); otherwise 409 `number_known` with `version`, and no record is created. `expected_version` is ignored. Trace: CUJ1, CUJ2, CUJ9; OD-6, OD-8; ADR 0005, 0026, 0060.

#### Scenario: Creation without a date
- **WHEN** ("+79991234567", "Ann") is added without `date`
- **THEN** 201, `outcome` is `changed`, `record.date` is `1876-03-07`, and `version` is a new version

#### Scenario: Number known
- **WHEN** the number has a record on 2020-01-01 and ("+79991234567", "Ann") is added without `date`
- **THEN** 409 `number_known` with the number's `version`; no record is created and the version is not changed

#### Scenario: An identical retry
- **WHEN** an add with `date` is repeated
- **THEN** 200, `outcome` is `unchanged`, and `version` is the same

#### Scenario: A retry without a date
- **WHEN** an add without `date` that created a record is repeated
- **THEN** 200, `outcome` is `unchanged`, `record.date` is `1876-03-07`, and `version` is the same; there is one record (OD-8)

#### Scenario: Without a date, a record on 1876-03-07 with a different name
- **WHEN** the number has one record on 1876-03-07 "Bob" and ("+79991234567", "Ann") is added without `date`
- **THEN** 409 `number_known` with `version`; the record is not changed

#### Scenario: Name conflict
- **WHEN** (number, date) already has the name "Bob" and "Ann" is added with that date
- **THEN** 409 `name_conflict` with `current_name` "Bob" and `version`; the record is not changed

#### Scenario: A version in an add
- **WHEN** an add contains `"expected_version":"garbage"`
- **THEN** the field is ignored and there is no error for it

### Requirement: Retry after a connection drop
The system SHALL NOT introduce idempotency keys: retrying an add with a date and a rename to the same name is safe while the number's records are unchanged, retrying an add without a date after success (a record on 1876-03-07 with the same name) gives 200 `unchanged`, and retrying a rename or delete with the earlier version after success gives `version_conflict`; the client finds out the outcome by re-reading the number. Trace: CUJ1, CUJ9, CUJ12; OD-6, OD-8; ADR 0026 items 5–6, 0064 (Q-54).

#### Scenario: A delete response is lost
- **WHEN** the response to a delete is lost and the client repeats the request
- **THEN** 409 `version_conflict`; search by number shows the record is absent

### Requirement: Absent member and `null`
The system SHALL treat a JSON body member with the value `null` as equivalent to an absent one. Absent member: `number` — `invalid_input`, `invalid_format`; `name` — `empty`; `date` on add — not an error (substitution, `unchanged` and `number_known` — "Adding a record"), on rename and delete — `missing`; `expected_version` on rename and delete — `missing`, on add it is ignored. A member of the wrong type — `wrong_type`. Trace: CUJ1, CUJ9; ADR 0062 item 3.

#### Scenario: An add with `"date": null`
- **WHEN** an add contains `"date":null` for a new number
- **THEN** same as without `date`: 201 with the date 1876-03-07; for a known number — 200 `unchanged` (a record on 1876-03-07 with the same name) or 409 `number_known`

#### Scenario: No number
- **WHEN** an add contains `"number":null` or has no `number`
- **THEN** 400 `invalid_input`, field `number`, reason `invalid_format`

#### Scenario: No name
- **WHEN** an add has no `name` or has `"name":null`
- **THEN** 400 `invalid_input`, field `name`, reason `empty`

#### Scenario: No expected version
- **WHEN** a rename contains `"expected_version":null`
- **THEN** 400 `invalid_input`, field `expected_version`, reason `missing`
