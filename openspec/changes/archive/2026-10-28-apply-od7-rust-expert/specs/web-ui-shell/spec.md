## MODIFIED Requirements

### Requirement: Other refusals
For a refusal not described above (`internal_error`, `request_timeout`, `snapshot_revoked`, `host_not_allowed`, an unknown code), the page SHALL show «<Действие> не выполнено: ошибка сервера (<code>)» (<Action> not performed: server error (<code>)) with a «Повторить» (Retry) button; client 4xx errors without their own text show «запрос отклонён (<code>)» (request rejected (<code>)). A connection closed without a response shows «нет связи с сервером» (no connection to the server). Trace: CUJ8–CUJ9, CUJ11; ADR 0062 item 4.

#### Scenario: A read timeout
- **WHEN** a search received 503 `request_timeout`
- **THEN** «Поиск не выполнен: ошибка сервера (request_timeout)» (Search not performed: server error (request_timeout)), «Повторить» (Retry)

#### Scenario: An unknown code
- **WHEN** the server responded with an unknown code and status 500
- **THEN** the server-error text with that code is shown; the page does not break
