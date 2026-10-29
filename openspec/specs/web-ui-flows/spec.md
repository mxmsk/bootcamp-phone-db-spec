# web-ui-flows Specification

## Purpose
This capability defines the observable behavior of the web interface in the user journeys CUJ1–CUJ6 and CUJ12 on top of the accepted HTTP API v1: the add form and detection of a known number, the date and overwrite prompts, the look of the results by number and by name words, "next" pagination, delete with confirmation, the reaction to stale data, and message texts. The page shell, sign-in, security policy and common refusals belong to the capability `web-ui-shell`; the API does not change.

## Requirements

### Requirement: Add form: fields and the date field
The add form SHALL contain the fields «Номер» ("Number") and «Имя» ("Name"); the field «Дата» ("Date") MUST be hidden until the server has answered "number known" (CUJ2) or until an overwrite prompt is shown for a record with a date. There is no separate "edit" button. This is the default (D-82). Trace: CUJ1, CUJ2, CUJ3; OD-6; ADR 0005 item 5, 0090 (Q-28, Q-62).

#### Scenario: First opening
- **WHEN** the user opens the add form
- **THEN** the fields «Номер» and «Имя» and the save button are visible; there is no date field

#### Scenario: Number turned out to be known
- **WHEN** an add without a date received `409 number_known` (requirement "Known number is determined by the server response")
- **THEN** the field «Дата» appears in the form, with the entered number and name unchanged

#### Scenario: No "edit" button
- **WHEN** any result row is shown
- **THEN** the row has no "edit" button; a name is corrected only through overwrite (CUJ3)

### Requirement: Adding a new number (CUJ1)
On `201` (created) or `200` `unchanged` (retry, OD-8) to an add without a date, the UI SHALL show a save message per the requirement "What to show after saving"; the record must be findable by number search and by a word prefix of the name. The UI MUST NOT fill in a date by itself. Trace: CUJ1; OD-6, OD-8; ADR 0005, 0090 (Q-61).

#### Scenario: Saving an unknown number
- **WHEN** the user enters «+79991234567» and «Ann» for an unknown number and submits the form
- **THEN** `POST /v1/records` contains `number` and `name` and does not contain `date`; the UI shows the save with the date `1876-03-07` from the response

#### Scenario: Retry without a date after a lost response
- **WHEN** an add without a date created the record, the response was lost, and the user submitted the same form again
- **THEN** the server answered `200` `unchanged`; «Запись уже есть, данные не изменились» ("The record already exists, the data is unchanged") is shown, and there is no date prompt (OD-8)

#### Scenario: Finding the saved record
- **WHEN** after saving, a search by number «+79991234567» and by words «ann» is performed
- **THEN** both results contain the row «Ann», «+79991234567»

#### Scenario: Number with separators
- **WHEN** «+7 (999) 123-45-67» is entered
- **THEN** the UI sends the input as is; the message and the results show the canonical form «+79991234567» from the response

### Requirement: Date prompt for a known number (CUJ2)
For a known number (`409 number_known` to an add without a date) the UI SHALL show the message «Номер {номер} уже есть в базе. Введите дату, с которой номер закрепляется за этим именем» ("Number {number} is already in the database. Enter the date from which the number is assigned to this name"), the number history (date, name, by date descending) and the field «Дата»; without a date a save MUST NOT be sent. The date being mandatory in this step is a rule of the UI itself (without a date the server would answer "number known") and the only exception to "Value errors in the form". Trace: CUJ2; OD-6; ADR 0002, 0007, 0091 (Q-37).

#### Scenario: Date prompt
- **WHEN** the number is known and has records for 2020-01-01 «Bob» and 2010-05-05 «Ann»
- **THEN** the UI shows the message, both records in this order and the field «Дата»; the entered number and name are kept

#### Scenario: Date not entered
- **WHEN** the user submits the form with an empty field «Дата» for a known number
- **THEN** no request goes to the server (a UI rule); next to the date field: «Дата: укажите дату» ("Date: specify a date")

#### Scenario: Declining to enter a date
- **WHEN** the user presses «Отмена» ("Cancel") in the date prompt or changes the number
- **THEN** nothing is saved, the field «Дата» is hidden, and the entered number and name remain

