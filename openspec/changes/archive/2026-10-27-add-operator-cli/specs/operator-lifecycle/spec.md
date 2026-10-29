# operator-lifecycle Specification

## Purpose
Operator commands on the server machine related to the lifecycle: the `npdb` program and its subcommands, creating an empty database, starting the server, the settings file, clean shutdown and signals, exit codes, operator messages, restore by starting on a backup, program upgrade, and the load-generation tool for verifying 10^8. Issuing and revoking tokens, backup and the channel to a running server are the capability of the `control` subtask; the server's behavior after startup is `server-runtime`. Operator-facing messages are in Russian; they are quoted below in Russian with an English gloss in parentheses.

## ADDED Requirements

### Requirement: One `npdb` program with subcommands
The operator SHALL work with the database through a single program `npdb`: the subcommand is the first argument (`init`, `serve`, subcommands that control a running server), parameters are `--name value`; `npdb --version` and `npdb --help` are available without a database. An unknown subcommand or parameter is a refusal with code 2 before any file is accessed. Trace: CUJ13, CUJ10, CUJ15; ADR 0075 item 1 [DEFAULT, D-54].

#### Scenario: Help without a database
- **WHEN** the operator runs `npdb --help` on a machine without a database
- **THEN** the list of subcommands and parameters is printed, code 0, no files are touched

#### Scenario: Unknown subcommand
- **WHEN** the operator runs `npdb frobnicate`
- **THEN** a usage error message, code 2, no file is opened

#### Scenario: No administration through the API
- **WHEN** one looks for an operation to create a database, issue a token or make a backup in HTTP API v1
- **THEN** it is not there; operator operations are only the `npdb` subcommands (X-3)

### Requirement: Database and settings directories are set explicitly
The subcommands `init` and `serve` and the `npdb-fill` program SHALL require the parameters `--db` (database directory) and `--config` (settings directory); there are no defaults or environment variables, a missing parameter is a usage error, code 2. The settings directory SHALL NOT lie inside the database directory or vice versa. Trace: CUJ13, CUJ15; ADR 0075 items 2, 3 [DEFAULT, D-55]; ADR 0028 item 9.

#### Scenario: Missing parameter
- **WHEN** the operator runs `npdb serve --db /data/db` without `--config`
- **THEN** code 2, the message names the missing parameter, `LOCK` is not taken

#### Scenario: Settings inside the database
- **WHEN** the operator runs `npdb init --db /data/db --config /data/db/cfg`
- **THEN** a refusal («настройки внутри базы вошли бы в копию» — settings inside the database would end up in the backup), code 2, nothing is created

#### Scenario: Symbolic link
- **WHEN** the settings path is a symbolic link to a directory inside the database
- **THEN** the check goes by canonical paths, the refusal is as above

### Requirement: `init` creates an empty database
The command `npdb init` SHALL create an empty database in the `--db` directory per ADR 0054 item 2 (`LOCK`, then the incomplete-creation marker `init.partial`, `data`, `dwb`, the first WAL segment, `control` last, after which the marker is removed) and print the result to standard output: the database path, the settings path, `format_version`, `db_id`, the program version. Trace: CUJ13, X-2; ADR 0015 item 1, ADR 0054 item 2, ADR 0075 items 4, 6.

#### Scenario: Creation and start
- **WHEN** the operator runs `npdb init` on a nonexistent directory, then `npdb serve`
- **THEN** creation finishes with code 0, the server starts, prints the UI address (CUJ13), search by number on an empty database gives an empty list

#### Scenario: Directory is empty
- **WHEN** `init` is run on an existing empty directory
- **THEN** the database is created in it, no marker is left in the directory

#### Scenario: Creation interrupted
- **GIVEN** `init` was interrupted before `control` was written; `LOCK`, the `init.partial` marker, `data`, `dwb`, `wal` remain in the directory
- **WHEN** the operator repeats `init`
- **THEN** the remnants are replaced, the database is created, code 0

### Requirement: Creation does not overwrite or touch foreign files
`init` SHALL refuse with code 11, changing and deleting nothing, if the database directory contains `control`, contains a database data file (`data`, `dwb`, the WAL) without the `init.partial` marker, or contains a file that is not part of the database; with code 6 if `LOCK` is held. The remnants of an interrupted `init` are only database files, with the marker and without `control`; only they are replaced. Trace: CUJ13, X-2; ADR 0015 item 2 [USER]; OD-3 Q-7; ADR 0054 items 1–2; ADR 0075 item 4.

