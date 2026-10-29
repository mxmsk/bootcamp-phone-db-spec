## REMOVED Requirements

### Requirement: Known number is determined by number search
**Reason**: OD-6 (Q-160): a number is known from the server's response to an add without a date, and there is no prior number query.
**Migration**: replaced by the requirement "Known number is determined by the server response".

## MODIFIED Requirements

### Requirement: Add form: fields and the date field
The add form SHALL contain the fields «Номер» (Number) and «Имя» (Name); the «Дата» (Date) field MUST be hidden until the server has answered "number known" (CUJ2) or until an overwrite prompt is shown for a record with a date. There is no separate «изменить» (edit) button. This is a default (D-82). Trace: CUJ1, CUJ2, CUJ3; OD-6; ADR 0005 item 5, 0090 (Q-28, Q-62).

#### Scenario: First opening
- **WHEN** the user opens the add form
- **THEN** the fields «Номер» and «Имя» and a save button are visible; there is no date field

#### Scenario: The number turned out to be known
- **WHEN** an add without a date received `409 number_known` (requirement "Known number is determined by the server response")
- **THEN** the «Дата» field appears in the form with the entered number and name unchanged

#### Scenario: No «изменить» button
- **WHEN** any result row is shown
- **THEN** the row has no «изменить» button; fixing a name is only through overwrite (CUJ3)

### Requirement: Adding a new number (CUJ1)
On `201` for an add without a date, the UI SHALL show a save message per the requirement "What to show after saving"; the record must be findable by number search and by word prefix of the name. The UI MUST NOT substitute the date itself. Trace: CUJ1; OD-6; ADR 0005, 0090 (Q-61).

#### Scenario: Saving an unknown number
- **WHEN** the user enters «+79991234567» and «Ann» for an unknown number and submits the form
- **THEN** `POST /v1/records` contains `number` and `name` and does not contain `date`; the UI shows the save with the date `1876-03-07` from the response

#### Scenario: Finding what was saved
- **WHEN** after saving, a search by number «+79991234567» and by words «ann» is run
- **THEN** both results contain the row «Ann», «+79991234567»

#### Scenario: A number with separators
- **WHEN** «+7 (999) 123-45-67» is entered
- **THEN** the UI sends the input as is; the message and the results show the canonical form «+79991234567» from the response

### Requirement: Date prompt for a known number (CUJ2)
For a known number (`409 number_known` on an add without a date), the UI SHALL show the message «Номер {номер} уже есть в базе. Введите дату, с которой номер закрепляется за этим именем» (Number {number} is already in the database. Enter the date from which the number is assigned to this name), the number history (date, name, by date descending) and the «Дата» field; without a date, a save MUST NOT be sent. The date being mandatory at this step is a rule of the UI itself (without a date the server would answer "number known") and the only exception to "Value errors in the form". Trace: CUJ2; OD-6; ADR 0002, 0007, 0091 (Q-37).

#### Scenario: The date prompt
- **WHEN** the number is known and has records on 2020-01-01 «Bob» and 2010-05-05 «Ann»
- **THEN** the UI shows the message, both records in this order and the «Дата» field; the entered number and name are kept

#### Scenario: No date entered
- **WHEN** the user submits the form with an empty «Дата» field for a known number
- **THEN** no request goes to the server (a UI rule); the date field shows «Дата: укажите дату» (Date: specify a date)

#### Scenario: Declining to enter a date
- **WHEN** the user presses «Отмена» (Cancel) on the date prompt or changes the number
- **THEN** nothing is saved, the «Дата» field is hidden, and the entered number and name remain

#### Scenario: A new date
- **WHEN** the user enters «2024-05-01», on which the number has no record
- **THEN** the UI sends `POST /v1/records` with `number`, `name`, `date`; the record is created, the UI shows the save; records of other dates are not changed

#### Scenario: A future date
- **WHEN** the date «2999-01-01» is entered
- **THEN** the record is created: the UI imposes no upper bound apart from the server's

### Requirement: Overwrite prompt (CUJ3)
On `409 name_conflict`, the UI SHALL show the dialog «На дату {дата} у номера {номер} уже записано имя «{current_name}». Заменить его на «{введённое имя}»? Старое имя будет стёрто» (For the date {date}, the number {number} already has the name "{current_name}". Replace it with "{entered name}"? The old name will be erased) with the buttons «Да, заменить» (Yes, replace) and «Нет» (No); the record does not change until the answer. Trace: CUJ3; OD-6; ADR 0005 item 9, 0006, 0091 (Q-37).

#### Scenario: A conflict
- **WHEN** (number, 2024-05-01) has "Bob" recorded and the user saves "Ann" on that date
- **THEN** the UI shows the dialog with "Bob" and "Ann"; server data is not changed

#### Scenario: The name differs by case
- **WHEN** «ANN» is saved and «Ann» is entered on the same date
- **THEN** the same dialog is shown (there is no silent success)

#### Scenario: A matching name
- **WHEN** the same name is entered (after space trimming and NFC)
- **THEN** the server answered `unchanged`; there is no dialog, and the result is shown per "What to show after saving"

#### Scenario: The date from the request
- **WHEN** the overwrite dialog is shown (an add without a date does not give `name_conflict`: it gives `409 number_known` and the date prompt, CUJ2)
- **THEN** the record's date is the date of the add that was sent; it is shown in the dialog text

### Requirement: "Yes" answer to overwrite
On «Да, заменить», the UI SHALL send `POST /v1/records/rename` with `number`, `date` (the date of the add that was sent), the entered `name` and `expected_version` from the `name_conflict` body; there MUST NOT be repeated submissions while the request is executing. Trace: CUJ3, CUJ12; ADR 0005, 0025, 0060 (Q-34, Q-90).

