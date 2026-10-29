# web-ui-shell — the web interface shell

## Purpose
The UI shell: how the server serves the page, how the user signs in by a link with a token, the page security policy and the common rules for showing refusals (stale token, server start and shutdown, "read-only" and "storage error", value errors, the date format). The user journeys CUJ1–CUJ6 and CUJ12 are built on these rules (capability `web-ui-flows`).

## ADDED Requirements

### Requirement: UI page is served without a token only on loopback
The server SHALL serve the UI shell (the page `/` and the resources under `/ui/`) with the methods GET and HEAD on the loopback listener in working mode, without a token and without database data; other methods get 405 `method_not_allowed`, an unknown path under `/ui/` gets 404 `route_not_found`; on the network listener the paths `/` and `/ui/…` get 404 `route_not_found`, like any path without a version (`http-api`, "API version in the path"). Trace: CUJ13; ADR 0011 item 1, 0067 item 5, 0085.

#### Scenario: Opening the page
- **WHEN** a browser requests `GET /` on the loopback listener with a valid Host, without an `Authorization` header
- **THEN** the response is 200 with an HTML page in Russian, containing no database records and no token

#### Scenario: Page resources
- **WHEN** the page requests its script and styles under `/ui/`
- **THEN** they are served without a token, 200

#### Scenario: Wrong method and unknown resource
- **WHEN** `POST /` or `GET /ui/unknown` is sent
- **THEN** respectively 405 `method_not_allowed` and 404 `route_not_found`

#### Scenario: Network listener
- **WHEN** `/` or a resource under `/ui/` is requested on the network listener, including from the loopback address
- **THEN** the response is 404 `route_not_found`, the page is not served (`http-api`, "API version in the path")

### Requirement: UI resources are embedded in the program
The server program SHALL contain all UI resources (the page, the script, the styles) within itself: the UI needs neither a resource directory on disk nor network access. This is a default value (D-76). Trace: CUJ13; item 1.2 of the original statement; ADR 0085 item 2.

#### Scenario: A machine with no network and no extra files
- **GIVEN** the machine has only the program, the database and the settings, and there is no network
- **WHEN** the user opens the link from the server output
- **THEN** the page loads and works

#### Scenario: Program update
- **WHEN** the server is replaced by a program of another version and restarted
- **THEN** the page and its script are served from one and the same program version, with no mixing of old and new resources

### Requirement: Interface language is Russian
All UI texts (labels, buttons, messages) SHALL be in Russian; the page declares the language `ru`. Trace: CUJ16, CUJ13; brief §2 (the CUJ16 and CUJ12 texts); ADR 0085 item 3.

#### Scenario: Page language
- **WHEN** the page is opened in a browser with any interface language
- **THEN** all texts are Russian, and `<html>` has `lang="ru"`

#### Scenario: A server message is not shown in place of a UI text
- **WHEN** a server response contains `message`
- **THEN** the UI shows its own Russian text by `code`, and does not need `message`

### Requirement: Strict page security policy
Every shell response SHALL carry `Content-Security-Policy` with `default-src 'none'`, `script-src 'self'`, `style-src 'self'`, `connect-src 'self'`, `base-uri 'none'`, `form-action 'none'`, `frame-ancestors 'none'`, as well as `Referrer-Policy: no-referrer` and `X-Content-Type-Options: nosniff`; the page contains no inline scripts or styles. Trace: CUJ13; ADR 0067 item 5 (P-2); ADR 0087.

#### Scenario: Shell headers
- **WHEN** `/`, the script and the styles are requested
- **THEN** every response has the listed headers

#### Scenario: No inline code
- **WHEN** the page HTML is checked
- **THEN** it has no `<script>` with code, no event-handler attributes (`onclick` and the like) and no `style` attribute

#### Scenario: Embedding in a foreign page
- **WHEN** another page tries to show the UI in a frame
- **THEN** the browser does not allow the embedding (`frame-ancestors 'none'`)

### Requirement: No external resources and no cookies
The page SHALL load only resources of its own origin and call only the API of its own origin; the server MUST NOT set a cookie in any response, the page MUST NOT read or write cookies or `localStorage`. Trace: CUJ13; P-1, P-5; ADR 0065 item 6, 0087.

