# web-ui-shell Specification

## Purpose
UI shell: how the server serves the page, how the user signs in through a link with a token, the page security policy, and the common rules for showing refusals (stale token, server start and shutdown, "read-only" and "storage error", value errors, date format). The user journeys CUJ1–CUJ6 and CUJ12 are built on these rules (capability `web-ui-flows`).

## Requirements

### Requirement: UI page is served without a token only on loopback
The server SHALL serve the UI shell (the page `/` and the resources under `/ui/`) with the GET and HEAD methods on the loopback listener in normal operation, without a token and without database data; other methods get 405 `method_not_allowed`, an unknown path under `/ui/` gets 404 `route_not_found`; on the network listener the paths `/` and `/ui/…` get 404 `route_not_found`, like any path without a version (`http-api`, "API version in the path"). Trace: CUJ13; ADR 0011 item 1, 0067 item 5, 0085.

#### Scenario: Opening the page
- **WHEN** a browser requests `GET /` on the loopback listener with a valid Host and without an `Authorization` header
- **THEN** the response is 200 with an HTML page in Russian, which contains no database records and no token

#### Scenario: Page resources
- **WHEN** the page requests its script and styles under `/ui/`
- **THEN** they are served without a token, 200

#### Scenario: Wrong method and unknown resource
- **WHEN** `POST /` or `GET /ui/unknown` is sent
- **THEN** the responses are 405 `method_not_allowed` and 404 `route_not_found` respectively

#### Scenario: Network listener
- **WHEN** `/` or a resource under `/ui/` is requested on the network listener, including from a loopback address
- **THEN** the response is 404 `route_not_found`, the page is not served (`http-api`, "API version in the path")

### Requirement: UI resources are embedded in the program
The server program SHALL contain all UI resources (page, script, styles) inside itself: the UI needs neither a resource directory on disk nor network access. This is the default (D-76). Trace: CUJ13; item 1.2 of the original wording; ADR 0085 item 2.

#### Scenario: Machine without a network and without extra files
- **GIVEN** the machine has only the program, the database and the settings, and no network
- **WHEN** the user opens the link from the server output
- **THEN** the page loads and works

#### Scenario: Program update
- **WHEN** the server is replaced with a program of another version and restarted
- **THEN** the page and its script are served from one and the same program version, with no mixing of old and new resources

### Requirement: Interface language is Russian
All UI texts (labels, buttons, messages) SHALL be in Russian; the page declares the language `ru`. Trace: CUJ16, CUJ13; brief §2 (texts of CUJ16, CUJ12); ADR 0085 item 3.

#### Scenario: Page language
- **WHEN** the page is opened in a browser with any interface language
- **THEN** all texts are Russian, and `<html>` has `lang="ru"`

#### Scenario: Server message is not shown instead of the UI text
- **WHEN** the server response contains `message`
- **THEN** the UI shows its own Russian text based on `code`, and does not need `message`

### Requirement: Strict page security policy
Every shell response SHALL carry `Content-Security-Policy` with `default-src 'none'`, `script-src 'self'`, `style-src 'self'`, `connect-src 'self'`, `base-uri 'none'`, `form-action 'none'`, `frame-ancestors 'none'`, as well as `Referrer-Policy: no-referrer` and `X-Content-Type-Options: nosniff`; the page has no inline scripts or styles. Trace: CUJ13; ADR 0067 item 5 (P-2); ADR 0087.

#### Scenario: Shell headers
- **WHEN** `/`, the script and the styles are requested
- **THEN** every response has the listed headers

#### Scenario: No inline code
- **WHEN** the page HTML is inspected
- **THEN** it has no `<script>` with code, no event-handler attributes (`onclick` and the like) and no `style` attribute

#### Scenario: Embedding in a foreign page
- **WHEN** another page tries to show the UI in a frame
- **THEN** the browser does not allow the embedding (`frame-ancestors 'none'`)

### Requirement: No external resources and no cookies
The page SHALL load only resources of its own origin and call only the API of its own origin; the server MUST NOT set a cookie in any response, and the page MUST NOT read or write cookies or `localStorage`. Trace: CUJ13; P-1, P-5; ADR 0065 item 6, 0087.

#### Scenario: Page requests
- **WHEN** the page runs from opening to closing
- **THEN** all its requests go to the server origin; there are no fonts, scripts, images or analytics from other addresses

#### Scenario: Cookies
- **WHEN** the server responses to page and API requests are inspected
- **THEN** there is no `Set-Cookie` header, and the page does not use `Cookie`

