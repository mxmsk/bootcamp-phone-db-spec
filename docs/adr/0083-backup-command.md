# 0083. Backup command: target, progress, cancellation, outcomes
- Status: Accepted ([OURS]: items 1, 2, 4–7, 9); Accepted (OD-6) ([DEFAULT]: item 3 — valid target, D-70; item 8 — cancellation, D-69)
- Label: [OURS] (items 1, 2, 4–7, 9); [DEFAULT] (item 3 — D-70; item 8 — D-69)
- Date: 2026-10-27
- CUJ: CUJ15, CUJ14, X-2, X-5

## Context
ADR 0016 [USER]: a backup by operator command without stopping the server, a snapshot at a single point in time. ADR 0057 [OURS] defines copying inside the
process (snapshot on receiving the command, one backup at a time, space check, a non-empty target is a refusal, page verification) and leaves to area
4 the form of the command itself, the display of outcomes, and cancellation. Q-41: parameters, outcomes; Q-60: who executes.

## Decision
1. [OURS] Command: `npdb backup --config <settings directory> --target <target>`. The target parameter is the path of the target directory; the command turns a relative path into an absolute one using
   its own working directory and passes it to the server (the server performs the backup as its own user; the path is interpreted on the server machine). There are no other parameters
   (the speed and the space reserve are the settings `backup.read_rate_mib_per_second` and `backup.space_reserve_percent`, ADR 0076 item 2, not command parameters). The backup is made by the server process (ADR 0057 item 1); the command is a
   client of the channel (ADR 0080), online mode is mandatory (ADR 0081 item 4).
2. [OURS] The order of refusals before copying starts: (1) the server is starting or stopping — refusal; (2) a backup is already running — "backup
   already running" (ADR 0057 item 1; the second command changes nothing and does not wait, code 27); (3) the snapshot is opened when the command is received (ADR 0057 item 2) — its moment
   is the backup point; (4) target check (item 3); (5) space check (item 4). Any refusal at (4)–(5) releases the snapshot; server clients see nothing.
   The "read-only" and "storage error" states of the source — ADR 0057 item 7.
