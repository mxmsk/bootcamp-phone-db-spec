# 0078. Operator messages: channel, format, list
- Status: Accepted (OD-6)
- Label: [DEFAULT] (items 1, 6 — D-59, D-60); [OURS] (items 2–5, 7–9)
- Date: 2026-10-27
- CUJ: CUJ8, CUJ13, CUJ14, CUJ15, CUJ16

## Context
Q-66: the channel through which the server tells the operator the reason for a refusal and events (process output, exit code, log) is the "visible part" of the question. Areas 2 and 3 defined the content of the messages
(ADR 0053 items 2, 3, 7, 0054 item 4, 0058 item 4, 0074 item 4) and the prohibitions: the UI address goes only to standard output, not to the log (ADR 0074 item 5); tokens, `Authorization`, bodies, numbers and names are not in the log (ADR 0070 item 9).
Q-133: which numbers are visible to the operator ("unauthorized" refusals, aborted handshakes).

## Decision
1. [DEFAULT, D-59] There are two channels: standard error (stderr) — all operator messages and the request log; standard output (stdout) of the `serve` process — only the UI address (ADR 0074 item 5), one line. There is no permanent log
   file and no rotation: the operator or the service manager redirects stderr (`2>>file`, the service log). Rationale: stderr is the diagnostics channel by command-line process convention; separating stdout and stderr
   makes it possible to redirect the log to a file without writing into it the link with the token (ADR 0074 item 5). Alternatives: our own log file (rotation, permissions, disk space — a new area of failures); syslog/journald
   (platform-specific, different on macOS); everything to stdout (the link would end up in the log).
2. [OURS] Format: one entry is one UTF-8 line, with no line breaks inside, `key=value` pairs separated by spaces, a value with spaces or special characters in double quotes with escaping `\"`, `\\`, `\n`.
   Required keys in order: `time` (UTC, RFC 3339, seconds, `Z`), `level` (`INFO`, `WARN`, `ERROR`), `event` (a stable ASCII identifier from item 3), `msg` (text in Russian, one sentence); then the event keys.
   Event identifiers and keys are part of the operator interface: in v1 they are only added (ADR 0013 by analogy). The `msg` text is for humans, not for parsing.
3. [OURS] Required events (Q-66). Each is one entry; the keys in parentheses are required:

   | `event` | Level | When | Keys |
   |---|---|---|---|
   | `start` | INFO | start of startup | `version`, `db`, `config`, `format_version` of the database |
   | `prev_stop` | INFO | after the checks | `kind` = `clean` \| `crash` \| `copy`; for `crash` — `dropped_bytes`, `recovery` (result) |
   | `format_upgrade` | INFO | the database is older than the program | `from`, `to`; for a heavy upgrade — `done_percent` at least once per 10 s, `duration_s` at the end |
   | `index_rebuild` | INFO | rebuild | `reason`, `done_percent`, `eta_s` at least once per 10 s; `duration_s` at the end |
   | `listening` | INFO | listener opened | `addr`, `kind` = `loopback` \| `network` |
   | `storage_state` | INFO/WARN/ERROR | the start is not in the normal state, and every transition | `state` = `normal` \| `read_only` \| `storage_error`, `source`, `errno`, `file`, `offset` or `page` |
   | `ready` | INFO | working mode | `startup_s` |
   | `start_failed` | ERROR | start refusal | `reason` (item 6 of ADR 0077), `exit_code`; depending on the reason: `file`, `offset` or `lsn`, `errno`, `addr`, `key` |
   | `settings_missing`, `cert_expired`, `key_permissions`, `no_ipv6_loopback` | WARN | see ADR 0076, 0066 | `file` or `not_after` |
   | `config_dir_unwritable` | WARN | the settings directory is not writable on a crash start or a start from backup (ADR 0077 item 9) | `path`, `errno` |
   | `auth_counters` | INFO | item 6 | `unauthorized`, `handshake_aborted`, `rate_limited` for the period |
   | `stop_begin` | INFO | signal | `cause` = `TERM` \| `INT` \| `command` |
   | `stop_repeat` | INFO | repeated signal | `cause` |
   | `sighup_ignored` | INFO | SIGHUP | — |
   | `stop_done` | INFO | before exit 0 | `duration_s` |
   | `stop_timeout` | ERROR | item 4 of ADR 0077 | `timeout_s` |
   | `request` | INFO | request (if `log.requests`) | `method`, `path` without the query string, `status`, `duration_ms`, `client` (the name of the network client, if any) |

   Control events (token issuance and revocation without the secret, backup: start, commit number, snapshot time, end, refusal) belong to the `control` subcommand, the format is the same.
4. [OURS] The reason for a start refusal is always in `start_failed` and before the process closes; for codes 5–10 and 12 the entry contains the data the operator needs to act: the directory (`db`), file, offset or LSN, `errno`, address and port,
   settings key, «создана версией N, программа знает до M» ("created by version N, the program knows up to M").
5. [OURS] Secrets: stderr contains no tokens (of any kind), no `Authorization` header, no UI address, no key or certificate values, no request bodies, numbers, or names (ADR 0070 item 9, 0065 item 7); the name of a network client is allowed.
   The only place for an address with an embedded token is the `serve` stdout (ADR 0074 item 5).
6. [DEFAULT, D-60] Counters (Q-133): once per `log.counters_seconds` (60 s), if the increment is non-zero — an `auth_counters` entry: the number of "unauthorized" refusals, aborted TLS handshakes and request-limit refusals for the period.
   No metrics endpoint (there is no administration through the API, X-3). Rationale: the operator needs a "token brute-force" signal (ADR 0067 item 3) without a new interface. Alternatives: an entry for every refusal (a stream of refusals floods stderr and
   would become a way to fill the operator's disk); counters on a command request (the `control` channel, an extra operation).
7. [OURS] The language of `msg` is Russian; `event` and keys are ASCII Latin. Reason: the owner's operator is Russian-speaking (the specification and the CUJ16 texts are in Russian), while parsing and search use the stable `event`. There is no localization.
8. [OURS] A failure to write to stderr or stdout (closed terminal, closed pipe, `EPIPE`, `EIO`) does not crash the server and does not change its state: the message is lost, the rest of the work continues (SIGPIPE is not a termination).
   Consequence: a UI address message that did not reach a closed stdout is not printed again (the address once, CUJ13).
9. [OURS] Messages are not buffered longer than the write: each entry is flushed immediately (a start refusal is visible before the process exits); the order of entries is the order of events.

## Alternatives considered
- JSON lines (one record per line): parsed more precisely, read worse by eye; `key=value` satisfies both.
- DEBUG/TRACE levels and level configuration: not needed by the v1 operator, out of scope.
- Messages in English: the industry standard, but the owner and the CUJ16 texts are Russian; events stay in Latin script.

## Consequences
- Q-66 is closed (channel, format, list); Q-133 is closed (item 6). D-59, D-60 go to the owner in the area 4 package.
- When run under a service manager, stdout also ends up in the service log together with the UI address: the operator manual says so (ADR 0079 item 6; area 3 design).
- Test: the list of events — `operator-lifecycle` scenarios; absence of secrets — a search for the token across all of stderr.

## Sources
- ADR 0053, 0054 item 4, 0058 item 4, 0065 item 7, 0067 item 3, 0070 item 9, 0074 items 4, 5; Q-66, Q-133; CUJ13, CUJ14, CUJ16.
