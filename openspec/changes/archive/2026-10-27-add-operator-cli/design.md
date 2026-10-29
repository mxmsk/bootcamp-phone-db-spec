## Scope of the area

Area 4 defines everything the operator does on the server machine from the command line: the `npdb` program and its subcommands, creating an empty database, starting and stopping the server, the settings file, signals, exit codes, operator messages, the control channel to a running server, tokens, backup, restore by starting on a backup,
program upgrade, and the load-generation tool for verifying 10^8. Data semantics (area 1), storage (area 2), the API, access and the server process (area 3) are accepted; they do not change, except for one `server-runtime` requirement ("Startup order", a MODIFIED delta). There is no administration through the HTTP API (ADR 0011 item 4, X-3);
the look of the UI is area 5. The requirements for area 4 accumulated by areas 2 and 3 are met (`openspec/changes/archive/2026-10-27-add-storage-engine/design.md`, `.../2026-10-27-add-server-api/design.md`, "Requirements for other areas"): database creation without overwriting, the backup command, files and permissions of the settings directory, server settings, channels and codes,
a shutdown deadline of at least 60 s, SIGHUP, the stop command, failure counters, the composition of the token list.

## Map of capabilities, ADRs and CUJs

| Capability | ADR | CUJs and requirements |
|---|---|---|
| `operator-lifecycle` (33 requirements, 105 scenarios) | 0075–0079 | CUJ7, CUJ8, CUJ10, CUJ11, CUJ13–CUJ16; X-1, X-2, X-3 |
| `operator-control` (29, 81) | 0080–0084 | CUJ7, CUJ10, CUJ11, CUJ13–CUJ16; X-2, X-5 |
| `server-runtime` (MODIFIED: 1 requirement, 6 scenarios) | 0077 item 1, 0080 item 2, 0081 item 1 | CUJ13, CUJ14; X-2 |

In total 62 added requirements and 186 scenarios in two delta specs and one modified requirement of an accepted capability; ADR 0075–0084 are 10 decisions. Accepted [OURS] — 0080 (items 1–4, 6–8), 0081 (items 1–3, 5, 6 except refusal codes), 0082 (items 1, 2, 5–8, item 9 except the warning), 0083 (items 1, 2, 4–7, 9), 0084 (items 1–4, 6–8); the other ADRs (0075–0079) are Proposed,
since they contain [DEFAULT]; the [OURS] items in them (0075 items 3–8, 0076 items 1, 2, 4–8, 0077 items 1–3, 6, 7, 8, 0078 items 2–5, 7–9, 0079 items 1–3, 5–7) are decisions of the area. This area changes none of the owner decisions ([USER]); the "never overwrite" rule (ADR 0015 item 2, OD-3 Q-7) is carried out without narrowing.

Boundaries between capabilities. `operator-lifecycle` is the `serve` process and everything visible at startup and shutdown: the program, directories, `init`, the settings file and its errors, locking of the settings directory, the token file at startup, signals, exit codes (a single table for all processes), the format and list of server messages, restore, upgrade, `npdb-fill`;
`operator-control` is the commands that act on a running or stopped server: the channel and access, tokens, backup, stop by command, the protocol, output streams and codes 20–36. The single source of codes is `operator-lifecycle`, "Exit codes"; the range 20–39 is given to `operator-control` (ADR 0084 item 3). The single source of the settings-directory contents and keys is `operator-lifecycle`, "Settings file `settings.toml`";
the startup order is `server-runtime`, "Startup order" (MODIFIED).

Dependence on owner decisions and the [DEFAULT] package: the values are collected in `docs/open-questions.md`, "Area 4: defaults" (D-54…D-75).

## Subtask reconciliation

| Seam | Single decision | Where |
|---|---|---|
| The `control.socket` key, the `control.lock`, `control.sock` files | the key and files are listed in `operator-lifecycle` and ADR 0076; `operator-control` refers to them | `operator-lifecycle` ("Settings file `settings.toml`", "What is configurable and the defaults"), ADR 0076 items 2, 7 |
| Startup order and the channel | `LOCK` → settings → database present → `control.lock`, `tokens`, channel → listeners → checks | `server-runtime` (MODIFIED), ADR 0077 item 1, 0080 item 2, 0081 item 1 |
| Start-refusal codes caused by the settings directory | `control.lock` busy — 12; `tokens` unparsable or newer — 3 (`key=tokens`); unreadable — 10; long socket path — 3 | `operator-lifecycle` ("Exit codes", "Token file at startup"), ADR 0077 item 6 |
| Stop confirmation | the server confirms any exit with code 0, including before the database is opened; 35 — only an exit without confirmation | `operator-control` ("Stop command"), `operator-lifecycle`, ADR 0084 item 8 |
| Keys that a command reads | `control.socket` and `network.enabled`; the server checks the rest | `operator-lifecycle`, `operator-control`, ADR 0080 item 3 |
| Settings directory not writable | planned start — code 10; crash start and start on a backup — per `storage-durability` | `operator-lifecycle`, ADR 0077 item 9 |

## Fix pass after the Opus review

The Opus review of area 4 (5 BLOCKER, 5 contradictions) was resolved in one fix pass; there is no repeat review. The owner decisions and the accepted areas 1–3 did not change, except for one MODIFIED requirement of `server-runtime` ("Startup order").