#### Scenario: Database already exists
- **WHEN** `init` is run on a directory with a database
- **THEN** code 11, the message «база уже есть» (database already exists), the database files are unchanged (checksums compared before and after)

#### Scenario: Database opened by a server
- **WHEN** `init` is run on a directory held by a running server
- **THEN** code 6, the database is unchanged

#### Scenario: Foreign files
- **WHEN** the directory contains `notes.txt`
- **THEN** code 11, the file is in place, no database is created

#### Scenario: Data files without `control`
- **GIVEN** the directory has `data`, `dwb`, the WAL, no `control` and no `init.partial` marker (for example, `control` was deleted)
- **WHEN** the operator runs `init`
- **THEN** code 11, the message «в каталоге файлы данных без `control`: база поверх них не создаётся; восстановите базу из резервной копии» (the directory has data files without `control`: a database is not created over them; restore the database from a backup), the files are unchanged (checksums compared before and after)

#### Scenario: Remnants with a foreign file
- **GIVEN** the directory has the remnants of an interrupted `init` with the marker and, besides them, `notes.txt`
- **WHEN** the operator runs `init`
- **THEN** code 11, nothing is deleted

#### Scenario: Directory with only `LOCK`
- **WHEN** the directory has only `LOCK` and no data
- **THEN** the database is created, code 0

### Requirement: `init` prepares the settings directory without creating extras
`init` SHALL create the settings directory if it does not exist (permissions 0700), check that it can be written to before creating the database, create `settings.toml` with defaults if the file does not exist, and SHALL NOT create or change `generation`, `tokens`, `local.token` or an existing `settings.toml`. A settings directory that is not writable is a refusal with code 10 before the database is created. Trace: CUJ13, CUJ14; ADR 0028 item 9, ADR 0054 items 5, 7, ADR 0075 item 5.

#### Scenario: First run
- **WHEN** there is no settings directory and the operator runs `init`
- **THEN** the directory is created with permissions 0700, it contains `settings.toml` with defaults, no other files

#### Scenario: Settings directory of another database
- **GIVEN** the settings directory has `generation`, `tokens` and a modified `settings.toml`
- **WHEN** `init` creates a new database with this settings directory
- **THEN** all three files are unchanged

#### Scenario: Settings not writable
- **WHEN** the settings directory is read-only
- **THEN** code 10 and no database is created

### Requirement: Server does not start on a directory without a database
`npdb serve` on a directory without `control` SHALL refuse with code 5, without listeners and a UI address, without creating a database or changing the directory. The hint `npdb init` is given only if there are no data files or creation is incomplete (the `init.partial` marker); with data files and no marker there is no hint, because `init` will not touch such a directory. Trace: CUJ13; ADR 0015 item 1 [USER]; OD-3 Q-7; ADR 0075 items 4, 8; `server-runtime` ("Startup order").

#### Scenario: Empty directory
- **WHEN** the operator runs `serve` on an empty directory
- **THEN** code 5, the message «в каталоге нет базы; создайте её командой `npdb init`» (there is no database in the directory; create it with `npdb init`), the event `start_failed reason=no_base`, the port is not occupied, the directory stays empty

#### Scenario: No directory
- **WHEN** `--db` points to a nonexistent path
- **THEN** code 5, the directory is not created

#### Scenario: Data without `control`
- **WHEN** the operator runs `serve` on a directory with `data`, `dwb`, the WAL and without `control`
- **THEN** code 5, `start_failed reason=no_base data_present=true`, the message «в каталоге есть файлы данных, но нет `control`; каталог не изменён; восстановите базу из резервной копии» (the directory has data files but no `control`; the directory is unchanged; restore the database from a backup), there is no `npdb init` hint, the files are unchanged

#### Scenario: Interrupted creation
- **WHEN** the operator runs `serve` on a directory with the remnants of an interrupted `init` (the `init.partial` marker, no `control`)
- **THEN** code 5, the message «создание базы не завершено; повторите `npdb init`» (database creation is not complete; repeat `npdb init`), the files are unchanged

### Requirement: `serve` runs in the foreground
`npdb serve --db … --config …` SHALL perform the startup order of `server-runtime` in the foreground: no daemonization, no pid file; the process lives until stopped. A usage error is code 2 before `LOCK` is taken. Trace: CUJ13, X-2; ADR 0077 item 1, ADR 0074 item 1.

