## Purpose
The capability defines the observable behavior of the web interface in user journeys CUJ1–CUJ6 and CUJ12 on top of the accepted HTTP API v1: the add form and detection of a known number, date and overwrite prompts, the look of results by number and by name words, "next" pagination, delete with confirmation, the reaction to stale data, message texts. The page shell, sign-in, the security policy, and common failures belong to the capability `web-ui-shell`; the API does not change.

## ADDED Requirements

### Requirement: Add form: fields and the date field
The add form SHALL contain the fields «Номер» ("Number") and «Имя» ("Name"); the «Дата» ("Date") field MUST be hidden until the number turns out to be known (CUJ2) or until an overwrite prompt is shown for a record with a date. There is no separate "edit" button. This is a default (D-82). Trace: CUJ1, CUJ2, CUJ3; ADR 0005 item 5, 0090 (Q-28, Q-62).

#### Scenario: First open
- **WHEN** the user opens the add form
- **THEN** the «Номер» and «Имя» fields and the save button are visible; there is no date field

#### Scenario: Number turned out to be known
- **WHEN** records are found for the entered number (requirement "Known number is determined by search by number")
- **THEN** the form shows the «Дата» field with the entered number and name unchanged

#### Scenario: No "edit" button
- **WHEN** any result row is shown
- **THEN** the row has no «изменить» ("edit") button; a name is corrected only through overwrite (CUJ3)

### Requirement: Known number is determined by search by number
When the form is submitted without a date, the UI SHALL first request `GET /v1/records/by-number` and treat the check as successful only on a response with `complete: true`: `count` 0 means the number is unknown and the add without a date is sent (CUJ1); if records exist, the date prompt is shown (CUJ2). A 200 response without a completeness marker, with records or without, and any failure are a failed check: the add without a date MUST NOT be sent. Trace: CUJ1, CUJ2; ADR 0005, 0060, 0090 (Q-28).

#### Scenario: Number unknown
- **WHEN** for the entered number `by-number` returned 200 with an empty `records`, `count` equal to 0, and `complete: true`
- **THEN** the UI sends `POST /v1/records` with `number` and `name` and without `date`

#### Scenario: Number known
- **WHEN** `by-number` returned a complete response (`complete: true`) with at least one record
- **THEN** the add is not sent; the UI shows the date prompt

#### Scenario: Invalid number on the check
- **WHEN** `by-number` answered `invalid_input` on the field `number`
- **THEN** the add is not sent; the error is shown at the «Номер» field (text — "Value errors in the form")

#### Scenario: Stream cut off before the first record
- **WHEN** `by-number` answered 200 but the stream ended without `count` and `complete` and without a single record
- **THEN** the add is not sent, the number is not considered unknown; what was entered in the form is kept; «Сохранение не выполнено: ошибка хранилища, обратитесь к оператору» ("Save failed: storage error, contact the operator") is shown, the form can be submitted again

#### Scenario: Stream cut off after some records
- **WHEN** `by-number` answered 200 with one or more records but without `complete`
- **THEN** the date prompt is not shown, the add is not sent; the same failure «Сохранение не выполнено: ошибка хранилища, обратитесь к оператору»

#### Scenario: Check failed
- **WHEN** `by-number` did not return 200 (server failure or connection drop)
- **THEN** the add is not sent, what was entered in the form is kept; the failure is shown per `web-ui-shell` with the action «Сохранение» ("Save") as for a request not related to writing (on a connection drop — «Сохранение не выполнено: нет связи с сервером» ("Save failed: no connection to the server"); "outcome unknown" is not shown, no write was sent)

#### Scenario: Number became known after the check
- **WHEN** between the check and the add another client added a record for this number
- **THEN** the add without a date is handled by the server as usual (a record on 1876-03-07, `unchanged`, or a name conflict), and the UI shows its outcome, including the date from the response (boundary accepted, ADR 0090 item 2)