#### Scenario: Page requests
- **WHEN** the page works from opening to closing
- **THEN** all its requests go to the server's origin; there are no fonts, scripts, images or analytics from other addresses

#### Scenario: Cookies
- **WHEN** the server's responses to page and API requests are checked
- **THEN** there is no `Set-Cookie` header, and the page does not use `Cookie`

### Requirement: Database data is shown only as text
The UI SHALL output names, numbers, dates and error texts as text, without parsing as HTML and without execution; a value from a server response MUST NOT end up in markup, a handler attribute, an address or `style`. Trace: CUJ4, CUJ5, CUJ13; ADR 0067 item 5 (P-2); ADR 0087.

#### Scenario: A name with markup
- **GIVEN** the database has the name `<img src=x onerror=alert(1)> <b>Ann</b>`
- **WHEN** it is shown in the results
- **THEN** the characters of this name are visible on the screen as they are, the script is not executed, there is no bold font

#### Scenario: A name with quotes and an ampersand
- **WHEN** the results contain the name `O'Brien & "Sons"`
- **THEN** it is shown verbatim

### Requirement: Sign-in by link: token from the fragment
The page SHALL take the UI token from the address fragment (64 hexadecimal characters) at load and at every change of the fragment in an already open tab (`hashchange`, the tab is not reloaded) and each time remove the fragment from the address bar; a new token replaces the old one, while the values entered in the forms and the results stay on the screen; the token MUST NOT remain in the address bar and MUST NOT go out in a page request. Trace: CUJ13; ADR 0065 item 1, 0067 item 5 (P-5); ADR 0086 items 1, 4.

#### Scenario: Opening the link from the server output
- **WHEN** the user opens `http://127.0.0.1:<port>/#<token>`
- **THEN** the UI is available at once, with no credentials entered, and the address in the browser bar became `http://127.0.0.1:<port>/`

#### Scenario: The fragment does not look like a token
- **WHEN** at load or on a fragment change it is empty or does not consist of 64 hexadecimal characters
- **THEN** it is not taken as a token, the fragment is removed from the address bar, the previous token (if any) does not change

#### Scenario: A new link in the same tab
- **GIVEN** the tab works with a token, its address is `http://127.0.0.1:<port>/`, values are entered in the form
- **WHEN** the user pastes `http://127.0.0.1:<port>/#<another token>` into the address bar of this tab (only the fragment changes, the page is not reloaded)
- **THEN** the new token is in effect, the address is again `http://127.0.0.1:<port>/`, the entered values and the results stayed in place

### Requirement: Token storage: where it is kept and what is not guaranteed
The page SHALL keep the token in the tab's `sessionStorage`, and if that is unavailable, in memory; the token MUST NOT end up in a cookie, `localStorage`, request addresses, texts or the console. "The token is not on disk" and "it disappears when the tab is closed" are not guaranteed (residual risks — ADR 0086); the mitigation is that the token is new at every start and is accepted only on loopback. A default value (D-77). Trace: CUJ13; P-1, P-2; ADR 0086 item 2.

#### Scenario: Page reload
- **WHEN** the user reloads the tab, the address has no fragment
- **THEN** the UI keeps working with the same token

#### Scenario: The token is not in a cookie or in localStorage
- **WHEN** the user has signed in by link and works in the UI
- **THEN** the token is not in a cookie or `localStorage`, nor in request addresses, nor in the text on the screen, nor in the console

#### Scenario: Residual risks accepted
- **WHEN** the browser saved `sessionStorage` to disk, returned it on "reopen closed tab" or session restore, duplicated the tab, or the link with the token stayed in the browser history and the terminal
- **THEN** the token may be available until the server restarts; the specification neither forbids this nor promises the opposite

#### Scenario: A restored tab
- **GIVEN** the browser returned a tab together with its `sessionStorage`, and the server has restarted since
- **WHEN** the user performs an action
- **THEN** the request gets 401, «Ссылка устарела…» (The link is stale…) is shown ("Old link after server restart"); if the server has not restarted, the UI works with the same token (the residual risk is accepted)

#### Scenario: A new tab without a link
- **WHEN** the user opens `http://127.0.0.1:<port>/` in a new tab
- **THEN** there is no token, «Нет ссылки для входа» (No sign-in link) is in effect

