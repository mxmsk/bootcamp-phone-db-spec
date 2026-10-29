## MODIFIED Requirements

### Requirement: Local-program token in a file
At every start the server SHALL write a new random local-program token to a file in the settings directory: the file is
created at once with "owner only" permissions (0600), replaced atomically, and the write completes before working mode: while the file
is being written, the listeners are already open, but every request gets "server starting" and the token is not checked. The file content is exactly 64 lowercase hexadecimal ASCII characters and one LF; the client strips whitespace at the edges. Trace: CUJ7; ADR 0012 item 1; ADR 0065 item 2.

#### Scenario: File permissions
- **WHEN** the server has written the token file
- **THEN** only the OS user the server runs as can read the file, and at no moment of the write is the file
  accessible to other users

#### Scenario: File is ready for working mode
- **WHEN** the server has switched to working mode
- **THEN** the file already contains the token of the current start; a request with it is accepted; there is no window in which tokens are checked while the
  file holds the previous token

#### Scenario: File format
- **WHEN** a program reads the token file
- **THEN** it contains 64 `0-9a-f` characters and a trailing LF; the client sends only the 64 characters (without the LF) in `Authorization: Bearer`; a token with whitespace at the edges is not sent

#### Scenario: File is inaccessible to the program
- **WHEN** a local program has no permission to read the token file
- **THEN** it cannot call the API with a valid token; there is no access that bypasses the file

### Requirement: UI and file tokens are accepted only on the loopback listener
UI and local-program tokens SHALL be accepted only on the loopback listener, the network token only on the network one (HTTPS): the network listener accepts only network tokens (also from a loopback address), the loopback listener (HTTP without TLS) accepts only UI and file tokens; a network token on it gets "unauthorized" under any network-listener setting. The value was accepted by the owner (D-46, OD-6), and refines OD-3, Q-27. Trace: CUJ7, CUJ13; ADR 0066 item 1.

#### Scenario: A file token presented not on the loopback listener
- **WHEN** a file token or a UI token is presented on the network listener (including by a client from a loopback address)
- **THEN** refusal "unauthorized" and data is unchanged (the token is not recognized even as valid, the response is no different from the
  response to an unknown token)

#### Scenario: Network token on loopback with the network enabled
- **GIVEN** the network listener is enabled
- **WHEN** a valid network token is presented on the loopback listener (HTTP without TLS)
- **THEN** refusal "unauthorized", the response is no different from the response to an unknown token; over HTTPS on the network listener the same token is accepted

### Requirement: TLS certificate source and replacement
The certificate and private key SHALL be provided by the operator as PEM files (chain and key) at the paths from the settings; the server does not
issue the certificate itself. Replacement is swapping the files and a planned restart; a restart does not change network tokens. Trace: CUJ8;
ADR 0066 item 5 (Q-69; D-50, accepted, OD-6).

#### Scenario: Certificate replacement
- **WHEN** the operator replaced the certificate and key files and restarted the server
- **THEN** new connections use the new certificate, network tokens remain valid without reissuing

#### Scenario: Client does not trust the certificate
- **WHEN** a client does not trust the server certificate
- **THEN** the connection is dropped on the client side before the token is sent; the token does not reach the server

### Requirement: Loopback protection: Host check
The loopback listener SHALL reject a request whose Host header is not equal to `127.0.0.1:<port>`, `[::1]:<port>` or
`localhost:<port>` (port is the listener's port; on port 80 a Host without a port is also allowed), with the refusal "host not allowed" before the token check. Trace: CUJ13; P-3, P-4, P-7e;
ADR 0067 item 1.

#### Scenario: Name substitution (DNS rebinding)
- **WHEN** a page from a foreign domain whose name points to 127.0.0.1 calls the loopback listener, and Host is the foreign
  name
- **THEN** refusal "host not allowed"; the token is not checked, no data is returned

#### Scenario: Allowed Host
- **WHEN** Host is equal to `127.0.0.1:<port>`
- **THEN** the request proceeds to the token check

#### Scenario: Port 80
- **GIVEN** the loopback listener is on port 80
- **WHEN** the browser sends Host `127.0.0.1` without a port
- **THEN** the request proceeds to the token check, the UI works via the printed link

#### Scenario: Host different from the listener's
- **WHEN** Host is absent, has a different port or a different name
- **THEN** the request is rejected (an absent Host is "invalid request" (`invalid_request`) under HTTP rules; everything else is "host not allowed")

### Requirement: A stream of invalid tokens does not displace legitimate clients
A stream of invalid tokens (every 401 refusal closes the connection, `server-runtime`, "Refusals that close the connection") SHALL be limited only by connection limits: a legitimate client gets no refusals caused by the stream while the source opens anonymous connections slower than the slot-protection threshold (`server-runtime`); with a faster stream, new connections, including legitimate ones, MAY get "too many connections". Trace: CUJ7, CUJ8; ADR 0072 items 2, 6 (D-53).

#### Scenario: Stream below the threshold
- **WHEN** a source opens anonymous connections with invalid tokens slower than the slot-protection threshold
- **THEN** other clients' requests are served without refusals caused by this stream

#### Scenario: Stream faster than the threshold
- **WHEN** a source opens anonymous connections faster than the slot-protection threshold
- **THEN** the server keeps answering on accepted connections; a new connection from any client may get "too many connections", and the client retries later

### Requirement: Request limit per network token
The server SHALL limit each network token independently: by default 100 requests per second, burst up to 100 (the value
is in the server settings), and an excess gets the refusal «слишком много запросов, повторите позже» (too many requests, retry later) with a `Retry-After` retry hint (the value is in `http-api`,
"Failure headers"). A request neither waits nor is queued. Trace: CUJ8, CUJ9; ADR 0012 item 7; ADR 0068
(Q-46; D-51, accepted, OD-6).

#### Scenario: Limit exceeded
- **WHEN** a network token sends requests faster than the limit
- **THEN** the excess requests immediately get «слишком много запросов, повторите позже» with Retry-After; when requests
  fall below the limit, they are served again without reissuing the token

#### Scenario: All request kinds
- **WHEN** a network token sends read and write requests
- **THEN** every request spends the same token limit; a "no permissions" refusal spends it too

#### Scenario: A limit refusal does not change data
- **WHEN** a change request got a limit refusal
- **THEN** data is unchanged (the limit is checked before the database state and before the area 1 checks)
