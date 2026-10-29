# server-runtime Specification

## Purpose
The server process: the single database owner, HTTP/1.1 on hyper, the request execution model on top of storage, limits and timeouts (including for network clients and the number stream), protection of connection slots, mapping storage states into responses without showing them in advance, start (including during an index rebuild) and clean shutdown.

The code, status and headers of each refusal and the single request check order are defined by `http-api` ("Client-failure codes and statuses", "Server-failure codes and statuses", "Request check order"); here refusals are named in words: "unauthorized" — `unauthorized`, "forbidden" — `forbidden`, "too many requests" — `rate_limited`, "server starting" — `server_starting`, "server stopping" — `server_stopping`, "too many connections" — `too_many_connections`, "payload too large" (body) — `payload_too_large`, "headers too large" — `headers_too_large`, "invalid request" — `invalid_request`, "request timeout" — `request_timeout`, "write outcome unknown" — `write_outcome_unknown`, "read-only" — `read_only`, "storage error" — `storage_error`, "snapshot revoked" — `snapshot_revoked`, "internal error" — `internal_error`.

## ADDED Requirements

### Requirement: One process owns the database and serves all clients
One server process SHALL own the database (taking `LOCK` as its first action) and serve the API and UI inside itself; there is no other process that opens the database files. Trace: X-2, CUJ13, CUJ15; ADR 0011 item 5, ADR 0015 item 3, ADR 0054 item 1, ADR 0070 item 1.

#### Scenario: A second server on the same database
- **WHEN** a second process starts on a directory with a database owned by a running server
- **THEN** it ends with the refusal «база уже открыта другим процессом» ("database already open by another process"), opens no listeners, the running server is unaffected

#### Scenario: UI and API are one process
- **WHEN** the UI and an API client work with the database at the same time
- **THEN** both requests are served by one process, and a change through the API is visible in the UI under the rules of `storage-snapshots` (after acknowledgment)

### Requirement: HTTP/1.1 on hyper, listeners per settings
The server SHALL accept requests over HTTP/1.1 on hyper with JSON on serde_json, on the loopback listener and, if the network listener is enabled, on the network HTTPS listener; HTTP/2 is not offered. Trace: brief §3 ("Interfaces", "Implementation"), CUJ7, CUJ8; ADR 0070 items 2, 3.

#### Scenario: HTTP/1.1 only
- **WHEN** a client offers HTTP/2 (ALPN `h2` on the network listener)
- **THEN** the connection works over HTTP/1.1

#### Scenario: The network listener is off
- **WHEN** the network listener is not enabled
- **THEN** there is no network listener, the loopback listener works (ADR 0012 item 3)

### Requirement: Listener bind error means start refusal
If a listener's address is taken or unavailable, the server SHALL refuse to start with a message to the operator (address, reason), not choose a fallback port and not print the UI address; the server SHALL NOT share a port with another process. Trace: CUJ13, X-2; ADR 0070 item 3, trap P-7.

#### Scenario: The port is taken by a foreign process
- **WHEN** the loopback listener's port is taken
- **THEN** the server does not start, the UI address is not printed, the operator gets the address and reason

#### Scenario: No fallback port
- **WHEN** the port is taken and a neighboring one is free
- **THEN** the server does not open the neighboring port

### Requirement: Storage does not block accepting and responding
The server SHALL perform storage reads and waiting for the writer outside the I/O threads: while readers are busy or the writer waits for a flush to stable storage, the server accepts connections, parses headers, checks the token and issues refusals. Trace: CUJ7, CUJ8, X-1; ADR 0055 item 2, ADR 0070 item 4.

#### Scenario: The writer waits for a flush
- **GIVEN** the writer is waiting for a flush to stable storage
- **WHEN** a request with an unknown token and a search by number arrive
- **THEN** the first gets "unauthorized" without waiting for the writer, the second runs on a snapshot

#### Scenario: All readers are busy
- **GIVEN** all threads of the reader pool are busy with long searches
- **WHEN** a request without a token arrives
- **THEN** the "unauthorized" refusal is issued without waiting for a reader thread

