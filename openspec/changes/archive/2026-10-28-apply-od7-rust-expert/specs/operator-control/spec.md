## MODIFIED Requirements

### Requirement: Backup target: empty or nonexistent
A backup SHALL be created only at a nonexistent path with an existing parent or in an empty directory; a nonempty target, a target that is not a directory, and a target inside or equal to the database directory or the
settings directory are a refusal without writing, nothing is overwritten or deleted.
Trace: CUJ15, CUJ14; ADR 0057 item 6; ADR 0015 item 2; ADR 0083 item 3 ([DEFAULT] D-70).

#### Scenario: Nonempty target
- **WHEN** the target is a directory with files
- **THEN** refusal «цель не пуста» ("target is not empty"), code 11, the directory contents are unchanged

#### Scenario: Nonexistent parent
- **WHEN** the target's parent does not exist
- **THEN** refusal «родительский каталог цели не существует» ("target parent directory does not exist"), code 28, nothing is created

#### Scenario: Target inside the database directory
- **WHEN** the target is inside the database directory
- **THEN** refusal «цель внутри каталога базы» ("target is inside the database directory"), code 28

#### Scenario: Empty directory
- **WHEN** the target is an existing empty directory
- **THEN** the backup is created in it

### Requirement: Space check before start and write errors
Before the backup starts, the command SHALL check the free space of the target: not less than the backup size (`data` plus the WAL up to X) plus a reserve (`backup.space_reserve_percent`, default 5%); less — a refusal stating the needed and the available amounts (code 29), nothing is created. Running out of space or a write error during the backup — the backup stops, the partial backup is deleted, the server keeps working without refusals (code 30). Trace: CUJ15, CUJ14; ADR 0057 items 6, 7; ADR 0083 item 4; ADR 0096.

#### Scenario: Not enough space before the start
- **WHEN** the target volume has less space than the backup size plus the reserve (default 5%)
- **THEN** refusal «недостаточно места: нужно N, доступно M» ("not enough space: need N, available M"), code 29, the target is not created

#### Scenario: Space ran out during the backup
- **WHEN** the target volume ran out of space during the backup
- **THEN** the partial backup is deleted, the command reports the reason with `errno`, code 30, the server's clients got no refusals

### Requirement: Backup cancellation
Interruption of the backup command (Ctrl-C, SIGINT, SIGTERM, closing the connection by the command itself) and a clean server shutdown SHALL cancel the backup: the partial backup is deleted, the checkpoints resume, code 33. A connection drop on the command without the server's final message (a server crash) is always code 24, not 33. There are no separate status and cancel commands. Trace: CUJ15; ADR 0057 item 1; ADR 0083 item 8 ([DEFAULT] D-69); ADR 0096.

#### Scenario: Ctrl-C
- **WHEN** the operator interrupts the command during the backup
- **THEN** the backup is cancelled, the target holds no partial backup, the server's clients got no refusals, a new backup command is accepted

#### Scenario: Server crash
- **WHEN** the server process crashed during the backup, and the command saw a connection break without a final message
- **THEN** code 24 ("outcome unknown"), not 33; the partial backup is not started as a backup

#### Scenario: Server shutdown
- **WHEN** the server shuts down cleanly during the backup
- **THEN** the command prints «копирование отменено: сервер останавливается» ("backup cancelled: server stopping"), code 33; the shutdown is clean and does not wait for the backup

### Requirement: Backup outcome: source and load
If a source page fails verification, the command SHALL report «копия не создана: ошибка хранилища» ("backup not created: storage error") with the file and offset (code 31), and the server SHALL switch to "storage error"; if the backup was interrupted by the modified-page cap — «копия не создана: слишком высокая нагрузка записи, повторите позже» ("backup not created: write load too high, retry later") (code 32), the partial backup is deleted. A "read-only" source does not prevent the backup. Trace: CUJ15, CUJ14, CUJ16; ADR 0057 items 4, 7, 12; ADR 0083 item 7; ADR 0096.

#### Scenario: Page corruption
- **WHEN** during the backup a page fails the checksum check
- **THEN** the backup is not created, the target is cleaned, a message with the file and offset, code 31

#### Scenario: Snapshot revoked
- **WHEN** the backup was interrupted by the modified-page cap (the backup uses no snapshot)
- **THEN** the message «копия не создана: слишком высокая нагрузка записи, повторите позже» ("backup not created: write load too high, retry later"), code 32, the partial backup is deleted

#### Scenario: Read-only source
- **WHEN** the database is in the "read-only" state and the operator starts a backup
- **THEN** the backup is created
