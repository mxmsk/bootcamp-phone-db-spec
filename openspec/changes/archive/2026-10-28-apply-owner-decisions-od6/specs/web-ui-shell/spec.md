## MODIFIED Requirements

### Requirement: "Server stopping" and connection drop
On 503 `server_stopping`, the page SHALL show «<Действие> не выполнено: сервер останавливается» (<Action> not performed: server stopping) without auto-retry (D-81). On a connection drop after a write was sent, or on 504 `write_outcome_unknown` — «<Действие>: нет подтверждения, исход неизвестен. Проверьте поиском по номеру» (<Action>: no confirmation, outcome unknown. Check with a number search) («Сохранение» (Save) or «Удаление» (Delete)); on a failure of other requests (search, the number history after "number known", re-reading) — «<Действие> не выполнено: нет связи с сервером» (<Action> not performed: no connection to the server) and «Повторить» (Retry). Trace: CUJ2, CUJ13, CUJ14; OD-6; ADR 0063, 0074, 0088.

#### Scenario: Shutdown
- **WHEN** an action received 503 `server_stopping`
- **THEN** this text is shown, the request is not repeated, and data is not changed

#### Scenario: The server is unavailable
- **WHEN** a connection to the server cannot be established during a search
- **THEN** «Поиск не выполнен: нет связи с сервером» (Search not performed: no connection to the server) and a «Повторить» (Retry) button are shown

#### Scenario: A drop during the number check
- **WHEN** the connection is lost on a `by-number` request after a `409 number_known` response to an add without a date
- **THEN** «Сохранение не выполнено: нет связи с сервером» and a «Повторить» button are shown; no record is created, and "outcome unknown" is not shown

#### Scenario: A write outcome is unknown
- **WHEN** the connection is dropped after a save was sent, or the response is 504 `write_outcome_unknown`
- **THEN** «Сохранение: нет подтверждения, исход неизвестен. Проверьте поиском по номеру» is shown, and no automatic retry is performed

#### Scenario: A delete outcome is unknown
- **WHEN** the connection is dropped after a delete was sent
- **THEN** «Удаление: нет подтверждения, исход неизвестен. Проверьте поиском по номеру» is shown