3. [DEFAULT] (D-70) Target: the path does not exist — the parent must exist and be writable by the server, the target is created; the path exists and is an
   empty directory — it is used; a non-empty directory or a non-directory — the refusal «цель не пуста» ("target is not empty") (code 11, like `target_exists` for `init`, ADR 0077 item 6), nothing is overwritten or deleted (ADR 0057 item 6, like
   database creation, ADR 0015 item 2). A target inside the database directory or the settings directory, or equal to them — the refusal «цель внутри каталога базы/настроек» ("target is inside the database/settings directory") (code 28; the same code applies when the target's parent does not exist). Rationale: a backup inside the
   server's working directory would mix the archive with the database and settings files and could end up in a settings transfer; the check is cheap. Alternative: allow
   anything that is empty — but for a database an "empty" subdirectory is acceptable only by accident; not creating missing parent directories — otherwise
   a typo in the path would silently create a tree on another disk.
4. [OURS] Space: before copying starts, the free space on the target volume is at least the snapshot size (the size of the database files at the moment of the snapshot) plus a reserve (`backup.space_reserve_percent`, 5% by default,
   ADR 0057 item 6). If less — the refusal «недостаточно места: нужно N, доступно M» ("not enough space: need N, available M") (both in GiB with one decimal place), code 29, nothing created. Running out of space or a
   write error during copying — copying stops, the partial backup is deleted (ADR 0057 item 6), a message with the reason and `errno`, code 30; the server continues
   working without refusals to clients.
5. [OURS] The command stays connected for the whole duration of the backup and prints progress: at the start — "snapshot N (UTC time), size X, estimate Y min at speed
   Z MiB/s"; then at least once per 10 s (like the index rebuild, ADR 0074 item 4 (d)): the fraction, copied out of total, speed, estimate of the remainder. On a terminal — a single
   updated line; not on a terminal — separate lines to the standard error stream. The fraction is by snapshot pages (ADR 0041).
6. [OURS] Success: standard output — one line with the backup path; the error stream — «Копия создана: путь, номер фиксации N, время снимка T (UTC), размер, длительность» ("Backup created: path, commit number N, snapshot time T (UTC), size, duration")
   (the number and time from the backup description, ADR 0057 item 2); a hint: "Tokens and settings are not part of the backup. A backup is a directory; to start from one archive repeatedly,
   duplicate it before the first start (ADR 0057 item 8)". Code 0. The backup is fully written only after the atomic write of the description (ADR 0057 item 5); before that
   it does not exist as a backup and the command does not print success.
7. [OURS] Refusals and errors during copying and the operator message: the source is corrupt (a page failed verification, ADR 0057 item 4) — "backup not created: storage
   error in file F, offset O"; the server enters "storage error" (ADR 0014 item 6); "snapshot revoked" (ADR 0057 item 12; ADR 0059 — the retention ceiling) — "backup not
   created: snapshot revoked under load, try again later"; the partial backup is deleted. Codes: 31 (source corrupt), 32 (snapshot revoked), ADR 0084 item 3.
8. [DEFAULT] (D-69) Cancellation: copying is bound to the command. Interrupting the command (Ctrl-C, SIGINT, SIGTERM, closing the terminal, losing the connection to the server)
   cancels the copy: the server stops copying, deletes the partial backup, releases the snapshot; the command (if alive) prints «копирование отменено» ("backup cancelled"), code 33.
   A clean server shutdown also cancels copying (ADR 0057 item 1): the command prints «копирование отменено: сервер останавливается» ("backup cancelled: server stopping"), code 33. Separate
   "backup status" and "cancel backup" commands are not introduced. Rationale: the brief has neither background tasks nor their statuses; the binding is simple
   (no orphaned backups, no "backup in progress" state after a restart) and sufficient: copying 40 GB takes ≈ 10 min (ADR 0059, D-32); the operator runs the command in a terminal
   or under a session manager. Alternatives: a background job with `backup status` and `backup cancel` commands — new product operations and state that
   has to be stored and cleaned up; copying continues after the command drops — an orphaned backup and a snapshot held until the retention ceiling (ADR 0059 item 2).
9. [OURS] Effect on server clients: during copying clients work without refusals (ADR 0016 item 3; latency growth — per the targets of ADR 0057 item 11); token commands
   and shutdown are processed during that time (ADR 0080 item 6). Copying does not prevent a restart: shutdown does not wait for copying (ADR 0057 item 1).

## Alternatives considered
- Copying on the command's side (the command reads the files): rejected, one process owns the database (X-2, ADR 0057).
- Copying into a non-empty target with `--force`: rejected, "nothing is overwritten" (ADR 0057 item 6).
- Resuming an interrupted backup: rejected, a partial backup is not a backup (ADR 0057 item 5).
- Showing progress through server messages without the command: rejected, the operator runs the command in one terminal and waits for the result there.

## Consequences
- Q-41 is closed for the command (parameters, outcomes); storage — ADR 0057.
- Lifecycle requirement: server messages (Q-66) about the start, completion and refusal of a backup (one entry each; target, commit number, time, size, refusal
  reason) — ADR 0084 item 4.
- Requirement for area 2/3: nothing new.

## Sources
- ADR 0014, 0015, 0016, 0028, 0041, 0055, 0056, 0057, 0059, 0074, 0080, 0081, 0084; `docs/owner-decisions.md` OD-3 (Q-10, Q-11).

- **Changed by OD-7 (2026-10-28).** Item 7: when copying is interrupted by the modified-page limit — «копия не создана: слишком высокая нагрузка записи, повторите позже» ("backup not created: write load too high, try again later"), code 32 (instead of "snapshot revoked"); the size for the space check is `data` plus the WAL up to X (ADR 0096).