#### Scenario: Replacement
- **WHEN** the user answers «Да, заменить», and the version is current
- **THEN** the `rename` request contains all four fields; the record's name is replaced, and the number's other dates are untouched; the UI shows the save

#### Scenario: A double click
- **WHEN** the user presses «Да, заменить» twice quickly
- **THEN** one request goes to the server; the buttons are unavailable until the response

#### Scenario: The version is stale
- **WHEN** `rename` answered `409 version_conflict`
- **THEN** "Stale data (CUJ12)" is executed

#### Scenario: Unchanged
- **WHEN** `rename` answered `200` with `outcome` equal to `unchanged`
- **THEN** the UI shows «Запись уже есть, данные не изменились», not an error

### Requirement: "No" answer to overwrite
On «Нет», the UI SHALL close the dialog without a request to the server and wait for new input; the entered number, name and date remain in the form. Trace: CUJ3; ADR 0005 item 9, 0091 (Q-37).

#### Scenario: A refusal
- **WHEN** the user answers «Нет»
- **THEN** data is not changed and there are no requests; the form contains the earlier number, name and date; focus is in the «Имя» field

#### Scenario: New input
- **WHEN** after «Нет» the user changes the name and submits the form
- **THEN** the UI sends an add with the same number and date and the new name, without a number query

### Requirement: Delete execution
On «Удалить» (Delete), the UI SHALL send `POST /v1/records/delete` with `number`, `date` and `expected_version`: the number version from the root of the by-number response or from a word-search result record; it MUST NOT substitute the date. Success is exactly one record, and the number's other records are intact. Trace: CUJ6; OD-6 (Q-128); ADR 0005, 0025, 0093 (Q-90).

#### Scenario: The request body
- **WHEN** the delete of the row («+79991234567», 2024-05-01) from the by-number results is confirmed, and the number version V is at the response root
- **THEN** the body contains these three values, and version V is taken from the response root, not from the row

#### Scenario: Other dates
- **WHEN** the number has records on three dates and one is deleted
- **THEN** the other two remain

#### Scenario: Read-only
- **WHEN** the server answered `503 read_only`
- **THEN** «Удаление не выполнено: база доступна только для чтения» (Delete not performed: the database is read-only); the record and the list are not changed

### Requirement: Flow message texts
The UI SHALL show «данные устарели, список обновлён» (data is stale, the list has been refreshed) verbatim, and the CUJ16 refusals and value error texts per `web-ui-shell` ("\"Read-only\" and \"storage error\" texts on action failure"), passing the request layer the action: «Сохранение» (Save) (add, the number history after "number known", replacement), «Удаление» (Delete), «Поиск» (Search); server details are not disclosed. This is a default (D-93). Trace: CUJ12, CUJ16; ADR 0011 item 6, 0088, 0094 (Q-65).

#### Scenario: Read-only on add
- **WHEN** an add received `503 read_only`
- **THEN** «Сохранение не выполнено: база доступна только для чтения» (Save not performed: the database is read-only) is shown

#### Scenario: Read-only on delete
- **WHEN** a delete received `503 read_only`
- **THEN** «Удаление не выполнено: база доступна только для чтения» is shown; the record and the list are not changed

#### Scenario: A storage error in search
- **WHEN** a search received `500 storage_error` or the by-number stream was cut without a completeness marker
- **THEN** «Поиск не выполнен: ошибка хранилища, обратитесь к оператору» (Search not performed: storage error, contact the operator) is shown; a partial result is not presented as complete

#### Scenario: Not shown in advance
- **WHEN** the UI is open and no action has been performed
- **THEN** the "read-only" and "storage error" states are not shown

## ADDED Requirements

### Requirement: Known number is determined by the server response
The UI SHALL send an add without a date immediately, without a prior number query; a `409 number_known` response means a known number (CUJ2): the UI requests `GET /v1/records/by-number` for the history and shows the date prompt, accepting the history as complete only with `complete: true`. The CUJ1/CUJ2 race is closed by a server rule, not by the UI (OD-6, Q-160). Trace: CUJ1, CUJ2; OD-6; ADR 0005, 0060, 0090 (Q-28).

#### Scenario: Number unknown
- **WHEN** the number has no records and the UI sent an add without a date
- **THEN** the server answered `201`; no number query was run before the add

#### Scenario: Number known
- **WHEN** an add without a date received `409 number_known`, and then `by-number` returned a complete response (`complete: true`) with records
- **THEN** no record is created; the UI shows the date prompt with the number's history

#### Scenario: An invalid number
- **WHEN** an add received `invalid_input` on the `number` field
- **THEN** the error is shown at the «Номер» field (text — "Value errors in the form")

#### Scenario: A truncated history
- **WHEN** after `409 number_known` the `by-number` stream ended without `count` and `complete`
- **THEN** the date prompt is not shown; the form input is kept; «Сохранение не выполнено: ошибка хранилища, обратитесь к оператору» (Save not performed: storage error, contact the operator) is shown, and the form can be submitted again

#### Scenario: History not received
- **WHEN** after `409 number_known`, `by-number` did not return 200 (a server refusal or a connection drop)
- **THEN** the form input is kept; the refusal is shown per `web-ui-shell` with the action «Сохранение» (on a connection drop — «Сохранение не выполнено: нет связи с сервером»; "outcome unknown" is not shown, no record is created)

#### Scenario: The number became known in a race
- **WHEN** another client added a record of this number immediately before the add without a date
- **THEN** the server answered `409 number_known`, no record on 1876-03-07 is created, and the UI shows the date prompt