#### Scenario: Start and wait
- **WHEN** the operator runs `serve` on a database
- **THEN** the process does not return control, after «готов» ("ready") it accepts requests

#### Scenario: Second server
- **WHEN** a second `serve` is started on the same database
- **THEN** code 6, the message «база уже открыта другим процессом» (the database is already opened by another process), the first server is unaffected

#### Scenario: Port occupied
- **WHEN** the loopback port is occupied by another process
- **THEN** code 7, the message contains the address and `errno`, the UI address is not printed, another port is not chosen

### Requirement: One server per settings directory
`serve` SHALL take an exclusive lock on the `control.lock` file of the settings directory after checking that the database exists and hold it until the process exits; if the lock is held — start refusal with code 12, `start_failed reason=config_locked` and the message «каталог настроек занят другим процессом» (settings directory is busy with another process), the listeners and the control channel are not opened, the first server's files are untouched. Trace: CUJ13, CUJ10, X-2; ADR 0081 item 1 [OURS]; ADR 0077 item 6; `server-runtime` ("Startup order").

#### Scenario: Second server with the same settings
- **WHEN** `serve` is started on another database with a settings directory held by a running server
- **THEN** code 12, the message «каталог настроек занят другим процессом» (settings directory is busy with another process), the first server and its control channel are unaffected

#### Scenario: Lock released after a crash
- **WHEN** the server terminated abnormally and starts again
- **THEN** the lock is free (the OS releases it), startup does not refuse because of `control.lock`

### Requirement: UI address is a single line on standard output
The standard output of `serve` SHALL contain exactly one line — the UI address with an embedded token of the form `http://127.0.0.1:<port>/#<token>`; it is printed after «готов» ("ready"), once per run, and all operator messages go to standard error. Trace: CUJ13; ADR 0074 item 5, ADR 0067 item 5, ADR 0078 item 1.

#### Scenario: Channel separation
- **WHEN** the operator redirects stderr to a file and starts the server
- **THEN** the file contains no address or token, and stdout has one line with the address

#### Scenario: New run
- **WHEN** the server is restarted
- **THEN** a new address is printed; the old one does not work (CUJ13)

#### Scenario: Start refusal
- **WHEN** startup ended with a refusal
- **THEN** nothing is printed to stdout

### Requirement: Settings file `settings.toml`
Settings SHALL be set by the `settings.toml` file (TOML) in the `--config` directory; the server reads it once right after `LOCK`; relative paths are from the settings directory. The other files of the settings directory have fixed names: `local.token`, `tokens`, `generation`, `control.lock`, `control.sock` (the path of the last is the `control.socket` key). Trace: CUJ13, CUJ10; item 1.3 of the original wording; ADR 0076 items 1, 4, 7.

#### Scenario: Edit and restart
- **WHEN** the operator changes `loopback.port` and restarts the server
- **THEN** the server listens on the new port

#### Scenario: Edit without restart
- **WHEN** the operator changes the file while the server is running
- **THEN** the running server does not change its settings

#### Scenario: Relative certificate path
- **WHEN** the file has `network.certificate = "tls/cert.pem"`
- **THEN** `<settings directory>/tls/cert.pem` is read

#### Scenario: Custom channel path
- **WHEN** the file has `control.socket = "run/c.sock"`
- **THEN** the server opens the channel at `<settings directory>/run/c.sock`, and control commands look for it there; the server does not refuse with code 3

### Requirement: Control commands read two keys
Control commands (`operator-control`) SHALL read from `settings.toml` only `control.socket` and `network.enabled`; the other keys are checked by the server. Trace: CUJ10; ADR 0080 item 3 [DEFAULT, D-74].

#### Scenario: Error in a key that is not the command's
- **WHEN** the file has the typo `loopback.prot = 7000`, the operator runs `npdb token-list`
- **THEN** the command works (it reads only `control.socket` and `network.enabled`), and the server does not start on this file (code 3)

#### Scenario: File does not parse
- **WHEN** `settings.toml` does not parse, the operator runs `npdb token-list`
- **THEN** the command looks for the channel at `control.sock` in the settings directory and prints no warnings about the network listener

### Requirement: What is configurable and the defaults
The settings file SHALL allow setting: the loopback port; the network listener (enable, addresses, certificate, key); the control channel path (`control.socket`, default `control.sock`); limits of connections, sizes and timeouts; the per-token request limit; the number of threads; snapshot timeouts; backup parameters; the shutdown deadline; the request log. Defaults — ADR 0076 item 2; a missing key takes the default. Trace: CUJ7, CUJ8, X-1; ADR 0076 item 2.

