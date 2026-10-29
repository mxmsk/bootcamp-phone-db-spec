## Why

Area 4 per `docs/process.md` ("Administration") gives the operator the only way to work with the database: create it, start and stop the server, issue and revoke network tokens, make a backup and restore from it, upgrade the program, and verify the 10^8 scale. Without it, CUJ10, CUJ13, CUJ14 and CUJ15 cannot be completed, and X-1 cannot be verified at the target volume (there are no operator operations in the HTTP API: ADR 0011 item 4, X-3). The decisions for the area are recorded in ADR 0075–0084; values not set by the brief or the owner are put to the owner as [DEFAULT] (D-54…D-75).

## What Changes

- The `npdb` program with subcommands (`init`, `serve`, `stop`, `token-issue`, `token-list`, `token-revoke`, `backup`, `--version`, `--help`), explicit `--db` and `--config`, creation of an empty database that never touches a directory containing data files and recognizes the remnants of an interrupted `init` only by an explicit marker, foreground operation,
  the `settings.toml` settings file, signals, the shutdown deadline, a single table of exit codes, the operator message channel and format, restore by starting on a backup, program upgrade, the operator manual, and the `npdb-fill` load-generation tool (ADR 0075–0079).
- The control channel to a running server is a Unix socket with a peer-user check, located in the settings directory; locking of the settings directory and token commands working while the server is stopped; token issuance, listing and revocation; backup (target, space, progress, cancellation, outcomes); the protocol, output streams and command codes (ADR 0080–0084).
- Fix pass after the Opus review (R-1…R-10): the `control.socket` key and the `control.lock`, `control.sock` files in `operator-lifecycle`; startup order with the channel (a MODIFIED requirement of `server-runtime`); start-refusal codes caused by the settings directory; the `init.partial` marker and the owner-decision rule Q-7;
  alignment with the accepted `storage-durability` for a read-only settings directory; stop confirmation on any exit with code 0; a verifiable foreign-user scenario; `root` as the file owner for an offline write; the keys that a command reads; `--config` for `npdb-fill`. Details are in `design.md`, "Fix pass after the Opus review".

## Capabilities

### New Capabilities

- `operator-lifecycle`: the program and subcommands, directories, database creation, startup and locking of the settings directory, the settings file and errors, tokens and the settings directory at startup, signals and shutdown, exit codes, operator messages, restore, upgrade, the manual, `npdb-fill`.
- `operator-control`: the control channel and access to it, the socket file, operation under load and storage failures, commands while the server is stopped and during start and shutdown, the stop command, the protocol and command codes, tokens (issuance, listing, revocation, storage), backup.

### Modified Capabilities

- `server-runtime`: the "Startup order" requirement — after checking that the database exists, the server takes the settings-directory lock, parses the token file and opens the control channel, and only then opens the listeners. The other requirements of areas 1–3 and `user-journeys` do not change.

## Impact

- ADR 0075–0084 (new, 10); they rely on ADR 0011–0013, 0015, 0016, 0028, 0054, 0057, 0065, 0074; ADR 0074 item 1 is extended with a reference to the channel step; the index is `docs/adr/README.md`.
- Requirements for area 5 and for the build are collected in `design.md`, "Requirements for other areas".
- Documents: `facts.md` (F-109…F-115), `docs/open-questions.md` (8 closed, 1 narrowed, 16 new: Q-142…Q-157, [DEFAULT] 22: D-54…D-75), `docs/adr/README.md`.
- No code: the change defines requirements, not implementation.
