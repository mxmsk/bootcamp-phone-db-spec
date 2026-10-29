## Purpose
Access to the server: three kinds of tokens (UI, local programs, network services), permissions, the refusals "unauthorized" /
"forbidden" / "rate limited" / "host not allowed", listeners (loopback and HTTPS), the TLS certificate, loopback protection and
the request limit per network token. Decisions: ADR 0011, 0012, 0065–0069.

The code, status and headers of each refusal and the single check order are set by `http-api` ("Client-failure codes and statuses", "Server-failure codes and statuses", "Request
check order"); here the refusals are named in prose: "unauthorized" is `unauthorized`, "forbidden" is `forbidden`,
"rate limited" (UI text «слишком много запросов, повторите позже» — too many requests, retry later) is `rate_limited`, "host not allowed" is `host_not_allowed`. The startup
state ("server starting", `server_starting`) and the connection caps are set by `server-runtime`.

## ADDED Requirements

### Requirement: UI token in the link
At every start the server SHALL create a new random UI token (after the listeners are taken, before working mode) and print
a link of the form `http://127.0.0.1:<port>/#<token>` once, after entering working mode (`server-runtime`, "UI address is
printed once and only to the operator"). The UI token is not equal to the local-program token.
Trace: CUJ13; ADR 0011 items 2–3; ADR 0065 item 1.

#### Scenario: Link after start
- **WHEN** the server has started
- **THEN** it prints a link containing the UI token in the fragment (after `#`), and the UI is available at the link immediately
- **AND** the UI token differs from the local-program token and from the token of the previous start

#### Scenario: Old link after a restart
- **WHEN** the server is restarted and has entered working mode, and the UI makes a request with the token of the previous start
- **THEN** the request is refused as "unauthorized"; the link from the new start's output works; requests during startup
  get "server starting" (`server-runtime`)

### Requirement: UI token does not reach logs, Referer or cookies
The UI token SHALL be delivered in the link fragment, which the browser does not send to the server; the UI page takes it from the
fragment, removes it from the address bar and passes it only in the `Authorization: Bearer` header. The server MUST NOT accept a
token in the query string or in a cookie and MUST NOT write tokens to a log. Trace: CUJ13; P-1, P-2, P-5.

#### Scenario: Token in the query string
- **WHEN** a request contains a token in a query-string parameter or in a cookie and has no `Authorization` header
- **THEN** the request is refused as "unauthorized" (the parameter and cookie are not read)

#### Scenario: Loading the UI shell without a token
- **WHEN** the browser requests the UI page by the link
- **THEN** the request for the shell (the UI static files) goes without a token and contains no database data; data is served only for
  API requests with the `Authorization` header

#### Scenario: Token in a log
- **WHEN** the server writes an operator message or a log (including about refusals)
- **THEN** the value of a token of any kind does not appear in it; for a network token the client name is given

### Requirement: Local-program token in a file
At every start the server SHALL write a new random local-program token to a file in the settings directory: the file
is created immediately with "owner only" permissions (0600), replaced atomically, and the write completes before working mode: while the file
is being written, the listeners are already open, but every request gets "server starting", the token is not checked.
Trace: CUJ7; ADR 0012 item 1; ADR 0065 item 2.

#### Scenario: File permissions
- **WHEN** the server has written the token file
- **THEN** only the OS user the server runs as can read the file, and at no moment of the write is the file
  available to other users

#### Scenario: File ready for working mode
- **WHEN** the server has entered working mode
- **THEN** the file already contains the token of the current start; a request with it is accepted; there is no window in which tokens are checked and the
  file holds the earlier token

#### Scenario: File inaccessible to the program
- **WHEN** a local program has no permission to read the token file
- **THEN** it cannot call the API with a valid token; there is no access bypassing the file

### Requirement: Local-program token after restart
After a server restart the earlier local-program token SHALL get the refusal "unauthorized", and a retry of the request with the
token re-read from the file SHALL succeed. The file is not deleted on clean shutdown; at the next start it is
overwritten. Trace: CUJ7.

#### Scenario: Old token, then re-read
- **WHEN** the server is restarted and has entered working mode, a program sends a request with the old token, gets
  "unauthorized", re-reads the file and repeats the request
- **THEN** the retry succeeds

#### Scenario: File from a stopped server
- **WHEN** the server is stopped, the file remained from the previous start, and a program presents its token to the new server
- **THEN** the refusal "unauthorized"; a stale token gives no one access

### Requirement: Permissions and limit of UI and local-program tokens
The UI and local-program tokens SHALL give full permissions (read and write) and MUST NOT fall under the per-token request limit. Trace: CUJ7, CUJ13; ADR 0012 items 2, 7 (OD-3, Q-24); ADR 0065 item 3.

#### Scenario: Write with the file token
- **WHEN** a local program with the token from the file adds, renames or deletes a record
- **THEN** the operation is executed (given that the area 1 checks pass)

#### Scenario: Any request rate
- **WHEN** a local program or the UI sends requests at any rate
- **THEN** there is no refusal by the per-token request limit

### Requirement: UI and file tokens are accepted only on the loopback listener
The UI and local-program tokens SHALL be accepted only on the loopback listener: the network listener accepts only network tokens, even if the client connected to it from a loopback address. This is a default value (D-46), it refines the owner decision "the file token is accepted from loopback" (OD-3, Q-27) and awaits their answer. Trace: CUJ7, CUJ13; ADR 0066 item 1.

#### Scenario: File token presented on a non-loopback listener
- **WHEN** the file token or the UI token is presented on the network listener (including by a client from a loopback address)
- **THEN** the refusal "unauthorized" and the data is unchanged (the token is not recognized even as valid, the response does not differ from
  the response to an unknown token)

### Requirement: Network token: client name and permissions
Every network token SHALL have a client name, unique among active tokens, and permissions "read" or "read and
write"; a token's permissions do not change. A token is a random string of at least 256 bits of entropy. Trace: CUJ10; ADR 0012 items 4–6.

#### Scenario: Token permissions do not change
- **WHEN** the operator needs different permissions for a client
- **THEN** a new token is issued; the issued token keeps its permissions

#### Scenario: Read token and write
- **WHEN** a "read" token calls add, rename or delete
- **THEN** the refusal "forbidden", the data is unchanged

#### Scenario: "Read and write" token
- **WHEN** a "read and write" token calls add, rename, delete or any search
- **THEN** the operation is executed by the same rules as for a local program (CUJ9)

#### Scenario: Read token searches
- **WHEN** a "read" token performs a search by number, by name words or by whole name
- **THEN** the search is executed

### Requirement: Network token storage
For a network token the server SHALL store only the hash (SHA-256), client name, permissions and issue time, in a file in the settings
directory with permissions 0600, separate from the data; the secret MUST NOT be present in storage or in a log, and tokens are not
included in the backup. Trace: CUJ10; ADR 0012 item 8 (OD-3, Q-10; OD-4, item 9); P-7d.

#### Scenario: Secret absent from storage
- **WHEN** the operator or another user reads the token file
- **THEN** the token secret cannot be recovered (the file holds only the hash), and only the owner can read the file

#### Scenario: Backup and restore
- **WHEN** a backup is made, the database is restored from it and the server is started with the same settings
- **THEN** the backup contains no tokens; the active tokens are the same; a revoked one is not valid

#### Scenario: Restart
- **WHEN** the server is restarted
- **THEN** unrevoked network tokens remain valid without reissuing

### Requirement: Constant-time token comparison
The server SHALL compare the presented token with the stored values in a time independent of the matching part: tokens
of all kinds are compared by digest (SHA-256) with a constant-time comparison function, with no early exit on the
first mismatch. Trace: CUJ7, CUJ8; P-7b.

#### Scenario: Invalid token
- **WHEN** a token is presented that matches the real one in its initial part
- **THEN** the refusal "unauthorized"; the response time does not depend on the length of the matching prefix

#### Scenario: One kind of refusal
- **WHEN** an unknown, stale, revoked token or a token of another kind is presented
- **THEN** the response is the same — "unauthorized"; one cannot tell from the response whether such a token existed

### Requirement: Network token revocation takes effect immediately
The server SHALL check the token on every request (not when the connection is opened), so that a revoked token gets the refusal
"unauthorized" from the next request, including on a connection opened before the revocation; a request being executed at the
moment of revocation completes. Trace: CUJ10; ADR 0012 items 4, 6.

#### Scenario: Request on an open connection
- **WHEN** the operator revokes a token and the client sends the next request on the earlier connection
- **THEN** the refusal "unauthorized"; the server is not stopped

#### Scenario: Request at the moment of revocation
- **WHEN** a token is revoked while a request with it is already being executed
- **THEN** this request completes with an ordinary response

#### Scenario: New name after revocation
- **WHEN** a token is revoked and a new token with the same client name is issued
- **THEN** the new token is valid, the revoked one does not come back to life

### Requirement: Access denials and check order
The server SHALL distinguish the refusals "unauthorized" (no token presented or not recognized), "forbidden" (token recognized, permissions do not allow the operation), "rate limited" (the limit) and "host not allowed" (Host outside the loopback list); the codes, statuses and check order are set by `http-api`; access is checked by headers only. Trace: CUJ7, CUJ8, CUJ9; ADR 0012 item 9; ADR 0063; ADR 0065 item 6.

#### Scenario: No token
- **WHEN** an API request contains no token
- **THEN** the refusal "unauthorized"

#### Scenario: Forbidden before the database state
- **WHEN** a "read" token calls add while the database is "read-only"
- **THEN** the refusal "forbidden" (access is checked before the database state)

#### Scenario: Limit before permissions
- **WHEN** a network "read" token over the limit calls add
- **THEN** the refusal "rate limited", not "forbidden"

#### Scenario: A refusal does not change data
- **WHEN** a request gets any access refusal
- **THEN** the data is unchanged, and no area 1 check was performed

### Requirement: Loopback listener
At every start the server SHALL accept the UI and local programs on the loopback listener: addresses 127.0.0.1 and ::1 (::1 — if
the system has IPv6 loopback), the port from the settings, without TLS. The listener MUST NOT accept connections from other machines.
Trace: CUJ13; ADR 0011 item 1; ADR 0066 items 1, 4.

#### Scenario: Access from the same machine
- **WHEN** a program or browser on the same machine connects to 127.0.0.1 or ::1 on the port from the settings
- **THEN** the connection is accepted

#### Scenario: Access from another machine
- **WHEN** a client from another machine connects to the loopback listener port
- **THEN** the connection is not accepted (the listener is not bound to external addresses)

#### Scenario: IPv6 loopback unavailable
- **WHEN** the system has no IPv6 loopback
- **THEN** the server starts on 127.0.0.1 and reports that ::1 is not used

### Requirement: Network listener over HTTPS is off by default
The network listener SHALL exist only when network access is enabled (an operator setting), accept only
HTTPS (TLS 1.2 and 1.3, TLS in the server itself) and only the API with network tokens, without the UI. By default network access is off.
Trace: CUJ8, CUJ10, CUJ13; ADR 0012 items 1–3 (OD-3, Q-8); ADR 0066 item 2.

#### Scenario: Network off by default
- **WHEN** the server is started without the network listener setting
- **THEN** there are no connections from other machines, no HTTPS listener (including on loopback), a network token on loopback
  gets the refusal "unauthorized"

#### Scenario: Network token over HTTPS
- **WHEN** network access is enabled and a client from another machine presents a valid network token over HTTPS
- **THEN** the request is accepted with the token's permissions

#### Scenario: Request not over HTTPS
- **WHEN** a client connects to the network port in plain text (HTTP)
- **THEN** the request is not served and no token is accepted anywhere; there is no redirect to HTTPS

#### Scenario: UI on the network listener
- **WHEN** a request to the network listener asks for the UI page
- **THEN** the page is not served

### Requirement: No certificate with network enabled means no start
If network access is enabled and the certificate or private key is not set, cannot be read, is of the wrong form or does not match
the other, the server SHALL not start at all (loopback does not start either) and report the reason to the operator. Trace: CUJ13;
ADR 0012 item 3 (OD-3, Q-8); ADR 0066 item 3.

#### Scenario: Certificate not set
- **WHEN** network access is enabled and the settings have no paths to the certificate and key
- **THEN** the server does not start and reports that there is no certificate; no listener is open

#### Scenario: Key does not match the certificate
- **WHEN** the files can be read but the key does not belong to the certificate
- **THEN** the server does not start and reports the reason

#### Scenario: Network off, no certificate
- **WHEN** network access is off and there is no certificate
- **THEN** the server starts; the certificate is not required

### Requirement: TLS certificate source and replacement
The certificate and private key SHALL be provided by the operator as PEM files (chain and key) at paths from the settings; the server
does not issue a certificate itself. Replacement is swapping the files and a planned restart; a restart does not change network tokens. Trace: CUJ8;
ADR 0066 item 5 (Q-69, [DEFAULT]).

#### Scenario: Certificate replacement
- **WHEN** the operator replaced the certificate and key files and restarted the server
- **THEN** new connections use the new certificate, network tokens remain valid without reissuing

#### Scenario: Client does not trust the certificate
- **WHEN** a client does not trust the server's certificate
- **THEN** the connection is dropped on the client side before the token is sent; the token does not reach the server

### Requirement: Loopback protection: Host check
The loopback listener SHALL reject a request whose Host header is not equal to `127.0.0.1:<port>`, `[::1]:<port>` or
`localhost:<port>` (the port is the listener's port) with the refusal "host not allowed" before the token check. Trace: CUJ13; P-3, P-4, P-7e;
ADR 0067 item 1.

#### Scenario: Name substitution (DNS rebinding)
- **WHEN** a page from a foreign domain whose name points to 127.0.0.1 calls the loopback listener, and Host is the foreign
  name
- **THEN** the refusal "host not allowed"; the token is not checked, data is not served

#### Scenario: Allowed Host
- **WHEN** Host equals `127.0.0.1:<port>`
- **THEN** the request proceeds to the token check

#### Scenario: Host different from the listener's
- **WHEN** Host is missing, has another port or another name
- **THEN** the request is rejected (a missing Host is "invalid request" (`invalid_request`) by the HTTP rules; the rest is "host not allowed")

### Requirement: Loopback protection: occupied port
If the loopback listener port (on any of the addresses) is occupied, the server SHALL not start and report it, without choosing another
port; the link with the UI token and the token file appear only after the port is successfully taken. Trace: CUJ13; P-7; ADR 0067 item 2.

#### Scenario: Port occupied
- **WHEN** the port from the settings is occupied by another process (including another user's)
- **THEN** the server does not start, reports «порт занят» (port occupied), tokens are not created or printed

#### Scenario: Token does not go to a stranger
- **WHEN** the server could not take the port
- **THEN** neither the link nor the token file of the current start exist, so a foreign process on this port does not get
  a valid token

### Requirement: Token brute-force protection
Tokens SHALL be random (at least 256 bits), a refusal MUST cost the server as much as a success, lockouts by the number of failures MUST NOT be applied (brute-force gives no result, while a lockout would allow disabling a legitimate client); the number of "unauthorized" refusals SHALL be available to the operator. Trace: CUJ7, CUJ8; P-7c; ADR 0067 item 3.

#### Scenario: Many invalid tokens
- **WHEN** a source sends many requests with invalid tokens
- **THEN** each gets "unauthorized"; the response is not more expensive than a success

#### Scenario: A refusal does not consume the limit
- **WHEN** a request gets "unauthorized" or "host not allowed"
- **THEN** the request limit of no network token is consumed

### Requirement: A stream of invalid tokens does not displace legitimate clients
A stream of invalid tokens (each refusal closes the connection) SHALL be limited only by the connection limits: a legitimate client gets no refusals caused by the stream while the source opens anonymous connections more slowly than the slot-protection threshold (`server-runtime`); at a faster flow new connections, including legitimate ones, MAY get "too many connections". Trace: CUJ7, CUJ8; ADR 0072 items 2, 6 (D-53).

#### Scenario: Flow below the threshold
- **WHEN** a source opens anonymous connections with invalid tokens more slowly than the slot-protection threshold
- **THEN** other clients' requests are executed without refusals caused by this stream

#### Scenario: Flow above the threshold
- **WHEN** a source opens anonymous connections faster than the slot-protection threshold
- **THEN** the server keeps answering accepted connections; a new connection of any client may get "too many connections", and the client retries later

### Requirement: Request limit per network token
The server SHALL limit each network token independently: by default 100 requests per second, a burst up to 100 (the value
is in the server settings), and an excess gets the refusal "rate limited" with the retry hint `Retry-After` (the value is in `http-api`,
"Failure headers"). A request does not wait and is not queued. Trace: CUJ8, CUJ9; ADR 0012 item 7; ADR 0068
(Q-46, [DEFAULT]).

#### Scenario: Limit exceeded
- **WHEN** a network token sends requests more often than the limit
- **THEN** the excess requests immediately get "rate limited" with Retry-After; when the requests
  fall below the limit, they are executed again without reissuing the token

#### Scenario: All kinds of requests
- **WHEN** a network token sends read and write requests
- **THEN** each request consumes the same token limit; a "forbidden" refusal consumes it too

#### Scenario: A limit refusal does not change data
- **WHEN** a change request got a limit refusal
- **THEN** the data is unchanged (the limit is checked before the database state and before the area 1 checks)

### Requirement: One client's limit is invisible to others
The limit SHALL be counted separately for each network token; exceeding the limit by one token MUST NOT cause refusals for
other tokens, local programs and the UI and not change their latencies beyond the general targets. Trace: CUJ8; ADR 0012 item 7 (OD-3,
Q-18, Q-24).

#### Scenario: One service exceeded the limit
- **WHEN** one network service exceeded its limit
- **THEN** requests of other network services, local programs and the UI are executed without refusals caused by another's limit

#### Scenario: Revocation and the limit
- **WHEN** a token is revoked
- **THEN** its limit counter is deleted; a new token with the same name gets its own full limit

#### Scenario: Restart
- **WHEN** the server is restarted
- **THEN** the limit counters start anew

### Requirement: Network listener: Host and browser pages
The network listener SHALL not check Host against a list (clients use any names and addresses of the server, the authenticity of
the server is checked by the client via the certificate) and MUST NOT answer CORS requests with consent; access is given only by the header
`Authorization: Bearer` with a network token. Trace: CUJ8; P-7e; ADR 0067 item 4.

#### Scenario: Any server name
- **WHEN** a client connects to the server by the DNS name listed in the certificate, with a valid token
- **THEN** the request is accepted regardless of the Host value

#### Scenario: Page of a foreign site
- **WHEN** a browser page of another origin tries to call the API with an Authorization header
- **THEN** the preflight request does not get permission, and the browser does not send the request
