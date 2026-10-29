# server-runtime Specification

## Purpose
The server process: the single owner of the database, HTTP/1.1 on hyper, the request execution model on top of storage, limits and deadlines (including for network clients and the by-number stream), protection of connection slots, mapping of storage states to responses without advance disclosure, startup (including during an index rebuild) and clean shutdown.

The code, status and headers of every refusal and the single order of request checks are set by `http-api` ("Client-failure codes and statuses", "Server-failure codes and statuses", "Request check order"); here refusals are named in words: "not authorized" — `unauthorized`, "no permissions" — `forbidden`, "too many requests" — `rate_limited`, "server starting" — `server_starting`, "server stopping" — `server_stopping`, "request too large" (body) — `payload_too_large`, "headers too large" — `headers_too_large`, "invalid request" — `invalid_request`, "request timeout" — `request_timeout`, "write outcome unknown" — `write_outcome_unknown`, "read-only" — `read_only`, "storage error" — `storage_error`, "snapshot revoked" — `snapshot_revoked`, "internal error" — `internal_error`.

## Requirements

### Requirement: One process owns the database and serves all clients
One server process SHALL own the database (its first action is to take `LOCK`) and serve the API and UI within itself; there is no other process that opens the database files. Trace: X-2, CUJ13, CUJ15; ADR 0011 item 5, ADR 0015 item 3, ADR 0054 item 1, ADR 0070 item 1.

#### Scenario: Second server on the same database
- **WHEN** a second process is started on a directory with a database owned by a running server
- **THEN** it exits with the refusal «база уже открыта другим процессом» ("database is already open by another process"), opens no listeners, the running server is not affected

#### Scenario: UI and API are one process
- **WHEN** the UI and an API client work with the database at the same time
- **THEN** one process serves both requests, and a change made through the API is visible in the UI under the rules of `storage-snapshots` (after acknowledgment)

### Requirement: HTTP/1.1 on hyper, listeners per settings
The server SHALL accept requests over HTTP/1.1 on hyper with JSON on serde_json, on the loopback listener and, if network listening is enabled, on the network HTTPS listener; HTTP/2 is not offered. Trace: brief §3 ("Interfaces", "Implementation"), CUJ7, CUJ8; ADR 0070 items 2, 3.

#### Scenario: HTTP/1.1 only
- **WHEN** a client offers HTTP/2 (ALPN `h2` on the network listener)
- **THEN** the connection works over HTTP/1.1

#### Scenario: Network listening is off
- **WHEN** network listening is not enabled
- **THEN** there is no network listener, the loopback listener works (ADR 0012 item 3)

### Requirement: Listener bind error means start refusal
If the listener address is occupied or unavailable, the server SHALL refuse to start with an operator message (address, reason), SHALL NOT pick a fallback port and SHALL NOT print the UI address; the server SHALL NOT share a port with another process. Trace: CUJ13, X-2; ADR 0070 item 3, trap P-7.

#### Scenario: Port occupied by a foreign process
- **WHEN** the loopback listener port is occupied
- **THEN** the server does not start, the UI address is not printed, the operator gets the address and the reason

#### Scenario: No fallback port
- **WHEN** the port is occupied and the adjacent one is free
- **THEN** the server does not open the adjacent port

### Requirement: Storage does not block accepting and responding
The server SHALL perform storage reads and waiting for the writer outside the I/O threads: while readers are busy or the writer waits for a flush to stable storage, the server accepts connections, parses headers, checks the token and issues refusals. Trace: CUJ7, CUJ8, X-1; ADR 0055 item 2, ADR 0070 item 4.

#### Scenario: Writer waits for a flush
- **GIVEN** the writer is waiting for a flush to stable storage
- **WHEN** a request with an unknown token and a search by number arrive
- **THEN** the first gets "not authorized" without waiting for the writer, the second is executed on a snapshot

#### Scenario: All readers are busy
- **GIVEN** all threads of the reader pool are busy with long searches
- **WHEN** a request without a token arrives
- **THEN** the "not authorized" refusal is issued without waiting for a reader thread