#### Scenario: Empty file
- **WHEN** `settings.toml` is empty
- **THEN** loopback 7878, the network listener off, the control channel `<settings directory>/control.sock`, limits and timeouts per ADR 0071, the token limit 100 requests with a reserve of 100

#### Scenario: Network listener
- **WHEN** `network.enabled = true`, an address, a certificate and a key are set
- **THEN** a network HTTPS listener opens at the given address

#### Scenario: Network disabled
- **WHEN** `network.enabled = false` and no certificate is set
- **THEN** the server starts, the certificate is not checked (CUJ13)

### Requirement: Settings error means start refusal
Invalid settings SHALL lead to a start refusal before the listeners are opened, code 3 and a message with all errors found (up to 20): key, value, reason. Errors: the file cannot be read or parsed; an unknown key or table; a value of the wrong type; a value outside the valid range. Trace: CUJ13, X-1; ADR 0076 items 3, 5; ADR 0071 item 1; ADR 0080 item 3.

#### Scenario: Typo in a key
- **WHEN** the file has `loopback.prot = 7000`
- **THEN** code 3, the message names `loopback.prot`; the port is unchanged and there are no listeners

#### Scenario: Body cap below the valid value
- **WHEN** `limits.body_bytes = 32768`
- **THEN** code 3, the reason is a value below 262144 (`records`: a valid name must not be rejected by size)

#### Scenario: Several errors
- **WHEN** the file has two errors
- **THEN** both are named in one message

#### Scenario: No file
- **WHEN** the settings directory exists, `settings.toml` does not
- **THEN** the server starts with defaults, WARN `settings_missing` [DEFAULT, D-56]

#### Scenario: Channel path too long
- **WHEN** the control channel file path (default or `control.socket`) is longer than the OS limit
- **THEN** code 3, `key=control.socket`, the message «путь канала команд слишком длинный, задайте `control.socket`» (control channel path is too long, set `control.socket`), no listeners

#### Scenario: No settings directory
- **WHEN** `--config` points to a nonexistent directory
- **THEN** code 10, no listeners

#### Scenario: Values out of range
- **WHEN** the file has port 0 or above 65535, an empty address list or a zero timeout
- **THEN** code 3, the message names the key, the value and the reason

### Requirement: Certificate is a separate refusal
With `network.enabled = true`, a missing or unreadable certificate, an invalid PEM format, or a mismatch between the certificate and the key SHALL lead to a start refusal with code 4 (loopback is not opened either); an expired certificate and a key readable by the group or others are a warning and a start. Trace: CUJ8, CUJ13; ADR 0066 items 3, 5; ADR 0076 item 6.

#### Scenario: No key file
- **WHEN** the network listener is enabled and the key file is missing
- **THEN** code 4, the settings key and path in the message, the ports are not opened

#### Scenario: Expired certificate
- **WHEN** the certificate's validity period has expired
- **THEN** start, WARN `cert_expired` with the date

#### Scenario: Key permissions
- **WHEN** the key is readable by the group
- **THEN** start, WARN `key_permissions`

### Requirement: Token file at startup
The `tokens` file SHALL be parsed before the listeners are opened: no file — the set is empty; the file does not parse or its `format` is newer than the program — start refusal with code 3, `start_failed reason=settings key=tokens`, the file is not overwritten; the file cannot be read because of access (permissions, I/O error) — code 10, `reason=file_access`. Trace: CUJ10, CUJ13; ADR 0081 item 5; ADR 0082 item 5; ADR 0077 item 6 [DEFAULT, D-72].

#### Scenario: Corrupted token file
- **WHEN** `tokens` does not parse
- **THEN** code 3, `key=tokens`, no listeners, the file is unchanged

#### Scenario: Format newer than the program
- **WHEN** `tokens` contains a `format` greater than the known one
- **THEN** code 3, `key=tokens`, the message «формат файла токенов новее программы» (token file format is newer than the program), the file is unchanged

#### Scenario: File unreadable
- **WHEN** `tokens` belongs to another user and is inaccessible to the server (permissions 0600)
- **THEN** code 10, `reason=file_access` with the path and `errno`

#### Scenario: No file
- **WHEN** `tokens` is missing
- **THEN** the token set is empty, the server starts

