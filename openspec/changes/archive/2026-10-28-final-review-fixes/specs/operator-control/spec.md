## MODIFIED Requirements

### Requirement: Channel works under load and storage failures
The channel MUST process token and stop commands when HTTP slots are exhausted and the database is read-only or in the "storage error" state that arose after the channel was opened, because tokens live outside the database. A server that started in "storage error" without a channel (the settings directory is not writable) holds the existing `control.lock` for reading, and commands answer «сервер работает без канала команд; остановка — сигналом SIGTERM» (the server is running without a control channel; stop it with SIGTERM), code 37. Backup — per ADR 0057 item 7.
Trace: CUJ10, CUJ16, CUJ15; ADR 0080 item 6; ADR 0065 item 4; ADR 0057 item 7.

#### Scenario: Revocation with exhausted slots
- **WHEN** all HTTP connections are busy and the operator revokes a token
- **THEN** the revocation is done and the next request with that token is rejected

#### Scenario: Read-only database
- **WHEN** the database is in the "read-only" state and the operator issues a token
- **THEN** the issuance is done

#### Scenario: Server without a channel after a crash
- **GIVEN** after a crash the settings directory is not writable, the server started in "storage error" without a control channel
- **WHEN** the operator runs stop, backup or a token command
- **THEN** the message «сервер работает без канала команд; остановка — сигналом SIGTERM», code 37; «сервер не запущен» ("server not running") (code 20) is not printed, the token file is unchanged

#### Scenario: Concurrent command cap
- **WHEN** concurrent channel connections exceed the cap
- **THEN** the extra ones get the refusal «канал занят» (channel busy), code 36

### Requirement: Command output and exit codes
A command SHALL print its result (secret, backup path, list rows) to standard output, and explanations, warnings, progress and errors to the error stream, in Russian;
each refusal reason has its own exit code: 0 done, 2 incorrect usage, 20–37 per the ADR 0084 item 3 table (37 — server running without a control channel) (the range 20–39 is given to control commands, ADR 0077 item 6).
Trace: CUJ10, CUJ15; ADR 0084 items 2, 3, 7; Q-66.

#### Scenario: Secret separated from explanations
- **WHEN** the operator issues a token and redirects standard output into a variable
- **THEN** the variable holds only the token secret on one line, explanations go to the error stream

#### Scenario: Code on a conditional refusal
- **WHEN** a command is rejected by a condition: the name is taken, there is no active token, a backup is already running or there is not enough space before the start
- **THEN** exit codes 25, 26, 27 and 29 respectively, the state did not change

#### Scenario: Code on invalid arguments
- **WHEN** the client name is not given when issuing a token
- **THEN** exit code 2, a usage message, nothing done

### Requirement: Backup cancellation
Interrupting the backup command (Ctrl-C, SIGINT, SIGTERM, the command itself closing the connection) and a clean server shutdown SHALL cancel the backup: the partial copy is deleted, the snapshot is released, code 33. A connection drop on the command without the server's final message (a server crash) is always code 24, not 33.
There are no separate status and cancel commands.
Trace: CUJ15; ADR 0057 item 1; ADR 0083 item 8 ([DEFAULT] D-69).

#### Scenario: Ctrl-C
- **WHEN** the operator interrupts the command during a backup
- **THEN** the backup is cancelled, the target contains no partial copy, server clients got no refusals, a new backup command is accepted

#### Scenario: Server crash
- **WHEN** the server process crashed during a backup, and the command saw a connection break without a final message
- **THEN** code 24 ("outcome unknown"), not 33; the partial copy is not started as a backup

#### Scenario: Server shutdown
- **WHEN** the server shuts down cleanly during a backup
- **THEN** the command prints «копирование отменено: сервер останавливается» (backup cancelled: the server is stopping), code 33; the shutdown is clean and does not wait for the backup