### Requirement: Heavy searches do not displace other reads
Word-prefix search SHALL occupy no more than ⌈P/2⌉ threads of the reader pool, where P = max(4, number of cores); other reads use any thread. Excess word searches wait; the wait counts toward the request deadline. Trace: CUJ4, CUJ5, X-1; ADR 0049 item 3, ADR 0070 item 6.

#### Scenario: A surge of word searches
- **GIVEN** P threads, and ⌈P/2⌉ heavy word searches are running
- **WHEN** more word searches and a search by number arrive
- **THEN** the extra word searches wait, the search by number is executed immediately

#### Scenario: Waiting counts toward the deadline
- **WHEN** a word search waits in the queue longer than the read processing deadline
- **THEN** it ends with the "request timeout" refusal without starting work

### Requirement: Snapshot is released on any request outcome
The server SHALL release the request snapshot on completion, error, deadline expiry, client connection drop, server stop and handler panic, and SHALL interrupt read work no later than after the next page read. Trace: CUJ4, CUJ8, X-1; ADR 0056 item 4, ADR 0070 item 7.

#### Scenario: Client dropped the connection
- **GIVEN** a long word search is running
- **WHEN** the client closed the connection
- **THEN** the search is stopped, the snapshot is released, the reader thread is free

#### Scenario: Read handler panic
- **WHEN** a read handler panics
- **THEN** the request gets "internal error", the snapshot is released, the process continues to work

### Requirement: Writer panic terminates the process
A panic in the writer thread or the checkpoint thread SHALL terminate the process immediately as a crash shutdown, without acknowledging unacknowledged operations. Trace: CUJ14, X-5; ADR 0028 item 10, ADR 0070 item 8.

#### Scenario: Writer panic
- **WHEN** the writer thread panics during an operation that has not received acknowledgment
- **THEN** the process terminates, the client did not get an acknowledgment, the next start is a crash recovery start, the operation is either present in full or absent (CUJ14)

### Requirement: Early refusal does not read the body or touch storage
The server SHALL perform request checks in the order of `http-api` ("Request check order"); access (token, permissions, request limit) SHALL be checked only from the headers, before reading the body. A refusal at an early step SHALL NOT lead to reading the body or touching storage; for a write, the database state is checked before the operation checks. Trace: CUJ7, CUJ8, CUJ16; ADR 0053 item 8, ADR 0063, ADR 0070 item 5.

#### Scenario: No token, large body
- **WHEN** a request without a token with a declared 200 KiB body arrives
- **THEN** "not authorized" is issued from the headers, the body is not read, storage is not called

#### Scenario: A "read" token tries to write
- **WHEN** a token with the read permission calls add with a body exceeding the cap
- **THEN** the refusal is "no permissions", not "request too large"

#### Scenario: State before input
- **GIVEN** the database is "read-only"
- **WHEN** an add with correct JSON but an invalid number arrives
- **THEN** the refusal is "read-only", not "invalid input" (`write-operations`, "Check order on write")

### Requirement: Connection limits
The server SHALL limit connections: 256 per loopback listener and 256 per network listener (the pools are separate) and 32 from one network address. If there is no one to displace ("Slot protection from anonymous clients") or the address cap is exceeded, the new connection is closed without a response, including under TLS. Values — OD-6 (D-52, D-53). Trace: CUJ7, CUJ8; brief §4; ADR 0071 (Q-52); ADR 0072 items 2, 6 (OD-7).

#### Scenario: A network surge does not take slots from the UI
- **GIVEN** the network pool is fully occupied
- **WHEN** the UI or a local program opens a connection on loopback
- **THEN** the connection is accepted

#### Scenario: One network address
- **GIVEN** an address has 32 connections, of which fewer than 16 are class U
- **WHEN** the same address opens a 33rd connection
- **THEN** it is closed without a response, connections of other addresses are not affected

### Requirement: Request size cap
The server SHALL reject a request whose query string and headers exceed 16 KiB ("headers too large"; if the refusal is produced by HTTP parsing — only status 431, without a body with `code`; there is no separate cap on the number of headers) and a request with a body larger than 32 KiB ("request too large", by `Content-Length` before reading the body, otherwise by the count of bytes read); a valid name within the body cap SHALL be accepted. Values: the 32 KiB body cap is an owner decision (OD-6, D-43; as in brief §4), the header cap is D-43; the body cap cannot be configured below 32 KiB; codes — `http-api`. Trace: CUJ1, CUJ5, CUJ7; OD-6; ADR 0064, 0071; `records`.