### Requirement: Adding a new number (CUJ1)
On `201` or `200` for an add without a date, the UI SHALL show the save message per the requirement "What to show after saving"; the record must be findable by number search and by word prefix of the name. The UI MUST NOT fill in the date itself. Trace: CUJ1; ADR 0005, 0090 (Q-61).

#### Scenario: Saving an unknown number
- **WHEN** the user enters «+79991234567» and «Ann» for an unknown number and submits the form
- **THEN** `POST /v1/records` contains `number` and `name` and does not contain `date`; the UI shows the save with the date `1876-03-07` from the response

#### Scenario: Find the saved record
- **WHEN** after saving, a search is run by number «+79991234567» and by words «ann»
- **THEN** both results contain the row «Ann», «+79991234567»

#### Scenario: Number with separators
- **WHEN** «+7 (999) 123-45-67» is entered
- **THEN** the UI sends the input as is; the message and results show the canonical form «+79991234567» from the response

### Requirement: Date prompt for a known number (CUJ2)
For a known number, the UI SHALL show the message «Номер {номер} уже есть в базе. Введите дату, с которой номер закрепляется за этим именем» ("Number {number} is already in the database. Enter the date from which the number is assigned to this name"), the number history (date, name, by date descending), and the «Дата» field; without a date the save MUST NOT be sent. A mandatory date in this step is a rule of the UI itself (the server does not require a date for add) and the only exception to "Value errors in the form". Trace: CUJ2; ADR 0002, 0007, 0091 (Q-37).

#### Scenario: Date prompt
- **WHEN** the number is known and has records on 2020-01-01 «Bob» and 2010-05-05 «Ann»
- **THEN** the UI shows the message, both records in this order, and the «Дата» field; the entered number and name are kept

#### Scenario: Date not entered
- **WHEN** the user submits the form with an empty «Дата» field for a known number
- **THEN** no request goes to the server (a UI rule); the date field shows «Дата: укажите дату» ("Date: specify the date")

#### Scenario: Declining to enter a date
- **WHEN** the user clicks «Отмена» ("Cancel") in the date prompt or changes the number
- **THEN** nothing is saved, the «Дата» field is hidden, the entered number and name remain

#### Scenario: New date
- **WHEN** the user enters «2024-05-01», for which the number has no record
- **THEN** the UI sends `POST /v1/records` with `number`, `name`, `date`; the record is created, the UI shows the save; records on other dates are unchanged

#### Scenario: Future date
- **WHEN** the date «2999-01-01» is entered
- **THEN** the record is created: the UI imposes no upper bound other than the server's

### Requirement: Date entry
The UI SHALL pass the entered date to the server without recomputation or parsing (the form «ГГГГ-ММ-ДД» ("YYYY-MM-DD") — `web-ui-shell`, "Date format in the UI") and show a date error at the «Дата» field per the `web-ui-shell` table; date validity (calendar, range 1876-03-07…9999-12-31) is decided by the server (except non-emptiness in the CUJ2 step), on an error no record is created, the input is kept. Trace: CUJ2, CUJ3; ADR 0024, 0089, 0090.

#### Scenario: Format hint
- **WHEN** the «Дата» field is shown
- **THEN** the format «ГГГГ-ММ-ДД» is written next to it

#### Scenario: Date earlier than allowed
- **WHEN** «1876-03-06» is entered
- **THEN** the server returned `invalid_input`/`date`/`out_of_range`; the field shows «Дата: не раньше 1876-03-07» ("Date: not earlier than 1876-03-07"); no record is created, the input is kept

#### Scenario: Nonexistent day
- **WHEN** «2021-02-30» is entered
- **THEN** the field shows «Дата: такого дня нет» ("Date: no such day")

#### Scenario: Other syntax
- **WHEN** «01.05.2024» is entered
- **THEN** the field shows «Дата: нужен вид ГГГГ-ММ-ДД» ("Date: the form YYYY-MM-DD is required"); the UI does not rewrite the value

