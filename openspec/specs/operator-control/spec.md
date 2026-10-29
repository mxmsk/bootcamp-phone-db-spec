# operator-control Specification

## Purpose
Operator commands that act on a running server: the channel from the command to the server on the same machine (there is no administration over the HTTP API),
issuing, listing and revoking network tokens, backup, the behavior of commands when the server is stopped, and the presentation of outcomes to the operator. Decisions: ADR 0011,
0012, 0016, 0057, 0065, 0080–0084. The program, settings (including `control.socket`), startup, signals, the code table and the message format are in `operator-lifecycle`;
here the program is `npdb`, parameters are named (`--config` is the settings directory, mandatory for all management subcommands).
Operator messages are product CLI strings and stay in Russian (quoted in «»), with an English gloss in parentheses where needed.

## Requirements

### Requirement: Control channel: local only, not HTTP
Operator commands SHALL reach the running server over a local channel — the server's Unix socket on the server machine; there is no administration over the HTTP API or
an administration network listener. The server MUST execute token and backup commands without stopping or restarting.
Trace: CUJ10, CUJ15; ADR 0011 item 4; ADR 0012 item 4; ADR 0016 item 1; ADR 0080 items 1, 7.

#### Scenario: A command reaches the running server
- **WHEN** the server is running and the operator on its machine runs a token or backup command
- **THEN** the command is executed by the server process itself without stopping or restarting

#### Scenario: No administration via the API
- **WHEN** a client tries to issue or revoke a token or start a backup with an API request (with any token, including the local one)
- **THEN** there is no such operation in the API, the request is rejected as a request to a nonexistent path (`http-api`), and the token set and the database do not change

### Requirement: Channel access: server user and root only
The server SHALL accept a command only from a process with the same effective OS user as the server's, or from `root`, checking the peer's credentials
independently of the socket file permissions; the socket file MUST be created with 0600 permissions. Otherwise the command is rejected before execution.
Trace: CUJ10, CUJ15; ADR 0080 items 4, 5 ([DEFAULT] D-62; Q-60).

#### Scenario: Same user
- **WHEN** the command is run by the user the server runs as
- **THEN** the command is accepted

#### Scenario: root
- **WHEN** the command is run by `root` (for example, via `sudo`)
- **THEN** the command is accepted

#### Scenario: Another user
- **WHEN** the command is run by another OS user, and the OS enforces the socket file and settings directory permissions
- **THEN** the connection is not established (the OS denies access), the command is not executed, nothing is changed, the message «нет доступа к каналу сервера» (no access to the server channel) and exit code 22; the connection does not reach the server, there is no record of it

#### Scenario: Socket file permissions are not required for protection
- **WHEN** the OS ignores the socket file permissions and a connection from a foreign user's process is accepted by the server
- **THEN** the server, by the peer's credentials, closes the connection without executing the command, and the command gets code 22
- **AND** `control_denied` with the peer uid is written to the operator log

### Requirement: Socket file, location, orphaned socket
The channel file SHALL live in the settings directory (`control.sock`, the path is `control.socket`), be created after taking `control.lock` and before the listeners, and be deleted on a clean shutdown and on a start refusal. The server MUST replace a previous process's file at startup, but not touch it while a live server owns it; a path that is too long — a start refusal. Trace: CUJ10, CUJ15; ADR 0080 items 2, 3, 8; ADR 0081 items 1, 2.

#### Scenario: Clean shutdown
- **WHEN** the server is cleanly stopped
- **THEN** there is no socket file in the settings directory

#### Scenario: Orphaned socket after a crash
- **WHEN** the server terminated abnormally and the socket file remained, then the server starts again
- **THEN** the start does not refuse because of the existing file and the channel works

#### Scenario: Second server
- **WHEN** a second server is started with the same settings directory while the first is running
- **THEN** the second does not start: code 12, the message «каталог настроек занят другим процессом» (the settings directory is in use by another process); the first server's socket file is intact and commands still reach the first

#### Scenario: Path too long
- **WHEN** the socket path exceeds the OS length limit
- **THEN** the server does not start: code 3, `key=control.socket`, the message «путь канала команд слишком длинный, задайте `control.socket`» (the control channel path is too long, set `control.socket`)