### Requirement: Settings directory is not writable
If the settings directory exists but cannot be written to, the server SHALL on a planned start refuse with code 10, and on a crash start and a start on a backup follow `storage-durability` ("Unavailable counter location or randomness source"): start in "storage error" without a control channel and without the local-program token. Trace: CUJ7, CUJ13, CUJ16; ADR 0065 item 2; ADR 0077 items 6, 9 [DEFAULT, D-71].

#### Scenario: Read-only settings directory on a planned start
- **WHEN** the settings directory is not writable, the previous shutdown was clean
- **THEN** code 10, `start_failed reason=file_access` with the path and `errno`, the listeners are closed

#### Scenario: Read-only settings directory after a crash
- **WHEN** after a crash shutdown the settings directory is not writable
- **THEN** the server starts, changes are rejected ("storage error"), search works, `run_state` is unchanged, `config_dir_unwritable` in stderr; code 10 is not returned

#### Scenario: Leaving the state
- **WHEN** the settings directory is writable again and the server is restarted
- **THEN** an ordinary crash start is performed with a new generation, the control channel is open

### Requirement: Clean shutdown on signals and command
SIGTERM, SIGINT and the stop command SHALL trigger a clean shutdown per `server-runtime`; the process ends with code 0 after the clean-shutdown marker is written (if the database was not opened — without writing it); `stop_begin` carries the cause (`TERM`, `INT`, `command`). An exit with code 0 by the stop command SHALL be confirmed to the command at any startup stage. Trace: CUJ14, CUJ15; ADR 0028 item 10; ADR 0054 item 8; ADR 0077 item 2; ADR 0084 item 8.

#### Scenario: Ctrl-C
- **WHEN** the operator presses Ctrl-C on a server running in a terminal
- **THEN** `stop_begin cause=INT`, shutdown, `stop_done`, code 0; the next start — `prev_stop kind=clean`

#### Scenario: SIGTERM from the service manager
- **WHEN** the process receives SIGTERM
- **THEN** the same with `cause=TERM`

#### Scenario: Signal before the listeners are opened
- **WHEN** SIGTERM arrives before the listeners are opened
- **THEN** the process ends with code 0, the database is unchanged

#### Scenario: Stop command before the database is opened
- **WHEN** the stop command is accepted by the channel while the server has not yet opened the listeners and has not changed the database
- **THEN** the process ends with code 0 without writing the marker, "stop confirmed" is sent to the command (it prints «сервер остановлен» (server stopped), code 0)

### Requirement: SIGHUP is ignored
The server SHALL ignore SIGHUP: not stop, not re-read the settings, write a single `sighup_ignored` message; closing the terminal does not stop the server and does not make the shutdown a crash. Trace: CUJ14; Q-136; ADR 0077 item 5 [DEFAULT, D-57]; ADR 0028 item 10 is unchanged.

#### Scenario: Terminal closed
- **WHEN** the server was started from a terminal and the terminal is closed (SIGHUP)
- **THEN** the server keeps running, requests are processed, shutdown is only by SIGTERM or the command

#### Scenario: Settings are not re-read
- **WHEN** the operator changed the settings file and sent SIGHUP
- **THEN** the earlier settings remain in effect

### Requirement: Shutdown deadline of at least 60 s and its own timer
The clean shutdown deadline SHALL be the setting `shutdown.timeout_seconds`, default 60 s, not below 60. If the process has not ended within the deadline from the signal, it SHALL terminate itself without writing the marker, with code 13 and the message `stop_timeout`; such a shutdown is a crash. Trace: CUJ14, X-1; Q-108; ADR 0028 item 10, ADR 0074 item 9, ADR 0077 item 4 [DEFAULT, D-58].

#### Scenario: Shutdown fits
- **WHEN** a server on a 10^8 database receives SIGTERM under normal load
- **THEN** code 0 no later than 35 s (`server-runtime`), no `stop_timeout`

#### Scenario: Stuck flush
- **WHEN** the storage steps did not finish within 60 s
- **THEN** the process terminated itself, code 13, `stop_timeout timeout_s=60`; the next start `prev_stop kind=crash`

#### Scenario: Deadline below the valid value
- **WHEN** the settings have `shutdown.timeout_seconds = 30`
- **THEN** start refusal with code 3