#### Scenario: New date
- **WHEN** the user enters «2024-05-01», for which the number has no record
- **THEN** the UI sends `POST /v1/records` with `number`, `name`, `date`; the record is created, the UI shows the save; records of other dates are unchanged

#### Scenario: Future date
- **WHEN** the date «2999-01-01» is entered
- **THEN** the record is created: the UI imposes no upper bound other than the server's

### Requirement: Date entry
The UI SHALL pass the entered date to the server without recalculation or parsing (the form «ГГГГ-ММ-ДД» (YYYY-MM-DD) is in `web-ui-shell`, "Date format in the UI") and show a date error next to the field «Дата» per the `web-ui-shell` table; date validity (calendar, range 1876-03-07…9999-12-31) is decided by the server (except non-emptiness in the CUJ2 step), on error no record is created, and the input is kept. Trace: CUJ2, CUJ3; ADR 0024, 0089, 0090.

#### Scenario: Format hint
- **WHEN** the field «Дата» is shown
- **THEN** the format «ГГГГ-ММ-ДД» is written next to it

#### Scenario: Date earlier than allowed
- **WHEN** «1876-03-06» is entered
- **THEN** the server returned `invalid_input`/`date`/`out_of_range`; next to the field «Дата: не раньше 1876-03-07» ("Date: not earlier than 1876-03-07"); no record is created, the input is kept

#### Scenario: Nonexistent day
- **WHEN** «2021-02-30» is entered
- **THEN** next to the field «Дата: такого дня нет» ("Date: no such day")

#### Scenario: Other syntax
- **WHEN** «01.05.2024» is entered
- **THEN** next to the field «Дата: нужен вид ГГГГ-ММ-ДД» ("Date: the form YYYY-MM-DD is required"); the UI does not rewrite the value

#### Scenario: Explicit date 1876-03-07
- **WHEN** «1876-03-07» is entered
- **THEN** the request goes with this date like with any other

### Requirement: Overwrite prompt (CUJ3)
On `409 name_conflict` the UI SHALL show the dialog «На дату {дата} у номера {номер} уже записано имя «{current_name}». Заменить его на «{введённое имя}»? Старое имя будет стёрто» ("For the date {date} number {number} already has the name «{current_name}». Replace it with «{entered name}»? The old name will be erased") with the buttons «Да, заменить» ("Yes, replace") and «Нет» ("No"); the record does not change until the answer. Trace: CUJ3; OD-6; ADR 0005 item 9, 0006, 0091 (Q-37).

#### Scenario: Conflict
- **WHEN** (number, 2024-05-01) holds «Bob», and the user saves «Ann» for this date
- **THEN** the UI shows the dialog with «Bob» and «Ann»; the server data is unchanged

#### Scenario: Name differs in case
- **WHEN** «ANN» is saved and «Ann» is entered for the same date
- **THEN** the same dialog is shown (no silent success)

#### Scenario: Matching name
- **WHEN** the same name is entered (after whitespace trimming and NFC)
- **THEN** the server answered `unchanged`; there is no dialog, the result is shown per "What to show after saving"

#### Scenario: Date from the request
- **WHEN** the overwrite dialog is shown (an add without a date does not produce `name_conflict`: it produces `409 number_known` and the date prompt, CUJ2)
- **THEN** the record's date is the date of the add that was sent; it is shown in the dialog text

### Requirement: "Yes" answer to overwrite
On «Да, заменить» the UI SHALL send `POST /v1/records/rename` with `number`, `date` (the date of the add that was sent), the entered `name` and `expected_version` from the `name_conflict` body; there MUST NOT be repeated submissions while the request is in progress. Trace: CUJ3, CUJ12; ADR 0005, 0025, 0060 (Q-34, Q-90).

#### Scenario: Replacement
- **WHEN** the user answers «Да, заменить», and the version is current
- **THEN** the `rename` request contains all four fields; the record's name is replaced, the other dates of the number are untouched; the UI shows the save

#### Scenario: Double press
- **WHEN** the user presses «Да, заменить» twice quickly
- **THEN** one request goes to the server; the buttons are unavailable until the response

