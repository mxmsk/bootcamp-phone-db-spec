## MODIFIED Requirements

### Requirement: Startup order
The server SHALL: take `LOCK`; read the settings and the certificate; check that the database exists; take the settings-directory lock (`control.lock`), parse the token file and open the operator control channel; open the listeners; perform startup checks, recovery, index rebuild and a new generation; create the UI token and write the local-program token file; enter working mode. A refusal before the listeners are opened leaves no listeners; after — the listeners are closed; on a refusal after the control channel is opened, it is closed and its file is removed; the operator gets the reason and a nonzero code. A settings directory that is not writable prevents taking `control.lock` and opening the channel: the decision for such a start is in `operator-lifecycle`, "Settings directory is not writable". Trace: CUJ13, CUJ14, X-2; ADR 0074 item 1, ADR 0015, ADR 0054; ADR 0077 item 1; ADR 0080 item 2; ADR 0081 item 1.

#### Scenario: Directory without a database
- **WHEN** the server starts on a directory without `control`
- **THEN** refusal «в каталоге нет базы» (there is no database in the directory), no listeners, the control channel is not open, `control.lock` is not taken, the UI address is not printed

#### Scenario: Certificate missing
- **WHEN** the network listener is enabled and there is no certificate
- **THEN** the server does not start, the operator gets the reason

#### Scenario: Corrupted database
- **WHEN** the startup check found an inconsistency
- **THEN** the listeners are closed, the operator gets the reason, the file, the offset or LSN, the process exits with a nonzero code, the data is not modified

#### Scenario: Control channel before the listeners
- **WHEN** an index rebuild is in progress at startup
- **THEN** the operator control channel is open from the moment the server took `control.lock` and before the listeners are opened; the HTTP listeners answer "server starting"

#### Scenario: Settings directory busy
- **WHEN** `control.lock` is held by another server
- **THEN** refusal «каталог настроек занят другим процессом» (settings directory is busy with another process), no listeners or control channel, the earlier server is unaffected

#### Scenario: Refusal after the channel is opened
- **WHEN** the startup check refused after the control channel was opened
- **THEN** the listeners and the channel are closed, the channel file is removed, the operator gets the reason and a nonzero code
