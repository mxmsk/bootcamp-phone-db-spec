## MODIFIED Requirements

### Requirement: Connection limits
The server SHALL limit connections: 256 per loopback listener and 256 per network listener (the pools are separate) and 32 from one network address. The values were accepted by the owner (OD-6, D-52, D-53); the rule for evicting an address's anonymous connections overrides the cap of 32 ("Anonymous connections from one network address"). Trace: CUJ7, CUJ8; brief §4; ADR 0071 (Q-52).

#### Scenario: A network flood does not take slots from the UI
- **GIVEN** the network pool is entirely full
- **WHEN** the UI or a local program opens a connection on loopback
- **THEN** the connection is accepted

#### Scenario: One network address
- **GIVEN** an address has 32 connections, of which fewer than 16 are class U
- **WHEN** the same address opens a 33rd connection
- **THEN** it gets the refusal "too many connections", other addresses' connections are unaffected

### Requirement: Request size cap
The server SHALL reject a request with a request line and headers larger than 16 KiB ("headers too large"; if the refusal is produced by HTTP parsing — status 431 only, without a body with `code`; there is no separate cap on the number of headers) and a request with a body larger than 32 KiB ("request too large", by `Content-Length` before reading the body, otherwise by the count of bytes read); a valid name within the body cap SHALL be accepted. Values: the 32 KiB body cap is an owner decision (OD-6, D-43; as in brief §4), the header cap is D-43; a body below 32 KiB cannot be configured; the codes are in `http-api`. Trace: CUJ1, CUJ5, CUJ7; OD-6; ADR 0064, 0071; `records`.

#### Scenario: A name with 20,000 whitespace characters at the edges
- **WHEN** a name of 200 letters with 20,000 U+3000 whitespace characters on each edge, written as `　`, is sent in a body larger than 32 KiB
- **THEN** refusal "request too large" before the body is read and before parsing; the name is not parsed (OD-6, D-43)

#### Scenario: A name with edges within the body cap
- **WHEN** a name of 200 letters with 1,000 U+3000 whitespace characters on each edge, written as `　` (≈ 12,000 bytes of edges), is sent in a body
- **THEN** the body is within the cap, the request is accepted, the server does not refuse on size (`records`, "Input size before normalization and request body cap")

#### Scenario: Body larger than the cap
- **WHEN** a request with a valid token declares `Content-Length` of 300 KiB
- **THEN** the refusal "request too large" is issued before the body is read and delivered to the client (ADR 0072 item 5)

#### Scenario: Body without a declared length
- **WHEN** a chunked body exceeded 32 KiB
- **THEN** reading stops, refusal "request too large"

#### Scenario: Headers larger than the cap
- **WHEN** a request line with headers takes 16,385 bytes
- **THEN** refusal "headers too large", the request is not executed

### Requirement: Anonymous connections from one network address
On the network listener, there SHALL be at most 16 class U connections from one address: a new one beyond that evicts the oldest class U connection of the same address, not someone else's; the protection period does not apply to such eviction. This rule overrides the cap of 32 connections per address: refusal by the cap of 32 happens only when the address has fewer than 16 class U connections. A default value (D-53). Trace: CUJ8; ADR 0072 item 2; P-7f.

#### Scenario: One network address does not evict others'
- **GIVEN** one network address holds 16 class U connections, the network listener pool is full
- **WHEN** this address opens one more anonymous connection
- **THEN** the oldest anonymous connection of the same address is closed, other addresses' connections are unaffected