### Requirement: Exit codes
The `npdb` and `npdb-fill` processes SHALL exit with a code from the table: 0 success and clean shutdown; 1 unexpected error; 2 usage; 3 settings and the token file; 4 certificate; 5 no database; 6 database opened by another process; 7 listener not bound; 8 database corrupted; 9 database newer than the program; 10 no access to files; 11 target exists; 12 settings directory busy; 13 shutdown deadline; 20–39 `control`; 40–49 `npdb-fill`. Trace: CUJ13–CUJ15; ADR 0077 item 6; Q-108.

#### Scenario: Settings directory busy
- **WHEN** `serve` is started with a settings directory held by another server
- **THEN** code 12, `start_failed reason=config_locked`

#### Scenario: Corrupted database
- **WHEN** the startup check found an inconsistency
- **THEN** code 8, the data is unchanged

#### Scenario: Newer than the program
- **WHEN** start on a database with a `format_version` greater than the known one
- **THEN** code 9

#### Scenario: Code distinguishes the reason
- **WHEN** two starts were refused: "no database" and «порт занят» ("port occupied")
- **THEN** codes 5 and 7; the reason can be determined from the code without parsing the text

#### Scenario: Adding codes
- **WHEN** a new v1 version is released
- **THEN** existing codes do not change meaning, new ones are only added

### Requirement: Operator message channel and format
Operator messages SHALL go to standard error, one line per event: `time` (UTC, RFC 3339), `level` (`INFO`, `WARN`, `ERROR`), `event` (a stable ASCII identifier), `msg` (Russian text), then the event keys as `key=value`; values with spaces are quoted with escaping. There is no permanent log file. Trace: CUJ13, CUJ14, CUJ16; Q-66; ADR 0078 items 1, 2, 7 [DEFAULT, D-59].

#### Scenario: Parsing an entry
- **WHEN** the server writes `ready`
- **THEN** a line of the form `time=2026-10-27T12:00:00Z level=INFO event=ready msg=«Сервер готов» startup_s=3` (msg means "Server ready")

#### Scenario: Redirection
- **WHEN** the operator runs `serve 2>> server.log`
- **THEN** the log is in the file, rotation is on the operator's side

#### Scenario: Special characters
- **WHEN** a value contains a newline or a quote
- **THEN** they are escaped, the entry stays a single line

### Requirement: Mandatory operator messages
The server SHALL write the events `start`, `prev_stop` (`clean`, `crash` with the number of discarded bytes and the recovery result, `copy`), `format_upgrade`, `index_rebuild` (at least once every 10 s), `listening`, `storage_state` (at startup if not in the normal state and at every transition), `ready`, `start_failed` with the reason and place, `stop_begin`, `stop_repeat`, `stop_done`, `stop_timeout`, warnings and, if enabled, `request`. Trace: CUJ13, CUJ14, CUJ16; Q-66; ADR 0078 item 3; ADR 0074 item 4.

#### Scenario: Start after SIGKILL
- **WHEN** the server starts after SIGKILL
- **THEN** `prev_stop kind=crash dropped_bytes=…` and the recovery result before `ready`

#### Scenario: Start refusal
- **WHEN** startup is rejected because of WAL corruption
- **THEN** `start_failed reason=corrupt` with the file and LSN, `exit_code=8`, before the process ends

#### Scenario: State transition
- **WHEN** the disk is full and the server went to "read-only", then space was freed and the write probe succeeded
- **THEN** two `storage_state` entries (`read_only` with `errno`, then `normal`); clients are shown nothing in advance (CUJ16)

#### Scenario: Request
- **WHEN** a request is accepted and `log.requests = true`
- **THEN** a `request` entry: method, path without the query string, status, duration, client name; without the token, body, numbers or names

### Requirement: Secrets do not appear in messages
Standard error SHALL NOT contain tokens of any kind, the `Authorization` header, the UI address, values of keys and certificates, request bodies, numbers or names; a network client's name is allowed. An address with an embedded token is only on `serve` stdout. Trace: CUJ13, CUJ10; ADR 0078 item 5; ADR 0070 item 9; ADR 0065 item 7.

#### Scenario: Searching the log for a token
- **WHEN** after work with the UI, a local program and a network client, stderr is searched for each issued token
- **THEN** none is found

### Requirement: Output failure does not crash the server
A write error to stdout or stderr (closed terminal, closed pipe) SHALL NOT terminate the process or change the server state; the message is lost, the UI address is not printed again. Trace: CUJ14; ADR 0078 item 8; ADR 0077 item 5.

#### Scenario: Closed pipe
- **WHEN** stderr is redirected to a program that has exited
- **THEN** the server keeps serving requests