### Requirement: Channel works under load and storage failures
The channel MUST handle token and shutdown commands when the HTTP slots are exhausted and the database is read-only or in the "storage error" state that arose after the channel was opened, because the tokens live outside the database. A server that started in "storage error" without a channel (the settings directory is not writable) holds the existing `control.lock` for reading, and commands answer «сервер работает без канала команд; остановка — сигналом SIGTERM» (the server is running without a control channel; stop it with SIGTERM), code 37. Backup — per ADR 0057 item 7.
Trace: CUJ10, CUJ16, CUJ15; ADR 0080 item 6; ADR 0065 item 4; ADR 0057 item 7.

#### Scenario: Revocation with exhausted slots
- **WHEN** all HTTP connections are occupied and the operator revokes a token
- **THEN** the revocation is performed and the next request with this token is rejected

#### Scenario: Read-only database
- **WHEN** the database is in the "read-only" state and the operator issues a token
- **THEN** the issue is performed

#### Scenario: Server without a channel after a crash
- **GIVEN** after a crash the settings directory is not writable, and the server started in "storage error" without a control channel
- **WHEN** the operator runs a shutdown, a copy or a token command
- **THEN** the message «сервер работает без канала команд; остановка — сигналом SIGTERM», code 37; «сервер не запущен» (server not running, code 20) is not printed, the token file is unchanged

#### Scenario: Cap on concurrent commands
- **WHEN** concurrent channel connections exceed the cap
- **THEN** the extra ones get the refusal «канал занят» (channel busy), code 36

### Requirement: Server not running
If the server is not running, token commands (issue, list, revoke) SHALL work on the token file without a server, and the changes take effect at the next start; the backup command and the shutdown command MUST refuse with «сервер не запущен» (server not running), code 20, creating nothing. Trace: CUJ10, CUJ13, CUJ15; ADR 0081 items 3–5; ADR 0057 item 1; ADR 0028 item 7 ([DEFAULT] D-64; Q-60).

#### Scenario: List without a server
- **WHEN** the server is not running and the operator requests the token list
- **THEN** the list is shown from the token file, with no secrets in it

#### Scenario: Issue without a server
- **WHEN** the server is not running and the operator issues a token
- **THEN** the token is shown once and the command reports that the change takes effect at startup
- **AND** after the server starts with the network listener enabled, the token is accepted with the stated permissions

#### Scenario: Backup without a server
- **WHEN** the server is not running and the operator starts a backup
- **THEN** no backup is created, the target is untouched, the message «сервер не запущен», code 20

#### Scenario: Token command while the server is starting or stopping
- **WHEN** the server holds the settings directory but the channel is not yet open or is already closed
- **THEN** the token command refuses «сервер запускается или останавливается, повторите позже» (the server is starting or stopping, retry later), the token file is unchanged, code 21

### Requirement: Offline write does not change file ownership
An offline command SHALL take `control.lock` for the duration of its work, require an existing settings directory (if there is none — code 10, nothing is created) and create and replace `tokens` and `control.lock` with 0600 permissions and the owner and group of the settings directory, whoever runs the command (including `root`). Trace: CUJ10, CUJ13; ADR 0081 items 3, 5, 7 [DEFAULT, D-73].

#### Scenario: root, server not running
- **GIVEN** the settings directory belongs to the user `npdb`, the server is not running
- **WHEN** `root` (via `sudo`) issues a token offline
- **THEN** `tokens` and `control.lock` are created with 0600 permissions and the owner `npdb`, not `root`
- **AND** a server started as `npdb` reads `tokens` and accepts the issued token

#### Scenario: No settings directory
- **WHEN** the server is not running, `--config` points to a nonexistent directory, the operator issues a token
- **THEN** code 10, the message «каталога настроек нет» (there is no settings directory), nothing is created

### Requirement: Command during server start and shutdown
While the server is starting (before the working mode), a token command and a backup command SHALL get the refusal «сервер запускается» (server starting), and during a clean shutdown — «сервер
останавливается» (server stopping); nothing is executed.
Trace: CUJ13, CUJ15, CUJ10; ADR 0074 items 3, 7; ADR 0083 item 2.

#### Scenario: Index rebuild at startup
- **WHEN** an index rebuild is in progress at startup and the operator issues a token
- **THEN** the issue is rejected with «сервер запускается», code 21, the token set is unchanged

#### Scenario: Shutdown
- **WHEN** the server is cleanly stopping and the operator starts a backup
- **THEN** the backup is rejected with «сервер останавливается», code 21

