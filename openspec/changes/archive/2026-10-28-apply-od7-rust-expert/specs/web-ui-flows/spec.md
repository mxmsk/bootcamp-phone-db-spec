## MODIFIED Requirements

### Requirement: Search by number (CUJ4)
For an entered number, the UI SHALL request `GET /v1/records/by-number` and show all the results as a single list with no «Дальше» ("Next"), in response order (date descending); the number comes from the response, in canonical form. A number of invalid shape is an error at the field, as in the add form. UI responsiveness is guaranteed up to about 10^4 records of a number; beyond that it is not guaranteed, and the results are not truncated. Trace: CUJ4; ADR 0007, 0035, 0061; ADR 0100 (OD-7, Q-163).

#### Scenario: All records of a number
- **WHEN** a number has 3 records
- **THEN** exactly 3 rows are shown by date DESC, with no «Дальше» ("Next")

#### Scenario: Exact match only
- **WHEN** a partial number «+7999» is entered
- **THEN** there are no rows of numbers that merely contain it; the result is what the server returned

#### Scenario: Input with separators
- **WHEN** «+7 999 123-45-67» is entered
- **THEN** the records of the number «+79991234567» are shown

#### Scenario: An incomplete stream
- **WHEN** a stream response (status 200) does not contain `complete: true` at the end
- **THEN** the UI does not accept the list as complete; it shows «Поиск не выполнен: ответ получен не полностью» (search not performed: response received incompletely) and «Повторить» (Retry) (`web-ui-shell`)

#### Scenario: A history of up to 10^4 records
- **WHEN** a number has about 10^4 records
- **THEN** all records are shown and the page is responsive

#### Scenario: More than 10^4 records
- **WHEN** a number has more than 10^4 records
- **THEN** responsiveness is not guaranteed; records are not truncated, and an incomplete stream is not accepted as complete