- R-1: the `control.socket` key (default `control.sock`) is added to the list of settings, the table of ADR 0076 item 2 and the value check; `control.sock` and `control.lock` are added to the list of settings-directory files (`operator-lifecycle`, ADR 0076 items 2, 5, 7).
- R-2: the startup order is defined by the MODIFIED requirement of `server-runtime` "Startup order": `LOCK` → read settings → check the database → `control.lock`, `tokens`, channel → listeners → …; the order of refusals 6, 3, 4, 5, 12, 3 or 10, 7, 8, 9 is fixed (ADR 0077 item 1, 0080 item 2, 0081 item 1; ADR 0074 item 1 is extended with a reference).
- R-3: a busy `control.lock` is code 12 (`config_locked`); `tokens` unparsable or of a newer format is code 3 (`key=tokens`); unreadable due to access is code 10; a socket path that is too long is a settings error, code 3 (D-72); the scenarios and the code table of `operator-lifecycle` and `operator-control` are aligned.
- R-4: `init` does not touch a directory that has data files (`data`, `dwb`, the WAL), even without `control` (owner decision Q-7, ADR 0015 item 2); the remnants of an interrupted `init` are recognized only by the `init.partial` marker, which `init` itself writes; `serve` on a directory with data and without `control` does not suggest `init` (code 5, `data_present=true`);
  with the marker the hint is "repeat `npdb init`" (ADR 0075 items 4, 8).
- R-5: lifecycle is aligned with the accepted `storage-durability` without changing it: a planned start with a read-only settings directory is code 10; a crash start and a start on a backup are "storage error", search works, there is no control channel and no local-program token (D-71, ADR 0077 item 9).
- R-6: the server confirms to the stop command any exit with code 0, including before the database is opened; the stop-during-startup scenarios name code 0; 35 is only an exit without confirmation (ADR 0084 item 8, ADR 0077 item 2).
- R-7: the "Another user" scenario requires an OS refusal (`EACCES` in `connect`, code 22) with no server record; the `control_denied` record with the peer uid appears only in the accepted-connection scenario (the OS ignores permissions) (ADR 0080 item 4, ADR 0081 item 3).
- R-8: an offline command creates `tokens` and `control.lock` with the owner and group of the settings directory, including when run by `root`; if there is no settings directory — code 10; the scenario "root, server not running" (D-73, ADR 0081 item 7).
- R-9: the command reads two keys from `settings.toml` — `control.socket` and `network.enabled`; the warning about a disabled network is the same online and offline (D-74, ADR 0080 item 3, ADR 0082 item 9).
- R-10: `npdb-fill` accepts a mandatory `--config`, takes `control.lock`, performs a crash start after a crash with a `generation` record; a settings directory that is not writable is code 10; `--from` after a crash equals the number of records from recovery, otherwise code 42 (D-75, ADR 0079 items 4, 5).

## Requirements for other areas

### UI (area 5)

- The UI address is a single line of `serve` standard output of the form `http://127.0.0.1:<port>/#<token>`; the port is the `loopback.port` setting (ADR 0078, `operator-lifecycle`, "UI address is a single line on standard output"). The UI does not get the address by any other path and does not write it to a log; after a restart the address is new, the old one does not work (CUJ13).
- Operator messages and exit codes are not UI; the UI does not show the state of the control channels or the state of `control.lock`.

### Server and build (area 3)

- A new `toml` crate (ADR 0076, cargo deny); the engine exports the write path as a library for `npdb-fill`; a separate build target for `npdb-fill`, not part of the delivery (ADR 0079).
- The server reads `settings.toml` right after `LOCK`, ignores SIGHUP, has its own shutdown timer `shutdown.timeout_seconds` (60 s, code 13), parses `tokens` before opening the listeners, opens the control channel before the listeners and confirms to the stop command any exit with code 0.
- The control channel does not share limits with the HTTP slots and works under "read-only" and "storage error" (ADR 0080 item 6); tokens are outside the database.

### Storage (area 2)

- No changes; `storage-durability` ("Unavailable counter location or randomness source") is carried out without edits; the `init.partial` marker is a service file only for the duration of creation and is not part of the database (ADR 0040).

### Documentation and delivery

- The operator manual (ADR 0079 item 6, `operator-lifecycle`, "Operator manual"): `sudo` is acceptable for control commands; a long settings path means `control.socket`; tokens and `generation` are carried over together with the settings; a backup is a directory, to be duplicated before the first start; backup is tied to the command (Ctrl-C cancels);
  a directory with `data` and without `control` means restore from a backup, `init` does not touch it; the numbers of the service manager's shutdown deadline (Q-144).

## Verified facts

`facts.md`, F-109…F-115: signal(7), `tokio::signal`, systemd `TimeoutStopSec=`, the `toml` crate, unix(7), `getpeereid(3)`, tokio `peer_cred`. Not verified (marked in `facts.md`): the value of `DefaultTimeoutStopSec` and the launchd analogue (F-111), ignoring SIGHUP in tokio (F-110), cargo deny for `toml` (F-112), socket file permissions on macOS and `peer_cred` by running (F-114, F-115).

## Open questions

`docs/open-questions.md`: 8 questions closed (section "Closed in area 4"), 1 narrowed (Q-45), new Q-142…Q-157 (14 (T), 2 (O): Q-145, Q-150), [DEFAULT] D-54…D-75 await the owner as a package.