### Requirement: Stop command
The command `npdb stop --config <directory>` SHALL pass a clean shutdown to the server over the channel (like SIGTERM, cause `command`), wait for the process to end and, on the confirmation "shutdown confirmed", print «сервер остановлен» (server stopped) (code 0). The server confirms any exit with code 0, including at an early startup stage without writing the marker. Not running — code 20; the process ended without confirmation — code 35. Trace: CUJ13–CUJ15; ADR 0074 items 7, 8; ADR 0077 item 2; ADR 0084 item 8.

#### Scenario: Clean shutdown by command
- **WHEN** the server is running and the operator runs `npdb stop`
- **THEN** the server writes `stop_begin cause=command`, stops cleanly, the command prints «сервер остановлен», code 0
- **AND** the next start reports `prev_stop kind=clean`

#### Scenario: Server not running
- **WHEN** the server is not running and the operator runs `npdb stop`
- **THEN** the message «сервер не запущен» (server not running), code 20

#### Scenario: Shutdown during startup
- **WHEN** an index rebuild is in progress at startup and the operator runs `npdb stop`
- **THEN** the shutdown is accepted, the rebuild is cancelled, the previous index is intact, the process ends with code 0 (ADR 0074 item 8), the server confirmed the shutdown, the command prints «сервер остановлен», code 0

#### Scenario: Shutdown at an early startup stage
- **WHEN** the channel is open, the database has not been opened yet and the operator runs `npdb stop`
- **THEN** the process ends with code 0 without writing the marker, the command prints «сервер остановлен», code 0, not 35

#### Scenario: Shutdown already in progress
- **WHEN** a shutdown is in progress and the operator runs `npdb stop` again
- **THEN** the message «остановка уже выполняется» (shutdown already in progress); it does not speed up the shutdown; the command waits for the process to end and prints the outcome

#### Scenario: Forced termination by deadline
- **WHEN** the shutdown did not finish within `shutdown.timeout_seconds` and the process ended by itself without a marker (server code 13)
- **THEN** the stop command reports «остановка завершилась без подтверждения штатной, см. сообщения сервера» (the shutdown ended without confirmation of a clean one, see the server messages), code 35

#### Scenario: Command interrupted
- **WHEN** the operator interrupts `npdb stop` (Ctrl-C) while waiting
- **THEN** the wait stops, the server shutdown continues

### Requirement: Channel protocol version
The command and the server SHALL compare the channel protocol version before execution; on a mismatch the command MUST NOT be executed, and the operator is told both versions and
how to fix it (restart the server).
Trace: CUJ11, CUJ10, CUJ15; ADR 0084 item 1; ADR 0013.

#### Scenario: Old-version server after a program update
- **WHEN** the program was updated, the server has not been restarted yet and the protocol versions differ
- **THEN** the command is not executed, a message with the command and server protocol versions, code 23

### Requirement: Command output and exit codes
A command SHALL print the result (secret, backup path, list rows) to standard output, and explanations, warnings, progress and errors — to the error stream in Russian;
each refusal reason has its own exit code: 0 done, 2 invalid usage, 20–37 per the ADR 0084 item 3 table (37 — the server runs without a control channel) (the range 20–39 is given to management commands, ADR 0077 item 6).
Trace: CUJ10, CUJ15; ADR 0084 items 2, 3, 7; Q-66.

#### Scenario: Secret separated from explanations
- **WHEN** the operator issues a token and redirects standard output to a variable
- **THEN** the variable holds only the token secret on one line, the explanations are in the error stream

#### Scenario: Code on a refusal by conditions
- **WHEN** a command is rejected by a condition: the name is taken, there is no valid token, a backup is already running or there is not enough space before starting
- **THEN** the exit codes are 25, 26, 27 and 29 respectively, the state did not change

#### Scenario: Code on invalid arguments
- **WHEN** the client name is not given when issuing a token
- **THEN** exit code 2, a usage message, nothing done

### Requirement: Command wait timeouts
A command SHALL wait for the connection no longer than 5 s, for the response to a token command no longer than 10 s, and during a backup consider the server lost after 60 s without a progress
message; on expiry — code 24 and a message about an unknown outcome.
Trace: CUJ10, CUJ15; ADR 0084 item 5 ([DEFAULT] D-68).

