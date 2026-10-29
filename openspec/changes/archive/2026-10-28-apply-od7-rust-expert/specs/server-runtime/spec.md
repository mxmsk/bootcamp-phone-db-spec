## RENAMED Requirements

- FROM: `### Requirement: Message boundary substitution is rejected`
- TO: `### Requirement: Message boundary substitution is ruled out`

## MODIFIED Requirements

### Requirement: Connection limits
The server SHALL limit connections: 256 per loopback listener and 256 per network listener (the pools are separate) and 32 per single network address. If there is nobody to evict ("Slot protection from anonymous clients") or the address cap is exceeded, the new connection is closed without a response, including under TLS. The values come from OD-6 (D-52, D-53). Trace: CUJ7, CUJ8; brief §4; ADR 0071 (Q-52); ADR 0072 items 2, 6 (OD-7).

#### Scenario: A network flood does not take slots from the UI
- **GIVEN** the network pool is fully occupied
- **WHEN** the UI or a local program opens a connection on loopback
- **THEN** the connection is accepted

#### Scenario: A single network address
- **GIVEN** the address has 32 connections, of which fewer than 16 are class U
- **WHEN** the same address opens a 33rd connection
- **THEN** it is closed without a response, and connections from other addresses are not affected

### Requirement: Slot protection from anonymous clients
When the listener pool is full, the server SHALL accept a new connection by closing the oldest connection that has made no request with a valid token (class U) if it is older than the protection period of 1 s, otherwise the connection idle the longest after a valid token (class I); connections with processing in progress (class A) and class U connections younger than 1 s are not evicted; if there is nobody to evict, the new connection is closed without a response. Trace: CUJ7, CUJ8; ADR 0072 items 1–2 (D-53; OD-7); P-6, P-7f.

#### Scenario: Anonymous connections have filled the pool
- **GIVEN** the loopback pool is occupied by 256 connections without a token, older than 1 s
- **WHEN** a local program opens a connection and presents a valid token
- **THEN** the oldest anonymous connection is closed and the program's request is served

#### Scenario: The pool is busy with processing
- **GIVEN** all 256 connections are processing requests with valid tokens
- **WHEN** a new connection arrives
- **THEN** the new connection is closed without a response and requests in progress are not affected

#### Scenario: An idle valid client is evicted
- **GIVEN** the pool has no anonymous connections older than 1 s, but has idle connections of clients
- **WHEN** a new connection arrives
- **THEN** the connection idle longer than the others is closed, and that client then opens a new one

### Requirement: Slot protection boundary
A legitimate connection SHALL be accepted when the stream of new anonymous connections is below the threshold of 256 per second per pool (pool of 256, protection period of 1 s), provided it sends a request with a valid token within the protection period; at a stream above the threshold, new connections, including legitimate ones, MAY be closed without a response. These are default values (D-53). Trace: CUJ7, CUJ8; ADR 0072 items 2, 6 (OD-7).

#### Scenario: A stream below the threshold
- **GIVEN** a source opens anonymous connections at a rate below 256 per second, and the pool is filled with them
- **WHEN** a legitimate client opens a connection and sends a request with a valid token within 1 s
- **THEN** the request is served and the connection is not closed

#### Scenario: A stream faster than the protection period
- **GIVEN** the loopback pool is occupied by 256 anonymous connections, all younger than 1 s
- **WHEN** a new connection arrives, including from a legitimate client
- **THEN** it is closed without a response and the anonymous ones are not evicted; the client retries later

### Requirement: Refusal before body read reaches the client
Refusals based on headers ("unauthorized", "too many requests", "no permissions", "request too large", "headers too large", "invalid request") SHALL be delivered to the client: before closing with unread input, the server closes the write side and drains the input for at most 1 s and 64 KiB; a connection with an unread body is not reused. Trace: CUJ7, CUJ8; ADR 0072 item 5, pitfall P-9, RFC 9112 §9.6.

#### Scenario: A request without a token with a large body
- **WHEN** a client without a token sends headers and a 200 KiB body
- **THEN** the client receives "unauthorized" rather than a connection reset without a response

#### Scenario: A refusal under the request limit
- **WHEN** a network token has exceeded the limit and sent a body
- **THEN** the "too many requests" refusal is delivered, and neither storage nor the reader pool is used

### Requirement: Message boundary substitution is ruled out
A request with both `Transfer-Encoding` and `Content-Length` SHALL be handled per RFC 9112 §6.1, as hyper does: `Content-Length` is ignored, the body is read according to `Transfer-Encoding`, and the connection is closed after the response. A request that cannot be parsed as HTTP/1.1 gets 400 "invalid request" (a body with `code` is not guaranteed) and is not executed. Trace: CUJ7, CUJ8; pitfall P-8, RFC 9112 §6.1, §6.3; ADR 0072 item 8 (OD-7); F-120.