#### Scenario: Version is stale
- **WHEN** `rename` answered `409 version_conflict`
- **THEN** "Stale data (CUJ12)" is carried out

#### Scenario: Unchanged
- **WHEN** `rename` answered `200` with `outcome` equal to `unchanged`
- **THEN** the UI shows «Запись уже есть, данные не изменились», not an error

### Requirement: "No" answer to overwrite
On «Нет» the UI SHALL close the dialog without a request to the server and wait for new input; the entered number, name and date remain in the form. Trace: CUJ3; ADR 0005 item 9, 0091 (Q-37).

#### Scenario: Refusal
- **WHEN** the user answers «Нет»
- **THEN** the data is unchanged, there are no requests; the form contains the same number, name and date; focus is in the field «Имя»

#### Scenario: New input
- **WHEN** after «Нет» the user changes the name and submits the form
- **THEN** the UI sends an add with the same number and date and the new name, without a by-number request

### Requirement: What to show after saving
After a successful add or replacement the UI SHALL show «Сохранено: {имя}, {номер}, с {дата}» ("Saved: {name}, {number}, from {date}") (for `unchanged`, «Запись уже есть, данные не изменились» instead), clear the form, hide the date field and show the history of this number (a by-number request). The message stays until the next action. This is the default (D-83). Trace: CUJ1, CUJ2, CUJ3; ADR 0011, 0091 (Q-61).

#### Scenario: Message and history
- **WHEN** a record is created
- **THEN** the message with the number, name and date from the response and the by-number results with all its records are shown

#### Scenario: Form clearing
- **WHEN** the save is successful
- **THEN** the fields «Номер», «Имя», «Дата» are empty

#### Scenario: Identical retry
- **WHEN** the server answered `200`, `unchanged`
- **THEN** the UI shows the message «Запись уже есть, данные не изменились», as a success

#### Scenario: History failed
- **WHEN** the save is successful but the number-history request failed
- **THEN** the save message stays; next to the results area «История номера не загружена: нет связи с сервером» ("Number history not loaded: no connection to the server") is shown (for an incomplete response, «…: ответ получен не полностью» ("…: the response was received incompletely")) and «Повторить» ("Retry"); the words «Сохранение не выполнено» ("Save not performed") are not used, and the save is not undone

### Requirement: Value errors in the form
The UI SHALL show an `invalid_input` error next to the field from `field` per the `web-ui-shell` table ("Value error: field and reason"), moving focus to that field; the input remains, the UI MUST NOT apply its own validity rules, except for the date being mandatory in the CUJ2 step ("Date prompt for a known number (CUJ2)"). The field «Имя» is labeled with the cap «не более 200 знаков» ("at most 200 characters"). Trace: CUJ1, CUJ2, CUJ3; ADR 0022, 0062, 0088 item 6.

#### Scenario: Number without «+»
- **WHEN** the response contains `number`, `invalid_format`, `hint` equal to `missing_plus`
- **THEN** next to the field «Номер» the `web-ui-shell` text for a number without «+» is shown; no record is created

#### Scenario: Name too long
- **WHEN** a name of 201 characters is sent in an add
- **THEN** next to the field «Имя» is the `too_long` error; the name is not truncated, the other form fields are unchanged

#### Scenario: Cap shown in advance
- **WHEN** the form is open
- **THEN** «не более 200 знаков» is written next to the field «Имя»

### Requirement: Search by number (CUJ4)
For the entered number the UI SHALL request `GET /v1/records/by-number` and show the whole results as one list without "next", in the response order (date descending); the number is from the response, in canonical form. A number of invalid shape gives an error next to the field as in the add form. UI responsiveness is guaranteed up to ≈ 10^4 records of a number; beyond that it is not guaranteed, and the results are not truncated. Trace: CUJ4; ADR 0007, 0035, 0061; ADR 0100 (OD-7, Q-163).

#### Scenario: All records of a number
- **WHEN** a number has 3 records
- **THEN** exactly 3 rows are shown by date DESC, without «Дальше» ("Next")