#### Scenario: Server did not respond
- **WHEN** the server accepted a token command but did not respond within 10 s
- **THEN** code 24 and the message «исход команды неизвестен: проверьте список токенов» (the command outcome is unknown: check the token list)

### Requirement: Operator log entries about commands
The server SHALL write to stderr (the `operator-lifecycle` format) the events `token_issued`, `token_revoked`, `backup_begin`, `backup_done`, `backup_failed`, `backup_cancelled`, `control_denied`, `control_protocol_mismatch` with the peer
uid and without secrets and token hashes.
Trace: CUJ10, CUJ15; ADR 0065 item 7; ADR 0078; ADR 0084 item 4.

#### Scenario: Issue and revocation are visible to the operator
- **WHEN** the operator issued a token to the client `billing` and later revoked it
- **THEN** the server wrote `token_issued client=billing` and `token_revoked client=billing` with `peer_uid`, and there is no secret or hash in the entries

### Requirement: Token issuance
The command `npdb token-issue --config <directory> --name <name> --access <read|read_write>` SHALL create a network token with a client name and permissions and show the secret once; the token MUST be accepted from the next request without a restart; «Готово» ("Done") only after a durable write. The warning about the network listener being off is by `network.enabled` from `settings.toml`, the same in any server state. Trace: CUJ10; ADR 0012 item 4; ADR 0065 item 8; ADR 0080 item 3 (D-74); ADR 0082.

#### Scenario: Issue with the read permission
- **WHEN** the operator issues a token with the name `billing` and the permission `read` while the server is running and the network listener is enabled
- **THEN** a secret of 64 hexadecimal characters is shown once, code 0
- **AND** the service's next request over HTTPS with this token is accepted with the read permission, a write is rejected as "no permission"

#### Scenario: Issue with the write permission
- **WHEN** the operator issues a token with the permission `read_write`
- **THEN** a write request with this token is accepted

#### Scenario: Server not stopped
- **WHEN** the issue is performed
- **THEN** the server was not restarted, other clients' requests were not interrupted

#### Scenario: Network listener off
- **WHEN** the operator issues a token while the network listener is off
- **THEN** the token is issued, the command warns «токен начнёт приниматься после включения сетевого приёма» (the token will start being accepted after the network listener is enabled), code 0 (D-67)

#### Scenario: Network listener off, server not running
- **WHEN** the server is not running, `network.enabled = false`, the operator issues a token
- **THEN** the same output: the token is shown, the same network-listener warning and the message «изменение вступит в силу при запуске» (the change will take effect at startup), code 0

### Requirement: Token secret is neither stored nor repeated
The secret SHALL be shown only in the response to the issue; only the SHA-256 of the token remains on disk, there is no secret or hash in the log, list or messages, and the secret cannot be obtained again
by any command. A lost token is replaced by revocation and a new issue.
Trace: CUJ10; ADR 0012 item 4; ADR 0065 items 4, 7; ADR 0082 item 1.

#### Scenario: Requesting the secret again
- **WHEN** after the issue the operator looks for the secret in the list, the token file or the server log
- **THEN** the secret is not there; there is no command to show it

#### Scenario: Secret lost
- **WHEN** the operator needs a client's token and the secret is lost
- **THEN** the operator revokes the token and issues a new one with the same name; the issue is performed

### Requirement: Client name and permissions at issuance
A client name SHALL consist of 1–64 characters `A–Z a–z 0–9 . _ -` and be case-sensitive; the permissions are `read` or `read_write`. An invalid name or permissions, a missing
parameter — a refusal with code 2 before any changes.
Trace: CUJ10; ADR 0012 item 4; ADR 0082 item 3 ([DEFAULT] D-65).

#### Scenario: Invalid name
- **WHEN** the operator issues a token with a name of 65 characters, with a space, or empty
- **THEN** the refusal «недопустимое имя клиента» (invalid client name), code 2, the token set is unchanged

#### Scenario: Invalid permissions
- **WHEN** the operator specifies the permission `admin`
- **THEN** the refusal «права: read или read_write» (permissions: read or read_write), code 2

#### Scenario: Case is distinguished
- **WHEN** a token `Billing` is issued and then a token `billing` is issued
- **THEN** both issues are performed