#### Scenario: Name with 20 000 whitespace characters at the edges
- **WHEN** a name of 200 letters with 20 000 whitespace characters U+3000 on each edge, written as `　`, is sent in a body larger than 32 KiB
- **THEN** the refusal "request too large" comes before reading the body and before parsing; the name is not parsed (OD-6, D-43)

#### Scenario: Name with edges within the body cap
- **WHEN** a name of 200 letters with 1 000 whitespace characters U+3000 on each edge, written as `　` (≈ 12 000 bytes of edges), is sent in the body
- **THEN** the body is within the cap, the request is accepted, the server does not refuse on size (`records`, "Input size before normalization and request body cap")

#### Scenario: Body larger than the cap
- **WHEN** a request with a valid token declares `Content-Length` of 300 KiB
- **THEN** the refusal "request too large" is issued before reading the body and delivered to the client (ADR 0072 item 5)

#### Scenario: Body without a declared length
- **WHEN** a chunked body exceeded 32 KiB
- **THEN** reading is stopped, refusal "request too large"

#### Scenario: Headers larger than the cap
- **WHEN** the query string with headers takes 16 385 bytes
- **THEN** refusal "headers too large", the request is not executed

### Requirement: Request read timeouts
The server SHALL close (without a response) a connection that has not transmitted the first request in full (including the TLS handshake) within 10 s, that idles between requests for 60 s, and that has not transmitted the body within 10 s after the headers; the headers of each following request — 10 s from the first byte. These are defaults. Trace: CUJ7, CUJ8; ADR 0071, trap P-6.

#### Scenario: Slow headers
- **WHEN** a client transmits headers slower than 10 s
- **THEN** the connection is closed, the slot is released

#### Scenario: Idle between requests
- **WHEN** a connection idles for 60 s after a response
- **THEN** the server closes it, the client opens a new one

#### Scenario: Slow body
- **WHEN** the body is transmitted for longer than 10 s after the headers
- **THEN** the request is not executed, the connection is closed

### Requirement: Read processing deadline of 20 s
A read that has not started its response within 20 s from the end of request parsing SHALL end with the "request timeout" refusal, the work is stopped and the snapshot is released; a valid word search on 10^7 (up to ≈ 1.4 s, recalculated proportionally from ≈ 14 s on 10^8) SHALL fit within the deadline. This is a default (D-52, accepted by the owner, OD-6; the 20 s deadline was chosen for 10^8, revision on a prototype — T). Trace: CUJ5, X-1; ADR 0071 item 2, ADR 0049 item 3.

#### Scenario: 14 s search
- **WHEN** a word search takes 14 s (margin: ≈ 1.4 s is expected on 10^7, OD-6)
- **THEN** it ends with a result, not a "request timeout"

#### Scenario: Search longer than the deadline
- **WHEN** read processing did not produce the first byte within 20 s
- **THEN** "request timeout", work is stopped, the snapshot is released; the "snapshot revoked" refusal (30 s ceiling) is not issued

### Requirement: Write deadline does not present unknown as not executed
A write that has not received an outcome within 10 s SHALL: if the writer has not yet accepted it — be withdrawn without consequences with the "request timeout" refusal; if accepted — receive the "write outcome unknown" refusal, remaining in the writer queue and being acknowledged or rejected. The server SHALL NOT report "not executed" for an accepted operation. Trace: CUJ9, CUJ12, CUJ14; ADR 0071 item 4, ADR 0026 items 5–6, trap P-12.

#### Scenario: Writer stuck in a flush
- **GIVEN** the writer accepted an operation and waits for a flush longer than 10 s
- **WHEN** the write deadline expired
- **THEN** the client gets "write outcome unknown"; the operation is either acknowledged later or rejected

#### Scenario: Client checks the outcome
- **WHEN** after "write outcome unknown" the client reads the number history and retries the rename with the version read before sending
- **THEN** if the operation was applied, the retry is "version conflict" (ADR 0026), otherwise — an ordinary result