#### Scenario: Two ways to specify length
- **WHEN** a request arrives with `Transfer-Encoding: chunked` and `Content-Length`
- **THEN** the body is read as chunked, `Content-Length` is not used, and the connection is closed after the response

#### Scenario: A second request inside the body
- **WHEN** the body contains bytes resembling a second request
- **THEN** it is not executed as a separate request

#### Scenario: An unparseable request
- **WHEN** a request arrives that cannot be parsed as HTTP/1.1
- **THEN** 400, the operation is not executed, and the connection is closed after the response

### Requirement: Refusals that close the connection
The server SHALL close the connection after the response with `Connection: close` (the write side is closed gracefully, the input is drained per "Refusal before body read reaches the client") on these refusals: 401 `unauthorized` (always), 400 on an unparseable request, 413, 431, 503 `server_stopping`, and also after the response to a request with both `Transfer-Encoding` and `Content-Length`; other refusals (403, 404, 409, 429, storage 5xx) do not close the connection if the body has been read. Trace: CUJ7, CUJ8; ADR 0072 items 1, 3, 5, 8.

#### Scenario: An invalid token
- **WHEN** a request with an invalid token received 401
- **THEN** the response contains `Connection: close` and the connection is closed after the response; the response is the same for any unrecognized token

#### Scenario: A permissions refusal
- **WHEN** a "read" token received 403 on an add with the body read
- **THEN** the connection stays open for the next request

### Requirement: Clean shutdown: HTTP side
On SIGTERM, SIGINT or an operator command, the server SHALL: close the listeners; close without a response connections whose request headers have not yet been received; answer a request with received headers with "server stopping" and close, without executing it; complete the writes already taken by the writer and reject the rest in the same way; give reads 5 s, then cut the streams without a terminating marker; perform the storage steps (ADR 0054 item 8), exit code 0. Trace: CUJ14, CUJ15; ADR 0028 item 10, ADR 0074 item 7 (OD-7).

#### Scenario: A write in flight
- **WHEN** the server receives SIGTERM while a write is being executed
- **THEN** the write is acknowledged or rejected, the client received a response, then the clean-shutdown marker is set and the exit code is 0

#### Scenario: A write not taken by the writer
- **WHEN** the signal arrived while the write body was still being read or the operation was waiting in the writer queue
- **THEN** the client receives 503 `server_stopping`, and the operation is neither executed nor acknowledged

#### Scenario: A new request on an open connection
- **WHEN** after the signal a request arrives on an open connection whose headers have already been received
- **THEN** "server stopping", the connection is closed, and the operation is not executed

#### Scenario: An idle connection
- **WHEN** after the signal a connection is idle: there are no request headers
- **THEN** it is closed without a response; a client that managed to write a request into the closing connection sees a disconnect

#### Scenario: A long stream
- **GIVEN** a number history is being streamed to a slow client
- **WHEN** SIGTERM is received
- **THEN** after 5 s the stream is cut without a terminating marker, the snapshot is released, and the shutdown is clean

#### Scenario: New connections
- **WHEN** after the signal a client opens a connection
- **THEN** the connection is rejected

### Requirement: By-number stream is exempt from the processing deadline
A number history SHALL be streamed without an overall deadline: after the first byte (with a 20 s deadline before it) the stream lives as long as the client accepts data; the next portion is read from the engine only after the bounded connection buffer has accepted the previous one. Idle time is 120 s without data accepted by the client: the connection is closed, the snapshot is released, and the stream is cut without a terminating marker. Trace: CUJ4, CUJ8, X-1; ADR 0035 item 6, ADR 0056 item 2, ADR 0072 item 4; Q-52; ADR 0100 (OD-7).

#### Scenario: A maximum history to a slow client
- **GIVEN** a history of 2 967 164 records and a client accepting data continuously, so the results take longer than 10 s and 120 s
- **WHEN** the results are being streamed
- **THEN** the processing deadline does not cut them, and they complete in full with the completeness marker

#### Scenario: The client stopped reading
- **WHEN** the client does not accept data for 120 s (the connection buffer is not freed)
- **THEN** the connection is closed without a terminating marker, the snapshot is released, and no write is delayed

#### Scenario: Memory does not grow
- **WHEN** a history of 10^6 records is streamed to a slow client
- **THEN** the server's output buffer is bounded by a small cap and a portion, and does not grow with the number of records