#### Scenario: Exact match only
- **WHEN** a part of a number «+7999» is entered
- **THEN** there are no rows of numbers that merely contain it; the result is what the server returned

#### Scenario: Input with separators
- **WHEN** «+7 999 123-45-67» is entered
- **THEN** the records of the number «+79991234567» are shown

#### Scenario: Incomplete stream
- **WHEN** the stream response (status 200) does not end with `complete: true`
- **THEN** the UI does not accept the list as complete; «Поиск не выполнен: ответ получен не полностью» ("Search not performed: the response was received incompletely") and «Повторить» are shown (`web-ui-shell`)

#### Scenario: History up to 10^4 records
- **WHEN** a number has ≈ 10^4 records
- **THEN** all records are shown, the page is responsive

#### Scenario: More than 10^4 records
- **WHEN** a number has more than 10^4 records
- **THEN** responsiveness is not guaranteed; records are not truncated, an incomplete stream is not taken for a complete one

### Requirement: Result row
A result row (by number and by words) SHALL contain on the left a «удалить» ("delete") button with a trash-can icon, then the name, the number in canonical form and the date «ГГГГ-ММ-ДД»; the name is shown in full and only as text. The columns are the same in both result types. This is the default (D-84). Trace: CUJ4, CUJ5, CUJ6; ADR 0092 (Q-29, Q-26).

#### Scenario: Row composition
- **WHEN** the server returned a record («Ann Smith», «+79991234567», 2024-05-01)
- **THEN** the row: a button with a trash can, «Ann Smith», «+79991234567», «2024-05-01»

#### Scenario: Name with markup
- **WHEN** the name is «<b>Ann</b>»
- **THEN** the screen literally shows «<b>Ann</b>»

#### Scenario: Long name
- **WHEN** a name of 200 characters
- **THEN** it is fully visible, wrapped across lines

### Requirement: Date 1876-03-07 and number owners in the results
The UI SHALL show the date 1876-03-07 like any other and MAY accompany it with the hint «дата по умолчанию для записей, добавленных без даты» ("the default date for records added without a date"); the UI MUST NOT distinguish past, current and future owners in the results and MUST NOT show a "current owner": only the list by date DESC is shown. This is the default (D-86, D-87). Trace: CUJ2, CUJ4; ADR 0002, 0007, 0092 (Q-1, Q-2).

#### Scenario: Zero date
- **WHEN** the results contain a record with the date 1876-03-07
- **THEN** the date column shows «1876-03-07»; an explicit and a substituted date are not distinguished

#### Scenario: Future date
- **WHEN** a number has a record with the date 2999-01-01
- **THEN** it is shown in its place in the order, without a mark and without computation against the browser clock

### Requirement: Empty results
For empty results by number the UI SHALL show «Записей с номером {номер} нет» ("There are no records with number {number}"), for results by words «По запросу «{запрос}» ничего не найдено» ("Nothing found for the query «{query}»"); this is not an error, with no «Дальше» and no delete buttons. This is the default (D-85). Trace: CUJ4, CUJ5, CUJ13; ADR 0034, 0092 (Q-29).

#### Scenario: Number not found
- **WHEN** `by-number` returned an empty list
- **THEN** the text that there are no records is shown; the style is not that of an error

#### Scenario: No name matches
- **WHEN** `by-words` returned an empty `records` and `next_cursor` equal to `null`
- **THEN** the text «ничего не найдено» with the entered query is shown

#### Scenario: Empty database
- **WHEN** a search is performed on a freshly created empty database
- **THEN** the same empty-results texts

#### Scenario: After deleting the last record
- **WHEN** the last record of the results is deleted
- **THEN** the empty-results text is shown

### Requirement: Name search: by word prefix, no exact search in the UI
The UI SHALL search by name with the request `GET /v1/records/by-words` and MUST NOT offer the exact search `by-name` (it exists only in the API, ADR 0011 item 4); next to the field there is the hint «Ищем по началу слов имени; регистр не важен» ("We search by the beginning of the name's words; case does not matter"). Trace: CUJ5, CUJ7; ADR 0008, 0011, 0031, 0092.

#### Scenario: Word beginning
- **WHEN** the database has «Ann Smith», «John Annabel», «Joann» and the query is «ann»
- **THEN** the first two rows are shown, «Joann» is not