#### Scenario: Operation not accepted by the writer
- **WHEN** the write deadline expired and the operation in the queue has not yet been taken by the writer
- **THEN** it is withdrawn, data is not changed, the client gets "request timeout"

### Requirement: By-number stream is exempt from the processing deadline
The number history SHALL be delivered as a stream without a total deadline: after the first byte (a 20 s deadline before it) the stream lives as long as the client accepts data; the next portion is read from the engine only after the bounded connection buffer has accepted the previous one. Idle time is 120 s without data accepted by the client: the connection is closed, the snapshot is released, the stream is cut off without a completion marker. Trace: CUJ4, CUJ8, X-1; ADR 0035 item 6, ADR 0056 item 2, ADR 0072 item 4; Q-52; ADR 0100 (OD-7).

#### Scenario: Maximum history to a slow client
- **GIVEN** a history of 2 967 164 records and a client that accepts data continuously, the results take longer than 10 s and 120 s
- **WHEN** the results are being delivered
- **THEN** the processing deadline does not cut them off, they complete in full with the completeness marker

#### Scenario: Client stopped reading
- **WHEN** the client does not accept data for 120 s (the connection buffer is not freed)
- **THEN** the connection is closed without a completion marker, the snapshot is released, writes are not delayed

#### Scenario: Memory does not grow
- **WHEN** a history of 10^6 records is delivered to a slow client
- **THEN** the server buffer for the results is bounded by a small cap and a portion, and does not grow with the number of records

### Requirement: Ordinary response write deadline of 10 s
An ordinary (non-streaming) response SHALL be written with progress: if the socket has not accepted data for 10 s, the connection is closed and the snapshot is released. The deadline is counted by the server code, not by a socket option. This is a default. Trace: CUJ7, CUJ8; ADR 0072 item 3, traps P-6, P-10.

#### Scenario: Client does not read the response
- **WHEN** a client sent a request and does not read the response, and the socket buffer is full
- **THEN** after 10 s the connection is closed and the slot is free

#### Scenario: Slow but accepting client
- **WHEN** a client accepts the response slowly but accepts data every 10 s
- **THEN** the response is written to the end

### Requirement: Slot protection from anonymous clients
With a full listener pool the server SHALL accept a new connection by closing the oldest connection without a request with a valid token (class U), if it is older than the protection period of 1 s, otherwise the longest-idle one after a valid token (class I); connections with processing in progress (class A) and class U younger than 1 s are not displaced; if there is no one to displace, the new connection is closed without a response. Trace: CUJ7, CUJ8; ADR 0072 items 1–2 (D-53; OD-7); P-6, P-7f.

#### Scenario: Anonymous connections filled the pool
- **GIVEN** the loopback pool is occupied by 256 connections without a token, older than 1 s
- **WHEN** a local program opens a connection and presents a valid token
- **THEN** the oldest anonymous connection is closed, the program's request is served

#### Scenario: Pool busy with processing
- **GIVEN** all 256 connections are processing requests with valid tokens
- **WHEN** a new connection arrives
- **THEN** the new connection is closed without a response, requests in progress are not affected

#### Scenario: An idle valid client is displaced
- **GIVEN** the pool has no anonymous connections older than 1 s, there are idle connections of clients
- **WHEN** a new connection arrives
- **THEN** the connection idle longer than the others is closed, the client opens a new one later

### Requirement: Anonymous connections from one network address
On the network listener there SHALL be no more than 16 class U connections from one address: a new one beyond this displaces the oldest class U connection of the same address, not a foreign one; the protection period does not apply to such displacement. This rule takes precedence over the cap of 32 connections per address: refusal by the 32 cap happens only when the address has fewer than 16 class U connections. This is a default (D-53). Trace: CUJ8; ADR 0072 item 2; P-7f.

#### Scenario: One network address does not displace others
- **GIVEN** one network address holds 16 class U connections, the network listener pool is full
- **WHEN** this address opens one more anonymous connection
- **THEN** the oldest anonymous connection of the same address is closed, connections of other addresses are not affected