### Requirement: Heavy searches do not displace other reads
Word-prefix search SHALL occupy no more than ⌈P/2⌉ threads of the reader pool, where P = max(4, number of cores); other reads use any thread. Excess word searches wait; the wait counts toward the request timeout. Trace: CUJ4, CUJ5, X-1; ADR 0049 item 3, ADR 0070 item 6.

#### Scenario: A surge of word searches
- **GIVEN** P threads, and ⌈P/2⌉ heavy word searches are running
- **WHEN** more word searches and a search by number arrive
- **THEN** the extra word searches wait, the search by number runs immediately

#### Scenario: The wait counts toward the timeout
- **WHEN** a word search waits in the queue longer than the read processing deadline
- **THEN** it ends with a "request timeout" refusal without starting work

### Requirement: Snapshot is released on any request outcome
The server SHALL release a request's snapshot on completion, error, timeout expiry, a client connection drop, server shutdown and handler panic, and SHALL interrupt read work no later than after the next page read. Trace: CUJ4, CUJ8, X-1; ADR 0056 item 4, ADR 0070 item 7.

#### Scenario: The client dropped the connection
- **GIVEN** a long word search is running
- **WHEN** the client closed the connection
- **THEN** the search is stopped, the snapshot is released, the reader thread is free

#### Scenario: A read handler panic
- **WHEN** a read handler panics
- **THEN** the request gets "internal error", the snapshot is released, the process continues to work

### Requirement: Writer panic terminates the process
A panic in the writer thread or the checkpoint thread SHALL terminate the process immediately as a crash shutdown, without acknowledging unacknowledged operations. Trace: CUJ14, X-5; ADR 0028 item 10, ADR 0070 item 8.

#### Scenario: A writer panic
- **WHEN** the writer thread panics during an operation that was not acknowledged
- **THEN** the process terminates, the client did not receive an acknowledgment, the next start is a crash start, the operation is either whole or absent (CUJ14)

### Requirement: Early refusal does not read the body or touch storage
The server SHALL perform request checks in the order of `http-api` ("Request check order"); access (token, permissions, request limit) SHALL be checked only from headers, before reading the body. A refusal at an early step SHALL NOT lead to reading the body or touching storage; for a write, the database state is checked before the operation checks. Trace: CUJ7, CUJ8, CUJ16; ADR 0053 item 8, ADR 0063, ADR 0070 item 5.

#### Scenario: No token, a large body
- **WHEN** a request without a token arrives with a declared body of 200 KiB
- **THEN** "unauthorized" is issued from headers, the body is not read, storage is not called

#### Scenario: A "read" token tries to write
- **WHEN** a token with the read permission calls add with a body exceeding the cap
- **THEN** the refusal is "forbidden", not "payload too large"

#### Scenario: State before input
- **GIVEN** the database is "read-only"
- **WHEN** an add arrives with correct JSON but an invalid number
- **THEN** the refusal is "read-only", not "invalid input" (`write-operations`, "Check order on write")

### Requirement: Connection limits
The server SHALL limit connections: 256 per loopback listener and 256 per network one (the pools are separate) and 32 from one network address. The values are defaults, awaiting the owner's decision. Trace: CUJ7, CUJ8; brief §4; ADR 0071 (Q-52).

#### Scenario: A surge from the network does not take slots from the UI
- **GIVEN** the network pool is entirely occupied
- **WHEN** the UI or a local program opens a connection on loopback
- **THEN** the connection is accepted

#### Scenario: One network address
- **WHEN** one network address opens a 33rd connection
- **THEN** it gets a "too many connections" refusal, connections of other addresses are unaffected

### Requirement: Request size cap
The server SHALL reject a request whose request line and headers exceed 16 KiB ("headers too large") and a request with a body over 256 KiB ("payload too large", by `Content-Length` before reading the body, otherwise by the count read); a valid name with any number of whitespace characters at the edges within the body cap SHALL be accepted. The values are defaults (D-43), a body below 256 KiB cannot be configured; the codes are in `http-api`. Trace: CUJ1, CUJ5, CUJ7; ADR 0064, 0071; `records`.

#### Scenario: A name with 20,000 whitespace characters at the edges
- **WHEN** a name of 200 letters with 20,000 U+3000 whitespace characters at each edge, written as `　`, is sent in the body
- **THEN** the body is within the cap, the request is accepted, the server does not refuse on size (`records`, "Input size before normalization and request body cap")