### Requirement: Database data is shown only as text
The UI SHALL output names, numbers, dates and error texts as text, without parsing as HTML and without execution; a value from a server response MUST NOT end up in markup, a handler attribute, an address or `style`. Trace: CUJ4, CUJ5, CUJ13; ADR 0067 item 5 (P-2); ADR 0087.

#### Scenario: Name with markup
- **GIVEN** the database has the name `<img src=x onerror=alert(1)> <b>Ann</b>`
- **WHEN** it is shown in the results
- **THEN** the characters of this name are visible on screen as is, no script runs, and there is no bold font

#### Scenario: Name with quotes and an ampersand
- **WHEN** the results contain the name `O'Brien & "Sons"`
- **THEN** it is shown verbatim

### Requirement: Sign-in by link: token from the fragment
The page SHALL take the UI token from the address fragment (64 hexadecimal characters) on load and on every fragment change in an already open tab (`hashchange`, the tab is not reloaded) and every time remove the fragment from the address bar; a new token replaces the previous one, while the values entered in the forms and the results stay on screen; the token MUST NOT remain in the address bar and MUST NOT be sent in a page request. Trace: CUJ13; ADR 0065 item 1, 0067 item 5 (P-5); ADR 0086 items 1, 4.

#### Scenario: Opening the link from the server output
- **WHEN** the user opens `http://127.0.0.1:<port>/#<token>`
- **THEN** the UI is available immediately, without entering credentials, and the address in the browser bar becomes `http://127.0.0.1:<port>/`

#### Scenario: Fragment does not look like a token
- **WHEN** on load or on a fragment change it is empty or does not consist of 64 hexadecimal characters
- **THEN** it is not treated as a token, the fragment is removed from the address bar, and the previous token (if any) does not change

#### Scenario: New link in the same tab
- **GIVEN** the tab works with a token, its address is `http://127.0.0.1:<port>/`, and values are entered in a form
- **WHEN** the user pastes `http://127.0.0.1:<port>/#<another token>` into this tab's address bar (only the fragment changes, the page is not reloaded)
- **THEN** the new token is in effect, the address is `http://127.0.0.1:<port>/` again, and the entered values and the results stay in place

### Requirement: Token storage: where it is kept and what is not guaranteed
The page SHALL keep the token in the tab's `sessionStorage`, and if that is unavailable, in memory; the token MUST NOT end up in cookies, `localStorage`, request addresses, texts or the console. "The token is not on disk" and "it disappears when the tab is closed" are not guaranteed (residual risks: ADR 0086); the mitigation is that the token is new on every start and is accepted only on loopback. Default (D-77). Trace: CUJ13; P-1, P-2; ADR 0086 item 2.

#### Scenario: Page reload
- **WHEN** the user reloads the tab, with an address without a fragment
- **THEN** the UI keeps working with the same token

#### Scenario: Token is not in cookies or localStorage
- **WHEN** the user has signed in by link and works in the UI
- **THEN** the token is not in cookies or `localStorage`, nor in request addresses, nor in the text on screen, nor in the console

#### Scenario: Residual risks accepted
- **WHEN** the browser has saved `sessionStorage` to disk, brought it back on "reopen closed tab" or session restore, duplicated the tab, or the link with the token stayed in the browser history and the terminal
- **THEN** the token may be available until the server restarts; the specification neither forbids this nor promises the opposite

#### Scenario: Restored tab
- **GIVEN** the browser brought a tab back together with its `sessionStorage`, and the server has restarted since
- **WHEN** the user performs an action
- **THEN** the request gets 401, and «Ссылка устарела…» ("The link is outdated…") is shown ("Old link after server restart"); if the server has not restarted, the UI works with the same token (the residual risk is accepted)

#### Scenario: New tab without a link
- **WHEN** the user opens `http://127.0.0.1:<port>/` in a new tab
- **THEN** there is no token, and «Нет ссылки для входа» ("No token: a link from server output is needed") applies

#### Scenario: sessionStorage unavailable
- **WHEN** access to `sessionStorage` fails with an error
- **THEN** the token is kept in page memory; a reload requires a new link

### Requirement: Token is sent only in the Authorization header
The page SHALL send the token only in the header `Authorization: Bearer <token>` of requests to the API of its own origin (`/v1/…`); the token MUST NOT be in a request address, body, cookie, text on screen or console messages. Trace: CUJ13; ADR 0065 item 6, 0067 item 5 (P-1, P-5); `api-access`.

