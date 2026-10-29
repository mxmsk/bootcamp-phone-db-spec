## MODIFIED Requirements

### Requirement: A stream of invalid tokens does not displace legitimate clients
A stream of invalid tokens (every 401 refusal closes the connection, `server-runtime`, "Refusals that close the connection") SHALL be limited only by the connection limits: a legitimate client gets no refusals caused by the stream as long as the source opens anonymous connections more slowly than the slot protection threshold; with a faster stream new connections, including legitimate ones, MAY be closed without a response. Trace: CUJ7, CUJ8; ADR 0072 items 2, 6 (D-53; OD-7).

#### Scenario: Stream below the threshold
- **WHEN** a source opens anonymous connections with invalid tokens more slowly than the slot protection threshold
- **THEN** the requests of the other clients are executed without refusals caused by this stream

#### Scenario: Stream faster than the threshold
- **WHEN** a source opens anonymous connections faster than the slot protection threshold
- **THEN** the server keeps responding on the accepted connections; a new connection of any client may be closed without a response, and the client retries later