#### Scenario: Explicit date 1876-03-07
- **WHEN** «1876-03-07» is entered
- **THEN** the request goes with this date like with any other

### Requirement: Overwrite prompt (CUJ3)
On `409 name_conflict`, the UI SHALL show the dialog «На дату {дата} у номера {номер} уже записано имя «{current_name}». Заменить его на «{введённое имя}»? Старое имя будет стёрто» ("On date {date}, number {number} already has the name "{current_name}". Replace it with "{entered name}"? The old name will be erased") with the buttons «Да, заменить» ("Yes, replace") and «Нет» ("No"); the record does not change until the answer. Trace: CUJ3; ADR 0005 item 9, 0006, 0091 (Q-37).

#### Scenario: Conflict
- **WHEN** (number, 2024-05-01) holds «Bob», and the user saves «Ann» on this date
- **THEN** the UI shows the dialog with «Bob» and «Ann»; server data is unchanged

#### Scenario: Name differs in case
- **WHEN** «ANN» is stored, «Ann» is entered for the same date
- **THEN** the same dialog is shown (no silent success)

#### Scenario: Matching name
- **WHEN** the same name is entered (after whitespace trimming and NFC)
- **THEN** the server answered `unchanged`; there is no dialog, the result is shown per "What to show after saving"

#### Scenario: Date from the request
- **WHEN** the dialog is shown for an add without a date
- **THEN** the record date is considered to be 1876-03-07; it is shown in the dialog text

### Requirement: "Yes" answer to overwrite
On «Да, заменить», the UI SHALL send `POST /v1/records/rename` with `number`, `date` (the date of the submitted add, and for an add without a date — 1876-03-07), the entered `name`, and `expected_version` from the `name_conflict` body; there MUST NOT be repeated submissions while the request is in flight. Trace: CUJ3, CUJ12; ADR 0005, 0025, 0060 (Q-34, Q-90).

#### Scenario: Replacement
- **WHEN** the user answers «Да, заменить», the version is current
- **THEN** the `rename` request contains all four fields; the record's name is replaced, the number's other dates are untouched; the UI shows the save

#### Scenario: Double click
- **WHEN** the user quickly clicks «Да, заменить» twice
- **THEN** one request goes to the server; the buttons are disabled until the response

#### Scenario: Version is stale
- **WHEN** `rename` answered `409 version_conflict`
- **THEN** "Stale data (CUJ12)" is performed

#### Scenario: Unchanged
- **WHEN** `rename` answered `200` with `outcome` equal to `unchanged`
- **THEN** the UI shows «Запись уже есть, данные не изменились» ("The record already exists, data unchanged"), not an error

### Requirement: "No" answer to overwrite
On «Нет», the UI SHALL close the dialog without a request to the server and wait for new input; the entered number, name, and date remain in the form. Trace: CUJ3; ADR 0005 item 9, 0091 (Q-37).

#### Scenario: Refusal
- **WHEN** the user answers «Нет»
- **THEN** data is unchanged, no requests; the form holds the same number, name, date; focus is in the «Имя» field

#### Scenario: New input
- **WHEN** after «Нет» the user changes the name and submits the form
- **THEN** the UI sends an add with the same number and date and the new name, without a new number check

### Requirement: What to show after saving
After a successful add or replacement, the UI SHALL show «Сохранено: {имя}, {номер}, с {дата}» ("Saved: {name}, {number}, from {date}") (for `unchanged`, «Запись уже есть, данные не изменились» instead), clear the form, hide the date field, and show the history of this number (a search by number). The message stays until the next action. This is a default (D-83). Trace: CUJ1, CUJ2, CUJ3; ADR 0011, 0091 (Q-61).

#### Scenario: Message and history
- **WHEN** a record is created
- **THEN** the message with the number, name, and date from the response and the results by number with all its records are shown

#### Scenario: Form cleared
- **WHEN** the save is successful
- **THEN** the «Номер», «Имя», «Дата» fields are empty

