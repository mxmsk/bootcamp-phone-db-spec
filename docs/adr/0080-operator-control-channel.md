# 0080. Operator control channel to a running server: Unix socket, access, location
- Status: Accepted ([OURS]: items 1–4, 6–8); Accepted (OD-6) ([DEFAULT]: item 5 — the set of permitted users, D-62; item 3, location — D-63; item 3, keys read by the command — D-74)
- Label: [OURS] (items 1–4, 6–8); [DEFAULT] (item 5 — D-62; item 3 — D-63, D-74)
- Date: 2026-10-27
- CUJ: CUJ10, CUJ15, X-2; item 1.2 of the original wording

## Context
ADR 0011 item 4 [USER]: there is no administration through the HTTP API. ADR 0012 item 4 and 0016 item 1 [USER]: tokens and backup are done by a command on the server
machine, without stopping the server. One process owns the database (X-2, ADR 0054), so the backup command and the command that changes the token set
of a running server must reach that process. Undecided (Q-60): how a command reaches the process and who may run it.
Q-60 is marked [b.5]: the "who may" part concerns the owner, so a default is given here.

## Decision
1. [OURS] The channel is a Unix socket (`AF_UNIX`, `SOCK_STREAM`) opened by the server process. It exists only on the server machine: administration has no network
   listener and no HTTP route (ADR 0011 item 4). Operator commands are clients of this socket; the server executes them itself
   (it is the owner of the database and of the token set).
2. [OURS] The server opens the socket at a startup step after taking `control.lock` (ADR 0081 item 1) and parsing `tokens`; this step comes after reading the settings (the socket path is the `control.socket` key, known) and checking that the database exists, and before the listeners
   (ADR 0074 item 1 with the area 4 addition, ADR 0077 item 1; `server-runtime`, "Startup order"), and closes it after the HTTP listeners are stopped, before the process exits. On a clean shutdown the socket file
   is deleted; on a crash it stays "orphaned" (no process listens on it) and is deleted by the next start (ADR 0081 item 2).
3. [OURS] Location: the file `control.sock` in the `--config` settings directory (next to `local.token` and `tokens`, ADR 0065, 0076 item 7; the settings directory has permissions 0700),
   the path is overridden by the `settings.toml` setting `control.socket` (a relative path is relative to the settings directory, ADR 0076 item 2; the key is added to the table of ADR 0076, see "Consequences").
   The command reads only two keys from `settings.toml`: `control.socket` (the channel path) and `network.enabled` (for the warning when issuing a token, ADR 0082 item 9; the same online and offline) — the rest are checked by the server; if the file is missing or cannot be parsed — `control.sock` and no network warning. [DEFAULT] (D-74): the alternative is to limit the warning to online mode (the command reads only `control.socket`): the output of the same command would depend on whether the server is running. Not in the database directory: the composition of the database directory is fixed by ADR 0040 and copied by ADR 0057;
   runtime files are not added to it. [DEFAULT] (D-63) in the part "settings directory, not the database directory"; the example "in the database directory" from the
   subtask statement is rejected for this reason. The socket path length cap: 108 bytes on Linux, may be 92 on other systems (unix(7), F-113);
   a path longer than the cap is a start refusal with code 3 (`reason=settings key=control.socket`, a settings error, ADR 0076 item 5, checked when reading the settings) with the message «путь канала команд слишком длинный, задайте `control.socket`» ("the control channel path is too long, set `control.socket`") (clients do not have to
   guess). Estimate: a typical settings directory fits; moving to a short path is done by the setting.
4. [OURS] Access, two independent layers. (a) The socket file is created with permissions 0600 and owned by the server's user: `umask` is set to `0177`
   before threads are created (at startup the process is single-threaded), `bind`, then the previous `umask`. On Linux, `connect` requires write permission on the socket
   (unix(7), F-113), so another user cannot connect; but POSIX says nothing about socket permissions, on some systems they are
   ignored, and "portable programs should not rely on them" (F-113). (b) Therefore, after `accept` the server checks the peer's credentials
   (SO_PEERCRED on Linux, `getpeereid` on macOS and BSD; in `tokio` — `UnixStream::peer_cred`, F-114, F-115) and accepts
   a command only if the peer's effective uid equals the server's effective uid or is 0 (item 5). Otherwise the connection is closed without executing the command, and
   the server operator gets the message «команда отклонена: чужой пользователь (uid …)» ("command rejected: foreign user (uid …)") (ADR 0084 item 4). The message is possible only for a connection that reached `accept`: when the OS enforces the permissions (settings directory 0700, socket 0600), a foreign user gets an OS refusal (`EACCES`) already at `connect`, the server knows nothing about it, and the command reports code 22 (ADR 0081 item 3 (c)).