#### Scenario: Search request
- **WHEN** the user performs a search
- **THEN** the request goes to `/v1/…` with the header `Authorization: Bearer <token>`, and the request address does not contain the token

#### Scenario: Token in the interface
- **WHEN** the user looks at the screen and the address bar after signing in
- **THEN** the token is nowhere to be seen

### Requirement: No token: a link from server output is needed
If on page load there is no token (neither in the fragment, nor in `sessionStorage` or memory: the tab was opened without a link), the page SHALL show «Нет ссылки для входа. Откройте ссылку, которую сервер напечатал при запуске» ("No sign-in link. Open the link the server printed at startup") and make no requests to the API; forms and results are not shown. The requirement does not apply to the state after a 401: there the form and the results remain ("Old link after server restart"). Trace: CUJ13; ADR 0067 item 5; ADR 0088 item 1.

#### Scenario: Address without a token
- **WHEN** the user opens `http://127.0.0.1:<port>/` in a new tab
- **THEN** this text is shown; there are no forms or results; there were no requests to `/v1/`

### Requirement: Old link after server restart
On a 401 `unauthorized` response to a request with a token, the page SHALL delete the token, not treat the action as performed, and show «Ссылка устарела: сервер был перезапущен. Нужна новая ссылка из вывода сервера» ("The link is outdated: the server was restarted. A new link from the server output is needed"); the form, what was entered in it, and the results remain, and the page is not reloaded. Until a new link arrives, actions make no requests and repeat this text. Trace: CUJ13, CUJ7; ADR 0011 item 2, 0065 item 6; ADR 0088 item 2.

#### Scenario: Open tab after a restart
- **GIVEN** the tab was opened before the server restart, and values are entered in a form
- **WHEN** after the restart the user presses «Найти» ("Find")
- **THEN** the results do not change, this text is shown; the request was not made; the form and the entered values are in place

#### Scenario: Repeated action without a link
- **GIVEN** «Ссылка устарела…» is shown, and there is no token
- **WHEN** the user presses «Найти» ("Find") or «Сохранить» ("Save") again
- **THEN** there are no requests to `/v1/`, the same text is shown

#### Scenario: New link
- **WHEN** the user pastes into the address bar of the same tab a link from the output of the new run ("Sign-in by link: token from the fragment")
- **THEN** the UI works with the new token without reloading the tab, the entered values and the results are in place, and the «Ссылка устарела…» text is removed; the earlier action is not repeated

#### Scenario: Not shown in advance
- **WHEN** the server is restarted and the user does nothing in the open tab
- **THEN** the page shows nothing and makes no requests by itself

### Requirement: Page does not poll the server
The page SHALL make requests to the API only in response to a user action (search, save, confirm, "next", retry) and retries after start refusals; there are no background status requests. Trace: CUJ16, X-5; ADR 0011 item 6, 0063 item 1; ADR 0088 item 3.

#### Scenario: Open tab with no actions
- **WHEN** the tab is open for 10 minutes and the user does nothing
- **THEN** there was not a single request to `/v1/`, and there are no warnings

#### Scenario: Storage state
- **GIVEN** the database is "read-only"
- **WHEN** the user opened the page and performed a search
- **THEN** the results are as usual, with no state warning

### Requirement: "Server starting": display and auto-retry
On 503 `server_starting` the page SHALL show «Сервер запускается, повторяем запрос…» ("The server is starting, retrying the request…"), repeat the same request after `Retry-After` seconds (5 if the header is absent or not an integer) for up to 60 s from the first refusal, then stop with a «Повторить» ("Retry") button (D-80). Trace: CUJ13, CUJ7; ADR 0063 item 8, 0074 item 2; ADR 0088 item 3.

#### Scenario: Server is ready soon
- **WHEN** a search request got 503 `server_starting` with `Retry-After: 5`, and after 5 s the server is ready
- **THEN** the request is repeated once, the results are shown, and the message is removed

#### Scenario: Startup drags on
- **WHEN** for 60 s the retries receive `server_starting`
- **THEN** the retries stop, «Сервер всё ещё запускается» ("The server is still starting") and a «Повторить» ("Retry") button are shown

#### Scenario: Write action
- **WHEN** a save received `server_starting`
- **THEN** there is no «Сохранение не выполнено» ("Save not performed") message until the retries end; the record was not written and a retry is safe

