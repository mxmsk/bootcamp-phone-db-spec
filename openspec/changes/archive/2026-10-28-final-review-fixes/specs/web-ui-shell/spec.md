## MODIFIED Requirements

### Requirement: "Server stopping" and connection drop
On 503 `server_stopping` the page SHALL show «<Действие> не выполнено: сервер останавливается» ("<Action> not performed: the server is stopping") with no auto-retry (D-81). A connection drop after a write was sent, or 504 `write_outcome_unknown` — «<Действие>: нет подтверждения, исход неизвестен. Проверьте поиском по номеру» ("<Action>: no confirmation, outcome unknown. Check by searching by number"), with no automatic actions. For other requests: no status received — «<Действие> не выполнено: нет связи с сервером» ("... no connection to the server") and «Повторить» ("Retry"); 200 received but the body has no `complete` — «<Действие> не выполнено: ответ получен не полностью» ("... the response was received incompletely") and «Повторить». Re-reading after an action — the texts of `web-ui-flows`. Trace: CUJ2, CUJ13, CUJ14; OD-6; ADR 0063, 0074, 0088.

#### Scenario: Shutdown
- **WHEN** an action received 503 `server_stopping`
- **THEN** this text is shown, the request is not retried, the data is unchanged

#### Scenario: Server unavailable
- **WHEN** the connection to the server cannot be established during a search
- **THEN** «Поиск не выполнен: нет связи с сервером» and a «Повторить» button are shown

#### Scenario: Stream cut off without a completeness marker
- **WHEN** a search request or `by-number` received 200 but the body ended without `complete` (connection drop, snapshot revocation, server shutdown)
- **THEN** «Поиск не выполнен: ответ получен не полностью» is shown (for the history after "number known" — «Сохранение не выполнено: …») and «Повторить»; partial results are not passed off as complete, and there is no «ошибка хранилища» text

#### Scenario: Connection drop while checking a number
- **WHEN** the connection is lost on the `by-number` request after a `409 number_known` response to an add without a date
- **THEN** «Сохранение не выполнено: нет связи с сервером» and a «Повторить» button are shown; the record is not created, and "outcome unknown" is not shown

#### Scenario: Write outcome unknown
- **WHEN** the connection is dropped after a save was sent, or the response is 504 `write_outcome_unknown`
- **THEN** «Сохранение: нет подтверждения, исход неизвестен. Проверьте поиском по номеру» is shown, and no automatic retry is made

#### Scenario: Delete outcome unknown
- **WHEN** the connection is dropped after a delete was sent
- **THEN** «Удаление: нет подтверждения, исход неизвестен. Проверьте поиском по номеру» is shown

### Requirement: "Read-only" and "storage error" texts on action failure
On 503 `read_only` the page SHALL show «<Действие> не выполнено: база доступна только для чтения» ("<Action> not performed: the database is read-only"), on 500 `storage_error` — «<Действие> не выполнено: ошибка хранилища, обратитесь к оператору» ("... storage error, contact the operator"), where the action is «Сохранение» (Save), «Удаление» (Delete) or «Поиск» (Search; for search — «не выполнен»); with no auto-retry and no details. Trace: CUJ16; ADR 0011 item 6, 0053, 0063; ADR 0088 item 5.

#### Scenario: Save in "read-only"
- **WHEN** a save received 503 `read_only`
- **THEN** «Сохранение не выполнено: база доступна только для чтения» is shown; the data is unchanged

#### Scenario: Delete on storage error
- **WHEN** a delete received 500 `storage_error`
- **THEN** «Удаление не выполнено: ошибка хранилища, обратитесь к оператору» is shown

#### Scenario: Search on storage error
- **WHEN** a search received 500 `storage_error`
- **THEN** «Поиск не выполнен: ошибка хранилища, обратитесь к оператору» is shown; partial results are not passed off as complete

#### Scenario: Success without warnings
- **WHEN** actions succeed although the storage state is faulty
- **THEN** no warnings about the state are shown
