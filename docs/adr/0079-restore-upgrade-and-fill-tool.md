# 0079. Restore by starting on a backup, program upgrade, test load-generation tool
- Status: Accepted (OD-6)
- Label: [OURS] (items 1–3, 5 except `--config` and code 42, 6, 7); [DEFAULT] (item 4 — D-61; items 4, 5, `--config` and the tool's crash start — D-75)
- Date: 2026-10-27
- CUJ: CUJ11, CUJ14, CUJ15, X-1, X-3

## Context
CUJ14 and CUJ15: the operator restores a corrupt database by starting the server on a backup (ADR 0016 item 4, ADR 0057). ADR 0013 [USER]: a new version within v1 opens databases of earlier versions on its own, without a command;
there is no program version rollback; the earlier program is not required to open a database after the new one. ADR 0040 item 4: a format newer than the program is a refusal. Q-53: verifying 10^8 requires a test load-generation tool, not a product operation
(the option chosen by the owner instead of batch loading, X-3); profile P0, inserts in random order — ADR 0044 item 7, `storage-format` ("File size at 10^7 records").

## Decision
1. [OURS] Restore is an ordinary `npdb serve --db <backup directory> --config <settings directory>`: there is no separate restore command (ADR 0016 item 4). A backup is a directory created by the backup command (the `control` subcommand);
   the server recognizes "backup" by `run_state = backup` (ADR 0054 item 4), reports `prev_stop kind=copy`, changes the version generation and clears the marker (ADR 0057 item 9). Tokens and `generation` are taken from the operator's settings directory
   (they are not part of the backup, ADR 0016 item 5): the same tokens, a revoked one stays revoked. The operator does not delete or overwrite the directory of the damaged database with a command: they restore into another directory or rename the old one.
2. [OURS] Repeated start from one archive: after the first start the backup directory stops being a backup (the marker is cleared, ADR 0057 item 8). To start from an archive again, the operator duplicates the directory before the first start.
   On a start from backup the server states this in the `prev_stop` message (key `hint`). Files copied by something other than the command (for example, the directory of a stopped database) are not considered a backup: they start by the marker on the medium, "restore" from them is not
   supported (ADR 0028 item 7, D-13 [USER]) — the operator manual (item 6).
3. [OURS] Program upgrade: the operator stops the server cleanly, installs the new program and starts it on the same database; there is no separate upgrade command (ADR 0013 item 3). If the database `format_version` is lower than the current one — a `format_upgrade` entry,
   the upgrade is done before the first operation is accepted (ADR 0040 item 4 (c)); a heavy upgrade and an index rebuild show progress (ADR 0058 item 4), clients get "server starting". If `format_version` or `features_incompat` is newer — a refusal,
   code 9, the message «база создана версией N, эта программа знает до M» ("the database was created by version N, this program knows up to M"), files unchanged. A backup made before the upgrade starts with the new program the same way (ADR 0013 item 4). Rollback is only restoring a backup made before the upgrade;
   the operator manual recommends a backup before every upgrade (a recommendation, not a program check). `npdb --version` prints the program version and the highest openable `format_version`.
4. [DEFAULT, D-61] The test load-generation tool is a separate program `npdb-fill`, not part of the product delivery (built as a separate package/target; `npdb` has no such subcommand, there is no API, X-3). It works on a stopped database:
   it takes `LOCK` like the server (held — code 6) and `control.lock` of the `--config` directory (held — code 12), opens an existing database with the engine (`init` creates it beforehand) and inserts records by the same write path as server operations (WAL, checkpoints, number versions,
   batch commit); there are no WAL bypasses and no bottom-up index construction. Opening is as for the server: if the previous termination (of the tool or the server) was a crash (SIGKILL, out of memory during a multi-hour load), a crash start with writing `generation` to the settings directory is required (ADR 0054 item 7), so `--config` is mandatory for the tool as well. If the settings directory is not writable in this case — refusal with code 10, the database is unchanged
   (the tool is a writer, it has no "storage error" mode with search; the operator fixes the cause or starts and cleanly stops `serve`). Rationale: the goal is to verify size and latencies on a database filled by the real write path in random order (ADR 0044, "Consequences": an ordered load
   gives a fill factor of 0.9 and understates the size); through the HTTP API 10^8 records at 100 records per second (the speed of a sequential client acknowledging each one, ADR 0068 item 1) is 10^8 / 100 / 86,400 ≈ 11.6 days. Alternatives for opening: refuse on any database with `run_state` ≠ `clean` (simpler, but after a failure at 10^8 the load would have to start over or be repaired by starting the server); open without writing `generation` (violates ADR 0028 item 9). Alternatives: loading through the server API
   (exercises the whole stack, but unacceptably slow and loads the write path, which we want to measure separately); batch load in the product (forbidden by X-3); building the file bottom-up (a different layout than a real database, the test's purpose is lost).
5. [OURS] `npdb-fill` parameters: `--db`, `--config` (mandatory, as for `serve`, ADR 0075 item 2), `--count` (number of records; default 10^8), `--seed` (default 1), `--profile p0` (the only one in v1: average name 22 characters, 2.5 words per name, 1.25 records per number, ADR 0044 item 7), `--from` (the index to
   continue from). The record with index i is a deterministic function of (`seed`, i), numbers and insertion order are pseudo-random, so two identical commands produce the same database, and an interrupted load continues with `--from`. Every 10 s — progress (inserted,
   records per second, size of `data`); at the end — a summary: number of records, size of `data`, bytes per record, time. SIGINT/SIGTERM — a clean engine shutdown (the clean-shutdown marker, as for the server), the summary and code 40; errors — per the common table (ADR 0077 item 6);
   code 41 — `--count` or parameters are invalid for the profile, 42 — the database already contains records when `--from` ≠ the number of records (after a crash termination the number of records is determined by recovery on open; the message names the actual number, the operator repeats with it). Messages — the ADR 0078 format, stderr.
6. [OURS] The operator manual (a document in the delivery) contains: the form of the commands and the exit codes; the location and contents of the settings directory and that `generation`, `tokens` and the certificate are not part of the backup and are carried with the settings (otherwise the version guarantee between generations
   is probabilistic, ≤ 2^-64 per event, ADR 0028 item 9); that restore is only from a backup created by the command, and files copied otherwise are not considered a backup (ADR 0028 item 7); how to start repeatedly from one archive (item 2); that a clean shutdown is SIGTERM, SIGINT
   or the command, with a deadline of at least 60 s, and the service manager sets its own deadline greater (ADR 0077 item 4); that SIGHUP is ignored and closing the terminal does not stop the server (ADR 0077 item 5); that under a service manager stdout with the UI address also ends up in the service log;
   that an upgrade is a program change and a start, rollback is restoring a backup made before the upgrade; the disk estimate for 10^8 (up to 1.8 × the file size plus the backup, ADR 0044 item 10).
7. [OURS] There is no separate backup verification without starting (Q-124: the backup is checked page by page on creation, ADR 0057 item 4; archive corruption is visible on start and on read). Not introduced: it is not in the brief (§3, "do not introduce operations that are not there").
8. **Changed by OD-6 (2026-10-28).** The verification scale is 10^7 records (OD-6): `--count` defaults to 10^7; through the HTTP API at 100 records per second 10^7 / 100 / 86,400 ≈ 1.2 days; disk headroom for 10^7 is up to 1.8 × the file size plus the backup (at 8 GB ≈ 22 GB); the size comparison is against 8 × 10^9 bytes (ADR 0044). D-61, D-75 accepted by the owner (OD-6).

## Alternatives considered
- An `npdb restore` command: rejected, "restore is starting the server on a backup" (brief §3; ADR 0016 item 4).
- Operator confirmation before a format upgrade: rejected, ADR 0013 item 3 [USER] (the server opens the database on its own).
- The load-generation tool as an `npdb fill` subcommand: rejected, the delivered executable would get a product bulk-insert operation, which X-3 forbids; a separate program is not part of the delivery and does not extend the API.
- Online loading with the server running: rejected (see item 4; X-2: one process owns the database).

## Consequences
- Q-53 is closed by the tool's composition (D-61 goes to the owner in the area 4 package: confirm the composition). Q-124 is closed (not introduced).
- D-75 goes to the owner in the area 4 package; continuation with `--from` after a crash termination at 10^8 is verified by the implementation (Q-156).
- Load measurements of search and write latencies on a filled database (ADR 0049, 0052) are separate implementation work, outside this tool; here only the load and the size (Q-114, Q-137).
- Implementation requirement: the engine exports the write path as a library suitable for both the server and `npdb-fill`.

## Sources
- ADR 0013, 0016, 0018, 0028 items 7, 9, 0040 item 4, 0044 items 7, 9, 10, 0054 items 1, 4, 0057 items 8, 9, 0058 item 4; `storage-format`; brief §3–4; Q-53, Q-124; CUJ11, CUJ14, CUJ15, X-1, X-3.