#### Scenario: Several words
- **WHEN** the query is «smith ann»
- **THEN** «Ann Smith» is found

#### Scenario: No exact search
- **WHEN** the user looks at the search form
- **THEN** there is no "exact search" mode, field or button; `by-name` is not called from the UI

### Requirement: "Next" pagination
Under the by-words list the UI SHALL show a «Дальше» button if `next_cursor` is not `null`, and on press append the next page under the rows already shown (the same request, `cursor`); with `null` there is no button; the total number of results and page numbers are not shown. This is the default (D-88). Trace: CUJ5; ADR 0032, 0033, 0061, 0092.

#### Scenario: There is a continuation
- **WHEN** there are more than 50 matches and the first 50 are shown
- **THEN** «Дальше» is under the list; on press rows 51 onward are added below, the earlier ones remain

#### Scenario: Last page
- **WHEN** the response contains `next_cursor` equal to `null`
- **THEN** there is no «Дальше» button

#### Scenario: Count is not shown
- **WHEN** results of any length are shown
- **THEN** there is no total number of results

#### Scenario: Invalid cursor
- **WHEN** the server answered `400 invalid_cursor`
- **THEN** the UI shows «Список изменился, повторите поиск» ("The list changed, repeat the search") with a retry-from-the-first-page button; the rows already shown remain

#### Scenario: Double press
- **WHEN** «Дальше» is pressed twice before the response
- **THEN** one page is requested

### Requirement: Row order matches the server response
The UI SHALL show rows in the response order (by name under ru rules, then date DESC, then number; by number: date DESC) without re-sorting and without hiding repeats: a record may arrive twice if the name changed between pages. Trace: CUJ4, CUJ5; ADR 0009, 0032, 0036, 0092.

#### Scenario: Alphabet, then date
- **WHEN** the response contains «Ann» 2020 and 2024 and «Bob» 2022
- **THEN** the rows: «Ann» 2024, «Ann» 2020, «Bob» 2022

#### Scenario: Repeat between pages
- **WHEN** a record appears on two pages with different names
- **THEN** both rows are shown, as the server returned them

#### Scenario: No local sorting
- **WHEN** pages are appended
- **THEN** new rows go after the earlier ones, the earlier ones are not rearranged

### Requirement: Name-search request errors
On `invalid_input` for the field `query` the UI SHALL show an error next to the search field per the `web-ui-shell` table («Запрос: введите слово» ("Query: enter a word"), «Запрос: слишком длинный» ("Query: too long")); empty results are not shown, the earlier results are removed, and the entered query is kept. Trace: CUJ5; ADR 0034, 0088 item 6.

#### Scenario: Empty query
- **WHEN** the query is empty, only spaces, or a single «-»
- **THEN** next to the field «Запрос: введите слово»; there are no name results, the empty-results text is not shown

#### Scenario: Too long
- **WHEN** a query of 201 characters
- **THEN** next to the field «Запрос: слишком длинный»

#### Scenario: Earlier results
- **WHEN** the request error occurs while results are already shown
- **THEN** the earlier results are removed, the entered query is kept