### Requirement: "Server stopping" and connection drop
On 503 `server_stopping` the page SHALL show «<Действие> не выполнено: сервер останавливается» ("<Action> not performed: the server is stopping") without auto-retry (D-81). A drop after a write was sent, or 504 `write_outcome_unknown`, gives «<Действие>: нет подтверждения, исход неизвестен. Проверьте поиском по номеру» ("<Action>: no confirmation, outcome unknown. Check with a search by number"), with no automatic actions. For other requests: no status received gives «<Действие> не выполнено: нет связи с сервером» ("<Action> not performed: no connection to the server") and «Повторить» ("Retry"); a received 200 whose body has no `complete` gives «<Действие> не выполнено: ответ получен не полностью» ("<Action> not performed: the response was received incompletely") and «Повторить». Re-reading after an action: the texts of `web-ui-flows`. Trace: CUJ2, CUJ13, CUJ14; OD-6; ADR 0063, 0074, 0088.

#### Scenario: Shutdown
- **WHEN** an action received 503 `server_stopping`
- **THEN** this text is shown, the request is not repeated, and the data is unchanged

#### Scenario: Server unavailable
- **WHEN** the connection to the server cannot be established during a search
- **THEN** «Поиск не выполнен: нет связи с сервером» ("Search not performed: no connection to the server") and a «Повторить» ("Retry") button are shown

#### Scenario: Stream cut off without a completeness marker
- **WHEN** a search or `by-number` request got 200, and the body ended without `complete` (drop, snapshot revocation, server shutdown)
- **THEN** «Поиск не выполнен: ответ получен не полностью» ("Search not performed: the response was received incompletely") is shown (for the history after "number known": «Сохранение не выполнено: …» ("Save not performed: …")) and «Повторить» ("Retry"); a partial result is not passed off as complete, and there is no "storage error" text

#### Scenario: Drop during the number check
- **WHEN** the connection is lost on a `by-number` request after a `409 number_known` response to an add without a date
- **THEN** «Сохранение не выполнено: нет связи с сервером» ("Save not performed: no connection to the server") and a «Повторить» ("Retry") button are shown; the record was not created, and "outcome unknown" is not shown

#### Scenario: Write outcome unknown
- **WHEN** the connection is dropped after a save was sent, or the response is 504 `write_outcome_unknown`
- **THEN** «Сохранение: нет подтверждения, исход неизвестен. Проверьте поиском по номеру» ("Save: no confirmation, outcome unknown. Check with a search by number") is shown, and no retry is performed automatically

#### Scenario: Delete outcome unknown
- **WHEN** the connection is dropped after a delete was sent
- **THEN** «Удаление: нет подтверждения, исход неизвестен. Проверьте поиском по номеру» ("Delete: no confirmation, outcome unknown. Check with a search by number") is shown

### Requirement: "Read-only" and "storage error" texts on action failure
On 503 `read_only` the page SHALL show «<Действие> не выполнено: база доступна только для чтения» ("<Action> not performed: the database is read-only"), on 500 `storage_error` «<Действие> не выполнено: ошибка хранилища, обратитесь к оператору» ("<Action> not performed: storage error, contact the operator"), where the action is «Сохранение» (Save), «Удаление» (Delete) or «Поиск» (Search) (for search the wording is «не выполнен»); no auto-retry and no details. Trace: CUJ16; ADR 0011 item 6, 0053, 0063; ADR 0088 item 5.

#### Scenario: Save in "read-only"
- **WHEN** a save received 503 `read_only`
- **THEN** «Сохранение не выполнено: база доступна только для чтения» is shown; the data is unchanged

#### Scenario: Delete on a storage error
- **WHEN** a delete received 500 `storage_error`
- **THEN** «Удаление не выполнено: ошибка хранилища, обратитесь к оператору» is shown

#### Scenario: Search on a storage error
- **WHEN** a search received 500 `storage_error`
- **THEN** «Поиск не выполнен: ошибка хранилища, обратитесь к оператору» is shown; a partial result is not passed off as complete

#### Scenario: Success without warnings
- **WHEN** actions succeed although the storage state is faulty
- **THEN** there are no state warnings at all

### Requirement: Value error: field and reason
For `invalid_input` the page SHALL show next to the field the message «<Поле>: <причина>» ("<Field>: <reason>") per the table: number, date, name, query × the reasons of the `records` dictionary; the text is built from `field`, `reason` and `hint`, not from `message`; the value in the field is not erased. «Дата: укажите дату» ("Date: specify a date") (`missing`) is also shown by the UI itself for an empty date in the CUJ2 step (`web-ui-flows`, "Date prompt for a known number (CUJ2)"). Trace: CUJ1, CUJ2, CUJ3, CUJ5; ADR 0020, 0062; ADR 0088 item 6.

