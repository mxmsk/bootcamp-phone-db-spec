# 0081. Commands with the server stopped: settings directory lock and mode selection
- Status: Accepted ([OURS]: items 1 (except the refusal code), 2, 3, 5 (except the codes), 6); Accepted (OD-6) ([DEFAULT]: item 4 — which commands work offline, D-64; items 1 and 5, start refusal codes, — D-72; item 7 — file owner on an offline write, D-73)
- Label: [OURS] (items 1–3, 5, 6, except the refusal codes); [DEFAULT] (item 4 — D-64; items 1, 5, refusal codes — D-72; item 7 — D-73)
- Date: 2026-10-27
- CUJ: CUJ10, CUJ13, CUJ15, X-2

## Context
The token set lives in the `tokens` file of the settings directory (ADR 0065 item 4), written by the running server (issuance and revocation must be visible to
the next request, ADR 0065 item 8). If the server is stopped, the operator may want to issue a token in advance (for example, before first enabling
the network listener or after a move: ADR 0012 item 8 "reissues"). Two writers of the file are a race: a lost issuance or a revocation that "comes back to life".
Q-60: what happens if the server is not running.

## Decision
1. [OURS] Settings directory lock: the file `control.lock` in the settings directory. The server takes an exclusive non-blocking lock on it
   (`flock(LOCK_EX | LOCK_NB)`, like `LOCK` of the database, ADR 0054 item 1, F-81) after `LOCK` of the database directory, reading the settings and checking that the database exists (the order is ADR 0077 item 1: settings are read earlier, and their content determines the socket path) and holds it until the process exits. If held —
   a start refusal with code 12, `reason=config_locked`: «каталог настроек занят другим процессом» ("the settings directory is held by another process") ([DEFAULT], D-72: code 12 is free in the table of ADR 0077 item 6; 6 means "the database is open", 10 — "no access", while here the database is free and access exists; the alternative is to reuse 10 or 6, but a script could not tell the reason apart)
   (two servers with the same settings would share tokens and the socket). The lock is released by the OS
   when the process exits, including a crash.
2. [OURS] Orphaned socket: the file `control.sock` with no listener. A server that has taken `control.lock` knows there is no other listener, and before `bind`
   it deletes the existing file (if the file exists, `bind` gives `EADDRINUSE`, unix(7), F-113). Without the lock the file is never deleted: neither
   by the server nor by a command.
3. [OURS] Mode selection by the token command, in order: (a) `connect` to the socket succeeded → online: the server executes the command; (b) `ENOENT` or
   `ECONNREFUSED` → there is no listener; the command tries to take `control.lock` (`LOCK_EX | LOCK_NB`): took it → offline (item 4), holds the lock until the end of the
   command; did not take it → the server is alive, but the channel is not yet open or already closed («сервер запускается или останавливается, повторите позже» ("the server is starting or stopping, try again later"), code 21);
   (c) `EACCES` → «нет доступа к каналу сервера: выполняйте команду от пользователя сервера или `root`» ("no access to the server channel: run the command as the server's user or `root`") (code 22), offline is not tried (the `tokens`
   file is inaccessible to the same user); the server does not learn of this refusal — a `control_denied` entry exists only for a connection accepted by the server (ADR 0080 item 4). The connection timeout is 5 s (D-68, ADR 0084 item 5). The command needs only the settings directory (`--config`, ADR 0075 item 2); `--db` is not needed.
4. [DEFAULT] (D-64) These work offline: issue a token, list tokens, revoke a token — directly on the `tokens` file, with the same logic and the same messages,
   the changes take effect at the server's next start (the command writes «сервер не запущен: изменение вступит в силу при запуске» ("server not running: the change takes effect at start"); the warning «сетевой приём выключен» ("network listener is off") — by `network.enabled` from `settings.toml`, as online, ADR 0080 item 3, D-74). These do not work
   offline: backup (ADR 0057 item 1: the backup is made by the server process, files of a stopped database are not considered a backup, ADR 0028 item 7) and
   the clean shutdown command (there is no server); both reply «сервер не запущен» ("server not running"), code 20. Rationale: tokens are not part of the database (OD-4 item 9) and issuing them before
   enabling the network is the real order of first setup (CUJ13); a backup without the server would mean byte-by-byte copying, which is forbidden. Alternative:
   tokens only online — simpler (the single writer is always the server), but the operator would have to start the server to issue a token for the network
   listener that does not yet exist without a token; rejected. An owner-level question: there is no conflict with the brief.
5. [OURS] An offline command neither reads nor changes the database and does not take `LOCK` of the database directory; it needs only the settings directory. If `tokens` does not exist, the set
   is empty (the first issuance creates the file with permissions 0600). If the file exists but cannot be parsed (format — ADR 0082 item 7), the command refuses and leaves the file
   untouched (code 34, «файл токенов повреждён» ("token file is corrupt"); starting the server on it is a refusal for the same reason: code 3, `start_failed reason=settings`, `key=tokens` — the file cannot be parsed or has a format newer than the program; a file that cannot be read because of access is code 10, `reason=file_access`: D-72;
   silently reading it "as empty" would erase revocations on the next write).
6. [OURS] An online command is executed by the server with the same code as offline (one implementation over one format, ADR 0082): the difference is only in
   who holds the lock and whether the set in the server's memory is updated. This way the observable behavior of the two modes does not diverge.
7. [DEFAULT, D-73] File owner on an offline write. An offline command creates and replaces `tokens` and creates `control.lock` with permissions 0600 and the owner and group of the **settings directory** (`fchown` by `stat` of the directory before `rename`/after creation), not of the user who ran the command; if the settings directory does not exist — code 10, nothing is created
   (the command does not create the settings directory: `init` does). Reason: the set of permitted users includes `root` (D-62), and an offline write under `sudo` would otherwise leave `tokens` and `control.lock` owned by `root` with permissions 0600; the server, running as its own user, could not read `tokens` (a start refusal, code 10) and could not write the change —
   a contradiction of `api-access` ("Network token storage"). Alternative: refuse if the command's uid does not equal the owner of the settings directory (stricter, but breaks the `sudo` administration for which `root` is permitted). A command from a user who is neither the directory owner nor `root` cannot write to a 0700 directory anyway.

## Alternatives considered
- Locking the `tokens` file itself: rejected, the file is replaced atomically (`rename`, ADR 0065 item 4), and the lock would be held on the old
  inode.
- Take `LOCK` of the database directory instead of `control.lock`: rejected, it ties tokens to the presence of a database (there is no database yet when the operator prepares tokens before
  creating one), and the command would write diagnostics into `LOCK` and could interfere with the owner.
- An offline command that starts the server for a while: rejected, an action not observable to the operator.
- Checking the process by the pid in `LOCK`: rejected, the pid is for diagnostics only (ADR 0054 item 1); pid reuse.

## Consequences
- The lifecycle requirements are met: the startup order is extended with the step "`control.lock`, `tokens`, channel" after checking that the database exists (ADR 0077 item 1; MODIFIED-delta of `server-runtime`); the refusal «каталог настроек занят другим процессом» ("the settings directory is held by another process") — code 12; `control.lock` and `control.sock` — in the file list of the settings directory (ADR 0076 item 7).
- Requirement for area 3: at startup the server reads `tokens` in full (ADR 0065 item 4) and on a parse error refuses to start.
- Q-60: the answer to "what if the server is not running".

## Sources
- ADR 0012 item 8, 0016, 0028 item 7, 0054 item 1, 0057 item 1, 0065; OD-4 item 9; F-81 (`flock`, behavior on process exit); F-113.
