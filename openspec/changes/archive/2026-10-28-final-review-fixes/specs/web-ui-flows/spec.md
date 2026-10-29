## MODIFIED Requirements

### Requirement: What to show after saving
After a successful add or replacement the UI SHALL show «Сохранено: {имя}, {номер}, с {дата}» ("Saved: {name}, {number}, from {date}"; for `unchanged`, instead — «Запись уже есть, данные не изменились», "The record already exists, the data is unchanged"), clear the form, hide the date field and show the history of this number (a search-by-number request). The message stays until the next action. This is a default (D-83). Trace: CUJ1, CUJ2, CUJ3; ADR 0011, 0091 (Q-61).

#### Scenario: Message and history
- **WHEN** a record is created
- **THEN** a message with the number, name and date from the response and the results by number with all its records are shown

#### Scenario: Clearing the form
- **WHEN** a save is successful
- **THEN** the «Номер», «Имя» and «Дата» fields are empty

#### Scenario: Identical retry
- **WHEN** the server answered `200`, `unchanged`
- **THEN** the UI shows the message «Запись уже есть, данные не изменились» as a success

#### Scenario: History failed
- **WHEN** the save is successful but the number history request failed
- **THEN** the save message stays; next to the results area «История номера не загружена: нет связи с сервером» ("Number history not loaded: no connection to the server"; for an incomplete response — «…: ответ получен не полностью») and «Повторить» are shown; the words «Сохранение не выполнено» are not used, and the save is not undone

### Requirement: Search by number (CUJ4)
The UI SHALL, for an entered number, request `GET /v1/records/by-number` and show all results as a single list without "next", in response order (date descending); the number from the response in canonical form is shown. A number with an invalid shape is an error at the field, as in the add form. Trace: CUJ4; ADR 0007, 0035, 0061.

#### Scenario: All records of a number
- **WHEN** a number has 3 records
- **THEN** exactly 3 rows are shown by date DESC, without «Дальше» ("Next")

#### Scenario: Exact match only
- **WHEN** a part of a number, «+7999», is entered
- **THEN** there are no rows of numbers that merely contain it; the result is what the server returned

#### Scenario: Input with separators
- **WHEN** «+7 999 123-45-67» is entered
- **THEN** the records of the number «+79991234567» are shown

#### Scenario: Incomplete stream
- **WHEN** the stream response (status 200) does not contain `complete: true` at the end
- **THEN** the UI does not accept the list as complete; «Поиск не выполнен: ответ получен не полностью» and «Повторить» are shown (`web-ui-shell`)

### Requirement: Flow message texts
The UI SHALL show «данные устарели, список обновлён» ("data is stale, the list has been refreshed") verbatim, and the CUJ16 refusals and value error texts — per `web-ui-shell` (the requirement '"Read-only" and "storage error" texts on action failure'), passing the request layer the action: «Сохранение» (add, number history after "number known", replacement), «Удаление», «Поиск»; server details are not disclosed. This is a default (D-93). Trace: CUJ12, CUJ16; ADR 0011 item 6, 0088, 0094 (Q-65).

#### Scenario: Read-only on add
- **WHEN** an add received `503 read_only`
- **THEN** «Сохранение не выполнено: база доступна только для чтения» is shown

#### Scenario: Read-only on delete
- **WHEN** a delete received `503 read_only`
- **THEN** «Удаление не выполнено: база доступна только для чтения» is shown; the record and the list are unchanged

#### Scenario: Storage error in search
- **WHEN** a search received `500 storage_error`
- **THEN** «Поиск не выполнен: ошибка хранилища, обратитесь к оператору» is shown; partial results are not passed off as complete

#### Scenario: Number stream cut off
- **WHEN** the number stream received 200 but ended without a completeness marker
- **THEN** «Поиск не выполнен: ответ получен не полностью» and «Повторить» are shown (`web-ui-shell`), not "storage error"

#### Scenario: Not shown in advance
- **WHEN** the UI is open and no action has been performed
- **THEN** the "read-only" and "storage error" states are not shown

### Requirement: Entered input is kept when a write fails
The UI SHALL, on any failure of an add or replacement (value error, conflict, server refusal, `read_only`, `storage_error`, `request_timeout`, `write_outcome_unknown`, connection drop), leave the entered number, name and date in the form; forms are cleared only after success. Trace: CUJ3, CUJ12, CUJ16; ADR 0026, 0094.

#### Scenario: Server refusal
- **WHEN** an add received `503 read_only`
- **THEN** the form fields are unchanged

#### Scenario: Outcome unknown
- **WHEN** the response is `504 write_outcome_unknown` or a connection drop after sending
- **THEN** the fields are kept; "outcome unknown" is shown per `web-ui-shell`; the UI re-reads and retries nothing by itself, and the user checks the outcome by searching by number (CUJ4); a repeated add without a date after a successful first attempt will receive `409 number_known` (a date prompt with the history where the record already exists), a replacement with the old version — "data is stale" (ADR 0026)

#### Scenario: Success
- **WHEN** a save is successful
- **THEN** the fields are cleared (the requirement "What to show after saving")

### Requirement: Known number is determined by the server response
The UI SHALL send an add without a date immediately, with no preliminary by-number request; a `409 number_known` response means a known number (CUJ2): the UI requests `GET /v1/records/by-number` for the history and shows the date prompt, accepting the history as complete only with `complete: true`. The CUJ1/CUJ2 race is closed by a server rule, not by the UI (OD-6, Q-160). Trace: CUJ1, CUJ2; OD-6; ADR 0005, 0060, 0090 (Q-28).

#### Scenario: Number unknown
- **WHEN** a number has no records and the UI sent an add without a date
- **THEN** the server answered `201`; no by-number request was made before the add

#### Scenario: Number known
- **WHEN** an add without a date received `409 number_known`, and then `by-number` returned a complete response (`complete: true`) with records
- **THEN** no record is created; the UI shows the date prompt with the number history

#### Scenario: Invalid number
- **WHEN** an add received `invalid_input` on the `number` field
- **THEN** the error is shown at the «Номер» field (the text — "Value errors in the form")

#### Scenario: History cut off
- **WHEN** after `409 number_known` the `by-number` stream ended without `count` and `complete`
- **THEN** the date prompt is not shown; what was entered in the form is kept; «Сохранение не выполнено: ответ получен не полностью» and «Повторить» are shown, and the form can be submitted again

#### Scenario: History not received
- **WHEN** after `409 number_known` `by-number` did not return 200 (server refusal or connection drop)
- **THEN** what was entered in the form is kept; the refusal is shown per `web-ui-shell` with the action «Сохранение» (on a connection drop — «Сохранение не выполнено: нет связи с сервером»; "outcome unknown" is not shown, the record is not created)

#### Scenario: Number became known in a race
- **WHEN** another client added a record of this number immediately before the add without a date
- **THEN** the server answered `409 number_known`, no record on 1876-03-07 is created, and the UI shows the date prompt