#### Scenario: Identical repeat
- **WHEN** the server answered `200`, `unchanged`
- **THEN** the UI shows the message «Запись уже есть, данные не изменились», as a success

#### Scenario: History failed
- **WHEN** the save is successful but the number history request failed
- **THEN** the save message stays; the error is shown next to the results area and does not cancel the save

### Requirement: Value errors in the form
The UI SHALL show an `invalid_input` error at the field from `field` per the `web-ui-shell` table ("Value error — field and reason"), moving focus to that field; the input remains, the UI MUST NOT apply its own validity rules, except the mandatory date in the CUJ2 step ("Date prompt for a known number (CUJ2)"). The «Имя» field is labeled with the limit «не более 200 знаков» ("at most 200 characters"). Trace: CUJ1, CUJ2, CUJ3; ADR 0022, 0062, 0088 item 6.

#### Scenario: Number without "+"
- **WHEN** the response contains `number`, `invalid_format`, `hint` equal to `missing_plus`
- **THEN** the «Номер» field shows the `web-ui-shell` text for a number without «+»; no record is created

#### Scenario: Name too long
- **WHEN** a name of 201 characters is sent in an add
- **THEN** the «Имя» field shows an error for `too_long`; the name is not truncated, the other form fields are unchanged

#### Scenario: Limit shown in advance
- **WHEN** the form is open
- **THEN** «не более 200 знаков» is written next to the «Имя» field

### Requirement: Search by number (CUJ4)
For an entered number, the UI SHALL request `GET /v1/records/by-number` and show the whole results as a single list without "next", in response order (date descending); the number from the response in canonical form is shown. A number of invalid shape is an error at the field as in the add form. Trace: CUJ4; ADR 0007, 0035, 0061.

#### Scenario: All records of a number
- **WHEN** the number has 3 records
- **THEN** exactly 3 rows by date DESC are shown, without «Дальше» ("Next")

#### Scenario: Exact match only
- **WHEN** a part of a number «+7999» is entered
- **THEN** there are no rows of numbers merely containing it; the result is what the server returned

#### Scenario: Input with separators
- **WHEN** «+7 999 123-45-67» is entered
- **THEN** the records of number «+79991234567» are shown

#### Scenario: Incomplete stream
- **WHEN** the stream response does not contain `complete: true` at the end
- **THEN** the UI does not accept the list as complete; the search failure per `web-ui-shell` is shown («Поиск не выполнен: ошибка хранилища, обратитесь к оператору» ("Search failed: storage error, contact the operator"))

### Requirement: Result row
A result row (by number and by words) SHALL contain, from the left, a «удалить» ("delete") button with a trash-can icon, then the name, the number in canonical form, and the date «ГГГГ-ММ-ДД»; the name is shown in full and only as text. The columns are the same in both results. This is a default (D-84). Trace: CUJ4, CUJ5, CUJ6; ADR 0092 (Q-29, Q-26).

#### Scenario: Row composition
- **WHEN** the server returned the record («Ann Smith», «+79991234567», 2024-05-01)
- **THEN** the row: a button with a trash can, «Ann Smith», «+79991234567», «2024-05-01»

#### Scenario: Name with markup
- **WHEN** the name is «<b>Ann</b>»
- **THEN** the screen shows literally «<b>Ann</b>»

#### Scenario: Long name
- **WHEN** the name is 200 characters
- **THEN** it is fully visible, wrapped across lines

### Requirement: Date 1876-03-07 and number owners in the results
The UI SHALL show the date 1876-03-07 like any other and MAY accompany it with the hint «дата по умолчанию для записей, добавленных без даты» ("default date for records added without a date"); the UI MUST NOT distinguish past, current, and future owners in the results and MUST NOT show a "current owner": only the list by date DESC is shown. This is a default (D-86, D-87). Trace: CUJ2, CUJ4; ADR 0002, 0007, 0092 (Q-1, Q-2).