#### Scenario: A body above the cap
- **WHEN** a request with a valid token declares a `Content-Length` of 300 KiB
- **THEN** the "payload too large" refusal is issued before reading the body and delivered to the client (ADR 0072 item 5)

#### Scenario: A body without a declared length
- **WHEN** a chunked body exceeded 256 KiB
- **THEN** reading stops, the refusal is "payload too large"

#### Scenario: Headers above the cap
- **WHEN** the request line with headers takes 16,385 bytes
- **THEN** the refusal is "headers too large", the request is not executed

### Requirement: Request read timeouts
The server SHALL close (without a response) a connection that has not sent the first request in full (including the TLS handshake) within 10 s, that idles between requests for 60 s, and that has not sent the body within 10 s after the headers; the headers of each subsequent request — 10 s from the first byte. The values are defaults. Trace: CUJ7, CUJ8; ADR 0071, trap P-6.

#### Scenario: Slow headers
- **WHEN** a client sends headers slower than 10 s
- **THEN** the connection is closed, the slot is freed

#### Scenario: Idle between requests
- **WHEN** a connection idles for 60 s after a response
- **THEN** the server closes it, the client opens a new one

#### Scenario: A slow body
- **WHEN** the body is sent for longer than 10 s after the headers
- **THEN** the request is not executed, the connection is closed

### Requirement: Read processing deadline of 20 s
A read that has not begun its response within 20 s of the end of request parsing SHALL end with a "request timeout" refusal, the work stops and the snapshot is released; a valid word search at 10^8 (up to ≈ 14 s) SHALL fit the deadline. The value is a default. Trace: CUJ5, X-1; ADR 0071 item 2, ADR 0049 item 3.

#### Scenario: A 14 s search
- **WHEN** a word search at 10^8 records takes 14 s
- **THEN** it ends with a result, not a "request timeout"

#### Scenario: A search longer than the deadline
- **WHEN** read processing gave no first byte within 20 s
- **THEN** "request timeout", the work is stopped, the snapshot is released; the "snapshot revoked" refusal (ceiling 30 s) is not issued

### Requirement: Write deadline does not present unknown as not executed
A write that got no outcome within 10 s SHALL: if the writer has not yet accepted it — be withdrawn without consequences with a "request timeout" refusal; if accepted — get a "write outcome unknown" refusal, remaining in the writer's queue and being acknowledged or rejected. The server SHALL NOT report "not executed" about an accepted operation. Trace: CUJ9, CUJ12, CUJ14; ADR 0071 item 4, ADR 0026 items 5–6, trap P-12.

#### Scenario: The writer is stuck in a flush
- **GIVEN** the writer accepted an operation and has been waiting for a flush for more than 10 s
- **WHEN** the write deadline expired
- **THEN** the client gets "write outcome unknown"; the operation is either acknowledged later or rejected

#### Scenario: The client checks the outcome
- **WHEN** after "write outcome unknown" the client reads the number history and repeats the rename with the version read before sending
- **THEN** if the operation was applied, the repeat is a "version conflict" (ADR 0026), otherwise an ordinary result

#### Scenario: The operation was not accepted by the writer
- **WHEN** the write deadline expired and the operation in the queue has not yet been taken by the writer
- **THEN** it is withdrawn, data is unchanged, the client gets "request timeout"

### Requirement: By-number stream is exempt from the processing deadline
Number history SHALL be delivered as a stream without a general deadline: after the first byte (a 20 s deadline before it) the stream lives as long as the client accepts data; the server reads the next chunk from the engine only after the previous one is accepted. Write idle is 120 s with no data accepted by the socket: the connection is closed, the snapshot is released, the stream is cut off without the final marker. Trace: CUJ4, CUJ8, X-1; ADR 0035 item 6, ADR 0056 item 2, ADR 0072 item 4; Q-52.

#### Scenario: Maximal history to a slow client
- **GIVEN** a history of 2,967,164 records and a client accepting data continuously, the delivery lasts longer than 10 s and 120 s
- **WHEN** delivery goes on
- **THEN** the processing deadline does not cut it off, it ends in full with the completeness marker