### Requirement: Failure counters are visible to the operator
Once per `log.counters_seconds` (default 60 s), if the increment is nonzero, the server SHALL write `auth_counters` with the number of "unauthorized" refusals, TLS handshake drops and request-limit refusals for the period. Trace: CUJ8; Q-133; ADR 0078 item 6 [DEFAULT, D-60]; ADR 0067 item 3.

#### Scenario: Token brute-force
- **WHEN** in a minute the server rejected 500 requests with an invalid token
- **THEN** one entry `auth_counters unauthorized=500`

#### Scenario: Quiet period
- **WHEN** there were no refusals
- **THEN** there is no entry

### Requirement: Restore is a server start on a backup
Restore SHALL be performed by an ordinary `npdb serve --db <backup directory> --config <settings directory>`; there is no separate command. The server reports `prev_stop kind=copy`, changes the version generation, clears the "backup" marker; tokens and `generation` are taken from the operator's settings directory. Trace: CUJ14, CUJ15; ADR 0016 item 4, ADR 0057 items 8–9, ADR 0079 item 1.

#### Scenario: Start on a backup
- **WHEN** the operator runs `serve` on a directory created by the backup command
- **THEN** searches return the data as of the backup point, `prev_stop kind=copy`, the earlier tokens are accepted, a revoked one is rejected

#### Scenario: Corrupted database
- **GIVEN** `serve` on a database ended with code 8
- **WHEN** the operator starts the server on a backup in another directory
- **THEN** the server works on the backup data; the corrupted database directory is not overwritten

#### Scenario: Repeated start from an archive
- **WHEN** the backup directory has already been started
- **THEN** it is no longer a backup (the marker is cleared); the first start's `prev_stop` entry hinted to duplicate the directory before starting (`hint`)

#### Scenario: Files of a stopped database
- **WHEN** the server starts on a directory copied not by the command
- **THEN** the start kind is by the marker on the medium, not `copy`; the operator manual says that restore from such files is not supported (ADR 0028 item 7)

### Requirement: Program upgrade opens existing v1 databases
A new v1 program SHALL open a database and a backup with a lower `format_version` without an operator command and without confirmation, reporting `format_upgrade` (`from`, `to`) and the progress of a heavy conversion or rebuild at least once every 10 s; clients meanwhile get "server starting". Trace: CUJ11, CUJ14, CUJ15; ADR 0013 items 2–3 [USER]; ADR 0040 item 4; ADR 0058 item 4; ADR 0079 item 3.

#### Scenario: In-place upgrade
- **WHEN** the operator stopped the server cleanly, installed the new version and started it on the same database
- **THEN** the database is opened, `format_upgrade` is reported, the search order and matches did not change

#### Scenario: Backup before the upgrade
- **WHEN** the new program is started on a backup made by the earlier version
- **THEN** the backup is opened and converted, `prev_stop kind=copy`

#### Scenario: Index rebuild
- **WHEN** the index format label did not match
- **THEN** `index_rebuild` with the reason, fraction and estimate; at the end — `duration_s`; then `ready`

### Requirement: Database newer than the program is not opened
The program SHALL refuse to start on a database with a greater `format_version` or an unknown `features_incompat` with code 9 and the message «база создана версией N, эта программа знает до M» (the database was created by version N, this program knows up to M); the files are not changed; there is no program version rollback — only restore of a backup made before the upgrade. Trace: CUJ11, CUJ14; ADR 0013 item 4 [USER]; ADR 0040 item 4 (b); ADR 0079 item 3.

#### Scenario: Earlier program started after the new one
- **WHEN** the earlier program is started on a database converted by the new one
- **THEN** code 9, `start_failed reason=newer_format`, the files are unchanged

#### Scenario: Rollback
- **WHEN** the operator needs the earlier version
- **THEN** they install the earlier program and start it on the backup made before the upgrade

### Requirement: Program version
`npdb --version` SHALL print the program version and the greatest `format_version` it opens; code 0, without accessing the database. Trace: CUJ11; ADR 0079 item 3.

#### Scenario: Check before an upgrade
- **WHEN** the operator runs `npdb --version`
- **THEN** a line with the version and the greatest openable format version

### Requirement: Operator manual
The delivery SHALL include an operator manual: subcommands and codes; the contents of the settings directory and carrying over `generation`, `tokens` and the certificate together with the settings; restore only from a backup created by the command, and repeated start from an archive; clean shutdown (a deadline of at least 60 s); SIGHUP; the UI address in the service log; upgrade and rollback; a disk estimate for 10^8. Trace: CUJ10, CUJ14, CUJ15; ADR 0079 item 6; ADR 0028 items 7, 9; ADR 0044 item 10.

