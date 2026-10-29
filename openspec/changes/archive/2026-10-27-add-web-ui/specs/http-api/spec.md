## MODIFIED Requirements

### Requirement: API version in the path
The system SHALL serve API v1 only under the prefix `/v1/`; a request with a path without a version or with another version SHALL get 404 with the code `route_not_found`, with no redirect and no default version. The exception is the UI shell (`web-ui-shell`): on the loopback listener the paths `/` and `/ui/…` SHALL be served as the UI shell (methods GET and HEAD, without a token and without database data), and on the network listener the same paths SHALL get 404 `route_not_found`, like any path without a version. Trace: CUJ7, CUJ11, CUJ13; ADR 0013, 0060 (Q-51), 0085.

#### Scenario: Path without a version
- **WHEN** a client requests `GET /records/by-number?number=%2B79991234567`
- **THEN** the response is 404, code `route_not_found`; there is no redirect

#### Scenario: Another version
- **WHEN** a client requests a path under `/v2/`
- **THEN** the response is 404, code `route_not_found`

#### Scenario: UI shell on the loopback listener
- **WHEN** `GET /` or a `GET` of a resource under `/ui/` without a token arrives at the loopback listener in working mode
- **THEN** the UI shell (`web-ui-shell`) is served, there is no database data in the response; a path outside `/`, `/ui/` and `/v1/` (for example, `/favicon.ico`) still gets 404 `route_not_found`

#### Scenario: UI shell on the network listener
- **WHEN** `GET /` or `GET /ui/<resource>` arrives at the network listener
- **THEN** the response is 404, code `route_not_found`; the shell is not served

#### Scenario: Shell during startup
- **WHEN** `GET /` arrives at the loopback listener during startup
- **THEN** the response is 503 `server_starting` (step 3 of the "Request check order"), the page is not served