#### Scenario: Zero date
- **WHEN** the results contain a record with the date 1876-03-07
- **THEN** the date column shows «1876-03-07»; explicit and filled-in dates are not distinguished

#### Scenario: Future date
- **WHEN** the number has a record with the date 2999-01-01
- **THEN** it is shown at its place in the order without a mark and without computation against the browser clock

### Requirement: Empty results
For empty results by number, the UI SHALL show «Записей с номером {номер} нет» ("There are no records with number {number}"), by words — «По запросу «{запрос}» ничего не найдено» ("Nothing found for query "{query}""); this is not an error, without «Дальше» and without delete buttons. This is a default (D-85). Trace: CUJ4, CUJ5, CUJ13; ADR 0034, 0092 (Q-29).

#### Scenario: Number not found
- **WHEN** `by-number` returned an empty list
- **THEN** the text saying there are no records is shown; the style is not an error style

#### Scenario: No name matches
- **WHEN** `by-words` returned an empty `records` and `next_cursor` equal to `null`
- **THEN** the «ничего не найдено» text with the entered query is shown

#### Scenario: Empty database
- **WHEN** a search is run on a freshly created empty database
- **THEN** the same empty-results texts

#### Scenario: After deleting the last record
- **WHEN** the last record of the results is deleted
- **THEN** the empty-results text is shown

### Requirement: Name search: by word prefix, no exact search in the UI
The UI SHALL search by name with the request `GET /v1/records/by-words` and MUST NOT offer the exact search `by-name` (it exists only in the API, ADR 0011 item 4); next to the field there is the hint «Ищем по началу слов имени; регистр не важен» ("We search by the beginning of name words; case does not matter"). Trace: CUJ5, CUJ7; ADR 0008, 0011, 0031, 0092.

#### Scenario: Word prefix
- **WHEN** the database has «Ann Smith», «John Annabel», «Joann» and the query is «ann»
- **THEN** the first two rows are shown, not «Joann»

#### Scenario: Several words
- **WHEN** the query is «smith ann»
- **THEN** «Ann Smith» is found

#### Scenario: No exact search
- **WHEN** the user looks at the search form
- **THEN** there is no mode, field, or button for «точный поиск» ("exact search"); `by-name` is not called from the UI

### Requirement: "Next" pagination
Under the word list, the UI SHALL show the «Дальше» button if `next_cursor` is not `null`, and on click append the next page below the displayed rows (the same query, `cursor`); on `null` there is no button; the total number of results and page numbers are not shown. This is a default (D-88). Trace: CUJ5; ADR 0032, 0033, 0061, 0092.

#### Scenario: Continuation exists
- **WHEN** there are more than 50 matches and the first 50 are shown
- **THEN** «Дальше» is under the list; on click rows 51 onward are appended below, the earlier ones remain

#### Scenario: Last page
- **WHEN** the response contains `next_cursor` equal to `null`
- **THEN** there is no «Дальше» button

#### Scenario: Count not shown
- **WHEN** results of any length are shown
- **THEN** there is no total number of results

#### Scenario: Invalid cursor
- **WHEN** the server answered `400 invalid_cursor`
- **THEN** the UI shows «Список изменился, повторите поиск» ("The list has changed, repeat the search") with a retry-from-the-first-page button; the shown rows remain

#### Scenario: Double click
- **WHEN** «Дальше» is clicked twice before the response
- **THEN** one page is requested

### Requirement: Row order matches the server response
The UI SHALL show rows in response order (by name under ru rules, then date DESC, then number; by number — date DESC) without re-sorting and without hiding repeats: a record may arrive twice if the name changed between pages. Trace: CUJ4, CUJ5; ADR 0009, 0032, 0036, 0092.

#### Scenario: Alphabet, then date
- **WHEN** the response contains «Ann» 2020 and 2024 and «Bob» 2022
- **THEN** the rows: «Ann» 2024, «Ann» 2020, «Bob» 2022