#### Scenario: Number without a plus
- **WHEN** the server answered `invalid_input`, field `number`, reason `invalid_format`, `hint` `missing_plus`
- **THEN** next to the number field: «Номер: нужен «+» в начале. Введите международный формат, например +79991234567» ("Number: a leading «+» is required. Enter the international format, for example +79991234567"); the entered value is kept

#### Scenario: Invalid number
- **WHEN** the reason is `invalid_format` without a hint
- **THEN** «Номер: неверный формат. «+», код страны и цифры, всего не более 15 цифр» ("Number: invalid format. «+», country code and digits, at most 15 digits in total")

#### Scenario: Date and name reasons
- **WHEN** `date` with `invalid_format`, `no_such_day`, `out_of_range`, `missing`; `name` with `empty`, `control_character`, `too_long`; `query` with `empty`, `too_long`
- **THEN** the messages are respectively «Дата: нужен вид ГГГГ-ММ-ДД» ("Date: the form YYYY-MM-DD is required"), «Дата: такого дня нет» ("Date: no such day"), «Дата: не раньше 1876-03-07» ("Date: not earlier than 1876-03-07"), «Дата: укажите дату» ("Date: specify a date"); «Имя: введите имя» ("Name: enter a name"), «Имя: недопустимый управляющий знак» ("Name: invalid control character"), «Имя: не длиннее 200 знаков» ("Name: at most 200 characters"); «Запрос: введите слово» ("Query: enter a word"), «Запрос: слишком длинный» ("Query: too long")

#### Scenario: Unknown reason
- **WHEN** a reason or code arrived that is not in the table
- **THEN** «<Действие> не выполнено: сервер отклонил значение (<код>)» ("<Action> not performed: the server rejected the value (<code>)") is shown; there is no other response data

### Requirement: Other refusals
For a refusal not described above (`internal_error`, `request_timeout`, `snapshot_revoked`, `host_not_allowed`, an unknown code), the page SHALL show «<Действие> не выполнено: ошибка сервера (<code>)» ("<Action> not performed: server error (<code>)") with a «Повторить» ("Retry") button; client errors of class 4xx without their own text get «запрос отклонён (<code>)» ("request rejected (<code>)"). A connection closed without a response gives «нет связи с сервером» ("no connection to the server"). Trace: CUJ8–CUJ9, CUJ11; ADR 0062 item 4.

#### Scenario: Read timeout
- **WHEN** a search received 503 `request_timeout`
- **THEN** «Поиск не выполнен: ошибка сервера (request_timeout)», «Повторить» ("Retry")

#### Scenario: Unknown code
- **WHEN** the server answered with an unknown code and status 500
- **THEN** the server-error text with this code is shown; the page does not break

### Requirement: Date format in the UI
The UI SHALL show and accept a date in the form YYYY-MM-DD (as in the API); a date is a calendar day without time and time zone, independent of the browser's locale and time zone. This is the default (D-78). Trace: CUJ1, CUJ2; brief §5; ADR 0024, 0089 item 1.

#### Scenario: Display
- **WHEN** a record with the date 2024-05-01 is shown in the results in any browser time zone
- **THEN** «2024-05-01» is shown

#### Scenario: Input
- **WHEN** the user enters «01.05.2024»
- **THEN** the server answers `invalid_format`, «Дата: нужен вид ГГГГ-ММ-ДД» is shown; the UI does not try to guess the format

#### Scenario: Boundary
- **WHEN** a record with the date 1876-03-07 is shown
- **THEN** it is shown as «1876-03-07» (like other dates)

### Requirement: Supported browsers
The UI SHALL work in the last two stable versions of Chrome, Edge, Firefox and Safari at the time of release; in a browser without `fetch`, response-stream reading or modules the page shows «Браузер не поддерживается» ("Browser not supported") (without JavaScript: «Нужен включённый JavaScript» ("JavaScript must be enabled")). This is the default (D-79). Trace: CUJ13; ADR 0089 item 2.

#### Scenario: Supported browser
- **WHEN** the user opens the link in a supported browser
- **THEN** all the journeys CUJ1–CUJ6 and CUJ12 are available

#### Scenario: Browser without the capabilities
- **WHEN** the browser has no response-stream reading
- **THEN** the text «Браузер не поддерживается» is shown, and no API requests are made

#### Scenario: JavaScript disabled
- **WHEN** JavaScript is turned off
- **THEN** the text «Нужен включённый JavaScript» is shown