#### Scenario: Moving to another machine
- **WHEN** the operator moves the database
- **THEN** the manual says that the settings (`tokens`, `generation`) are carried over together with the backup, otherwise tokens are reissued, and the version guarantee between generations is probabilistic (≤ 2^-64 per event)

#### Scenario: Service
- **WHEN** the operator configures a service manager
- **THEN** the manual requires a shutdown deadline greater than `shutdown.timeout_seconds` and warns about the UI address in the service log

### Requirement: Load-generation tool is a separate program
For verifying 10^8 records there SHALL exist a separate non-product program `npdb-fill`, not part of the delivery; there is no bulk insert in `npdb` or in the HTTP API. The tool works on a stopped database: it takes `LOCK` (held — code 6) and inserts records through the same write path as the server (WAL, checkpoints, number versions). Trace: X-1, X-3; Q-53; ADR 0079 item 4 [DEFAULT, D-61]; ADR 0018.

#### Scenario: No product operation
- **WHEN** one looks for an `npdb` subcommand or an API route for bulk insert
- **THEN** there are none (X-3); `npdb-fill` is not in the delivery

#### Scenario: Database held by a server
- **WHEN** `npdb-fill` is run on the database of a running server
- **THEN** code 6, the database is unchanged

#### Scenario: Result opened by the server
- **WHEN** loading is finished and `serve` is started
- **THEN** the server opens the database without an index rebuild; searches for records inserted by the tool work

### Requirement: Load-generation tool opens the database as a server
`npdb-fill` SHALL take `control.lock` of the `--config` directory (held — code 12) and open the database the same way as the server: after a crash shutdown — a crash start with a `generation` record; a settings directory that is not writable — code 10, the database is unchanged. Trace: X-1; ADR 0079 items 4, 5 [DEFAULT, D-75]; ADR 0054 item 7; `storage-durability`.

#### Scenario: Settings directory held by a server
- **WHEN** `npdb-fill` is run on another database with a settings directory held by a running server
- **THEN** code 12, the database is unchanged

#### Scenario: Crash open with a read-only settings directory
- **WHEN** after a crash shutdown `npdb-fill` is run and the settings directory is not writable
- **THEN** code 10, the database is unchanged; the operator starts and cleanly stops `serve` or removes the cause

### Requirement: Load profile and reproducibility
`npdb-fill` SHALL accept `--db`, `--config` (mandatory), `--count` (default 10^8), `--seed` (1), `--profile p0` and `--from`; record i is a deterministic function (`seed`, i); the P0 profile is a name of 22 characters, 2.5 words per name, 1.25 records per number; every 10 s — progress, at the end — a summary. After a crash shutdown `--from` equals the number of records from recovery, otherwise code 42. Trace: X-1; Q-53; ADR 0044 item 7; ADR 0079 item 5.

#### Scenario: Identical commands
- **WHEN** loading of empty databases is run twice with identical `--count` and `--seed`
- **THEN** the databases contain identical sets of records

#### Scenario: Interruption and continuation
- **WHEN** loading is interrupted by SIGINT at record k and run with `--from k`
- **THEN** the first stop is clean (clean-shutdown marker), code 40, the summary is printed; the continuation inserts records with indexes from k, there are no repeats

#### Scenario: Continuation after a crash shutdown
- **GIVEN** loading was killed by SIGKILL, the settings directory is writable
- **WHEN** the operator runs `npdb-fill` with the same `--db`, `--config`, `--seed` and `--from` equal to the number of records found by recovery
- **THEN** a crash start is performed with a new generation, loading continues without repeats

#### Scenario: Wrong continuation
- **WHEN** `--from` is not equal to the number of records in the database after a crash shutdown
- **THEN** code 42, the message names the actual number of records, the database is unchanged

#### Scenario: No `--config`
- **WHEN** `npdb-fill` is run without `--config`
- **THEN** code 2, the database is not opened

#### Scenario: Size check
- **WHEN** 10^8 records of the P0 profile are loaded
- **THEN** the summary contains the size of `data`; comparison with 64 × 10^9 bytes is the goal of `storage-format`

#### Scenario: Invalid parameter
- **WHEN** `--profile unknown`
- **THEN** code 41, the database is unchanged
