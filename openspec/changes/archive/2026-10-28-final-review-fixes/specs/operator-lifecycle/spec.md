## MODIFIED Requirements

### Requirement: Server does not start on a directory without a database
`npdb serve` on a directory without `control` SHALL refuse with exit code 5, with no listeners and no UI address, without creating a database and without changing the directory (except the service file `LOCK`, taken by the first startup step; a nonexistent directory is not created). The `npdb init` hint is given only if there are no data files or creation was not completed (the `init.partial` marker); with data files and no marker there is no hint, because `init` will not touch such a directory. Trace: CUJ13; ADR 0015 item 1 [USER]; OD-3 Q-7; ADR 0075 items 4, 8; `server-runtime` ("Startup order").

#### Scenario: Empty directory
- **WHEN** the operator runs `serve` on an empty directory
- **THEN** exit code 5, the message «в каталоге нет базы; создайте её командой `npdb init`» (no database in the directory; create it with `npdb init`), event `start_failed reason=no_base`, the port is not taken, the directory is left without a database (it contains at most the service file `LOCK`, with which `init` accepts it)

#### Scenario: The directory does not exist
- **WHEN** `--db` points to a nonexistent path
- **THEN** exit code 5 before the `LOCK` step, the directory is not created

#### Scenario: Data without `control`
- **WHEN** the operator runs `serve` on a directory with `data`, `dwb`, a WAL and no `control`
- **THEN** exit code 5, `start_failed reason=no_base data_present=true`, the message «в каталоге есть файлы данных, но нет `control`; каталог не изменён; восстановите базу из резервной копии» (the directory has data files but no `control`; the directory is unchanged; restore the database from a backup), no `npdb init` hint, files are unchanged

#### Scenario: Interrupted creation
- **WHEN** the operator runs `serve` on a directory with leftovers of an interrupted `init` (the `init.partial` marker, no `control`)
- **THEN** exit code 5, the message «создание базы не завершено; повторите `npdb init`» (database creation is not complete; repeat `npdb init`), files are unchanged

### Requirement: Settings directory is not writable
If the settings directory exists but cannot be written to, on a planned start the server SHALL refuse with code 10, and on a crash recovery start and a start from backup it SHALL follow `storage-durability` ("Unavailable counter location or randomness source"): start in "storage error" without a control channel and without a local-program token. Trace: CUJ7, CUJ13, CUJ16; ADR 0065 item 2; ADR 0077 items 6, 9 [DEFAULT, D-71].

#### Scenario: Read-only settings directory on a planned start
- **WHEN** the settings directory is not writable, the previous shutdown was clean
- **THEN** code 10, `start_failed reason=file_access` with the path and `errno`, listeners are closed

#### Scenario: Read-only settings directory after a crash
- **WHEN** after a crash shutdown the settings directory is not writable
- **THEN** the server starts, changes are rejected ("storage error"), search works, `run_state` is unchanged, `config_dir_unwritable` in stderr; code 10 is not returned; the server holds the existing `control.lock` for reading, operator commands answer with code 37 (`operator-control`)

#### Scenario: Leaving the state
- **WHEN** the settings directory is writable again and the server is restarted
- **THEN** a normal crash recovery start with a new generation is performed, the control channel is open