#### Scenario: sessionStorage is unavailable
- **WHEN** access to `sessionStorage` ends with an error
- **THEN** the token is kept in page memory; a reload requires a new link

### Requirement: Token is sent only in the Authorization header
The page SHALL send the token only in the header `Authorization: Bearer <token>` of requests to the API of its own origin (`/v1/…`); the token MUST NOT be in the request address, the body, a cookie, the text on the screen or console messages. Trace: CUJ13; ADR 0065 item 6, 0067 item 5 (P-1, P-5); `api-access`.

#### Scenario: A search request
- **WHEN** the user performs a search
- **THEN** the request goes to `/v1/…` with the header `Authorization: Bearer <token>`, and the request address does not contain the token

#### Scenario: The token in the interface
- **WHEN** the user looks at the screen and the address bar after sign-in
- **THEN** the token is nowhere

### Requirement: No token: a link from server output is needed
If at page load there is no token (neither in the fragment nor in `sessionStorage` or memory — the tab was opened without a link), the page SHALL show «Нет ссылки для входа. Откройте ссылку, которую сервер напечатал при запуске» (No sign-in link. Open the link the server printed at startup) and make no requests to the API; it shows no forms and no results. The requirement does not apply to the state after 401: there the form and the results stay ("Old link after server restart"). Trace: CUJ13; ADR 0067 item 5; ADR 0088 item 1.

#### Scenario: An address without a token
- **WHEN** the user opens `http://127.0.0.1:<port>/` in a new tab
- **THEN** this text is shown; there are no forms and no results; there were no requests to `/v1/`

### Requirement: Old link after server restart
On a 401 `unauthorized` response to a request with a token, the page SHALL delete the token, not consider the action executed, and show «Ссылка устарела: сервер был перезапущен. Нужна новая ссылка из вывода сервера» (The link is stale: the server was restarted. A new link from the server output is needed); the form, what was entered in it and the results stay, the page is not reloaded. Until a new link arrives, actions make no requests and repeat this text. Trace: CUJ13, CUJ7; ADR 0011 item 2, 0065 item 6; ADR 0088 item 2.

#### Scenario: An open tab after a restart
- **GIVEN** the tab was opened before the server restarted, values are entered in the form
- **WHEN** after the restart the user presses «Найти» (Search)
- **THEN** the results do not change, this text is shown; the request was not executed; the form and the entered values are in place

#### Scenario: A repeated action without a link
- **GIVEN** «Ссылка устарела…» is shown, there is no token
- **WHEN** the user presses «Найти» or «Сохранить» (Save) again
- **THEN** there are no requests to `/v1/`, the same text is shown

#### Scenario: A new link
- **WHEN** the user pastes into the address bar of the same tab the link from the output of the new start ("Sign-in by link: token from the fragment")
- **THEN** the UI works with the new token without reloading the tab, the entered values and the results are in place, the text «Ссылка устарела…» is removed; the earlier action is not repeated

#### Scenario: Not shown in advance
- **WHEN** the server is restarted, and the user does nothing in the open tab
- **THEN** the page shows nothing and makes no requests itself

### Requirement: Page does not poll the server
The page SHALL make API requests only in response to a user action (search, save, confirm, "next", retry) and retries on startup refusals; there are no background state requests. Trace: CUJ16, X-5; ADR 0011 item 6, 0063 item 1; ADR 0088 item 3.

#### Scenario: An open tab with no actions
- **WHEN** the tab is open for 10 minutes and the user does nothing
- **THEN** there was not a single request to `/v1/`, there are no warnings

#### Scenario: Storage state
- **GIVEN** the database is "read-only"
- **WHEN** the user opened the page and performed a search
- **THEN** the results are as usual, there is no state warning

### Requirement: "Server starting": display and auto-retry
On 503 `server_starting` the page SHALL show «Сервер запускается, повторяем запрос…» (Server is starting, retrying the request…), repeat the same request after `Retry-After` seconds (5 if the header is absent or is not an integer) up to 60 s from the first refusal, then stop with a «Повторить» (Retry) button (D-80). Trace: CUJ13, CUJ7; ADR 0063 item 8, 0074 item 2; ADR 0088 item 3.

#### Scenario: The server is ready soon
- **WHEN** a search request got 503 `server_starting` with `Retry-After: 5`, and after 5 s the server is ready
- **THEN** the request is repeated once, the results are shown, the message is removed

