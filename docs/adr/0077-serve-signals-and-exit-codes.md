# 0077. The `serve` command, signals, shutdown deadline and exit codes
- Status: Accepted (OD-6)
- Label: [OURS] (items 1–3, 6 except codes 3 and 12, 7, 8); [DEFAULT] (item 4 — D-58; item 5, SIGHUP — D-57; item 6, codes 3, 10 and 12 for the settings directory — D-72; item 9 — D-71)
- Date: 2026-10-27
- CUJ: CUJ7, CUJ13, CUJ14, CUJ15, CUJ16, X-2

## Context
ADR 0074 defines the startup order and the clean shutdown on the HTTP side, leaving to area 4 the channel and message format, the exit codes, a shutdown deadline of at least 60 s (Q-108), the handling of SIGHUP (Q-136),
and the stop command. ADR 0028 item 10: clean shutdown is the operator command, SIGTERM, SIGINT; everything else (including a forced exit on deadline before the marker is written) is a crash shutdown. ADR 0054 item 8 estimates
the storage steps at ≲ 30 s. Note: the SIGHUP question in open-questions is Q-136 (number Q-139 refers to OS users on loopback).

## Decision
1. [OURS] `npdb serve --db <database directory> --config <settings directory>` runs in the foreground: it does not daemonize, does not write a pid file, and does not restart itself. Running as a service is the job of the service
   manager (systemd, launchd, supervisor): it must give a shutdown deadline of at least 60 s and deliver SIGTERM. The order is ADR 0074 item 1, extended by one step: after step (3) ("no database") the server takes the `control.lock` lock of the settings directory (ADR 0081 item 1; if held — code 12), parses `tokens` (ADR 0081 item 5) and opens the control channel (ADR 0080 item 2), and only then opens the listeners (step (4)).
   Settings are read at step (2), so the socket path (`control.socket`) is known by the time the channel is opened; a refusal at any step after the channel is opened closes it and deletes the file. Refusal order: `LOCK` (6) → settings (3) → certificate (4) → no database (5) → `control.lock` (12) → `tokens` (3 or 10) → listeners (7) → checks (8, 9, 10) —
   this is an addition to the "Startup order" requirement of `server-runtime` (MODIFIED-delta). `serve` adds to the order the parsing of parameters
   (before everything else: a usage error is code 2, `LOCK` is not taken) and the message channel (ADR 0078).
2. [OURS] Signals: SIGTERM and SIGINT are a clean shutdown (ADR 0074 item 7); the stop command (the `control` subcommand) equals SIGTERM: the server receives it over the control channel and performs the same shutdown, the reason in the message is
   `command`. Handlers are installed right after parameter parsing: SIGTERM/SIGINT before the listeners open — exit without writing the marker (nothing has been written) with code 0; the same exit on the stop command (it is accepted from the moment the channel opens) is acknowledged to the command («остановка подтверждена» ("stop acknowledged"), ADR 0084 item 8): the acknowledgment is sent on any exit with code 0, not only after the marker is written; during checks and rebuild — ADR 0074 item 8 (the rebuild
   is cancelled, the previous index is intact). Until the handlers are installed (a fraction of a second) the default OS behavior applies: the signal terminates the process before `control` is read, the database is untouched.
3. [OURS] A repeated SIGTERM/SIGINT means «остановка уже выполняется» ("shutdown already in progress") (ADR 0074 item 8); SIGQUIT and other signals are not handled (the OS default action — a crash shutdown).
4. [DEFAULT, D-58] The clean shutdown deadline is the setting `shutdown.timeout_seconds`, 60 s by default, values below 60 are not accepted (ADR 0076 item 5). The countdown starts when the signal is received. If the process has not finished within this deadline
   (storage steps ADR 0054 item 8 (3)–(5) are not complete), the server terminates itself forcibly without writing the marker, with code 13 and the message `stop_timeout`: the shutdown is a crash shutdown (ADR 0028 item 10), clients will get
   a version conflict after the start. Rationale: a 35 s budget (ADR 0074 item 9) + 25 s margin; the process's own timer does not depend on the service manager and gives one visible outcome instead of a hung process (disk, `fsync`).
   Alternative: no timer, the deadline belongs only to the service manager (systemd kills forcibly on `TimeoutStopSec`, the default value was not verified, F-111): then the deadline depends on the environment and may turn out to be less than 35 s.
   The service manager must set its own deadline greater than `shutdown.timeout_seconds` (documented, ADR 0079 item 6).
5. [DEFAULT, D-57] SIGHUP is ignored: the server is not stopped, settings are not re-read, one message `sighup_ignored` is written. Rationale: SIGHUP terminates the process by default (signal(7), F-109), which on terminal
   close would become a crash shutdown (Q-136); making SIGHUP a clean shutdown is a change to the list in ADR 0028 item 10 and to `write-operations`, accepted earlier; ignoring does not change what was accepted and
   preserves the database. Consequence: closing the terminal does not stop the server; it can be stopped with SIGTERM or the stop command; a write to a closed terminal must not crash the process (ADR 0078 item 8).
   Alternatives: SIGHUP as a clean shutdown (safer for a "forgotten" process, but requires editing ADR 0028 item 10 and `write-operations`); leave it as a crash shutdown (rejected: data is intact, but versions are devalued, ADR 0028 item 6).
