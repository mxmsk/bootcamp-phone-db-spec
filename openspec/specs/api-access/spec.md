# api-access Specification

## Purpose
Access to the server: three kinds of tokens (UI, local programs, network services), permissions, the refusals "not authorized" /
"no permission" / "too many requests" / "host not allowed", listeners (loopback and HTTPS), the TLS certificate, loopback protection and
the per-network-token request limit. Decisions: ADR 0011, 0012, 0065–0069.

The code, status and headers of each refusal and the single check order are set by `http-api` ("Client-failure codes and statuses", "Server-failure codes and statuses", "Request check
order"); here the refusals are named in words: "not authorized" — `unauthorized`, "no permission" — `forbidden`,
«слишком много запросов, повторите позже» ("too many requests, retry later") — `rate_limited`, "host not allowed" — `host_not_allowed`. The start state
("server starting", `server_starting`) and the connection limits are set by `server-runtime`.

## Requirements

### Requirement: UI token in the link
At each start the server SHALL create a new random UI token (after taking the listeners, before the working mode) and print
a link of the form `http://127.0.0.1:<port>/#<token>` once, after switching to the working mode (`server-runtime`, "UI address is printed once and only to the operator"). The UI token is not equal to the local-program token.
Trace: CUJ13; ADR 0011 items 2–3; ADR 0065 item 1.

#### Scenario: Link after start
- **WHEN** the server has started
- **THEN** it prints a link containing the UI token in the fragment (after `#`), and the UI is available at once via the link
- **AND** the UI token differs from the local-program token and from the token of the previous start

#### Scenario: Old link after a restart
- **WHEN** the server has been restarted and switched to the working mode, and the UI makes a request with the token of the previous start
- **THEN** the request is refused as "not authorized"; the link from the output of the new start works; requests during startup
  get "server starting" (`server-runtime`)

### Requirement: UI token does not reach logs, Referer or cookies
The UI token SHALL be delivered in the link fragment, which the browser does not send to the server; the UI page takes it from the
fragment, removes it from the address bar and passes it only in the `Authorization: Bearer` header. The server MUST NOT accept the
token in the query string or in a cookie and MUST NOT write tokens to a log. Trace: CUJ13; P-1, P-2, P-5.

#### Scenario: Token in the query string
- **WHEN** a request contains a token in a query string parameter or in a cookie, and there is no `Authorization` header
- **THEN** the request is refused as "not authorized" (the parameter and the cookie are not read)

#### Scenario: Loading the UI shell without a token
- **WHEN** the browser requests the UI page via the link
- **THEN** the shell request (the UI static files) goes without a token and contains no database data; data is served only for
  API requests with the `Authorization` header

#### Scenario: Token in a log
- **WHEN** the server writes an operator message or a log entry (including about refusals)
- **THEN** the value of a token of any kind does not appear in it; for a network token the client name is given

### Requirement: Local-program token in a file
At each start the server SHALL write a new random local-program token to a file in the settings directory: the file
is created at once with "owner only" permissions (0600), replaced atomically, and the write completes before the working mode: while the file
is being written, the listeners are already open, but every request gets "server starting", and the token is not checked. The file content is exactly 64 lowercase hexadecimal ASCII characters and one LF; the client discards whitespace characters at the edges. Trace: CUJ7; ADR 0012 item 1; ADR 0065 item 2.

#### Scenario: File permissions
- **WHEN** the server has written the token file
- **THEN** only the OS user the server runs as can read the file, and at no moment of the write is the file
  accessible to other users

#### Scenario: The file is ready for the working mode
- **WHEN** the server has switched to the working mode
- **THEN** the file already contains the token of the current start; a request with it is accepted; there is no window in which tokens are checked while the
  file holds the old token

#### Scenario: File format
- **WHEN** a program reads the token file
- **THEN** it contains 64 `0-9a-f` characters and a trailing LF; the client sends only the 64 characters (without LF) in `Authorization: Bearer`; a token with whitespace characters at the edges is not sent

#### Scenario: File inaccessible to the program
- **WHEN** a local program has no permission to read the token file
- **THEN** it cannot call the API with a valid token; there is no access bypassing the file

### Requirement: Local-program token after restart
After a server restart the old local-program token SHALL be refused as "not authorized", and a retry of the request with the
token re-read from the file SHALL succeed. The file is not deleted on a clean shutdown; at the next start it is
overwritten. Trace: CUJ7.

#### Scenario: Old token, then re-reading
- **WHEN** the server has been restarted and switched to the working mode, a program sends a request with the old token, gets
  "not authorized", re-reads the file and repeats the request
- **THEN** the retry succeeds

#### Scenario: File left by a stopped server
- **WHEN** the server is stopped, the file remained from the previous start, and a program presents its token to the new server
- **THEN** "not authorized"; a stale token gives nobody access

### Requirement: Permissions and limit of UI and local-program tokens
UI and local-program tokens SHALL give full permissions (read and write) and MUST NOT fall under the per-token request limit. Trace: CUJ7, CUJ13; ADR 0012 items 2, 7 (OD-3, Q-24); ADR 0065 item 3.

#### Scenario: Write with the token from the file
- **WHEN** a local program with the token from the file adds, renames or deletes a record
- **THEN** the operation is performed (given that the area 1 checks pass)

#### Scenario: Any request rate
- **WHEN** a local program or the UI sends requests at any rate
- **THEN** there is no refusal for the per-token request limit

### Requirement: UI and file tokens are accepted only on the loopback listener
UI and local-program tokens SHALL be accepted only on the loopback listener, a network token — only on the network one (HTTPS): the network listener accepts only network tokens (including from a loopback address), the loopback listener (HTTP without TLS) — only UI and file tokens; a network token on it gets "not authorized" under any network-listener setting. The value was accepted by the owner (D-46, OD-6), refines OD-3, Q-27. Trace: CUJ7, CUJ13; ADR 0066 item 1.

#### Scenario: A file token presented not on the loopback listener
- **WHEN** a file token or a UI token is presented on the network listener (including by a client from a loopback address)
- **THEN** "not authorized" and the data is unchanged (the token is not recognized even if valid, and the response does not differ from
  the response to an unknown token)

#### Scenario: Network token on loopback with the network enabled
- **GIVEN** the network listener is enabled
- **WHEN** a valid network token is presented on the loopback listener (HTTP without TLS)
- **THEN** "not authorized", and the response does not differ from the response to an unknown token; over HTTPS on the network listener the same token is accepted

### Requirement: Network token: client name and permissions
Every network token SHALL have a client name, unique among valid tokens, and the permissions "read" or "read and
write"; a token's permissions do not change. A token is a random string of at least 256 bits of entropy. Trace: CUJ10; ADR 0012 items 4–6.

#### Scenario: Token permissions do not change
- **WHEN** the operator needs different permissions for a client
- **THEN** a new token is issued; the issued token's permissions stay the same

#### Scenario: Read token and write
- **WHEN** a "read" token calls an add, rename or delete
- **THEN** "no permission", the data is unchanged

#### Scenario: "Read and write" token
- **WHEN** a "read and write" token calls an add, rename, delete or any search
- **THEN** the operation is performed under the same rules as for a local program (CUJ9)

#### Scenario: Read token searches
- **WHEN** a "read" token performs a search by number, by word prefix of the name or by whole name
- **THEN** the search is performed

### Requirement: Network token storage
For a network token the server SHALL store only a hash (SHA-256), the client name, the permissions and the issue time, in a file in the settings
directory with 0600 permissions, separate from the data; the secret MUST NOT be present in storage or in a log, and tokens are not
part of a backup. Trace: CUJ10; ADR 0012 item 8 (OD-3, Q-10; OD-4, item 9); P-7d.

#### Scenario: The secret is absent from storage
- **WHEN** the operator or another user reads the token file
- **THEN** the token secret cannot be recovered (the file holds only the hash), and only the owner can read the file

#### Scenario: Backup and restore
- **WHEN** a backup is made, the database is restored from it and the server is started with the same settings
- **THEN** the backup contains no tokens; the valid tokens are the same; a revoked one is not valid

#### Scenario: Restart
- **WHEN** the server is restarted
- **THEN** unrevoked network tokens are valid without reissue

### Requirement: Constant-time token comparison
The server SHALL compare the presented token with the stored values in a time that does not depend on the matching part: tokens
of all kinds are compared by digest (SHA-256) with a constant-time comparison function, with no early exit on the
first mismatch. Trace: CUJ7, CUJ8; P-7b.

#### Scenario: Wrong token
- **WHEN** a token that matches the real one in its initial part is presented
- **THEN** "not authorized"; the response time does not depend on the length of the matching prefix

#### Scenario: One kind of refusal
- **WHEN** an unknown, stale or revoked token, or a token of another kind, is presented
- **THEN** the response is always the same — "not authorized"; the response does not reveal whether such a token existed

### Requirement: Network token revocation takes effect immediately
The server SHALL check the token on every request (not when the connection is opened), so a revoked token is refused as
"not authorized" from the very next request, including on a connection opened before the revocation; a request being executed at the
moment of revocation completes. Trace: CUJ10; ADR 0012 items 4, 6.

#### Scenario: Request on an open connection
- **WHEN** the operator revokes a token, and the client sends the next request on the same connection
- **THEN** "not authorized"; the server is not stopped

#### Scenario: Request at the moment of revocation
- **WHEN** a token is revoked while a request with it is already being executed
- **THEN** this request completes with an ordinary response

#### Scenario: New name after revocation
- **WHEN** a token is revoked and a new token with the same client name is issued
- **THEN** the new token is valid, and the revoked one does not come back to life

### Requirement: Access denials and check order
The server SHALL distinguish the refusals "not authorized" (a token is not presented or not recognized), "no permission" (the token is recognized, the permissions do not allow the operation), «слишком много запросов, повторите позже» ("too many requests, retry later") (the limit) and "host not allowed" (Host outside the loopback list); the codes, statuses and check order are set by `http-api`; access is checked by headers only. Trace: CUJ7, CUJ8, CUJ9; ADR 0012 item 9; ADR 0063; ADR 0065 item 6.

#### Scenario: No token
- **WHEN** an API request contains no token
- **THEN** "not authorized"

#### Scenario: No permission comes before the database state
- **WHEN** a "read" token calls an add while the database is "read-only"
- **THEN** "no permission" (access is checked before the database state)

#### Scenario: Limit comes before permissions
- **WHEN** a "read" network token over the limit calls an add
- **THEN** «слишком много запросов, повторите позже» ("too many requests, retry later"), not "no permission"

#### Scenario: A refusal does not change data
- **WHEN** a request gets any access refusal
- **THEN** the data is unchanged, and no area 1 check was performed

### Requirement: Loopback listener
At each start the server SHALL accept the UI and local programs on the loopback listener: addresses 127.0.0.1 and ::1 (::1 only if
the system has an IPv6 loopback), the port from the settings, without TLS. The listener MUST NOT accept connections from other machines.
Trace: CUJ13; ADR 0011 item 1; ADR 0066 items 1, 4.

#### Scenario: Access from the same machine
- **WHEN** a program or a browser on the same machine connects to 127.0.0.1 or ::1 on the port from the settings
- **THEN** the connection is accepted

#### Scenario: Access from another machine
- **WHEN** a client from another machine connects to the loopback listener's port
- **THEN** the connection is not accepted (the listener is not bound to external addresses)

#### Scenario: IPv6 loopback unavailable
- **WHEN** the system has no IPv6 loopback
- **THEN** the server starts on 127.0.0.1 and reports that ::1 is not used

### Requirement: Network listener over HTTPS is off by default
The network listener SHALL exist only when the network listener is enabled (an operator setting), accept only
HTTPS (TLS 1.2 and 1.3, TLS in the server itself) and only the API with network tokens, without the UI. By default the network listener is off.
Trace: CUJ8, CUJ10, CUJ13; ADR 0012 items 1–3 (OD-3, Q-8); ADR 0066 item 2.

#### Scenario: Network off by default
- **WHEN** the server is started without a network-listener setting
- **THEN** there are no connections from other machines, no HTTPS listener (including on loopback), and a network token on loopback
  is refused as "not authorized"

#### Scenario: Network token over HTTPS
- **WHEN** the network listener is enabled and a client from another machine presents a valid network token over HTTPS
- **THEN** the request is accepted with the token's permissions

#### Scenario: Request not over HTTPS
- **WHEN** a client connects to the network port in plain text (HTTP)
- **THEN** the request is not served and no token is accepted anywhere; there is no redirect to HTTPS

#### Scenario: UI on the network listener
- **WHEN** a request to the network listener asks for the UI page
- **THEN** the page is not served

### Requirement: No certificate with network enabled means no start
If the network listener is enabled but the certificate or the private key is not set, cannot be read, is malformed or they do not match
each other, the server SHALL not start at all (loopback is not started either) and report the reason to the operator. Trace: CUJ13;
ADR 0012 item 3 (OD-3, Q-8); ADR 0066 item 3.

#### Scenario: Certificate not set
- **WHEN** the network listener is enabled, and the settings have no certificate and key paths
- **THEN** the server does not start and reports that there is no certificate; no listener is open

#### Scenario: Key does not match the certificate
- **WHEN** the files are readable but the key does not belong to the certificate
- **THEN** the server does not start and reports the reason

#### Scenario: Network off, no certificate
- **WHEN** the network listener is off and there is no certificate
- **THEN** the server starts; a certificate is not required

### Requirement: TLS certificate source and replacement
The certificate and the private key SHALL be provided by the operator as PEM files (chain and key) at the paths from the settings; the server does not
issue a certificate itself. Replacement is swapping the files and a planned restart; a restart does not change network tokens. Trace: CUJ8;
ADR 0066 item 5 (Q-69; D-50, accepted, OD-6).

#### Scenario: Certificate replacement
- **WHEN** the operator replaced the certificate and key files and restarted the server
- **THEN** new connections use the new certificate, and network tokens are valid without reissue

#### Scenario: Client does not trust the certificate
- **WHEN** a client does not trust the server's certificate
- **THEN** the connection is dropped on the client side before the token is sent; the token does not reach the server

### Requirement: Loopback protection: Host check
The loopback listener SHALL reject a request whose Host header is not `127.0.0.1:<port>`, `[::1]:<port>` or
`localhost:<port>` (the port is the listener's port; on port 80 a Host without a port is also valid), with the refusal "host not allowed" before the token check. Trace: CUJ13; P-3, P-4, P-7e;
ADR 0067 item 1.

#### Scenario: Name substitution (DNS rebinding)
- **WHEN** a page from a foreign domain whose name points to 127.0.0.1 calls the loopback listener, and Host is the foreign
  name
- **THEN** "host not allowed"; the token is not checked, no data is served

#### Scenario: Allowed Host
- **WHEN** Host equals `127.0.0.1:<port>`
- **THEN** the request proceeds to the token check

#### Scenario: Port 80
- **GIVEN** the loopback listener is on port 80
- **WHEN** the browser sends Host `127.0.0.1` without a port
- **THEN** the request proceeds to the token check, and the UI works via the printed link

#### Scenario: Host different from the listener's
- **WHEN** Host is missing, contains a different port or a different name
- **THEN** the request is rejected (a missing Host is "invalid request" (`invalid_request`) under the HTTP rules; the rest is "host not allowed")

### Requirement: Loopback protection: occupied port
If the loopback listener's port (on any of the addresses) is occupied, the server SHALL not start and report it, without choosing another
port; the link with the UI token and the token file appear only after the port has been successfully taken. Trace: CUJ13; P-7; ADR 0067 item 2.

#### Scenario: Port occupied
- **WHEN** the port from the settings is occupied by another process (including another user's)
- **THEN** the server does not start, reports "port occupied", and no tokens are created or printed

#### Scenario: The token does not go to a stranger
- **WHEN** the server could not take the port
- **THEN** neither the link nor the token file of the current start exists, so a foreign process on this port does not get
  a valid token

### Requirement: Token brute-force protection
Tokens SHALL be random (at least 256 bits), a refusal MUST cost the server as much as a success, lockouts by the number of failures MUST NOT be applied (brute force gives nothing, while a lockout would let someone disable a legitimate client); the number of "not authorized" refusals SHALL be available to the operator. Trace: CUJ7, CUJ8; P-7c; ADR 0067 item 3.

#### Scenario: Many wrong tokens
- **WHEN** a source sends many requests with wrong tokens
- **THEN** each gets "not authorized"; the response is no cheaper than a success

#### Scenario: A refusal does not consume the limit
- **WHEN** a request gets "not authorized" or "host not allowed"
- **THEN** the request limit of no network token is consumed

### Requirement: A stream of invalid tokens does not displace legitimate clients
A stream of wrong tokens (every 401 refusal closes the connection, `server-runtime`, "Refusals that close the connection") SHALL be bounded only by the connection limits: a legitimate client gets no refusals caused by the stream while the source opens anonymous connections slower than the slot-protection threshold; with a faster stream new connections, including legitimate ones, MAY be closed without a response. Trace: CUJ7, CUJ8; ADR 0072 items 2, 6 (D-53; OD-7).

#### Scenario: Stream below the threshold
- **WHEN** a source opens anonymous connections with wrong tokens slower than the slot-protection threshold
- **THEN** other clients' requests are performed without refusals caused by this stream

#### Scenario: Stream faster than the threshold
- **WHEN** a source opens anonymous connections faster than the slot-protection threshold
- **THEN** the server keeps answering accepted connections; a new connection of any client may be closed without a response, and the client retries later

### Requirement: Request limit per network token
The server SHALL limit each network token independently: by default 100 requests per second, a burst of up to 100 (the value
is in the server settings), and an excess gets the refusal «слишком много запросов, повторите позже» ("too many requests, retry later") with a retry hint `Retry-After` (the value is in `http-api`,
"Failure headers"). A request does not wait and is not queued. Trace: CUJ8, CUJ9; ADR 0012 item 7; ADR 0068
(Q-46; D-51, accepted, OD-6).

#### Scenario: Limit exceeded
- **WHEN** a network token sends requests more often than the limit
- **THEN** the extra requests immediately get «слишком много запросов, повторите позже» ("too many requests, retry later") with Retry-After; when the requests
  fall below the limit, they are performed again without reissuing the token

#### Scenario: All kinds of requests
- **WHEN** a network token sends read and write requests
- **THEN** every request consumes the same token limit; a "no permission" refusal consumes it too

#### Scenario: A limit refusal does not change data
- **WHEN** a change request got a limit refusal
- **THEN** the data is unchanged (the limit is checked before the database state and before the area 1 checks)

### Requirement: One client's limit is invisible to others
The limit SHALL be counted separately for each network token; one token exceeding the limit MUST NOT cause refusals for
other tokens, local programs and the UI and MUST NOT change their latencies beyond the common targets. Trace: CUJ8; ADR 0012 item 7 (OD-3,
Q-18, Q-24).

#### Scenario: One service exceeded the limit
- **WHEN** one network service exceeded its limit
- **THEN** the requests of the other network services, local programs and the UI are performed without refusals because of another's limit

#### Scenario: Revocation and the limit
- **WHEN** a token is revoked
- **THEN** its limit counter is deleted; a new token with the same name gets its own full limit

#### Scenario: Restart
- **WHEN** the server is restarted
- **THEN** the limit counters start over

### Requirement: Network listener: Host and browser pages
The network listener SHALL not check Host against a list (clients use any names and addresses of the server, and the client verifies the server's
authenticity by the certificate) and MUST NOT answer CORS requests with consent; access is given only by the
`Authorization: Bearer` header with a network token. Trace: CUJ8; P-7e; ADR 0067 item 4.

#### Scenario: Any server name
- **WHEN** a client connects to the server by a DNS name listed in the certificate, with a valid token
- **THEN** the request is accepted regardless of the Host value

#### Scenario: Page of a foreign site
- **WHEN** a browser page of another origin tries to call the API with an Authorization header
- **THEN** the preflight request gets no permission, and the browser does not send the request