#### Scenario: The client stopped reading
- **WHEN** the socket accepts no data for 120 s
- **THEN** the connection is closed without the final marker, the snapshot is released, writes are not delayed

#### Scenario: Memory does not grow
- **WHEN** a history of 10^6 records is delivered to a slow client
- **THEN** the server's buffer for the delivery does not exceed a chunk and does not grow with the number of records

### Requirement: Ordinary response write deadline of 10 s
An ordinary (non-stream) response SHALL be written with progress: if the socket accepted no data for 10 s, the connection is closed and the snapshot is released. The deadline is counted by the server code, not by a socket option. The value is a default. Trace: CUJ7, CUJ8; ADR 0072 item 3, traps P-6, P-10.

#### Scenario: The client does not read the response
- **WHEN** a client sent a request and does not read the response, and the socket buffer is full
- **THEN** after 10 s the connection is closed and the slot is free

#### Scenario: A slow but accepting client
- **WHEN** a client accepts the response slowly but accepts data every 10 s
- **THEN** the response is written to the end

### Requirement: Slot protection from anonymous clients
When a listener's pool is full the server SHALL accept a new connection by closing the oldest connection that has had no request with a valid token (class U) if it is older than the protection period of 1 s, otherwise the one idle longest after a valid token (class I); connections with processing in progress (class A) and class U younger than 1 s are not evicted; if there is no one to evict — a "too many connections" refusal (no more than 16 at once). Trace: CUJ7, CUJ8; ADR 0072 items 1–2 (D-53); P-6, P-7f.

#### Scenario: Anonymous connections filled the pool
- **GIVEN** the loopback pool is occupied by 256 connections without a token, older than 1 s
- **WHEN** a local program opens a connection and presents a valid token
- **THEN** the oldest anonymous connection is closed, the program's request is served

#### Scenario: The pool is busy with processing
- **GIVEN** all 256 connections are processing requests of valid tokens
- **WHEN** a new connection arrives
- **THEN** the refusal is "too many connections", the requests in progress are unaffected

#### Scenario: An idle valid client is evicted
- **GIVEN** the pool has no anonymous connections older than 1 s, and has idle clients
- **WHEN** a new connection arrives
- **THEN** the connection idle longer than the others is closed, the client later opens a new one

### Requirement: Anonymous connections from one network address
On the network listener there SHALL be no more than 16 class U connections from one address: a new one beyond that evicts the oldest class U connection of the same address, not a foreign one; the protection period does not apply to such an eviction. The value is a default (D-53). Trace: CUJ8; ADR 0072 item 2; P-7f.

#### Scenario: One network address does not evict others
- **GIVEN** one network address holds 16 class U connections, the network listener's pool is full
- **WHEN** this address opens one more anonymous connection
- **THEN** the oldest anonymous connection of the same address is closed, connections of other addresses are unaffected

### Requirement: Slot protection boundary
A legitimate connection SHALL be accepted when the stream of new anonymous connections is below the threshold of 256 per second per pool (pool 256, protection period 1 s), if it sends a request with a valid token within the protection period; at a stream above the threshold new connections, including legitimate ones, MAY get "too many connections". The values are defaults (D-53). Trace: CUJ7, CUJ8; ADR 0072 items 2, 6.

#### Scenario: A stream below the threshold
- **GIVEN** a source opens anonymous connections at a rate below 256 per second, the pool is filled with them
- **WHEN** a legitimate client opens a connection and sends a request with a valid token within 1 s
- **THEN** the request is served, there is no "too many connections" refusal

#### Scenario: A stream faster than the protection period
- **GIVEN** the loopback pool is occupied by 256 anonymous connections, all younger than 1 s
- **WHEN** a new connection arrives, including from a legitimate client
- **THEN** it gets "too many connections", anonymous ones are not evicted; the client retries later

### Requirement: Refusal before body read reaches the client
Refusals from headers ("unauthorized", "too many requests", "forbidden", "payload too large", "headers too large", "too many connections", "invalid request") SHALL be delivered to the client: before closing with unread input the server shuts the write side and drains the input for no more than 1 s and 64 KiB; a connection with an unread body is not reused. Trace: CUJ7, CUJ8; ADR 0072 item 5, trap P-9, RFC 9112 §9.6.

