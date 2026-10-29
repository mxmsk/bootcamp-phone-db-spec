## MODIFIED Requirements

### Requirement: Request size cap
The server SHALL reject a request whose request line and headers exceed 16 KiB ("headers too large") and a request with a body over 32 KiB ("request too large", by `Content-Length` before reading the body, otherwise by the count of bytes read); a valid name within the body cap SHALL be accepted. Values: the 32 KiB body cap is an owner decision (OD-6, D-43; as in brief §4), the header cap is D-43; the body cap cannot be configured below 32 KiB; codes — `http-api`. Trace: CUJ1, CUJ5, CUJ7; OD-6; ADR 0064, 0071; `records`.

#### Scenario: A name with 20 000 whitespace characters at the edges
- **WHEN** a name of 200 letters with 20 000 whitespace characters U+3000 at each edge, written as `　`, is sent in a body over 32 KiB
- **THEN** a "request too large" refusal before the body is read and before parsing; the name is not parsed (OD-6, D-43)

#### Scenario: A name with edges within the body cap
- **WHEN** a name of 200 letters with 1 000 whitespace characters U+3000 at each edge, written as `　` (about 12 000 bytes of edges), is sent in a body
- **THEN** the body is within the cap, the request is accepted, and the server does not refuse on size (`records`, "Input size before normalization and request body cap")

#### Scenario: A body over the cap
- **WHEN** a request with a valid token declares `Content-Length` of 300 KiB
- **THEN** a "request too large" refusal is issued before the body is read and delivered to the client (ADR 0072 item 5)

#### Scenario: A body without a declared length
- **WHEN** a chunked body exceeded 32 KiB
- **THEN** reading stops, and a "request too large" refusal

#### Scenario: Headers over the cap
- **WHEN** the request line with headers takes 16 385 bytes
- **THEN** a "headers too large" refusal, and the request is not executed

### Requirement: Read processing deadline of 20 s
A read that has not started its response within 20 s of the end of request parsing SHALL end with a "request timeout" refusal, the work stops and the snapshot is released; a valid word search at 10^7 (up to about 1.4 s, scaled proportionally from about 14 s at 10^8) SHALL fit within the deadline. A default value (D-52, accepted by the owner, OD-6; the 20 s deadline was chosen for 10^8, revision on a prototype — (T)). Trace: CUJ5, X-1; ADR 0071 item 2, ADR 0049 item 3.

#### Scenario: A 14 s search
- **WHEN** a word search takes 14 s (a margin: about 1.4 s is expected at 10^7, OD-6)
- **THEN** it completes with a result rather than a "request timeout"

#### Scenario: A search longer than the deadline
- **WHEN** read processing gave no first byte within 20 s
- **THEN** "request timeout", the work stops and the snapshot is released; the "snapshot revoked" refusal (30 s ceiling) is not issued

### Requirement: Shutdown fits the deadline of area 4
A shutdown on a signal SHALL complete within 35 s at a database of 10^7 under normal load (up to 5 s for reads plus about 30 s for storage); the forced-termination deadline (area 4) is at least 60 s. A forced termination before the marker is written is a crash shutdown. Trace: CUJ14, X-1; ADR 0054 item 8, ADR 0074 item 9; Q-108.

#### Scenario: A shutdown under load
- **WHEN** SIGTERM is received during 100 writes per second and 10 reads
- **THEN** the process exits with code 0 within 35 s, and the marker is written

#### Scenario: The deadline expired
- **WHEN** the process was forcibly terminated before the marker was written
- **THEN** the next start is a crash start, and the data is intact (`storage-durability`)