### Requirement: Client name is unique among active tokens
Issuing a token with a name taken by a valid token SHALL be rejected (code 25, no changes); a revoked token's name MUST be issuable again as a new entry, and a revoked
token MUST NOT come back to life. Two concurrent issues of one name — one is performed.
Trace: CUJ10; ADR 0012 item 5; OD-3 Q-9; OD-4 item 4; ADR 0082 items 7, 9.

#### Scenario: Name taken
- **WHEN** a valid token `billing` exists and the operator issues another `billing`
- **THEN** the refusal «имя занято» (name taken) with the hint «чтобы сменить права, отзовите токен и выдайте новый» (to change permissions, revoke the token and issue a new one), code 25, the set is unchanged

#### Scenario: Name after revocation
- **WHEN** the token `billing` is revoked and a new `billing` is issued
- **THEN** the issue is performed, the new token is valid, the old one is still rejected as "not authorized"

#### Scenario: Concurrent issues
- **WHEN** two commands concurrently issue a token with one name
- **THEN** one issue is performed, the second got «имя занято»

### Requirement: Permission change only via a new token
There SHALL NOT be a "change permissions" command; to change a client's permissions, the operator revokes the token and issues a new one, the old token does not change permissions.
Trace: CUJ10; ADR 0012 item 6; ADR 0082 item 9.

#### Scenario: Raising permissions
- **WHEN** a client with a `read` token needs write permissions
- **THEN** the operator revokes the token and issues a new one with `read_write`; the old token gets "not authorized", its permissions did not change

### Requirement: Token list
The command `npdb token-list --config <directory> [--show active|all]` SHALL print the valid tokens (client name, permissions, issue time in UTC, state), in order of issue; with `--show all` — also revoked ones with the revocation
time. There MUST be no secrets or hashes in the list; "last used" is not shown.
Trace: CUJ10; ADR 0012 item 4; ADR 0065, "Consequences"; ADR 0082 item 4 ([DEFAULT] D-66; Q-40).

#### Scenario: List contents
- **WHEN** `billing` (`read`) and `etl` (`read_write`) were issued and the list is requested
- **THEN** two rows are printed with the name, permissions, issue time and the state «действует» (active), no secrets or hashes

#### Scenario: Revoked ones
- **WHEN** `billing` is revoked, the list is requested without `--show`, then with `--show all`
- **THEN** the first list does not contain `billing`, the second does, with the state «отозван» (revoked) and the revocation time

#### Scenario: Empty list
- **WHEN** there are no tokens
- **THEN** «действующих токенов нет» (there are no valid tokens), code 0

### Requirement: Token revocation by client name
The command `npdb token-revoke --config <directory> --name <name>` SHALL revoke the valid token with this name; from the next request the token is rejected as "not authorized", including on a connection opened before the
revocation, and a request being executed at the moment of revocation completes. The server does not stop.
Trace: CUJ10, CUJ7; ADR 0012 items 4, 6; ADR 0065 items 6, 7; ADR 0082 items 2, 6 (Q-40).

#### Scenario: Revocation with an open connection
- **WHEN** a service holds an open connection and the operator revokes its token
- **THEN** the command prints «готово» (done) and the next request on this connection gets "not authorized"

#### Scenario: Request at the moment of revocation
- **WHEN** a request with the token is being executed at the moment of revocation
- **THEN** this request completes, the next one is rejected

#### Scenario: Restart after revocation
- **WHEN** a token is revoked and the server is restarted
- **THEN** the token is still rejected

#### Scenario: No such token
- **WHEN** the operator revokes a name that has no valid token (never issued, already revoked)
- **THEN** the message «действующего токена с именем X нет» (there is no valid token named X), code 26, the set is unchanged

### Requirement: "Done" after a durable write; write failure leaves no change
An issue or revoke command SHALL print «готово» only after the token file is written to stable storage and the set is switched in the server; on a write error the set in the file and in the server MUST stay
the same, code 10 (file not written).
Trace: CUJ10, CUJ14; ADR 0065 items 4, 8 (D-45); ADR 0082 items 5, 6.

#### Scenario: No space for the token file
- **WHEN** writing the token file is impossible (no space or the settings directory is not writable)
- **THEN** the issue or revocation is not performed, a message with the reason, code 10, the server keeps accepting the old tokens

#### Scenario: The next request sees the change
- **WHEN** the command printed «готово» for a revocation
- **THEN** any request started after that is no longer accepted with the revoked token