#### Scenario: A request without a token with a large body
- **WHEN** a client without a token sends headers and a body of 200 KiB
- **THEN** the client gets "unauthorized", not a connection reset without a response

#### Scenario: A refusal by request limit
- **WHEN** a network token exceeded the limit and sent a body
- **THEN** the "too many requests" refusal is delivered, storage and the reader pool are not used

### Requirement: Message boundary substitution is rejected
A request with `Transfer-Encoding` and `Content-Length` together, as well as a request that does not parse as HTTP/1.1, SHALL get an "invalid request" refusal (even during start, `http-api`, "Request check order") and not be executed; the connection is closed after the response. Trace: CUJ7, CUJ8; trap P-8, RFC 9112 §6.3; ADR 0072 item 8.

#### Scenario: Two ways to set the length
- **WHEN** a request arrives with `Transfer-Encoding: chunked` and `Content-Length`
- **THEN** the operation is not executed, the client gets "invalid request", the connection is closed

#### Scenario: A second request inside the body
- **WHEN** the body contains bytes that look like a second request
- **THEN** it is not executed as a separate request

### Requirement: Storage states are not shown in advance
The server SHALL NOT reveal the "read-only" and "storage error" states by anything other than the refusal of the action the state prevented: there is no state resource, no state header or field in successful responses. The operator is told the state in the message channel. Trace: CUJ16, X-5; ADR 0011 item 6, ADR 0063.

#### Scenario: Successful responses in "read-only"
- **GIVEN** the database is "read-only"
- **WHEN** a client runs searches
- **THEN** the responses do not differ from those of the normal state and contain no state marker

#### Scenario: No state resource
- **WHEN** a client looks for a way to learn the storage state
- **THEN** there is no such resource in API v1

### Requirement: Change refusal under storage states
A write operation under "read-only" SHALL get a "read-only" refusal, under "storage error" and a disk write error — "storage error" (it is stronger); data is not changed. The server does not cache the state: each write goes to the engine (the probe, ADR 0053 item 8). Trace: CUJ16; ADR 0053 items 5–8, ADR 0063.

#### Scenario: Read-only
- **WHEN** a change arrives under "read-only"
- **THEN** the refusal is "read-only", data is unchanged

#### Scenario: Space freed
- **GIVEN** the "read-only" state due to `ENOSPC`
- **WHEN** the operator freed space and the client repeated the change
- **THEN** the change is performed and acknowledged without a restart

#### Scenario: A disk write error
- **WHEN** a WAL write ended with `EIO`
- **THEN** the refusal is "storage error", there is no acknowledgment

#### Scenario: Both states
- **GIVEN** "read-only" and "storage error" are both in effect
- **WHEN** a change arrives
- **THEN** the refusal is "storage error"

### Requirement: Search under "storage error"
A read that needs a corrupted page SHALL get a "storage error" refusal (a full error response if before the first byte); a read that does not touch corrupted pages SHALL run; an incomplete result is not delivered as complete. Trace: CUJ16, CUJ4, CUJ5; ADR 0053 item 4, ADR 0063.

#### Scenario: A search does not touch the corrupted page
- **GIVEN** "storage error" due to a page not needed by a search by number
- **WHEN** this search runs
- **THEN** the result is complete and ordinary

#### Scenario: A search needs the corrupted page
- **WHEN** a search touches a page marked corrupted
- **THEN** the refusal is "storage error", without partial results

#### Scenario: Corruption in the middle of a stream
- **WHEN** corruption is found in a number history stream after the headers were sent
- **THEN** the connection is cut off without a final chunk and without the completeness marker; the client does not take the response for complete

### Requirement: Snapshot revocation is a separate repeatable reason
Snapshot revocation (the short-read ceiling of 30 s, the retention cap) SHALL give a "snapshot revoked" refusal, distinct from "storage error" and "internal error"; in a stream after the headers were sent — a connection cut without the final marker. The server state does not change. Trace: CUJ4, CUJ8, CUJ16; ADR 0056 item 2, ADR 0063.

