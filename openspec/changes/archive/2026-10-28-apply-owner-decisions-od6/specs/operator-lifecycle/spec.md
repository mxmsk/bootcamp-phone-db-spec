## MODIFIED Requirements

### Requirement: Shutdown deadline of at least 60 s and its own timer
The clean shutdown deadline SHALL be the setting `shutdown.timeout_seconds`, default 60 s, not below 60. If the process has not exited within the deadline from the signal, it SHALL terminate itself without writing the marker, with code 13 and the message `stop_timeout`; such a shutdown is a crash shutdown. Trace: CUJ14, X-1; Q-108; ADR 0028 item 10, ADR 0074 item 9, ADR 0077 item 4 [DEFAULT, D-58].

#### Scenario: The shutdown fits
- **WHEN** a server on a database of 10^7 receives SIGTERM under normal load
- **THEN** code 0 within 35 s (`server-runtime`), no `stop_timeout`

#### Scenario: A hung flush
- **WHEN** the storage steps did not complete within 60 s
- **THEN** the process terminated itself, code 13, `stop_timeout timeout_s=60`; the next start is `prev_stop kind=crash`

#### Scenario: A deadline below the allowed
- **WHEN** the settings have `shutdown.timeout_seconds = 30`
- **THEN** start refusal with code 3

### Requirement: Load-generation tool is a separate program
For checking 10^7 records, a separate non-product program `npdb-fill` SHALL exist, not included in the distribution; there is no bulk insert in `npdb` or in the HTTP API. The tool works on a stopped database: it takes `LOCK` (occupied — code 6) and inserts records by the same write path as the server (WAL, checkpoints, number versions). Trace: X-1, X-3; Q-53; ADR 0079 item 4 [DEFAULT, D-61]; ADR 0018; OD-6.

#### Scenario: No product operation
- **WHEN** a `npdb` subcommand or an API route for bulk insert is sought
- **THEN** there is none (X-3); `npdb-fill` is absent from the distribution

#### Scenario: The database is occupied by a server
- **WHEN** `npdb-fill` is started on the database of a running server
- **THEN** code 6, and the database is not changed

#### Scenario: The result is opened by the server
- **WHEN** filling has completed and `serve` is started
- **THEN** the server opens the database without an index rebuild; searches over the records inserted by the tool work

### Requirement: Load profile and reproducibility
`npdb-fill` SHALL accept `--db`, `--config` (required), `--count` (default 10^7), `--seed` (1), `--profile p0` and `--from`; record i is a deterministic function of (`seed`, i); profile P0 — a name of 22 characters, 2.5 words per name, 1.25 records per number; every 10 s — progress, at the end — a summary. After a crash termination, `--from` equals the number of records from recovery, otherwise code 42. Trace: X-1; Q-53; ADR 0044 item 7; ADR 0079 item 5; OD-6.

#### Scenario: Identical commands
- **WHEN** filling of empty databases is run twice with the same `--count` and `--seed`
- **THEN** the databases contain identical sets of records

#### Scenario: Interruption and continuation
- **WHEN** filling is interrupted by SIGINT at record k and started with `--from k`
- **THEN** the first stop is clean (clean-shutdown marker), code 40, the summary is printed; the continuation inserts records with indexes from k, with no repeats

#### Scenario: Continuation after a crash termination
- **GIVEN** filling was killed by SIGKILL, the settings directory is writable
- **WHEN** the operator starts `npdb-fill` with the same `--db`, `--config`, `--seed` and `--from` equal to the number of records found by recovery
- **THEN** a crash recovery start with a new generation is performed, and filling continues with no repeats

#### Scenario: An incorrect continuation
- **WHEN** `--from` does not equal the number of records in the database after a crash termination
- **THEN** code 42, the message names the actual number of records, and the database is not changed

#### Scenario: No `--config`
- **WHEN** `npdb-fill` is started without `--config`
- **THEN** code 2, and the database is not opened

#### Scenario: A size check
- **WHEN** 10^7 records of profile P0 have been filled
- **THEN** the summary contains the size of `data`; comparison with 8 × 10^9 bytes is a `storage-format` target

#### Scenario: An invalid parameter
- **WHEN** `--profile unknown`
- **THEN** code 41, and the database is not changed

### Requirement: Settings error means start refusal
Invalid settings SHALL lead to start refusal before the listeners are opened, code 3 and a message with all errors found (up to 20): key, value, reason. Errors: the file cannot be read or parsed; an unknown key or table; a value of the wrong type; a value outside the allowed range. Trace: CUJ13, X-1; ADR 0076 items 3, 5; ADR 0071 item 1; ADR 0080 item 3.

#### Scenario: A typo in a key
- **WHEN** the file has `loopback.prot = 7000`
- **THEN** code 3, the message names `loopback.prot`; the port is not changed and there are no listeners

#### Scenario: A body cap below the allowed
- **WHEN** `limits.body_bytes = 32767`
- **THEN** code 3, the reason is a value below 32768 (the lower bound equals the default, OD-6, D-43)

#### Scenario: A body cap at the lower bound
- **WHEN** `limits.body_bytes = 32768`
- **THEN** the value is accepted

#### Scenario: Several errors
- **WHEN** the file has two errors
- **THEN** both are named in one message

#### Scenario: No file
- **WHEN** the settings directory exists but `settings.toml` does not
- **THEN** the server starts on defaults, WARN `settings_missing` [DEFAULT, D-56]

#### Scenario: A control channel path that is too long
- **WHEN** the control channel socket file path (the default or `control.socket`) is longer than the OS limit
- **THEN** code 3, `key=control.socket`, the message «путь канала команд слишком длинный, задайте `control.socket`» (the control channel path is too long, set `control.socket`), no listeners

#### Scenario: No settings directory
- **WHEN** `--config` points to a nonexistent directory
- **THEN** code 10, no listeners

#### Scenario: Values outside the allowed range
- **WHEN** the file has a port of 0 or above 65535, an empty address list or a zero deadline
- **THEN** code 3, the message names the key, the value and the reason

### Requirement: Operator manual
The distribution SHALL include an operator manual: subcommands and codes; the contents of the settings directory and carrying `generation`, `tokens`, the certificate along with the settings; restore only from a backup made by the command, and a repeated start from an archive; clean shutdown (a deadline of at least 60 s); SIGHUP; the UI address in the service log; upgrade and rollback; a disk estimate for 10^7 (up to 1.8 × the file size plus a backup; at 8 GB about 22 GB). Trace: CUJ10, CUJ14, CUJ15; OD-6; ADR 0079 item 6; ADR 0028 items 7, 9; ADR 0044 item 10.

#### Scenario: Moving to another machine
- **WHEN** the operator moves the database
- **THEN** the manual says the settings (`tokens`, `generation`) are carried along with the backup, otherwise the tokens are reissued, and the version guarantee between generations is probabilistic (≤ 2^-64 per event)

#### Scenario: The service
- **WHEN** the operator configures a service manager
- **THEN** the manual requires a stop deadline greater than `shutdown.timeout_seconds` and warns about the UI address in the service log