5. [DEFAULT] (D-62) The set of permitted users: the OS user the server runs as, and `root` (uid 0). Rationale: `root` already reads `tokens`,
   `local.token` and the database directory (permissions 0600 do not stop it), refusing it protects nothing and breaks administration through `sudo`;
   the database owner by OS is the server's user. A group or any local user is not permitted. Alternatives: (1) any local
   user — rejected: issuing and revoking tokens would be available to anyone who can log in to the machine; (2) only the server's uid, without `root` —
   rejected: running the command with `sudo` would not work; (3) a configurable list of uids or a group — rejected: the brief does not require it, it is a new setting.
   An owner-level question; a conflict with the brief is impossible (the brief does not name the set of users).
6. [OURS] The server does not trust the content of a command: all fields are validated (ADR 0082), the socket is read by a thread/task separate from the HTTP pools (ADR 0071)
   and does not share connection limits with them: issuing and revoking a token must work when HTTP slots are exhausted (CUJ10: revocation, including under
   abuse). Concurrent channel connections — at most 8 (the value is an implementation decision); beyond that — the refusal «канал занят» ("channel busy").
7. [OURS] The channel carries the commands: issue a token, list tokens, revoke a token (ADR 0082), backup (ADR 0083) and the clean shutdown command
   (`npdb stop`, ADR 0084 item 8; equal to SIGTERM, ADR 0077 item 2; the channel for it is the same one, with the same access check). The channel has no other
   administrative operations: test loading, switching the "read-only" mode (out of scope, ADR 0018), etc. do not exist.
8. [OURS] The `control.sock` file is not part of the backup (the backup builds the database directory, ADR 0057 item 13); a server started on a backup creates its own.

## Alternatives considered
- A separate TCP administration port on loopback: rejected, any local user can reach it, without peer credentials; a token for
  it is one more secret (P-5, P-7); ADR 0011 item 4 forbids administration through HTTP.
- A named pipe (FIFO) or a "mailbox" file with polling: rejected, no reply and no peer credentials, no progress and cancellation.
- Signals (SIGUSR1) and re-reading a file: rejected, a signal has no arguments and no reply; "done" after the change is visible (D-45) cannot be expressed.
- An administrator token/password in a file: rejected, one more secret, and OS permissions on the socket plus the peer check give the same without it.
- Running the backup as a separate program on the files: impossible, one process owns the database (ADR 0057 item 1).
- The socket in the database directory: rejected (item 3), it changes the composition of the database directory of ADR 0040.

## Consequences
- Specification: `operator-control` (channels, access, online/offline, messages).
- The lifecycle requirements are met: the `control.socket` key in the settings table of ADR 0076 item 2; the files `control.sock` and `control.lock` in the file list of ADR 0076 item 7; the startup order (channel after `control.lock`, before the listeners) — ADR 0077 item 1 and the MODIFIED-delta of `server-runtime`; deleting the socket file on
  shutdown and on start refusal; an operator message about refusing a foreign user (ADR 0084).
- Closes Q-60 (the channel; "who may" — by default, D-62) and part of Q-66 (the channel for replies to commands).
- Platforms: Unix-like (Linux, macOS), like `flock` in ADR 0054 item 1; Windows is not supported (the brief does not name a platform, checked in Q-147).

## Sources
- ADR 0011, 0012, 0016, 0018, 0040, 0054, 0057, 0065, 0074; `docs/owner-decisions.md` OD-3 (Q-9), OD-4 (item 9).
- Facts: F-113 (unix(7)), F-114 (`getpeereid`), F-115 (tokio `peer_cred`) (`facts.md`).
- Pitfalls P-5, P-7.
- **Changed by the final fix pass (2026-10-28, ADR 0095).** A server without a channel (the settings directory is not writable) holds the existing `control.lock` for reading; commands reply with code 37.

- **Changed by OD-7 (2026-10-28).** Item 4 (a): `umask` is not changed; the socket is created in a 0700 directory and after `bind` gets `chmod 0600` (ADR 0101).