#### Scenario: A short read revoked
- **WHEN** the snapshot of a short read is revoked before the first response byte
- **THEN** "snapshot revoked", no partial result, a retry is allowed

#### Scenario: The retention cap
- **WHEN** the retention cap is exceeded and a number stream is revoked
- **THEN** the connection is cut without the final marker, the server does not enter "storage error"

### Requirement: Client does not receive environment details
A response to a client SHALL NOT contain `errno`, file names, offsets, page numbers or system error text; they are reported to the operator. Other engine errors outside the kind table SHALL give "internal error", a tree key above 65,535 bytes (D-36) — "internal error" without a state change. Trace: CUJ16; ADR 0053 item 6, ADR 0063.

#### Scenario: An error with errno
- **WHEN** a write ended with `EIO` naming a file
- **THEN** the client gets "storage error" without the file and `errno`, the operator gets the source, `errno`, file, offset

#### Scenario: An unexpected engine error
- **WHEN** the engine returns an error outside the kind table
- **THEN** the client gets "internal error", details go to the operator

### Requirement: Startup order
The server SHALL: take `LOCK`; read the settings and certificate; check that the database exists; open the listeners; perform the start checks, recovery, index rebuild and the new generation; create the UI token and write the local-program token file; enter working mode. A refusal before the listeners are opened — without listeners; after — the listeners are closed, the operator gets the reason, a nonzero exit code. Trace: CUJ13, CUJ14, X-2; ADR 0074 item 1, ADR 0015, ADR 0054.

#### Scenario: A directory without a database
- **WHEN** the server starts on a directory without `control`
- **THEN** the refusal is "no database", no listeners, the UI address is not printed

#### Scenario: The certificate is missing
- **WHEN** the network listener is enabled and there is no certificate
- **THEN** the server does not start, the operator gets the reason

#### Scenario: A corrupted database
- **WHEN** the start check found an inconsistency
- **THEN** the listeners are closed, the operator gets the reason, file, offset or LSN, the process ends with a nonzero code, data is not modified

### Requirement: "Server starting" during start and rebuild
From the opening of the listeners to working mode (including token creation and writing the token file) the server SHALL answer any request on any path on any listener with a "server starting" refusal with a retry hint, without reading the body, without checking the route and token and without touching storage. Trace: CUJ7, CUJ13, CUJ14, X-1; ADR 0058 item 2, ADR 0074 items 2, 3; the order is in `http-api`.

#### Scenario: A request during a rebuild
- **GIVEN** a rebuild of the order index is in progress
- **WHEN** a search or write with any token arrives
- **THEN** "server starting", data is unchanged

#### Scenario: An old token after a restart
- **WHEN** during start a request with an old token arrives
- **THEN** "server starting"; after working mode begins a retry gets "unauthorized", the program re-reads the token file (CUJ7)

#### Scenario: No tokens yet
- **WHEN** a request arrives before the tokens of this run are created
- **THEN** it is not checked for a token: "server starting"

#### Scenario: The port is not released
- **WHEN** start takes longer than usual
- **THEN** the port is held by the server, a foreign process cannot take it

### Requirement: Token file is ready before working mode
The UI token and the local-program token file of this run SHALL be created before working mode; there is no window in which the server checks tokens while the token file is still the old one (listeners accept connections earlier, but requests in this window get "server starting" without a token check). Trace: CUJ7, CUJ13; ADR 0074 item 6.

#### Scenario: The first request after readiness
- **WHEN** a program reads the token file after the «готов» ("ready") message
- **THEN** the token is valid

#### Scenario: Reading the file during start
- **WHEN** a program reads the file during a rebuild and sends a request
- **THEN** "server starting"; after readiness a retry with the earlier token gets "unauthorized" (CUJ7)

### Requirement: Operator messages at startup
At start the server SHALL tell the operator: the directory and version; the kind of the previous shutdown ("clean", "crash", "start from backup"); for a crash — the number of discarded WAL bytes; during a rebuild — the reason, share and time estimate at least once every 10 s; a refusal — the reason and location; the listener addresses; the storage state if it is not normal; «готов» ("ready"). The channel and format belong to area 4. Trace: CUJ13, CUJ14, CUJ16; ADR 0074 item 4, ADR 0053, ADR 0054 item 4, ADR 0058 item 4.

