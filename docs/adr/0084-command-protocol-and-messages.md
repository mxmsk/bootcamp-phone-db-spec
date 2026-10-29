# 0084. Command protocol, output streams, exit codes, operator log entries about commands, stop by command
- Status: Accepted ([OURS]: items 1–4, 6–8); Accepted (OD-6) ([DEFAULT]: item 5 — wait timeouts, D-68)
- Label: [OURS] (items 1–4, 6–8); [DEFAULT] (item 5 — D-68)
- Date: 2026-10-27
- CUJ: CUJ10, CUJ15, CUJ7, CUJ13, CUJ14

## Context
The token, backup and stop commands (ADR 0082, 0083, item 8 below) go through the channel of ADR 0080. The general rules for launching and messages are the lifecycle subtask: the program `npdb`, the subcommand is the
first argument, parameters are `--name value` (ADR 0075 item 1), the exit code table, where 20–39 are given to the control commands (ADR 0077 item 6), the format of server entries in stderr: the line
`time level event msg` and the event keys (ADR 0078 items 2, 3). Here is what concerns the client side of the control commands and the server's entries about them.

## Decision
1. [OURS] Channel protocol: one connection is one command. Messages are "length + JSON" frames (`serde_json`, brief §3: the crate is specified). The client sends first
   `hello` with the channel protocol version number and the command with arguments; the server replies with events (`progress` for backup; `stopping` for stop) and one final
   result (success with data or a refusal with a kind and text). The protocol version is an integer, changed on an incompatible change to the messages; a match of the program
   version is not required. A mismatch — the server replies with the refusal `protocol_mismatch` (with the server's version), the command: «версия протокола команды N, сервера M: перезапустите сервер программой той версии, что у команды» ("command protocol version N, server's M: restart the server with the program version that the command has") (code 23). Reason: after a program upgrade (ADR 0013) the running server stays old until restart, while the commands are already new.
2. [OURS] Streams: standard output carries only a script-usable result (the token secret; list rows; the backup path); the standard error stream carries everything else
   (explanations, warnings, progress, errors; command messages are in Russian, not in the server entry format of ADR 0078 item 2, which belongs to the `serve` process). One entry is one line (except the
   list and the hint). The secret is not printed to the error stream.
3. [OURS] Exit codes of the control commands (ADR 0077 item 6: the common table; 0 only if done, 20–39 are for this subtask; the common codes 2, 10, 11 are reused):

   | Code | Meaning | Commands |
   |---|---|---|
   | 0 | done (for `stop` — a clean shutdown is acknowledged) | all |
   | 2 | usage error: missing parameter, unknown parameter, invalid client name or permissions, `--show` is not `active`/`all` | all |
   | 10 | token file not written: no space or no access (the common `file_access`) | `token-issue`, `token-revoke` |
   | 11 | the backup target exists and is non-empty or is not a directory (the common `target_exists`) | `backup` |
   | 20 | server not running | `backup`, `stop` |
   | 21 | the server is starting or stopping (tokens, backup), or the channel is not open or already closed | `token-*`, `backup` |
   | 22 | no access to the channel | all |
   | 23 | protocol version mismatch | all |
   | 24 | no reply in time or the server is lost: outcome unknown | all |
   | 25 | client name taken | `token-issue` |
   | 26 | there is no active token with this name | `token-revoke` |
   | 27 | a backup is already running | `backup` |
   | 28 | the backup target is invalid otherwise: no parent, inside the database or settings directory | `backup` |
   | 29 | not enough space before copying starts | `backup` |
   | 30 | write or space error during copying | `backup` |
   | 31 | source corrupt (a page failed verification) | `backup` |
   | 32 | snapshot revoked (retention ceiling) | `backup` |
   | 33 | backup cancelled (by interrupting the command or by server shutdown) | `backup` |
   | 34 | token file corrupt or of a format newer than the program | `token-*` |
   | 35 | shutdown ended without a clean-shutdown acknowledgment | `stop` |
   | 36 | channel busy: the limit of concurrent connections | all |
   | 37–39 | reserved | |

   Code 1 is an unexpected error (common). A backup script needs 27, 29, 30, 31, 32 to be distinguishable (retry later, free up space, check the disk), so there is no single "failure code".
4. [OURS] Server entries about commands (the channel and format are ADR 0078: stderr, `key=value`; events are only added, ADR 0078 item 2): `token_issued` (INFO; `client`, `access`), `token_revoked` (INFO; `client`),
   `backup_begin` (INFO; `target`, `snapshot_lsn`, `snapshot_time`, `size_bytes`), `backup_done` (INFO; `target`, `commit_number`, `snapshot_time`, `size_bytes`, `duration_s`),
   `backup_failed` (ERROR; `target`, `reason` = `no_space` \| `write_error` \| `source_damaged` \| `snapshot_revoked` \| `target`, `errno`, `file`, `offset`), `backup_cancelled` (WARN; `target`, `cause` = `command` \| `stop`),
   `control_denied` (WARN; `peer_uid`, `peer_pid`), `control_protocol_mismatch` (WARN; `peer_uid`, `client_protocol`, `server_protocol`). `token_issued`, `token_revoked` and `backup_*` also carry `peer_uid` and
   `peer_pid` (if the OS provides it: `UCred::pid`, F-115): it is visible who issued and revoked a token. A network client's name in an entry is allowed (ADR 0078 item 5). The token secret and hash, and command arguments unrelated to the event, are not written (ADR 0065 item 7).
5. [DEFAULT] (D-68) Command client wait timeouts: connection — 5 s; reply to a token command — 10 s; backup — the server sends `progress` at least once per 10 s, silence longer than 60 s counts as loss of the
   server; stop — no deadline (the process finishes on its own by `shutdown.timeout_seconds`, ADR 0077 item 4). On timeout: code 24, the message «сервер не ответил за N с; исход команды неизвестен: проверьте `npdb token-list` (токены) или цель копии» ("the server did not reply within N s; the command's outcome is unknown: check `npdb token-list` (tokens) or the backup target") — on a dropped connection the backup is cancelled by the server itself (ADR 0083 item 8). Rationale: writing the token file takes milliseconds; 10 s covers a slow
   disk but does not make the operator wait forever; progress once per 10 s (ADR 0074 item 4 (d), ADR 0078) gives a margin of six intervals. Alternatives: no deadline — the operator "hangs" on a stuck server; a deadline for the whole
   backup — depends on the database size (up to tens of minutes at 10^8).
6. [OURS] The token secret passes only over the socket from the server to the command and to the command's standard output; it is not written to the server log or to error messages (ADR 0065 item 7, ADR 0078 item 5). An operator who redirected
   the output to a file or to a shell with history is responsible for that file: the command writes «токен показан один раз» ("token is shown once") to the error stream.
7. [OURS] The texts of command refusals are in Russian, not a bare code; the machine-readable kind of the refusal (`name_taken`, `not_found`, `backup_busy`, `target_not_empty`, `target_invalid`, `no_space`, `source_damaged`, `snapshot_revoked`,
   `server_starting`, `server_stopping`, `protocol_mismatch`) is in the protocol and tests; for the operator it is the exit code (item 3). The verbatim wording of the texts is an implementation decision (like Q-65 for the UI), the specification requirements are by meaning and fields.
8. [OURS] The stop command: `npdb stop --config <settings directory>` (ADR 0077 item 2: equal to SIGTERM; the server receives it over the channel, the `stop_begin` entry has the cause `command`). It is accepted at any stage of startup and operation from the moment the channel opens (ADR 0074 item 8; before the channel, SIGTERM applies, ADR 0077 item 2); access is as for all commands.
   The server replies `stopping`, then sends «остановка подтверждена» ("stop acknowledged") and exits; the acknowledgment is sent on any exit with code 0: after the clean-shutdown marker is written (ADR 0054 item 8) or, if the database has not been opened or changed yet (an early startup stage, before the listeners open), without writing the marker (ADR 0077 item 2);
   the command waits for the connection to close and prints «сервер остановлен» ("server stopped") (code 0).
   The connection closed without the acknowledgment (forced termination on deadline, code 13 on the server, or a crash) — «остановка завершилась без подтверждения штатной, см. сообщения сервера» ("shutdown ended without clean-shutdown acknowledgment, see server messages"), code 35. If a shutdown is already in progress — the message
   «остановка уже выполняется» ("shutdown already in progress") (ADR 0074 item 8), the command waits for the end and prints the outcome; a repeated command does not speed up the shutdown. Interrupting the command itself (Ctrl-C) stops the waiting but not the shutdown. Server not running — code 20
   (not 0: the operator distinguishes «остановил» ("stopped it") from «и так не работал» ("was not running anyway")). A shutdown during a backup cancels the backup (ADR 0057 item 1; code 33 for the backup command).
9. **Changed by OD-6 (2026-10-28).** A full rebuild takes up to a few minutes at 10^7 (was up to tens of minutes at 10^8, OD-6).
- **Changed by the final fix pass (2026-10-28, ADR 0095).** Added code 37 «сервер работает без канала команд» ("the server is running without a control channel"); a connection drop during backup without a final message is code 24.

## Alternatives considered
- HTTP inside a Unix socket (hyper): rejected, ADR 0011 item 4 — administration not through HTTP; our own frame is simpler and does not mix with the API.
- A binary protocol: rejected, volumes are minimal, JSON is debuggable by eye.
- The secret and explanations together in standard output: rejected, a script could not separate the secret.
- One code for all command refusals: rejected (item 3, the backup script).
- Protocol version = program version: rejected, v1 patches without message changes must not break commands (ADR 0013).
- The server entry format for the command's own messages: rejected, a human reads the command at a terminal; the `key=value` format is for the server log (ADR 0078).
- `stop` on a stopped server gives code 0 (idempotent): rejected, a script could not tell «остановил» ("stopped it") from "was not running".

## Consequences
- Closes Q-66 for operator commands (streams, codes, entries about commands) and the stop command left by `lifecycle` (ADR 0075 item 1, ADR 0077 item 2).
- Requirements for lifecycle: add the events of item 4 to the event list of ADR 0078 item 3; the range 20–39 is used by codes 20–36 (37–39 reserved).
- Q-133 (operator metrics) is not affected.

## Sources
- ADR 0013, 0054 item 8, 0065 item 7, 0074 items 4, 7, 8, 0075 item 1, 0077 items 2, 4, 6, 0078, 0080–0083; F-115 (tokio `UCred`); `docs/open-questions.md` Q-66.