#### Scenario: The start dragged on
- **WHEN** for 60 s the retries get `server_starting`
- **THEN** the retries stop, «Сервер всё ещё запускается» (The server is still starting) and a «Повторить» button are shown

#### Scenario: A write action
- **WHEN** a save got `server_starting`
- **THEN** there is no «Сохранение не выполнено» (Saving failed) message until the retries end; the write was not executed and a retry is safe

### Requirement: "Server stopping" and connection drop
On 503 `server_stopping` the page SHALL show «<Действие> не выполнено: сервер останавливается» (<Action> failed: the server is stopping) with no auto-retry (D-81). On a connection drop after a write was sent or on 504 `write_outcome_unknown` — «<Действие>: нет подтверждения, исход неизвестен. Проверьте поиском по номеру» (<Action>: no confirmation, outcome unknown. Check with a number search) («Сохранение» (Saving) or «Удаление» (Deleting)); on a failure of other requests (search, number check, re-read) — «<Действие> не выполнено: нет связи с сервером» (<Action> failed: no connection to the server) and «Повторить». Trace: CUJ13, CUJ14; ADR 0063, 0074, 0088.

#### Scenario: Shutdown
- **WHEN** an action got 503 `server_stopping`
- **THEN** this text is shown, the request is not repeated, data is not changed

#### Scenario: Server unavailable
- **WHEN** the connection to the server cannot be established during a search
- **THEN** «Поиск не выполнен: нет связи с сервером» (Search failed: no connection to the server) and a «Повторить» button are shown

#### Scenario: A drop during the number check
- **WHEN** the connection is lost on the `by-number` request before an add without a date
- **THEN** «Сохранение не выполнено: нет связи с сервером» and a «Повторить» button are shown; the add was not sent, "outcome unknown" is not shown

#### Scenario: Write outcome unknown
- **WHEN** the connection is dropped after a save was sent, or the response is 504 `write_outcome_unknown`
- **THEN** «Сохранение: нет подтверждения, исход неизвестен. Проверьте поиском по номеру» is shown, no automatic retry is performed

#### Scenario: Delete outcome unknown
- **WHEN** the connection is dropped after a delete was sent
- **THEN** «Удаление: нет подтверждения, исход неизвестен. Проверьте поиском по номеру» is shown

### Requirement: "Read-only" and "storage error" texts on action failure
On 503 `read_only` the page SHALL show «<Действие> не выполнено: база доступна только для чтения» (<Action> failed: the database is read-only), on 500 `storage_error` — «<Действие> не выполнено: ошибка хранилища, обратитесь к оператору», where the action is «Сохранение», «Удаление» or «Поиск» (Search) (for search — «не выполнен»); with no auto-retry and no details. Trace: CUJ16; ADR 0011 item 6, 0053, 0063; ADR 0088 item 5.

#### Scenario: Save while "read-only"
- **WHEN** a save got 503 `read_only`
- **THEN** «Сохранение не выполнено: база доступна только для чтения» is shown; data is not changed

#### Scenario: Delete on a storage error
- **WHEN** a delete got 500 `storage_error`
- **THEN** «Удаление не выполнено: ошибка хранилища, обратитесь к оператору» is shown

#### Scenario: Search on a storage error
- **WHEN** a search got 500 `storage_error` or the stream was cut with no completeness marker
- **THEN** «Поиск не выполнен: ошибка хранилища, обратитесь к оператору» is shown; a partial result is not passed off as complete

#### Scenario: Success without warnings
- **WHEN** actions succeed although the storage state is faulty
- **THEN** there are no state warnings at all

### Requirement: Value error: field and reason
For `invalid_input` the page SHALL show at the field the message «<Поле>: <причина>» (<Field>: <reason>) per the table: number, date, name, query × the reasons of the `records` dictionary; the text is built from `field`, `reason` and `hint`, not from `message`; the value in the field is not erased. «Дата: укажите дату» (`missing`) is also shown by the UI itself on an empty date in the CUJ2 step (`web-ui-flows`, "Date prompt for a known number (CUJ2)"). Trace: CUJ1, CUJ2, CUJ3, CUJ5; ADR 0020, 0062; ADR 0088 item 6.