#### Scenario: A start after a crash shutdown
- **WHEN** the server starts after SIGKILL
- **THEN** the operator gets "crash" and the number of discarded WAL tail bytes

#### Scenario: A rebuild
- **WHEN** a rebuild lasts longer than 10 s
- **THEN** the operator periodically gets: reason, share, time estimate; on completion the duration

#### Scenario: A start in "read-only"
- **WHEN** the files are not writable, the previous shutdown was clean
- **THEN** the operator gets a "read-only" message; clients are not shown it in advance (ADR 0011 item 6)

### Requirement: UI address is printed once and only to the operator
The UI address with the embedded token SHALL be printed after «готов» ("ready") and after all listeners have bound successfully, to the process's standard output, once per run; it is not written to the server log or to files. After a restart a new address is printed. Trace: CUJ13; ADR 0011 item 2, ADR 0074 item 5; trap P-5.

#### Scenario: Printing after readiness
- **WHEN** the server entered working mode
- **THEN** the address is printed after «готов» ("ready"); the user opens it and works (CUJ13)

#### Scenario: A log without a token
- **WHEN** the operator reads the server log
- **THEN** it has no tokens, `Authorization` header, address with an embedded token, request bodies, numbers or names

#### Scenario: An old link
- **WHEN** the server is restarted
- **THEN** the old address does not work, the new one is printed in the output

### Requirement: Clean shutdown: HTTP side
On SIGTERM, SIGINT or an operator command the server SHALL: close the listeners; on open connections answer a new request with "server stopping" with a close, without executing it; finish accepted writes by acknowledging or rejecting them; give running reads 5 s, then cancel them, cutting streams off without the final marker; then perform the storage steps (ADR 0054 item 8) and exit with code 0. Trace: CUJ14, CUJ15; ADR 0028 item 10, ADR 0074 item 7.

#### Scenario: A write in flight
- **WHEN** the server receives SIGTERM while a write is running
- **THEN** the write is acknowledged or rejected, the client got a response, then the clean-shutdown marker and code 0

#### Scenario: A new request on an open connection
- **WHEN** after the signal a request arrives on an open connection
- **THEN** "server stopping", the connection is closed, the operation is not executed

#### Scenario: A long stream
- **GIVEN** a number history is being delivered to a slow client
- **WHEN** SIGTERM is received
- **THEN** after 5 s the stream is cut off without the final marker, the snapshot is released, the shutdown is clean

#### Scenario: New connections
- **WHEN** after the signal a client opens a connection
- **THEN** the connection is refused

### Requirement: Repeated signal does not speed up shutdown
A repeated SIGTERM or SIGINT during shutdown SHALL NOT interrupt it and SHALL produce the message «остановка уже выполняется» ("shutdown already in progress"); a shutdown during start cancels the rebuild (the earlier index is intact) and ends the process. Trace: CUJ14; ADR 0028 item 10, ADR 0058 item 3, ADR 0074 item 8.

#### Scenario: Two signals in a row
- **WHEN** a second SIGINT arrives during shutdown
- **THEN** the shutdown continues, the clean-shutdown marker is written, code 0

#### Scenario: Shutdown during a rebuild
- **WHEN** SIGTERM arrives during an index rebuild
- **THEN** the rebuild is cancelled, the earlier index is intact, the process ends, the next start rebuilds anew

### Requirement: Shutdown fits the deadline of area 4
A shutdown on a signal SHALL end no later than 35 s on a database of 10^8 under normal load (up to 5 s for reads plus ≲ 30 s of storage); the forced-termination deadline (area 4) is at least 60 s. Forced termination before the marker is written is a crash shutdown. Trace: CUJ14, X-1; ADR 0054 item 8, ADR 0074 item 9; Q-108.

#### Scenario: Shutdown under load
- **WHEN** SIGTERM is received during writes of 100 operations per second and 10 reads
- **THEN** the process ends with code 0 no later than 35 s, the marker is written

#### Scenario: The deadline expired
- **WHEN** the process is forcibly terminated before the marker is written
- **THEN** the next start is a crash start, data is intact (`storage-durability`)