#### Scenario: Response lost
- **WHEN** the response to an issue is lost (a drop after the file was written)
- **THEN** the command reports «исход неизвестен» (outcome unknown), the token is in the list; the operator revokes it and issues a new one

### Requirement: Token storage: file, permissions, outside the backup, restart
The token set SHALL be stored in the `tokens` file of the settings directory with 0600 permissions, not be part of a backup and not be changed by a backup, by a start from a backup, or by a
server restart; an unparseable file — a refusal of commands and of startup, without overwriting.
Trace: CUJ10, CUJ15; ADR 0012 item 8; ADR 0016 item 5; ADR 0065 item 4; ADR 0081 item 5; ADR 0082 items 5, 8; OD-3 Q-10; OD-4 item 9.

#### Scenario: Restart
- **WHEN** the server is restarted
- **THEN** unrevoked tokens are valid without reissue

#### Scenario: Backup and tokens
- **WHEN** a backup is made and the server is started on the backup with the same settings
- **THEN** there is no token file in the backup, and the server accepts the same tokens; a revoked one is rejected

#### Scenario: Corrupted file
- **WHEN** the token file does not parse
- **THEN** token commands refuse «файл токенов повреждён» (the token file is corrupted) (code 34) and do not change the file; the server does not start (code 3, `key=tokens`, `operator-lifecycle`, "Token file at startup")

#### Scenario: First issue without a file
- **WHEN** there is no token file (first setup)
- **THEN** the set is empty; the first issue creates the file with 0600 permissions

### Requirement: Backup command
The command `npdb backup --config <directory> --target <target>` SHALL create a backup by the running server: users and programs keep working without refusals, the backup is a consistent cut as of the moment the
command is accepted (operations acknowledged to clients before acceptance are included entirely, later ones are not). A relative target path is counted from the command's working directory.
Trace: CUJ15, X-2; ADR 0016 items 1, 2; ADR 0057 items 1, 2; ADR 0083 items 1, 2.

#### Scenario: Backup while the server is running
- **WHEN** the operator starts a backup while clients are working
- **THEN** the server is not stopped, UI and API requests are handled without refusals

#### Scenario: Backup point
- **WHEN** a client got a write acknowledgment before the command, and another write is acknowledged after the command is accepted
- **THEN** the first write is in the backup, the second is not, no partial ones

#### Scenario: Relative path
- **WHEN** the operator specifies the target `backups/b1` from the directory `/srv/ops`
- **THEN** the backup is created in `/srv/ops/backups/b1` on the server machine, if `backups` exists

### Requirement: One backup at a time
While one backup is running, a second backup command SHALL be rejected with the message «копирование уже выполняется» (a backup is already running) (code 27) and change nothing; the first backup continues.
Trace: CUJ15; ADR 0057 item 1; ADR 0083 item 2.

#### Scenario: Second backup
- **WHEN** a backup is running and the operator starts another backup command
- **THEN** the second gets «копирование уже выполняется», code 27; the first completes successfully

### Requirement: Backup target: empty or nonexistent
A backup SHALL be created only at a nonexistent path with an existing parent or in an empty directory; a non-empty target, a non-directory target and a target inside or equal to the database or
settings directory — a refusal without writing, nothing is overwritten or deleted.
Trace: CUJ15, CUJ14; ADR 0057 item 6; ADR 0015 item 2; ADR 0083 item 3 ([DEFAULT] D-70).

#### Scenario: Non-empty target
- **WHEN** the target is a directory with files
- **THEN** the refusal «цель не пуста» (the target is not empty), code 11, the directory content is unchanged

