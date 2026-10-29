## MODIFIED Requirements

### Requirement: Adding a new number (CUJ1)
On `201` (created) or `200` `unchanged` (retry, OD-8) for an add without a date, the UI SHALL show a save message per the requirement "What to show after saving"; the record must be findable by number search and by word prefix of the name. The UI MUST NOT substitute the date itself. Trace: CUJ1; OD-6, OD-8; ADR 0005, 0090 (Q-61).

#### Scenario: Saving an unknown number
- **WHEN** the user enters «+79991234567» and «Ann» for an unknown number and submits the form
- **THEN** `POST /v1/records` contains `number` and `name` and does not contain `date`; the UI shows the save with the date `1876-03-07` from the response

#### Scenario: A retry without a date after a lost response
- **WHEN** an add without a date created a record, the response was lost, and the user submitted the same form again
- **THEN** the server answered `200` `unchanged`; «Запись уже есть, данные не изменились» (The record already exists, data unchanged) is shown, and there is no date prompt (OD-8)

#### Scenario: Finding what was saved
- **WHEN** after saving, a search by number «+79991234567» and by words «ann» is run
- **THEN** both results contain the row «Ann», «+79991234567»

#### Scenario: A number with separators
- **WHEN** «+7 (999) 123-45-67» is entered
- **THEN** the UI sends the input as is; the message and the results show the canonical form «+79991234567» from the response

### Requirement: Entered input is kept when a write fails
On any failure of an add or replacement (value error, conflict, server refusal, `read_only`, `storage_error`, `request_timeout`, `write_outcome_unknown`, connection drop), the UI SHALL leave the entered number, name and date in the form; forms are cleared only after success. Trace: CUJ3, CUJ12, CUJ16; ADR 0026, 0094.

#### Scenario: A server refusal
- **WHEN** an add received `503 read_only`
- **THEN** the form fields are not changed

#### Scenario: Outcome unknown
- **WHEN** the response is `504 write_outcome_unknown` or the connection dropped after sending
- **THEN** the fields are kept; «исход неизвестен» (outcome unknown) is shown per `web-ui-shell`; the UI re-reads and retries nothing itself, and the user checks the outcome by number search (CUJ4); a repeated add without a date after a successful first attempt gets `200` `unchanged` («Запись уже есть, данные не изменились»; OD-8), and a replacement with the earlier version gets «данные устарели» (data is stale) (ADR 0026)

#### Scenario: Success
- **WHEN** the save succeeds
- **THEN** the fields are cleared (requirement "What to show after saving")

### Requirement: Known number is determined by the server response
The UI SHALL send an add without a date immediately, without a prior number query; a `409 number_known` response means a known number (CUJ2), and `200` `unchanged` means an identical retry (OD-8): the UI requests `GET /v1/records/by-number` for the history and shows the date prompt, accepting the history as complete only with `complete: true`. The CUJ1/CUJ2 race is closed by a server rule, not by the UI (OD-6, Q-160). Trace: CUJ1, CUJ2; OD-6, OD-8; ADR 0005, 0060, 0090 (Q-28).

#### Scenario: Number unknown
- **WHEN** the number has no records and the UI sent an add without a date
- **THEN** the server answered `201`; no number query was run before the add

#### Scenario: Number known
- **WHEN** an add without a date received `409 number_known`, and then `by-number` returned a complete response (`complete: true`) with records
- **THEN** no record is created; the UI shows the date prompt with the number's history

#### Scenario: A retry after an unknown outcome
- **WHEN** the number has a record on 1876-03-07 with the same name and the UI sent an add without a date
- **THEN** the server answered `200` `unchanged`; the date prompt and `by-number` are not run, and the result is shown per "What to show after saving"

#### Scenario: An invalid number
- **WHEN** an add received `invalid_input` on the `number` field
- **THEN** the error is shown at the «Номер» (Number) field (text — "Value errors in the form")

#### Scenario: A truncated history
- **WHEN** after `409 number_known` the `by-number` stream ended without `count` and `complete`
- **THEN** the date prompt is not shown; the form input is kept; «Сохранение не выполнено: ответ получен не полностью» (Save not performed: response received incompletely) and «Повторить» (Retry) are shown, and the form can be submitted again

#### Scenario: History not received
- **WHEN** after `409 number_known`, `by-number` did not return 200 (a server refusal or a connection drop)
- **THEN** the form input is kept; the refusal is shown per `web-ui-shell` with the action «Сохранение» (Save) (on a connection drop — «Сохранение не выполнено: нет связи с сервером» (Save not performed: no connection to the server); "outcome unknown" is not shown, no record is created)

#### Scenario: The number became known in a race
- **WHEN** another client added a record of this number immediately before the add without a date
- **THEN** the server answered `409 number_known`, no record on 1876-03-07 is created, and the UI shows the date prompt