6. [OURS] Exit codes (a common table for all subcommands of `npdb` and `npdb-fill`; the code is 0 only if the operation was performed):

   | Code | Meaning | Event |
   |---|---|---|
   | 0 | success; clean shutdown | `stop_done` |
   | 1 | unexpected error | `internal_error` |
   | 2 | usage error: missing or invalid parameter, unknown subcommand | `usage` |
   | 3 | settings error (ADR 0076 item 5); the `tokens` file cannot be parsed or has a format newer than the program (`key=tokens`, ADR 0081 item 5) | `start_failed` reason=`settings` |
   | 4 | certificate or key (ADR 0066 item 3) | reason=`certificate` |
   | 5 | no database in the directory (no `control`) | reason=`no_base` |
   | 6 | the database is already open by another process (`LOCK` is held) | reason=`locked` |
   | 7 | listener not bound: address and `errno` | reason=`bind` |
   | 8 | the database is corrupt or does not open as consistent, the backup is incomplete, the signature is foreign | reason=`corrupt` |
   | 9 | the database was created by a newer version of the program | reason=`newer_format` |
   | 10 | no access to files: permissions, "read-only", out of space, the token file is not written or cannot be read, the settings directory does not exist | reason=`file_access` |
   | 11 | the target already exists: the database on `init`, the backup target | `target_exists` |
   | 12 | the settings directory is held by another process (`control.lock`, ADR 0081 item 1) | reason=`config_locked` |
   | 13 | the shutdown did not finish in time (crash shutdown) | `stop_timeout` |
   | 20–39 | codes of the control subcommands (the `control` subcommand) |  |
   | 40–49 | codes of the load-generation tool (ADR 0079 item 5) |  |

   The code distinguishes the cause, not the words of the message; the message (ADR 0078) carries the details. Codes are part of the operator interface: in v1 codes are only added (ADR 0013, by analogy with the API). Termination by a signal
   (SIGKILL) gives the process no code (the shell shows 128 + the number) — a crash shutdown.
7. [OURS] A panic of the writer or of the checkpoint thread terminates the process immediately (ADR 0070 item 8); the code is non-zero (1, or the one adopted by the runtime for a panic — not promised); there is no marker — a crash shutdown.
8. [OURS] Nothing is required before `serve` except `init` (ADR 0075); a second `serve` on the same database — code 6, on the same ports with another database — code 7, with the same settings directory and another database — code 12.
9. [DEFAULT, D-71] The settings directory is not writable (it exists, but a file cannot be created in it). The accepted `storage-durability` ("Unavailable counter location or randomness source") requires that on a crash start and a start from backup the server not refuse but start in "storage error" with working search;
   a hard refusal with code 10 on any start would make this scenario unreachable, because `local.token`, `control.lock` and the socket live in the same directory. Therefore: (a) a planned start (after a clean shutdown) — refusal with code 10, `reason=file_access`, listeners closed (a server that accepts writes without a local-program token
   file and without a control channel is half-broken, and `storage-durability` does not touch the planned start); (b) a crash start and a start from backup — the start continues per `storage-durability`: "storage error", changes are rejected, search works (the UI token is created in memory); the control channel is not opened (`control.lock` and the socket
   cannot be created), the local-program token file is not written, the operator gets `config_dir_unwritable` (WARN, the path and `errno`); `npdb stop` is unavailable, shutdown is by signal. Rationale: bringing the lifecycle in line with the accepted `storage-durability` without changing it; the accepted behavior "refuse rather than work without a required file" is preserved where `storage-durability` is silent.
   Alternatives: change `storage-durability` (MODIFIED: always refuse with code 10) — changes the accepted area 2 for the convenience of the lifecycle; allow a planned start without `local.token` — local programs would be left with a stale token without a visible reason. A settings directory that does not exist at all — always code 10 (an operator typo, not a volume failure).

## Alternatives considered
- Daemonization inside the program (double `fork`, pid file): rejected, ADR 0054 item 1 does not require a pid, and `LOCK` already provides uniqueness; service managers do this more reliably; extra code and state.
- Codes from `sysexits.h` (64–78): not examined; our own table is shorter and not tied to the BSD convention.
- One non-zero code for all refusals: rejected, the purpose is to distinguish refusals (area 3 design, "Administration" section).
- Forced termination on a second signal: rejected in ADR 0074 item 8.

## Consequences
- D-57, D-58, D-71, D-72 go to the owner in the area 4 package. Q-108 is closed (deadline, codes); Q-136 is closed (SIGHUP).
- The service manager needs a shutdown deadline > 60 s; the operator manual (ADR 0079 item 6) says so.

## Sources
- ADR 0013, 0028 item 10, 0054 items 1, 8, 0066 item 3, 0070 item 8, 0074 items 1, 7–9; Q-108, Q-136; F-109…F-111.
- **Changed by the final fix pass (2026-10-28, ADR 0095).** The check that the database directory exists precedes `LOCK`; a leftover `LOCK` in an empty directory is acceptable.