#### Scenario: Nonexistent parent
- **WHEN** the target's parent does not exist
- **THEN** the refusal «родительский каталог цели не существует» (the target's parent directory does not exist), code 28, nothing is created

#### Scenario: Target inside the database directory
- **WHEN** the target is inside the database directory
- **THEN** the refusal «цель внутри каталога базы» (the target is inside the database directory), code 28

#### Scenario: Empty directory
- **WHEN** the target is an existing empty directory
- **THEN** the backup is created in it

### Requirement: Space check before start and write errors
Before the backup starts, the command SHALL check the target's free space: at least the backup size (`data` plus the WAL up to X) plus a reserve (`backup.space_reserve_percent`, default 5 %); less — a refusal stating the needed and available amounts (code 29), nothing is created. Running out of space or a write error during the backup — the backup stops, the partial backup is deleted, the server keeps working without refusals (code 30). Trace: CUJ15, CUJ14; ADR 0057 items 6, 7; ADR 0083 item 4; ADR 0096.

#### Scenario: Little space before the start
- **WHEN** the target volume has less space than the backup size plus the reserve (default 5 %)
- **THEN** the refusal «недостаточно места: нужно N, доступно M» (not enough space: N needed, M available), code 29, the target is not created

#### Scenario: Space ran out during the backup
- **WHEN** space on the target volume ran out during the backup
- **THEN** the partial backup is deleted, the command reports the reason with `errno`, code 30, the server's clients got no refusals

### Requirement: Backup progress
The command SHALL, at the start, print the snapshot number, the snapshot time (UTC), the size and a duration estimate, and then at least once every 10 s — the share, volume, speed and an estimate of the remainder; the command stays connected until
the backup ends.
Trace: CUJ15; ADR 0083 item 5; ADR 0074 item 4 (d).

#### Scenario: Backup progress
- **WHEN** copying the database takes longer than 30 s
- **THEN** a line with the share, copied volume, speed and estimate of the remainder appears in the command's error stream at least once every 10 s

#### Scenario: Not a terminal
- **WHEN** the command's error stream is redirected to a file
- **THEN** progress is written as separate lines, not as redrawing

### Requirement: Backup cancellation
Interrupting the backup command (Ctrl-C, SIGINT, SIGTERM, closing the connection by the command itself) and a clean server shutdown SHALL cancel the backup: the partial backup is deleted, checkpoints resume, code 33. A connection drop at the command without a final server message (a server crash) is always code 24, not 33. There are no separate status and cancel commands. Trace: CUJ15; ADR 0057 item 1; ADR 0083 item 8 ([DEFAULT] D-69); ADR 0096.

#### Scenario: Ctrl-C
- **WHEN** the operator interrupts the command during a backup
- **THEN** the backup is cancelled, the target contains no partial backup, the server's clients got no refusals, a new backup command is accepted

#### Scenario: Server crash
- **WHEN** the server process crashed during a backup, and the command saw a connection break without a final message
- **THEN** code 24 ("outcome unknown"), not 33; the partial backup is not started as a backup

#### Scenario: Server shutdown
- **WHEN** the server is cleanly stopping during a backup
- **THEN** the command prints «копирование отменено: сервер останавливается» (backup cancelled: the server is stopping), code 33; the shutdown is clean and does not wait for the backup

### Requirement: Backup outcome: success
On success the command SHALL print the backup path to standard output, and to the error stream — the commit number, the snapshot time (UTC), the size, the duration and hints: tokens and settings are not part of the backup, for repeated starts from one archive
the backup directory is duplicated before the first start; code 0. Success MUST be printed only after the backup description is fully written.
Trace: CUJ15; ADR 0057 items 2, 5, 8; ADR 0083 item 6.

#### Scenario: Successful backup
- **WHEN** the backup is finished
- **THEN** the path, the commit number and snapshot time, hints are printed; code 0
- **AND** a server started on this directory returns the data as of the snapshot

#### Scenario: Failure before completion
- **WHEN** the server process crashed during the backup
- **THEN** the command does not print success (code 24), the target has no directory that starts as a backup

### Requirement: Backup outcome: source and load
If a source page fails validation, the command SHALL report «копия не создана: ошибка хранилища» (backup not created: storage error) with the file and offset (code 31), and the server SHALL move to "storage error"; if the backup is interrupted by the cap on modified pages — «копия не создана: слишком высокая нагрузка записи, повторите позже» (backup not created: write load too high, retry later) (code 32), the partial backup is deleted. A "read-only" source does not hinder the backup. Trace: CUJ15, CUJ14, CUJ16; ADR 0057 items 4, 7, 12; ADR 0083 item 7; ADR 0096.

#### Scenario: Page corruption
- **WHEN** during the backup a page fails the checksum check
- **THEN** no backup is created, the target is cleaned, a message with the file and offset, code 31

#### Scenario: Snapshot revoked
- **WHEN** the backup is interrupted by the cap on modified pages (a backup does not use a snapshot)
- **THEN** the message «копия не создана: слишком высокая нагрузка записи, повторите позже», code 32, the partial backup is deleted

#### Scenario: Read-only source
- **WHEN** the database is in the "read-only" state and the operator starts a backup
- **THEN** the backup is created