#### Scenario: An address has 32 connections, 16 of them anonymous
- **GIVEN** an address has 32 connections, 16 of them class U
- **WHEN** this address opens a new connection
- **THEN** the oldest class U connection of this address is closed, the new one is accepted (the address's connection count does not grow); there is no refusal by the 32 cap

### Requirement: Slot protection boundary
A legitimate connection SHALL be accepted when the stream of new anonymous connections is below the threshold of 256 per second per pool (pool 256, protection period 1 s), if it sends a request with a valid token within the protection period; above the threshold, new connections, including legitimate ones, MAY be closed without a response. These are defaults (D-53). Trace: CUJ7, CUJ8; ADR 0072 items 2, 6 (OD-7).

#### Scenario: Stream below the threshold
- **GIVEN** a source opens anonymous connections at a rate below 256 per second, the pool is filled by them
- **WHEN** a legitimate client opens a connection and sends a request with a valid token within 1 s
- **THEN** the request is served, the connection is not closed

#### Scenario: Stream faster than the protection period
- **GIVEN** the loopback pool is occupied by 256 anonymous connections, all younger than 1 s
- **WHEN** a new connection arrives, including from a legitimate client
- **THEN** it is closed without a response, the anonymous ones are not displaced; the client retries later

### Requirement: Refusal before body read reaches the client
Refusals made from headers ("not authorized", "too many requests", "no permissions", "request too large", "headers too large", "invalid request") SHALL be delivered to the client: before closing with unread input, the server closes the write side and reads the remaining input for no more than 1 s and 64 KiB; a connection with an unread body is not reused. Trace: CUJ7, CUJ8; ADR 0072 item 5, trap P-9, RFC 9112 §9.6.

#### Scenario: Request without a token with a large body
- **WHEN** a client without a token sends headers and a 200 KiB body
- **THEN** the client gets "not authorized", not a connection reset without a response

#### Scenario: Request limit refusal
- **WHEN** a network token exceeded the limit and sent a body
- **THEN** the "too many requests" refusal is delivered, storage and the reader pool are not used

### Requirement: Message boundary substitution is ruled out
A request with `Transfer-Encoding` and `Content-Length` SHALL be handled per RFC 9112 §6.1, as hyper does: `Content-Length` is ignored, the body is read by `Transfer-Encoding`, the connection is closed after the response. A request that cannot be parsed as HTTP/1.1 gets 400 "invalid request" (a body with `code` is not guaranteed) and is not executed. Trace: CUJ7, CUJ8; trap P-8, RFC 9112 §6.1, §6.3; ADR 0072 item 8 (OD-7); F-120.

#### Scenario: Two ways to set the length
- **WHEN** a request arrives with `Transfer-Encoding: chunked` and `Content-Length`
- **THEN** the body is read as chunked, `Content-Length` is not used, the connection is closed after the response

#### Scenario: Second request inside the body
- **WHEN** the body contains bytes that look like a second request
- **THEN** it is not executed as a separate request

#### Scenario: Unparseable request
- **WHEN** a request that cannot be parsed as HTTP/1.1 arrives
- **THEN** 400, the operation is not executed, the connection is closed after the response

### Requirement: Storage states are not shown in advance
The server SHALL NOT disclose the "read-only" and "storage error" states by anything other than the refusal of the action that the state prevented: there is no state resource, no state header or field in successful responses. The state is reported to the operator in the message channel. Trace: CUJ16, X-5; ADR 0011 item 6, ADR 0063.

#### Scenario: Successful responses in "read-only"
- **GIVEN** the database is "read-only"
- **WHEN** a client performs searches
- **THEN** the responses do not differ from responses of the normal state and contain no state marker

#### Scenario: No state resource
- **WHEN** a client looks for a way to learn the storage state
- **THEN** there is no such resource in API v1

### Requirement: Change refusal under storage states
A write operation under "read-only" SHALL get the "read-only" refusal, under "storage error" and a disk write error — "storage error" (it is stronger); data is not changed. The server does not cache the state: every write goes to the engine (a probe, ADR 0053 item 8). Trace: CUJ16; ADR 0053 items 5–8, ADR 0063.

#### Scenario: Read-only
- **WHEN** a change arrives under "read-only"
- **THEN** refusal "read-only", data is not changed

#### Scenario: Space freed
- **GIVEN** the "read-only" state due to `ENOSPC`
- **WHEN** the operator freed space and the client repeated the change
- **THEN** the change is executed and acknowledged without a restart

#### Scenario: Disk write error
- **WHEN** a WAL write ended with `EIO`
- **THEN** refusal "storage error", no acknowledgment

#### Scenario: Both states
- **GIVEN** "read-only" and "storage error" are in effect
- **WHEN** a change arrives
- **THEN** refusal "storage error"

### Requirement: Search under "storage error"
A read that needs a corrupted page SHALL get the "storage error" refusal (a full error response, if before the first byte); a read that does not touch corrupted pages SHALL be executed; an incomplete result is not delivered as complete. Trace: CUJ16, CUJ4, CUJ5; ADR 0053 item 4, ADR 0063.

#### Scenario: Search does not touch the corrupted page
- **GIVEN** a "storage error" caused by a page not needed by the search by number
- **WHEN** this search is executed
- **THEN** the result is complete and ordinary

#### Scenario: Search needs the corrupted page
- **WHEN** a search touches a page marked as corrupted
- **THEN** refusal "storage error", without partial results

#### Scenario: Corruption in the middle of a stream
- **WHEN** corruption is found in the number history stream after the headers were sent
- **THEN** the connection is cut off without a final chunk and without the completeness marker; the client does not take the response as complete

### Requirement: Snapshot revocation is a separate repeatable reason
Snapshot revocation (short-read ceiling of 30 s, retention cap) SHALL give the "snapshot revoked" refusal, different from "storage error" and "internal error"; in a stream after the headers were sent — a connection cut off without a completion marker. The server state does not change. Trace: CUJ4, CUJ8, CUJ16; ADR 0056 item 2, ADR 0063.

#### Scenario: Short read revoked
- **WHEN** the snapshot of a short read was revoked before the first byte of the response
- **THEN** "snapshot revoked", no partial result, retry is allowed

#### Scenario: Retention cap
- **WHEN** the retention cap is exceeded and the by-number stream is revoked
- **THEN** the connection is cut off without a completion marker, the server does not go into "storage error"

### Requirement: Client does not receive environment details
A response to a client SHALL NOT contain `errno`, file names, offsets, page numbers or system error text; these are reported to the operator. Other engine errors outside the kind table SHALL give "internal error", a tree key above 65 535 bytes (D-36) — "internal error" without a state change. Trace: CUJ16; ADR 0053 item 6, ADR 0063.

#### Scenario: Error with errno
- **WHEN** a write ended with `EIO` naming a file
- **THEN** the client gets "storage error" without the file and `errno`, the operator gets the source, `errno`, file, offset

#### Scenario: Unexpected engine error
- **WHEN** the engine returns an error outside the kind table
- **THEN** the client gets "internal error", the details go to the operator

### Requirement: Startup order
The server SHALL: make sure the database directory exists (if not — «базы нет» ("no database"), code 5, nothing is created); take `LOCK`; read the settings and the certificate; check that the database is present; take the settings directory lock (`control.lock`), parse the token file and open the operator control channel; open the listeners; perform start checks, restore, index rebuild and the new generation; create the UI token and write the local-program token file; enter working mode. A refusal before the listeners are opened — without listeners; after — the listeners are closed; on a refusal after the control channel is opened it is closed and its file is deleted; the operator gets the reason, a non-zero code. A settings directory that is not writable does not allow creating `control.lock` and opening the channel (an existing `control.lock` is taken for reading): the decision on such a start is in `operator-lifecycle`, "Settings directory is not writable". Trace: CUJ13, CUJ14, X-2; ADR 0074 item 1, ADR 0015, ADR 0054; ADR 0077 item 1; ADR 0080 item 2; ADR 0081 item 1.

#### Scenario: Directory without a database
- **WHEN** the server is started on a directory without `control`
- **THEN** refusal «базы нет» ("no database"), no listeners, the control channel is not opened, `control.lock` is not taken, the UI address is not printed

#### Scenario: Certificate missing
- **WHEN** network listening is enabled, there is no certificate
- **THEN** the server does not start, the operator gets the reason

#### Scenario: Corrupted database
- **WHEN** the start check found an inconsistency
- **THEN** the listeners are closed, the operator gets the reason, file, offset or LSN, the process exits with a non-zero code, data is not modified

#### Scenario: Control channel before the listeners
- **WHEN** an index rebuild is running at startup
- **THEN** the operator control channel is open from the moment the server took `control.lock`, and before the listeners are opened; HTTP listeners answer "server starting"

#### Scenario: Settings directory busy
- **WHEN** `control.lock` is held by another server
- **THEN** refusal «каталог настроек занят другим процессом» ("settings directory is busy with another process"), no listeners and no control channel, the earlier server is not affected

#### Scenario: Refusal after the channel is opened
- **WHEN** the start check refused after the control channel was opened
- **THEN** the listeners and the channel are closed, the channel file is deleted, the operator gets the reason and a non-zero code

### Requirement: "Server starting" during start and rebuild
From the opening of the listeners to working mode (including token creation and writing the token file) the server SHALL answer any request on any path on any listener with the "server starting" refusal with a retry hint, without reading the body, without checking the route and the token and without touching storage. Trace: CUJ7, CUJ13, CUJ14, X-1; ADR 0058 item 2, ADR 0074 items 2, 3; the order — `http-api`.

#### Scenario: Request during a rebuild
- **GIVEN** a rebuild of the order index is running
- **WHEN** a search or a write with any token arrives
- **THEN** "server starting", data is not changed

#### Scenario: Old token after a restart
- **WHEN** a request with an old token arrives during startup
- **THEN** "server starting"; after working mode the retry gets "not authorized", the program re-reads the token file (CUJ7)

#### Scenario: No tokens yet
- **WHEN** a request arrives before the tokens of this start are created
- **THEN** it is not checked for a token: "server starting"

#### Scenario: Port is not released
- **WHEN** startup takes longer than usual
- **THEN** the port is held by the server, a foreign process cannot take it

### Requirement: Token file is ready before working mode
The UI token and the local-program token file of this start SHALL be created before working mode; there is no window in which the server checks tokens while the token file is the old one (the listeners accept connections earlier, but requests in this window get "server starting" without a token check). Trace: CUJ7, CUJ13; ADR 0074 item 6.

#### Scenario: First request after readiness
- **WHEN** a program reads the token file after the «готов» ("ready") message
- **THEN** the token is valid

#### Scenario: Reading the file during startup
- **WHEN** a program reads the file during a rebuild and sends a request
- **THEN** "server starting"; after readiness, a retry with the old token gets "not authorized" (CUJ7)

### Requirement: Operator messages at startup
The server SHALL report to the operator at startup: the directory and version; the kind of the previous stop ("clean", "crash", "start from backup"); for a crash — the number of discarded WAL bytes; during a rebuild — the reason, the fraction and the time estimate at least once every 10 s; a refusal — the reason and the place; the listener addresses; the storage state, if it is not normal; «готов» ("ready"). The channel and format — area 4. Trace: CUJ13, CUJ14, CUJ16; ADR 0074 item 4, ADR 0053, ADR 0054 item 4, ADR 0058 item 4.

#### Scenario: Start after a crash shutdown
- **WHEN** the server starts after SIGKILL
- **THEN** the operator gets "crash" and the number of discarded WAL tail bytes

#### Scenario: Rebuild
- **WHEN** a rebuild takes longer than 10 s
- **THEN** the operator gets periodically: the reason, fraction, time estimate; on completion the duration

#### Scenario: Start in "read-only"
- **WHEN** files are not writable, the previous stop was clean
- **THEN** the operator gets a message about "read-only"; clients are not shown it in advance (ADR 0011 item 6)

### Requirement: UI address is printed once and only to the operator
The UI address with the embedded token SHALL be printed after «готов» ("ready") and after all listeners are bound successfully, to the process standard output, once per start; it is not written to the server log or to files. After a restart a new address is printed. Trace: CUJ13; ADR 0011 item 2, ADR 0074 item 5; trap P-5.

#### Scenario: Printing after readiness
- **WHEN** the server entered working mode
- **THEN** the address is printed after «готов» ("ready"); the user opens it and works (CUJ13)

#### Scenario: Log without a token
- **WHEN** the operator reads the server log
- **THEN** it contains no tokens, no `Authorization` header, no address with an embedded token, no request bodies, numbers or names

#### Scenario: Old link
- **WHEN** the server is restarted
- **THEN** the old address does not work, the new one is printed in the output

### Requirement: Clean shutdown: HTTP side
On SIGTERM, SIGINT or an operator command the server SHALL: close the listeners; close without a response the connections on which request headers have not yet been received; answer a request with received headers with "server stopping" with closing, without executing it; complete the writes taken by the writer, reject the rest the same way; give reads 5 s, then cut off the streams without a completion marker; perform the storage steps (ADR 0054 item 8), code 0. Trace: CUJ14, CUJ15; ADR 0028 item 10, ADR 0074 item 7 (OD-7).

#### Scenario: Write in flight
- **WHEN** the server receives SIGTERM while a write is being executed
- **THEN** the write is acknowledged or rejected, the client got a response, then the clean-shutdown marker and code 0

#### Scenario: Write not taken by the writer
- **WHEN** the signal arrived while the write body was still being read or the operation waited in the writer queue
- **THEN** the client gets 503 `server_stopping`, the operation is not executed and not acknowledged

#### Scenario: New request on an open connection
- **WHEN** after the signal a request arrives on an open connection whose headers have already been received
- **THEN** "server stopping", the connection is closed, the operation is not executed

#### Scenario: Idle connection
- **WHEN** after the signal a connection is idle: there are no request headers
- **THEN** it is closed without a response; a client that managed to write a request into the closing connection sees a break

#### Scenario: Long stream
- **GIVEN** a number history is being delivered to a slow client
- **WHEN** SIGTERM is received
- **THEN** after 5 s the stream is cut off without a completion marker, the snapshot is released, the shutdown is clean

#### Scenario: New connections
- **WHEN** after the signal a client opens a connection
- **THEN** the connection is refused

### Requirement: Repeated signal does not speed up shutdown
A repeated SIGTERM or SIGINT during shutdown SHALL NOT interrupt it and SHALL produce the message «остановка уже выполняется» ("shutdown already in progress"); a stop during startup cancels the rebuild (the earlier index is intact) and terminates the process. Trace: CUJ14; ADR 0028 item 10, ADR 0058 item 3, ADR 0074 item 8.

#### Scenario: Two signals in a row
- **WHEN** a second SIGINT arrives during shutdown
- **THEN** the shutdown continues, the clean-shutdown marker is written, code 0

#### Scenario: Stop during a rebuild
- **WHEN** SIGTERM arrives during an index rebuild
- **THEN** the rebuild is cancelled, the earlier index is intact, the process terminates, the next start rebuilds again

### Requirement: Shutdown fits the deadline of area 4
A shutdown on a signal SHALL complete no later than 35 s on a 10^7 database under normal load (up to 5 s for reads plus ≲ 30 s for storage); the forced termination deadline (area 4) is at least 60 s. A forced termination before the marker is written is a crash shutdown. Trace: CUJ14, X-1; ADR 0054 item 8, ADR 0074 item 9; Q-108.

#### Scenario: Shutdown under load
- **WHEN** SIGTERM is received during 100 write operations per second and 10 reads
- **THEN** the process exits with code 0 no later than 35 s, the marker is written

#### Scenario: Deadline expired
- **WHEN** the process is forcibly terminated before the marker is written
- **THEN** the next start is a crash recovery start, data is intact (`storage-durability`)

### Requirement: Refusals that close the connection
The server SHALL close the connection after a response with `Connection: close` (the write side is closed in steps, the input is read to the end per "Refusal before body read reaches the client") for the refusals: 401 `unauthorized` (always), 400 for an unparseable request, 413, 431, 503 `server_stopping`, and also after the response to a request with `Transfer-Encoding` and `Content-Length`; other refusals (403, 404, 409, 429, storage 5xx) do not close the connection if the body was read. Trace: CUJ7, CUJ8; ADR 0072 items 1, 3, 5, 8.

#### Scenario: Invalid token
- **WHEN** a request with an invalid token got 401
- **THEN** the response contains `Connection: close` and the connection is closed after the response; the response is the same for any unrecognized token

#### Scenario: Permission refusal
- **WHEN** a "read" token got 403 on an add with the body read
- **THEN** the connection stays open for the next request
