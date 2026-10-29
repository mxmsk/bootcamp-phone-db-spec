# 0075. Operator program, database and settings directories, creating an empty database
- Status: Accepted (OD-6)
- Label: [DEFAULT] (items 1, 2 — D-54, D-55); [OURS] (items 3–8)
- Date: 2026-10-27
- CUJ: CUJ13, CUJ14, CUJ15, X-2, X-3

## Context
Brief §3 and CUJ13: an empty database is created by a separate operator command, without it the server does not start; ADR 0015 [USER]: creating over an existing database is a refusal,
a second server on the same database is a refusal. ADR 0054 item 2 sets the creation mechanics (`LOCK`, `control` last with `O_EXCL`), ADR 0040 item 7 — the content of `data`,
ADR 0028 item 9 and ADR 0054 item 5 — the `generation` counter in the settings directory outside the backup, ADR 0065 — the token files there too. Not defined: the program name,
the command form, where directory paths come from, what the command creates outside the database directory.

## Decision
1. [DEFAULT, D-54] One program `npdb` with subcommands: `init` (create an empty database, this ADR), `serve` (the server, ADR 0077), subcommands for managing a running
   server (issuing, listing and revoking tokens, backup, stop) — the `control` subtask; `npdb --version` and `npdb --help`. The subcommand is the first argument, the rest are
   named parameters `--name value`. Rationale: one executable — one version of code and format for the server and the operator commands (the backup and token command
   works with the same format and the same settings directory as the server); the operator installs and updates one file. Alternative: separate programs (`npdb-server`,
   `npdb-admin`): separate launch permissions and a smaller server, but two files with a risk of version drift. The product is one program; the load-generation test tool is a separate
   non-product one (ADR 0079 item 4).
2. [DEFAULT, D-55] Directories are given explicitly and are mandatory for `init` and `serve`: `--db <database directory>` and `--config <settings directory>`. There are no defaults or environment variables;
   a missing parameter is a usage error. Rationale: there is no hidden place where the database, tokens and the generation counter end up; the operator will not create the database and settings where they are not looked for;
   the `generation` counter and tokens must lie outside the database directory (ADR 0028 item 9), a default "inside the database" would violate this, and platform defaults (XDG, `/var/lib`) were not verified.
   Alternative: platform defaults (more convenient, but ambiguous on Linux and macOS and tied to the OS user).
3. [OURS] The settings directory cannot lie inside the database directory and vice versa: `init` refuses («настройки внутри базы вошли бы в копию» ("settings inside the database would end up in the backup")). Reason: tokens and `generation` must not land in the
   backup (ADR 0016 item 5, ADR 0057 item 3). The check is by canonical paths (with symbolic links resolved).
4. [OURS] `init` performs ADR 0054 item 2: takes `LOCK` (busy — refusal), creates the database directory, then the incomplete-creation marker — the file `init.partial` (flushed to stable storage before any other file), `data`, `dwb`, the first WAL segment,
   `control` last (`O_EXCL`), after which it durably deletes the marker; an existing `control` — refusal, nothing is changed. The marker is a service file of `init` itself, a working database does not have it and it is not part of the ADR 0040 database composition (a backup is built anew, ADR 0057 item 13; a marker left next to a finished
   database by a break between `control` and deletion is harmless: `serve` ignores it, `init` refuses because of `control`). A database directory is acceptable for `init` if it does not exist, is empty, contains only `LOCK`, or contains leftovers of an interrupted `init`: files of the database composition (`LOCK`, `data`, `dwb`, WAL) when the marker `init.partial` is present and `control` is absent;
   such leftovers are replaced. Any other directory — refusal with code 11, nothing is deleted or changed: in particular, a directory with any data files (`data`, `dwb`, WAL) without the marker is not touched even if `control` is absent (lost or deleted) — it is not a "leftover",
   but possibly a working database, and `init` will not replace it (ADR 0015 item 2 [USER], the owner's decision Q-7: "never overwrites"). Basis: a set of names does not distinguish an interrupted creation from a database with a lost `control`; only an explicit marker, written by `init` itself, does. An interrupted creation is not a database (ADR 0054 item 2), re-creation is allowed; foreign files are not touched.
5. [OURS] Outside the database directory `init` behaves as follows: the settings directory is created if absent (permissions 0700); it is checked that it can be written to (a probe file is created and deleted) — otherwise a refusal before the database is created,
   because without write access to the settings directory the very first crash start could not create the generation counter (ADR 0054 item 7); if `settings.toml` is absent — it is created with defaults (ADR 0076 item 3, values in comments),
   an existing one is not changed; `generation`, `tokens`, `local.token` are not created and existing ones are not changed (an absent `generation` — counter 0, ADR 0054 item 5 (a); `tokens` is created by the first issuance, `local.token` — by starting the server).
   This way `init` on database No. 2 with the settings directory of database No. 1 breaks nothing.
6. [OURS] Result: standard output — the summary (database path, settings path, `format_version`, `db_id`, program version) and a launch hint; errors — per ADR 0078; the code — per the table of ADR 0077 item 6: 0 success, 2 usage error, 6 `LOCK` busy,
   10 no access, 11 the database already exists, the directory contains foreign files or data files without `control` (item 4).
7. [OURS] Creation neither starts the server nor opens listeners; on a machine where a server already runs on another database, `init` is allowed (the lock is on the database directory).
8. [OURS] The server on a directory without `control` does not start (ADR 0015 item 1; code 5). If there are no data files in the directory (empty, no directory, only `LOCK`), it reports: «в каталоге нет базы; создайте её командой `npdb init`» ("there is no database in the directory; create it with the `npdb init` command"); with the marker `init.partial` — «создание базы не завершено; повторите `npdb init`» ("database creation was not completed; repeat `npdb init`");
   if there are data files but no marker — «в каталоге есть файлы данных, но нет `control`; каталог не изменён; восстановите базу из резервной копии» ("the directory has data files but no `control`; the directory is unchanged; restore the database from a backup") (`start_failed reason=no_base data_present=true`), with no `init` hint: `init` will not touch such a directory anyway (item 4). There is no auto-creation at start.

## Alternatives considered
- Automatic database creation on `serve` (the `--create` flag): rejected, ADR 0015 item 1 [USER].
- Overwriting under the `--force` flag: rejected, "never overwrites" (ADR 0015 item 2 [USER]).
- A default settings directory inside the database directory: rejected, `generation` and tokens would end up in the backup (item 3).
- `init` creates `generation` with the value 0: not needed (absence = 0, ADR 0054 item 5), and an extra file in a shared settings directory could overwrite someone else's counter by mistake.
- `init` finds and deletes any files in the directory: rejected, a risk of losing others' data.
- Leftovers by a set of names (without a marker) or by the size and content of `data` ("looks empty"): rejected, a set of names and "similarity" do not distinguish an interrupted creation from a working database with a lost `control`; the marker is written by `init` itself and is unambiguous.

## Consequences
- D-54 and D-55 go to the owner as an area 4 package.
- The `init.partial` marker is the only new file in the database directory, and only for the duration of creation; the check in the implementation — Q-155.
- The `control` subcommands accept the same `--db` and `--config` if they need a directory (ADR 0075 as a common convention; the choice of channel — their ADR).
- Database creation is the only subcommand writing to the database directory without the server; the others (`serve`, `fill`) take `LOCK` per ADR 0054.

## Sources
- ADR 0013, 0015, 0016 item 5, 0028 item 9, 0040 item 7, 0054 items 2, 5, 7, 0057 item 3, 0065 items 2, 4; brief §3 "Scale and reliability"; CUJ13; X-2, X-3.