#### Scenario: Repeat between pages
- **WHEN** a record appears on two pages with different names
- **THEN** both rows are shown, as the server returned

#### Scenario: No local sorting
- **WHEN** pages are appended
- **THEN** new rows follow the earlier ones, the earlier ones are not reordered

### Requirement: Name-search request errors
On `invalid_input` for the field `query`, the UI SHALL show an error at the search field per the `web-ui-shell` table («Запрос: введите слово» ("Query: enter a word"), «Запрос: слишком длинный» ("Query: too long")); empty results are not shown in this case, the previous results are removed, the entered query is kept. Trace: CUJ5; ADR 0034, 0088 item 6.

#### Scenario: Empty query
- **WHEN** the query is empty, only whitespace, or a single «-»
- **THEN** the field shows «Запрос: введите слово»; there are no name results, the empty-results text is not shown

#### Scenario: Too long
- **WHEN** the query is 201 characters
- **THEN** the field shows «Запрос: слишком длинный»

#### Scenario: Previous results
- **WHEN** the query error occurred while results were already shown
- **THEN** the previous results are removed, the entered query is kept

### Requirement: Row "delete" button (CUJ6)
The UI SHALL show in every row of the results by number and by words, on the left, a «удалить» button with a trash-can icon and the accessible name «Удалить» ("Delete") (including the row's name, number, and date). Trace: CUJ6; ADR 0005, 0093.

#### Scenario: Button in every row
- **WHEN** results by number or by words contain rows
- **THEN** each has, on the left, a button with a trash can and the name «Удалить»

#### Scenario: Absent in empty results
- **WHEN** the results are empty
- **THEN** there are no delete buttons

### Requirement: Delete confirmation
On clicking «удалить», the UI SHALL open the dialog «Удалить запись: {имя}, {номер}, {дата}?» ("Delete record: {name}, {number}, {date}?") with the buttons «Удалить» and «Отмена» ("Cancel"); the delete request MUST NOT be sent before confirmation; initial focus is on «Отмена», Esc equals «Отмена». Confirmation before delete is an owner decision (CUJ6); the dialog text, buttons, focus, and Esc are a default (D-94). Trace: CUJ6; ADR 0005, 0093.

#### Scenario: Dialog
- **WHEN** the user clicks «удалить» in the row («Ann», «+79991234567», 2024-05-01)
- **THEN** a dialog with these data; the server is not called

#### Scenario: Double confirmation
- **WHEN** «Удалить» is clicked twice before the response
- **THEN** one request goes to the server

### Requirement: Cancel in the delete dialog
On «Отмена» (button, Esc, or a click outside the dialog), the UI SHALL close the dialog without requests and without messages, leave the results as they are, and return focus to the row's button. The UI MUST NOT offer to undo an already performed delete. This is a default (D-89). Trace: CUJ6, CUJ12; ADR 0093 (Q-63).

#### Scenario: Refusal
- **WHEN** the user clicks «Отмена»
- **THEN** data and list are unchanged, no requests

#### Scenario: No way back
- **WHEN** a record is deleted
- **THEN** the UI has no «вернуть» ("restore") or «отменить удаление» ("undo delete") button

### Requirement: Delete execution
On «Удалить», the UI SHALL send `POST /v1/records/delete` with `number`, `date`, and `expected_version` from the results row; the date MUST NOT be filled in. Success is exactly one record; the number's other records are intact. Trace: CUJ6; ADR 0005, 0025, 0093 (Q-90).

#### Scenario: Request body
- **WHEN** the delete of the row («+79991234567», 2024-05-01, version V) is confirmed
- **THEN** the body contains these three values

#### Scenario: Other dates
- **WHEN** the number has records on three dates and one is deleted
- **THEN** the other two remain

#### Scenario: Read-only
- **WHEN** the server answered `503 read_only`
- **THEN** «Удаление не выполнено: база доступна только для чтения» ("Delete failed: the database is read-only"); the record and the list are unchanged

### Requirement: Re-query after delete
After `200` for a delete, the UI SHALL re-request the original results and show them updated: by number — the same number; by words — the same query from the first page, as many pages as were shown; then the message «Запись удалена» ("Record deleted"). This is a default (D-90). Trace: CUJ6; ADR 0093 (Q-35).

#### Scenario: Results by number
- **WHEN** a record is deleted from results by number
- **THEN** a new `by-number` for this number is run, the list is shown without the deleted row

#### Scenario: Results by words, three pages
- **WHEN** 3 pages are shown and a row on the second is deleted
- **THEN** requests for the first, second, and third pages of the same query are run; the list is without the deleted row

#### Scenario: Re-query failed
- **WHEN** the delete is done, the re-query failed
- **THEN** the row is removed from the list, «Запись удалена, но список обновить не удалось» ("Record deleted, but the list could not be refreshed") is shown with the «Обновить» ("Refresh") button

#### Scenario: List became empty
- **WHEN** the only record is deleted
- **THEN** the empty-results text is shown

### Requirement: Stale data (CUJ12)
On `409 version_conflict` for a delete or replacement, the UI SHALL NOT consider the action performed, SHALL show «данные устарели, список обновлён» ("data is stale, list refreshed"), and re-request the number history (`by-number` for the row's number or the overwrite's number). Trace: CUJ12; ADR 0006, 0025, 0094.

#### Scenario: Stale delete
- **WHEN** after the row was shown the number was changed in another tab, the user confirms the delete
- **THEN** the record is not deleted; «данные устарели, список обновлён» is shown; `by-number` is run

#### Scenario: Stale replacement
- **WHEN** after the overwrite dialog the number was changed in parallel, the user answers «Да, заменить»
- **THEN** the record is not changed; the same message; `by-number` is run

#### Scenario: Another date changed
- **WHEN** another record of the same number is changed
- **THEN** the action is still considered stale

#### Scenario: Re-read failed
- **WHEN** `by-number` after the conflict did not return a complete response (200 with `complete: true`)
- **THEN** the message «данные устарели, список обновлён» is not shown as a performed refresh; «Обновить список не удалось» ("Could not refresh the list") is shown; the action is not repeated

### Requirement: What is re-read on conflict
On a conflict from results by number, the UI SHALL replace the results with the re-read result; from results by words — show the re-read number history in the dialog and, in the rows of this number in the results, remove deleted records and update names and versions, leaving the rest of the results, the order, and the position unchanged. This is a default (D-91). Trace: CUJ12; ADR 0094 (Q-36).

#### Scenario: From results by number
- **WHEN** a conflict on delete in results by number
- **THEN** the results are replaced with the re-read ones, rows by date DESC

#### Scenario: From results by words
- **WHEN** a conflict on deleting a row in results by words
- **THEN** the dialog shows the number history; the number's rows in the results are updated or removed; the other rows and the shown pages are the same

#### Scenario: Row position
- **WHEN** the updated name in a row would change the order
- **THEN** the row stays in place until a new search

### Requirement: Offer to retry the delete
After a delete conflict, the UI SHALL show the number history and, if the record (number, date) still exists, the current record with the buttons «Повторить удаление» ("Retry delete") and «Закрыть» ("Close"); the retry sends the version from the re-read; if the record is gone — «Записи уже нет» ("The record no longer exists") and only «Закрыть». Trace: CUJ6, CUJ12; ADR 0025, 0094.

#### Scenario: Record exists
- **WHEN** the record remains, its name was changed in another tab
- **THEN** the current name and «Повторить удаление» are shown; the delete goes with the new version and after success — a re-query

#### Scenario: Record is gone
- **WHEN** the record was deleted in parallel
- **THEN** «Записи уже нет»; no retry is offered

#### Scenario: User does not retry
- **WHEN** «Закрыть» is clicked
- **THEN** data does not change

#### Scenario: Stale again
- **WHEN** the retry got `version_conflict` again
- **THEN** the same rule applies

### Requirement: Offer to retry the replacement
After a replacement conflict, the UI SHALL keep the entered name and date in the form, show the number history and the buttons «Повторить сохранение» ("Retry save") and «Закрыть»; the retry is a new add with the same number, name, and date, the outcome is decided by the server: created, unchanged, or a new overwrite prompt with the current name and version. Trace: CUJ3, CUJ12; ADR 0025, 0094.

#### Scenario: Name is kept
- **WHEN** the replacement is rejected as stale
- **THEN** the «Имя» field contains the entered name

#### Scenario: Record vanished
- **WHEN** the record was deleted in parallel and the user clicks «Повторить сохранение»
- **THEN** a record (number, date) with the entered name is created, the save is shown as usual

#### Scenario: Name already the same
- **WHEN** the record on the date already has the entered name
- **THEN** the server answered `unchanged`; «Запись уже есть, данные не изменились», not an error

#### Scenario: Name changed in parallel
- **WHEN** a different name is stored on the date
- **THEN** a new overwrite dialog with the current name and version

### Requirement: Record vanished between read and action
On `404 record_not_found` for a delete or replacement, the UI SHALL handle the response as "Stale data" (re-read the number) and offer an action based on the re-read data, rather than show a "not found" error. This is a default (D-92). Trace: CUJ3, CUJ6, CUJ12; ADR 0025, 0094 (Q-68).

#### Scenario: Not found on replacement
- **WHEN** `rename` returned `record_not_found`
- **THEN** the re-read and the «Повторить сохранение» offer are performed

#### Scenario: Not found on delete
- **WHEN** `delete` returned `record_not_found`
- **THEN** «Записи уже нет» with the re-read history

### Requirement: Flow message texts
The UI SHALL show «данные устарели, список обновлён» verbatim, and the CUJ16 failures and value error texts per `web-ui-shell` ("Texts of "read-only"…"), passing the request layer the action: «Сохранение» (number check, add, replacement), «Удаление» ("Delete"), «Поиск» ("Search"); server details are not disclosed. This is a default (D-93). Trace: CUJ12, CUJ16; ADR 0011 item 6, 0088, 0094 (Q-65).

#### Scenario: Read-only on add
- **WHEN** the add received `503 read_only`
- **THEN** «Сохранение не выполнено: база доступна только для чтения» ("Save failed: the database is read-only") is shown

#### Scenario: Read-only on delete
- **WHEN** the delete received `503 read_only`
- **THEN** «Удаление не выполнено: база доступна только для чтения» is shown; the record and the list are unchanged

#### Scenario: Storage error in search
- **WHEN** the search received `500 storage_error` or the number stream was cut off without a completeness marker
- **THEN** «Поиск не выполнен: ошибка хранилища, обратитесь к оператору» is shown; partial results are not passed off as complete

#### Scenario: Not shown in advance
- **WHEN** the UI is open and no action was performed
- **THEN** the "read-only" and "storage error" states are not shown

### Requirement: Entered input is kept when a write fails
On any failure of an add or replacement (value error, conflict, server failure, `read_only`, `storage_error`, `request_timeout`, `write_outcome_unknown`, connection drop), the UI SHALL keep the entered number, name, and date in the form; forms are cleared only after success. Trace: CUJ3, CUJ12, CUJ16; ADR 0026, 0094.

#### Scenario: Server failure
- **WHEN** the add received `503 read_only`
- **THEN** the form fields are unchanged

#### Scenario: Outcome unknown
- **WHEN** the response is `504 write_outcome_unknown` or a drop after sending
- **THEN** the fields are kept; the "outcome unknown" message per `web-ui-shell` is shown; the UI re-reads the number history; a retry by the user is safe (ADR 0026)

#### Scenario: Success
- **WHEN** the save is successful
- **THEN** the fields are cleared (requirement "What to show after saving")
