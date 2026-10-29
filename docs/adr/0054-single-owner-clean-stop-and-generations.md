# 0054. Single database owner, clean-shutdown marker, version generations
- Status: Accepted
- Label: [OURS] (mechanism per the owner's decisions ADR 0015, 0028 items 7, 9 [USER])
- Date: 2026-10-27
- CUJ: CUJ13, CUJ14, CUJ15, CUJ16, X-2, X-5

## Context
ADR 0015 [USER]: a second server on the same database is a refusal «база уже открыта другим процессом» ("the database is already opened by another process"). ADR 0028: clean shutdown is defined by
item 10 and differs from crash shutdown by a marker on the medium; the version across generations is variant (c) of item 9 [USER, OD-5, D-17]: a generation
counter outside the backup next to the settings plus a random epoch of at least 128 bits; unavailability of the counter location or of the randomness
source is a database state refusal. Here: the storage mechanics and edge cases (Q-78, Q-99, Q-107, Q-108).

## Decision
1. [OURS] Database owner: the `LOCK` file in the database directory. As its first action at start, before reading any other file, the server opens
   it (with the close-on-`exec` flag) and takes an exclusive non-blocking lock (`flock(LOCK_EX | LOCK_NB)`; in Rust —
   `File::try_lock`, which calls exactly that, F-76). The descriptor is held until the process ends; the OS releases the lock on any process
   termination: on Linux — "when all descriptors are closed" (F-73), on macOS verified by a run (`kill -9` of the owner — the lock
   is free, F-81). Busy (`EWOULDBLOCK`/`EAGAIN`) — start refusal «база уже открыта другим процессом» ("the database is already opened by another process") (ADR 0015 item 3, X-2); any other lock
   error — a start refusal with its own cause, not with this text. `LOCK` holds the process identifier — for diagnostics, not for decisions.
2. [OURS] The database creation command (area 4) takes the same lock and creates `control` with `O_EXCL` (create, not overwrite): on a directory
   with an existing `control` it is a refusal; an existing database is not changed (ADR 0015 item 2). Creation: the directory, `LOCK`, `data` with a header and
   empty indexes, `dwb`, the first WAL segment, `control` last — a flush of each file and of the directory (ADR 0050 item 3); a database that has
   no `control` is not a database (the server does not start, ADR 0015 item 1).
3. [OURS] Lock boundaries: it is advisory — a process that does not take it (`cp`, an editor, another program) is not stopped (F-73, F-74);
   a child process that inherited the descriptor holds the lock after the owner dies (measurement, F-81) — the server does not spawn
   processes with this descriptor; network file systems are not supported (not verified). The minimum Rust version for `File::try_lock` is 1.89 (F-75),
   higher than `rust-version = "1.88"` from the "Build and tests" section of area 1's `design.md` (1.88 is an ICU4X requirement, not a ceiling): either raise to 1.89 or take
   the lock via `libc::flock` (a separate ADR on the crate, ADR 0017 item 3) (Q-118).
4. [OURS] The stop marker is the `run_state` field in `control` ∈ {`clean`, `running`, `backup`}. The start kind by state at start: `clean` —
   planned (the generation does not change); `running` — crash; `backup` — start from backup (ADR 0028 items 5, 7, 10). For files copied not by the command, the marker
   determines the start kind (ADR 0028 item 7); binding the marker to the medium (Q-99) is not introduced: a reliable volume identifier is not verified, ADR 0028 item 7 does not require it. The database identifier
   `db_id` (random 128 bits at creation) is checked across the database files (ADR 0053 item 1) and protects against mixing files of different databases, not against copies.
5. [OURS] An abnormal start (crash or from backup) after successful checks (ADR 0053) and before accepting operations: (a) the generation counter location —
   the `generation` file in the server settings directory (next to the tokens; path and permissions — area 4): two 4 KiB slots with a number and CRC;
   the directory is writable, the file is absent — the counter is 0 (first start or a move without settings); present but both slots invalid —
   unavailable; (b) `G_s` is read; `G = max(G_s, G_run from control) + 1`; it is written to the slot with the lower number, the flush of the file (and of the directory on creation) is durable
   (ADR 0050); (c) the epoch `E` — 16 bytes from the OS randomness source (Linux — `getrandom(2)`, macOS — `getentropy`, F-77, F-78); (d) into a `control` slot
   are written `G_run = G`, `E_run = E`, `run_state = running`, the WAL epoch + 1 (ADR 0051 item 5 (g)), flush; (e) accept operations. The initial version of the generation is (G, E, 0).
   The observable number version is (G, E, S) (ADR 0051 item 7), 32 bytes; clients compare it only for equality (ADR 0027 item 5; representation — area 3).
   The maximum with `G_run` is [OURS]: if the settings rolled back below what is recorded in the database itself, the generation still grows (ADR 0028 item 9 (c) allows this).
6. [OURS] Planned start: `G_run`, `E_run` are taken from `control`, the settings are neither read nor written; `run_state = running` is durably written to `control` before
   the first operation is accepted (ADR 0028 item 10). All issued versions remain valid (ADR 0006 item 4).
7. [OURS] Unavailability of the counter location (no settings directory, no write permission, `EROFS`, both slots invalid, a flush error) or of the randomness source (a
   `getrandom`/`getentropy` error) at an abnormal start — a database state refusal (ADR 0028 item 8): "storage error" (ADR 0053 item 6 (e)); `run_state`
   does not change; write operations are not executed; search works; versions in the results are of the previous generation (`G_run`, `E_run`, S), good for reading only: no write on them will
   pass (Q-107). Exit — fix the cause and restart (ADR 0053 item 8 (a)). Not a "start refusal": the database is intact, reading is useful to the operator.
8. [OURS] Clean shutdown on the storage side (ADR 0028 item 10): on operator command or SIGTERM/SIGINT: (1) stop accepting operations; (2) acknowledge or reject each accepted
   operation (none is left in flight); (3) flush the WAL; (4) a full checkpoint (all dirty pages; ADR 0051 item 4);
   (5) write to `control`: `redo_start_lsn` = the end of the WAL, `next_S`, `run_state = clean`, flush (ADR 0050); (6) release the lock, exit code 0. The marker is step (5), the last.
   Deadline: if the process is terminated before it (forcibly, by the area 4 deadline, an error in steps 3–5), the marker is not written — the shutdown is a crash (Q-108). Estimated duration of steps (3)–(5): ≲ 30 s
   (ADR 0052: background writing up to 5.8 GiB in 5 minutes of load), for area 4 a deadline of at least 60 s is recommended.
9. [OURS] Backup: the `backup` marker in the backup's `control` is set by the copy command; it is cleared only in a database started from the backup (item 5 (d)), not in the archive (ADR 0028 item 5).
   A start from backup performs item 5, including the recovery pass (ADR 0051 item 5), if the backup contains a WAL.
10. [OURS] Testability (ADR 0028 item 9): the randomness source and the counter storage are injectable interfaces; a test with a deterministic source checks: growth of `G` and change of `E` on a
    crash start and a start from backup; no change on a planned start; refusal on a source or storage error; a rollback of `G_s` does not produce matching versions with different `E`; two starts
    from one archive give different `G`; the capacity of `E` is not less than 128 bits.
11. **Changed by OD-6 (2026-10-28).** The scale is 10^7 records (ADR 0017); the numbers and estimates for 10^8 in this ADR are the history of the calculation, the effective values for 10^7 are in ADR 0044, 0049, 0052, 0058, 0059; non-linear estimates are recomputed on the prototype (T, `docs/open-questions.md`).
- **Changed by the final fix pass (2026-10-28, ADR 0095).** `serve` checks that the database directory exists before `LOCK` (if not — exit code 5); a leftover `LOCK` is acceptable in a directory without a database.

## Alternatives considered
- A file with a process identifier instead of a lock: after `kill -9` or power loss it remains stale, and "alive" cannot be told from "dead" without checking the
  process (races, number reuse). Rejected.
- POSIX locks `fcntl(F_SETLK)`: released when any descriptor of the same file in the process is closed (a property of the family; the wording was not found in the pages checked — not verified,
  F-73); `flock` is tied to the open file description. Not chosen.
- A lock on the `data` or `control` file: any other open-close of it in the process risks releasing it; a separate `LOCK` isolates. Not chosen.
- A TCP port as a lock: it protects the address, not the database. Rejected.
- Always a new generation at every start: planned restarts would devalue versions (ADR 0028 item 7 (3)). Rejected.
- Store `G` and `E` per number: 24 bytes × 10^8 = 2.4 GB and re-evaluation at start (ADR 0028 item 6 (d)). Rejected; (G, E) is one per start (ADR 0051 item 7).
- Binding the marker to a volume identifier (Q-99): the identifier's reliability is not verified; not introduced.
- Put `generation` in the database directory: it would enter the backup and lose its meaning (ADR 0028 item 9 (b)).

## Consequences
- Q-78 is closed (with ADR 0051), Q-107 is closed (item 7), Q-108: the deadline and codes — area 4, storage provides the estimate of item 8; Q-99: not introduced (stays a question if the owner wants strengthening).
- Area 4: the settings directory is writable at an abnormal start; document that `generation` is not part of the backup and moves with the settings; shutdown deadline ≥ 60 s;
  a message about the kind of the previous stop (ADR 0028; Q-66).
- Area 3: the version is an opaque value of 32 bytes; the message «база уже открыта другим процессом» ("the database is already opened by another process").
- Every abnormal start: two durable writes (≈ 10 ms on the development machine, ADR 0052) plus the end-of-recovery entry.

## Sources
- ADR 0006, 0015, 0027, 0028 (items 5–10), 0050, 0051, 0053; `docs/owner-decisions.md`, OD-3 (Q-7, Q-77), OD-4 (item 5), OD-5 (D-13, D-17).
- F-73…F-78, F-81 (`facts.md`); `research/durability/flock-test.py`.