#### Scenario: A number without a plus
- **WHEN** the server responded `invalid_input`, field `number`, reason `invalid_format`, `hint` `missing_plus`
- **THEN** at the number field: «Номер: нужен «+» в начале. Введите международный формат, например +79991234567» (Number: a "+" is needed at the start. Enter the international format, for example +79991234567); what was entered is kept

#### Scenario: An invalid number
- **WHEN** the reason is `invalid_format` with no hint
- **THEN** «Номер: неверный формат. «+», код страны и цифры, всего не более 15 цифр» (Number: invalid format. "+", the country code and digits, no more than 15 digits in total)

#### Scenario: Date and name reasons
- **WHEN** `date` with `invalid_format`, `no_such_day`, `out_of_range`, `missing`; `name` with `empty`, `control_character`, `too_long`; `query` with `empty`, `too_long`
- **THEN** the messages are respectively «Дата: нужен вид ГГГГ-ММ-ДД», «Дата: такого дня нет», «Дата: не раньше 1876-03-07», «Дата: укажите дату»; «Имя: введите имя», «Имя: недопустимый управляющий знак», «Имя: не длиннее 200 знаков»; «Запрос: введите слово», «Запрос: слишком длинный» (Date: the form YYYY-MM-DD is needed; Date: there is no such day; Date: not earlier than 1876-03-07; Date: specify a date; Name: enter a name; Name: invalid control character; Name: no longer than 200 characters; Query: enter a word; Query: too long)

#### Scenario: An unknown reason
- **WHEN** a reason or code arrived that is not in the table
- **THEN** «<Действие> не выполнено: сервер отклонил значение (<код>)» (<Action> failed: the server rejected the value (<code>)) is shown; there is no other response data

### Requirement: Other refusals
For a refusal not described above (`internal_error`, `request_timeout`, `too_many_connections`, `snapshot_revoked`, `host_not_allowed`, an unknown code) the page SHALL show «<Действие> не выполнено: ошибка сервера (<code>)» (<Action> failed: server error (<code>)) with a «Повторить» button; client errors of class 4xx without a text of their own — «запрос отклонён (<code>)» (request rejected (<code>)). Trace: CUJ8–CUJ9, CUJ11; ADR 0062 item 4.

#### Scenario: Read timeout
- **WHEN** a search got 503 `request_timeout`
- **THEN** «Поиск не выполнен: ошибка сервера (request_timeout)», «Повторить»

#### Scenario: Unknown code
- **WHEN** the server responded with an unknown code with status 500
- **THEN** the server error text with this code is shown; the page does not break

### Requirement: Date format in the UI
The UI SHALL show and accept a date in the form YYYY-MM-DD (as in the API); a date is a calendar day without time and zone, independent of the browser's locale and time zone. A default value (D-78). Trace: CUJ1, CUJ2; brief §5; ADR 0024, 0089 item 1.

#### Scenario: Display
- **WHEN** a record with the date 2024-05-01 is shown in the results in any browser time zone
- **THEN** «2024-05-01» is shown

#### Scenario: Input
- **WHEN** the user enters «01.05.2024»
- **THEN** the server responds `invalid_format`, «Дата: нужен вид ГГГГ-ММ-ДД» is shown; the UI does not try to guess the format

#### Scenario: Boundary
- **WHEN** a record with the date 1876-03-07 is shown
- **THEN** it is shown as «1876-03-07» (like other dates)

### Requirement: Supported browsers
The UI SHALL work in the two latest stable versions of Chrome, Edge, Firefox and Safari at the time of release; in a browser without `fetch`, response stream reading or modules the page shows «Браузер не поддерживается» (Browser not supported) (without JavaScript — «Нужен включённый JavaScript»). A default value (D-79). Trace: CUJ13; ADR 0089 item 2.

#### Scenario: A supported browser
- **WHEN** the user opens the link in a supported browser
- **THEN** all the paths CUJ1–CUJ6 and CUJ12 are available

#### Scenario: A browser without the capabilities
- **WHEN** the browser has no response stream reading
- **THEN** the text «Браузер не поддерживается» is shown, no API requests are made

#### Scenario: JavaScript is off
- **WHEN** JavaScript is disabled
- **THEN** the text «Нужен включённый JavaScript» is shown