### Requirement: Row "delete" button (CUJ6)
The UI SHALL show in every row of the results by number and by words, on the left, a «удалить» button with a trash-can icon and the accessible name «Удалить» ("Delete") (including the row's name, number and date). Trace: CUJ6; ADR 0005, 0093.

#### Scenario: Button in every row
- **WHEN** results by number or by words contain rows
- **THEN** each has on the left a button with a trash can and the name «Удалить»

#### Scenario: Absent in empty results
- **WHEN** the results are empty
- **THEN** there are no delete buttons

### Requirement: Delete confirmation
On pressing «удалить» the UI SHALL open the dialog «Удалить запись: {имя}, {номер}, {дата}?» ("Delete the record: {name}, {number}, {date}?") with the buttons «Удалить» ("Delete") and «Отмена» ("Cancel"); the delete request MUST NOT be sent before confirmation; initial focus is on «Отмена», Esc equals «Отмена». Confirmation before delete is an owner decision (CUJ6); the dialog text, buttons, focus and Esc are the default (D-94). Trace: CUJ6; ADR 0005, 0093.

#### Scenario: Dialog
- **WHEN** the user presses «удалить» in a row («Ann», «+79991234567», 2024-05-01)
- **THEN** the dialog with this data; the server is not called

#### Scenario: Double confirmation
- **WHEN** «Удалить» is pressed twice before the response
- **THEN** one request goes to the server

### Requirement: Cancel in the delete dialog
On «Отмена» (the button, Esc or a click outside the dialog) the UI SHALL close the dialog without requests and without messages, leave the results as they are and return focus to the row's button. The UI MUST NOT offer undoing an already performed delete. This is the default (D-89). Trace: CUJ6, CUJ12; ADR 0093 (Q-63).

#### Scenario: Refusal
- **WHEN** the user presses «Отмена»
- **THEN** the data and the list are unchanged, there are no requests

#### Scenario: No way back
- **WHEN** a record is deleted
- **THEN** the UI has no «вернуть» ("restore") or "undo delete" button

### Requirement: Delete execution
On «Удалить» the UI SHALL send `POST /v1/records/delete` with `number`, `date` and `expected_version`: the number version from the root of the by-number response, or from the record in the by-words results; the date MUST NOT be filled in. Success is exactly one record; the other records of the number are intact. Trace: CUJ6; OD-6 (Q-128); ADR 0005, 0025, 0093 (Q-90).

#### Scenario: Request body
- **WHEN** the delete of the row («+79991234567», 2024-05-01) from the by-number results is confirmed, and the number version V is in the response root
- **THEN** the body contains these three values, version V is taken from the response root, not from the row

#### Scenario: Other dates
- **WHEN** a number has records for three dates and one is deleted
- **THEN** the other two remain

#### Scenario: Read-only
- **WHEN** the server answered `503 read_only`
- **THEN** «Удаление не выполнено: база доступна только для чтения»; the record and the list are unchanged

### Requirement: Re-query after delete
After `200` to a delete the UI SHALL re-request the original results and show them updated: by number, the same number; by words, the same query from the first page, as many pages as were shown; then the message «Запись удалена» ("Record deleted"). This is the default (D-90). Trace: CUJ6; ADR 0093 (Q-35).

#### Scenario: Results by number
- **WHEN** a record is deleted from results by number
- **THEN** a new `by-number` for this number is performed, the list without the deleted row is shown

#### Scenario: Results by words, three pages
- **WHEN** 3 pages are shown and a row on the second is deleted
- **THEN** requests for the first, second and third pages of the same query are performed; the list is without the deleted row

#### Scenario: Re-query failed
- **WHEN** the delete is done, the re-query failed
- **THEN** the row is removed from the list, «Запись удалена, но список обновить не удалось» ("Record deleted, but the list could not be refreshed") is shown with a «Обновить» ("Refresh") button

#### Scenario: List became empty
- **WHEN** the only record is deleted
- **THEN** the empty-results text is shown

### Requirement: Stale data (CUJ12)
On `409 version_conflict` for a delete or replacement the UI SHALL not treat the action as performed, show «данные устарели, список обновлён» ("data is stale, the list is refreshed") and re-request the number history (`by-number` for the number of the row or of the overwrite). Trace: CUJ12; ADR 0006, 0025, 0094.

#### Scenario: Stale delete
- **WHEN** after a row is shown the number is changed by another tab, and the user confirms the delete
- **THEN** the record is not deleted; «данные устарели, список обновлён» is shown; `by-number` is performed

#### Scenario: Stale replacement
- **WHEN** after the overwrite dialog the number is changed in parallel, and the user answers «Да, заменить»
- **THEN** the record is unchanged; the same message; `by-number` is performed

#### Scenario: Another date changed
- **WHEN** another record of the same number is changed
- **THEN** the action is still considered stale

#### Scenario: Re-read failed
- **WHEN** `by-number` after the conflict did not return a complete response (200 with `complete: true`)
- **THEN** the message «данные устарели, список обновлён» is not shown as a completed refresh; «Обновить список не удалось» ("The list could not be refreshed") is shown; the action is not repeated

### Requirement: What is re-read on conflict
On a conflict from results by number the UI SHALL replace the results with the re-read result; from results by words, show the re-read number history in the dialog and in the rows of this number in the results remove the deleted records and update names and versions, leaving the rest of the results, their order and position unchanged. This is the default (D-91). Trace: CUJ12; ADR 0094 (Q-36).

#### Scenario: From results by number
- **WHEN** a conflict on delete in the by-number results
- **THEN** the results are replaced with the re-read ones, rows by date DESC

#### Scenario: From results by words
- **WHEN** a conflict on deleting a row in the by-words results
- **THEN** the number history is in the dialog; the number's rows in the results are updated or removed; the other rows and the shown pages are the same

#### Scenario: Row position
- **WHEN** an updated name in a row would change the order
- **THEN** the row stays in place until a new search

### Requirement: Offer to retry the delete
After a delete conflict the UI SHALL show the number history and, if the record (number, date) still exists, the current record with the buttons «Повторить удаление» ("Retry delete") and «Закрыть» ("Close"); the retry sends the version from the re-read; if the record is gone, «Записи уже нет» ("The record no longer exists") and only «Закрыть». Trace: CUJ6, CUJ12; ADR 0025, 0094.

#### Scenario: Record exists
- **WHEN** the record remained, the name was changed by another tab
- **THEN** the current name and «Повторить удаление» are shown; the delete goes with the new version and after success a re-query

#### Scenario: Record gone
- **WHEN** the record was deleted in parallel
- **THEN** «Записи уже нет»; no retry is offered

#### Scenario: User does not retry
- **WHEN** «Закрыть» is pressed
- **THEN** the data does not change

#### Scenario: Stale again
- **WHEN** the retry again received `version_conflict`
- **THEN** the same rule applies

### Requirement: Offer to retry the replacement
After a replacement conflict the UI SHALL leave the entered name and date in the form, show the number history and the buttons «Повторить сохранение» ("Retry save") and «Закрыть»; the retry is a new add with the same number, name and date, and the server decides the outcome: created, unchanged, or a new overwrite prompt with the current name and version. Trace: CUJ3, CUJ12; ADR 0025, 0094.

#### Scenario: Name is kept
- **WHEN** a replacement is rejected as stale
- **THEN** the field «Имя» contains the entered name

#### Scenario: Record vanished
- **WHEN** the record was deleted in parallel and the user presses «Повторить сохранение»
- **THEN** a record (number, date) with the entered name is created, the save is shown as usual

#### Scenario: Name already the same
- **WHEN** the record for the date already has the entered name
- **THEN** the server answered `unchanged`; «Запись уже есть, данные не изменились», not an error

#### Scenario: Name changed in parallel
- **WHEN** a different name is recorded for the date
- **THEN** a new overwrite dialog with the current name and version

### Requirement: Record vanished between read and action
On `404 record_not_found` for a delete or replacement the UI SHALL handle the response as "Stale data" (re-read the number) and offer an action based on the re-read data, rather than showing a "not found" error. This is the default (D-92). Trace: CUJ3, CUJ6, CUJ12; ADR 0025, 0094 (Q-68).

#### Scenario: Not found on replacement
- **WHEN** `rename` returned `record_not_found`
- **THEN** the re-read and the «Повторить сохранение» offer are performed

#### Scenario: Not found on delete
- **WHEN** `delete` returned `record_not_found`
- **THEN** «Записи уже нет» with the re-read history

### Requirement: Flow message texts
The UI SHALL show «данные устарели, список обновлён» verbatim, and the CUJ16 refusals and value-error texts per `web-ui-shell` ("\"Read-only\" and \"storage error\" texts on action failure"), passing the request layer the action: «Сохранение» (Save) (add, number history after "number known", replacement), «Удаление» (Delete), «Поиск» (Search); server details are not disclosed. This is the default (D-93). Trace: CUJ12, CUJ16; ADR 0011 item 6, 0088, 0094 (Q-65).

#### Scenario: Read-only on add
- **WHEN** an add received `503 read_only`
- **THEN** «Сохранение не выполнено: база доступна только для чтения» is shown

#### Scenario: Read-only on delete
- **WHEN** a delete received `503 read_only`
- **THEN** «Удаление не выполнено: база доступна только для чтения» is shown; the record and the list are unchanged

#### Scenario: Storage error in search
- **WHEN** a search received `500 storage_error`
- **THEN** «Поиск не выполнен: ошибка хранилища, обратитесь к оператору» is shown; a partial result is not passed off as complete

#### Scenario: By-number stream cut off
- **WHEN** the by-number stream got 200 but ended without a completeness marker
- **THEN** «Поиск не выполнен: ответ получен не полностью» and «Повторить» are shown (`web-ui-shell`), not "storage error"

#### Scenario: Not shown in advance
- **WHEN** the UI is open and no action was performed
- **THEN** the "read-only" and "storage error" states are not shown

### Requirement: Entered input is kept when a write fails
On any failure of an add or replacement (value error, conflict, server refusal, `read_only`, `storage_error`, `request_timeout`, `write_outcome_unknown`, connection drop) the UI SHALL leave the entered number, name and date in the form; forms are cleared only after success. Trace: CUJ3, CUJ12, CUJ16; ADR 0026, 0094.

#### Scenario: Server refusal
- **WHEN** an add received `503 read_only`
- **THEN** the form fields are unchanged

#### Scenario: Outcome unknown
- **WHEN** the response is `504 write_outcome_unknown` or a drop after sending
- **THEN** the fields are kept; "outcome unknown" is shown per `web-ui-shell`; the UI re-reads and retries nothing by itself, and the user checks the outcome by search by number (CUJ4); a repeated add without a date after a successful first attempt gets `200` `unchanged` («Запись уже есть, данные не изменились»; OD-8), a replacement with the old version gets "data is stale" (ADR 0026)

#### Scenario: Success
- **WHEN** the save is successful
- **THEN** the fields are cleared (requirement "What to show after saving")

### Requirement: Known number is determined by the server response
The UI SHALL send an add without a date immediately, without a preliminary by-number request; the response `409 number_known` means a known number (CUJ2), and `200` `unchanged` an identical retry (OD-8): the UI requests `GET /v1/records/by-number` for the history and shows the date prompt, accepting the history as complete only with `complete: true`. The CUJ1/CUJ2 race is closed by a server rule, not by the UI (OD-6, Q-160). Trace: CUJ1, CUJ2; OD-6, OD-8; ADR 0005, 0060, 0090 (Q-28).

#### Scenario: Number unknown
- **WHEN** a number has no records and the UI sent an add without a date
- **THEN** the server answered `201`; no by-number request was made before the add

#### Scenario: Number known
- **WHEN** an add without a date received `409 number_known`, and then `by-number` returned a complete response (`complete: true`) with records
- **THEN** no record is created; the UI shows the date prompt with the number history

#### Scenario: Retry after an unknown outcome
- **WHEN** a number has a record for 1876-03-07 with the same name and the UI sent an add without a date
- **THEN** the server answered `200` `unchanged`; the date prompt and `by-number` are not performed, the result is shown per "What to show after saving"

#### Scenario: Invalid number
- **WHEN** an add received `invalid_input` for the field `number`
- **THEN** the error is shown next to the field «Номер» (text: "Value errors in the form")

#### Scenario: History cut off
- **WHEN** after `409 number_known` the `by-number` stream ended without `count` and `complete`
- **THEN** the date prompt is not shown; what was entered in the form is kept; «Сохранение не выполнено: ответ получен не полностью» and «Повторить» are shown, and the form can be submitted again

#### Scenario: History not received
- **WHEN** after `409 number_known` `by-number` did not return 200 (server refusal or connection drop)
- **THEN** what was entered in the form is kept; the refusal is shown per `web-ui-shell` with the action «Сохранение» (on a connection drop: «Сохранение не выполнено: нет связи с сервером»; "outcome unknown" is not shown, the record is not created)

#### Scenario: Number became known in a race
- **WHEN** another client added a record of this number immediately before the add without a date
- **THEN** the server answered `409 number_known`, no record for 1876-03-07 is created, the UI shows the date prompt