#### Scenario: An address has 32 connections, 16 of them anonymous
- **GIVEN** an address has 32 connections, of which 16 are class U
- **WHEN** this address opens a new connection
- **THEN** the oldest class U connection of this address is closed, the new one is accepted (the address's connection count does not grow); there is no refusal by the cap of 32

### Requirement: Message boundary substitution is rejected
A request with both `Transfer-Encoding` and `Content-Length`, as well as a request that does not parse as HTTP/1.1, SHALL get the refusal "invalid request" (status 400; a body with `code` is not guaranteed if the refusal is produced by HTTP parsing; even during startup, `http-api`, "Request check order") and SHALL NOT be executed; the connection is closed after the response. Trace: CUJ7, CUJ8; pitfall P-8, RFC 9112 §6.3; ADR 0072 item 8.

#### Scenario: Two ways to specify the length
- **WHEN** a request arrives with `Transfer-Encoding: chunked` and `Content-Length`
- **THEN** the operation is not executed, the client gets "invalid request", the connection is closed

#### Scenario: A second request inside the body
- **WHEN** the body contains bytes that look like a second request
- **THEN** it is not executed as a separate request

### Requirement: Startup order
The server SHALL: make sure the database directory exists (if not — "no database", code 5, nothing is created); take `LOCK`; read the settings and the certificate; check that the database is present; take the settings directory lock (`control.lock`), parse the token file and open the operator control channel; open the listeners; perform start checks, recovery, index rebuild and the new generation; create the UI token and write the local-program token file; switch to working mode. A refusal before listeners are opened has no listeners; after — the listeners are closed; on a refusal after the control channel is opened, it is closed and its file deleted; the operator gets the reason and a nonzero code. A settings directory that is not writable does not allow creating `control.lock` and opening the channel (an existing `control.lock` is taken for reading): the decision for such a start is in `operator-lifecycle`, "Settings directory is not writable". Trace: CUJ13, CUJ14, X-2; ADR 0074 item 1, ADR 0015, ADR 0054; ADR 0077 item 1; ADR 0080 item 2; ADR 0081 item 1.

#### Scenario: Directory without a database
- **WHEN** the server starts on a directory without `control`
- **THEN** refusal "no database", no listeners, the control channel is not open, `control.lock` is not taken, the UI address is not printed

#### Scenario: Certificate missing
- **WHEN** the network listener is enabled and there is no certificate
- **THEN** the server does not start, the operator gets the reason

#### Scenario: Damaged database
- **WHEN** the start check found an inconsistency
- **THEN** listeners are closed, the operator gets the reason, file, offset or LSN, the process exits with a nonzero code, data is not modified

#### Scenario: Control channel before listeners
- **WHEN** an index rebuild is in progress at startup
- **THEN** the operator control channel is open from the moment the server took `control.lock`, and before the listeners open; HTTP listeners answer "server starting"

#### Scenario: Settings directory is busy
- **WHEN** `control.lock` is held by another server
- **THEN** refusal «каталог настроек занят другим процессом» ("settings directory is held by another process"), no listeners and no control channel, the earlier server is unaffected

#### Scenario: Refusal after the channel is opened
- **WHEN** the start check refused after the control channel was opened
- **THEN** listeners and the channel are closed, the channel file is deleted, the operator gets the reason and a nonzero code

### Requirement: Clean shutdown: HTTP side
On SIGTERM, SIGINT or an operator command, the server SHALL: close the listeners; on open connections answer a new request with "server stopping" and close, without executing it; finish the writes taken by the writer with an acknowledgement or a rejection, and reject those not taken (in the writer queue, with an unread body) with "server stopping" without executing them; give running reads 5 s, then cancel them, breaking the streams without a completion marker; then perform the storage steps (ADR 0054 item 8) and exit with code 0. Trace: CUJ14, CUJ15; ADR 0028 item 10, ADR 0074 item 7.

#### Scenario: Write in flight
- **WHEN** the server gets SIGTERM while a write is executing
- **THEN** the write is acknowledged or rejected, the client got a response, then the clean-shutdown marker and code 0

#### Scenario: Write not taken by the writer
- **WHEN** the signal arrived while the write body was still being read or the operation was waiting in the writer queue
- **THEN** the client gets 503 `server_stopping`, the operation is neither executed nor acknowledged

#### Scenario: New request on an open connection
- **WHEN** after the signal a request arrives on an open connection
- **THEN** "server stopping", the connection is closed, the operation is not executed

#### Scenario: Long stream
- **GIVEN** a number history is being streamed to a slow client
- **WHEN** SIGTERM is received
- **THEN** after 5 s the stream is broken without a completion marker, the snapshot is released, the shutdown is clean

#### Scenario: New connections
- **WHEN** after the signal a client opens a connection
- **THEN** the connection is rejected

## ADDED Requirements

### Requirement: Refusals that close the connection
The server SHALL close the connection after a response with `Connection: close` (the write side is closed in steps, input is read to the end per "Refusal before body read reaches the client") on these refusals: 401 `unauthorized` (always), 400 on a message boundary error, 413, 431, 503 `server_stopping`, "too many connections"; other refusals (403, 404, 409, 429, storage 5xx) do not close the connection if the body has been read. Trace: CUJ7, CUJ8; ADR 0072 items 1, 3, 5.

#### Scenario: Invalid token
- **WHEN** a request with an invalid token got 401
- **THEN** the response contains `Connection: close` and the connection is closed after the response; the response is the same for any unrecognized token

#### Scenario: Permissions refusal
- **WHEN** a "read" token got 403 on an add whose body was read
- **THEN** the connection stays open for the next request
