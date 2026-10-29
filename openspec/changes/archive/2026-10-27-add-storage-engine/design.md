## Area boundaries

Area 2 defines a dedicated storage engine: the file and page format, indexes for the three searches and the result order, durability and
crash recovery, the integrity check, a consistent cut for a backup, a single database owner, storage of
versions and generations, the scale of 10^8 records and the size and latency targets. The semantics of data and operations was accepted in area 1
(`records`, `write-operations`, `name-search`, `number-search`, `result-ordering`) and does not change. Out of the area, with no requirements
here: the protocol, API error codes and bodies, tokens (area 3), operator commands (area 4), the UI look and texts (area 5); everything
area 2 requires from them is collected in the section "Requirements for other areas". Off-the-shelf storages and search engines are forbidden
(ADR 0017 item 4); crates are allowed only with an ADR and a cargo deny check (ADR 0017 item 3).

The storage requirements accumulated by area 1 are met (`openspec/changes/archive/2026-10-27-add-data-model/design.md`, "Storage"):
the record key (number, date) with no scan on search by number; the name in NFC byte by byte; keys of any length without truncation (ADR 0041, 0045 item 6);
the number version in one indivisible write with the record itself (ADR 0042, 0051); a shared version counter, generation and epoch without scanning
numbers (ADR 0051 item 7, 0054); shutdown and backup markers on the medium (ADR 0054); a snapshot for number history (ADR 0055, 0056);
result order and continuation from any position without cursor state (ADR 0047); indexes as a function of records with a format label and
rebuild (ADR 0045, 0058); acknowledgment after durability (ADR 0050).

## Map of capabilities, ADRs and CUJs

| Capability | ADR | CUJs and requirements |
|---|---|---|
| `storage-format` (22 requirements, 64 scenarios) | 0040–0044 | CUJ1–CUJ7, CUJ9, CUJ11–CUJ16; X-1, X-2, X-4, X-5 |
| `storage-indexes` (13, 33) | 0045–0049 | CUJ1, CUJ2, CUJ4–CUJ9, CUJ11, CUJ12, CUJ14, CUJ15; X-1, X-5 |
| `storage-durability` (28, 76) | 0050–0054 | CUJ1, CUJ3, CUJ6, CUJ8, CUJ9, CUJ12–CUJ16; X-1, X-2, X-5 |
| `storage-snapshots` (27, 69) | 0055–0059 | CUJ1, CUJ4–CUJ6, CUJ8–CUJ16; X-1, X-2, X-5 |

In total 90 requirements and 242 scenarios in four delta-specs; ADR 0040–0059 are 20 decisions: Accepted [OURS] — 0040, 0042, 0043,
0045 (except item 6), 0046–0048, 0050, 0051, 0054, 0055, 0057, 0058 and 0056 (except item 1 (b); the numbers are in 0059), 0053 (except items 5 (errno classification) and 8); Proposed [DEFAULT] — 0041 items 1 and 5, 0044 items 6
and 10, 0045 item 6, 0049 items 2–5, 0052, 0053 items 5 and 8, 0056 item 1 (b), 0059. This area changes none of the owner's decisions ([USER]); the area 1 requirements are unchanged (no MODIFIED delta).

Boundaries between capabilities. `storage-format` defines the page, keys, record and header encoding, free pages and size;
`storage-indexes` — what lies in the trees R, O, W, D and how to read them; `storage-durability` — the WAL, recovery, checks,
states and the owner; `storage-snapshots` — readers on snapshots, backup and rebuild on format change. Shared objects:
`META` (page 0) — format, `control` — durability, the free page list and snapshots — format and snapshots, the key format label —
indexes, format and snapshots. Requirements refer to each other by requirement names, not by ADR numbers.

Dependence on owner decisions and the [DEFAULT] package: all numeric targets and two client-visible features are marked [DEFAULT] and
collected in `docs/open-questions.md`, "Area 2: defaults" (D-20…D-36).

## Subtask reconciliation

The subtasks (format, indexes, durability, snapshots) were written in parallel; the seams below are reconciled to a single value, and the ADRs and specs are fixed
only in the affected phrases. The subtasks' decisions did not change in meaning.

| No. | Discrepancy | Single value | Rationale | Fixed in |
|---|---|---|---|---|
| 1 | Spill-to-overflow threshold: 2 KiB (ADR 0041) and "a quarter of a page" (ADR 0045 item 6) | 2,048 bytes of cell payload, 256 bytes in the page; key ceiling 65,535 | The page layout is a matter of the format; the 2 KiB threshold gives at least 7 cells per page (a quarter — 4) and better fill; SQLite (F-53) is only a precedent for the technique | ADR 0045 item 6, alternatives; ADR 0041, "Consequences" |
| 2 | Format version: META (ADR 0040) and `control` (ADR 0053 item 1 (a)) | `format_version` only in META; `control` and the WAL keep their own layout labels | A single source of truth; two labels mean two checks at open, both not newer than the program | ADR 0053 item 1 (a); ADR 0040, "Consequences"; spec `storage-durability` |
| 3 | Number version: 32 bytes (ADR 0049), 8 bytes (ADR 0051), varint (ADR 0042) | In the number header — `S`, a u64 varint (≤ 5 bytes at S < 2^35); `G` (8 bytes) and `E` (16 bytes) — only in `control`; `(G, E, S)` in 32 bytes — only in the API response | The number version is one per start (ADR 0051 item 7); a copy of the generation in every number would cost ≈ 2.4 GB and re-estimation at start | ADR 0049 (assumption, item 1, recalculation `output-version5-2026-10-27.txt`: 38.9 GB instead of 42.0), 0051 items 7–8, 0052, 0042; spec `storage-durability` |
| 4 | Rebuild time: 10–30 min (ADR 0045, 0049) and 20 s–2.5 min (ADR 0058) | A full rebuild of O, W, D ≈ 10–30 min at 10^8, target ≤ 60 min (D-26); 20 s–2.5 min — only the order index | The ADR 0058 estimate concerned a single order index (key computation measured, F-84); W and D not measured, the model gives ≈ 2 min with a margin of 5–15 times | ADR 0058 items 5, "Alternatives", "Consequences"; ADR 0059 item 4 refers to ADR 0049 item 5; a scenario in `storage-indexes` |
| 5 | A page with LSN and CRC in the header (durability requirement) and the ADR 0041 layout | They match: `checksum` (0), `page_type`, `page_no`, `page_lsn` (16) in the 32-byte header; WAL changes do not touch the `checksum` field (the sum is computed at write, ADR 0043 item 3), `page_lsn` after application equals the LSN of the entry | The durability requirement is met by the format; the page storage is confirmed, no revision of items 3–4 of ADR 0051 is needed | ADR 0051 (context, item 2), ADR 0041, "Consequences" |
| 6 | Snapshot "visibility after durability" and retention of earlier versions (ADR 0055, 0056) versus the free page list with `freed_at` (ADR 0044) | Two kinds of retention: (a) pages removed from the trees wait in the free list; `freed_at` is the LSN of a WAL entry, the snapshot remembers the published LSN, a page is handed out if there is no snapshot with an LSN below `freed_at`; (b) earlier published versions of pages changed after the snapshot are held in memory, beyond the budget — in a temporary file `retain` (D-35; the `data` file holds the checkpoint version, not the latest durable one); a reader sees only published versions | The LSN continues across restarts, so the `freed_at` of an earlier run is below the LSN of any new snapshot (there are no snapshots after a restart); the commit number N starts anew with a restart | ADR 0044 items 1–2, 0055 item 3, 0056 items 1, 7, 0040 item 6; specs `storage-format`, `storage-snapshots` |
| 7 | Size target: ≤ 50 GB (ADR 0049) and ≤ 64 GB (ADR 0044) | A single [DEFAULT] D-21: the `data` file ≤ 64 GB for 10^8 records of profile P0 | The subtasks' calculations and the variants are below, "File size target" | ADR 0044 item 6, 0049 item 1; spec `storage-indexes`; profile P0 replaces the reference corpus of ADR 0049 (2,106 names versus 12) |
| 8 | WAL entry cap 128 KiB (ADR 0051) and the worst case of the indexes: 200 words, up to ≈ 204 leaves per record (ADR 0045 item 9 (d), 0049) | 4 MiB (204 leaves × 16 KiB ≈ 3.3 MiB plus the inner page paths); a typical record ≈ 2 KiB | 128 KiB did not cover inserting a 200-word name (≈ 180 KiB of indexes), let alone a replacement | ADR 0051 items 2, 8 |
| 9 | Index format label: 4 components (ADR 0045 item 7) and 2 components (ADR 0058, spec `storage-snapshots`) | The label from ADR 0045 item 7 (key format, `icu_collator`, word tables, hash H64), a u64 `key_format_label` in the descriptor of each derived tree (ADR 0040 item 3); R has none | A rebuild is needed when any component changes | spec `storage-snapshots` |
| 10 | The post-rebuild check "number of index entries = number of data records" is wrong for W and D | O — the number of R records; W — the sum of the number of distinct words and one posting per whole name; D — the number of word postings | W has several postings per record | ADR 0058 item 3; spec `storage-snapshots` |
| 11 | The "backup" marker "in the backup database header" (ADR 0057 item 8) and "no markers in `data`" (ADR 0040 item 5) | In the backup description and in the backup's `control` | The single source of truth about the start kind is `control` (ADR 0054 item 4) | ADR 0057 item 8; spec `storage-snapshots` |
| 12 | Number-stream lifetime: "area 3 request timeout, the value is in ADR 0049" (ADR 0046 item 3) and a 120 s ceiling (ADR 0056) | The stream has no general lifetime ceiling, client idle is 120 s (D-30, fix after review R-6); the area 3 request timeout may be shorter — a requirement on area 3 | ADR 0049 has no stream lifetime | ADR 0046 item 3, "Consequences" |
| 13 | File growth is flushed with the checkpoint (ADR 0044 item 3), the `data` length is checked at open (ADR 0040 item 6) | The length is checked against the checkpoint state before WAL replay; a page beyond the end of the file during replay is treated as zero, the file is extended | Without this a failure between the WAL and the growth flush would produce a false "truncated" refusal | ADR 0040 item 6; spec `storage-format` |
| 14 | Page header size: 64 bytes (calculations of ADR 0045, 0049) and 40 (ADR 0041) | 40 bytes; the index calculations are conservative | A difference of 24 bytes per page does not change the conclusions | ADR 0041, "Consequences"; ADR 0045 item 9 (a), 0049 |
| 15 | `crc32c` speed: 22.7 GiB/s (F-58) and 8.4–12.2 GiB/s (F-85) | Both measurements are kept; the smaller one is taken for copying | Different builds and methodology; the choice of algorithm and crate does not depend on the difference | `facts.md` (F-58, F-85) |

The assumptions of the indexes subtask about the format (ADR 0045 item 9) are met as follows: (a) a 16 KiB page, a 40-byte header — ADR 0041; (b) overflow
pages — ADR 0041 items 5–6; (c) a reader sees a consistent snapshot of all trees as of the start, not blocking writes —
ADR 0055; (d) an operation's write is atomic, up to ≈ 204 leaves are one WAL entry of up to 4 MiB (ADR 0051 item 2); (e) inner pages
≈ 115 MiB in a cache ≥ 256 MiB — an assumption of the latency calculation, a requirement on the page cache (versions for snapshots are not part of the cache, ADR 0056 item 1); (f) a checksum of every page,
a read error is a "storage error" — ADR 0043, 0053. Inner-cell aggregates (two u64 in D) — `agg_len` = 16 in the tree descriptor
(ADR 0040 item 3).

### File size target at 10^8 records

A single [DEFAULT] (D-21, ADR 0044 item 6; ADR 0049 item 1 refers to it). The default is no more than 64 GB for `data`.
The subtasks' calculations:

- file-format (`research/file-format/sizing.py`, F-62): profile P0 — a 22-character name (42.1 bytes), sort key 35.2, 2.5 words,
  1.25 records per number; 404 B per record; 61.2 GB without prefix compression at fill 0.67; 52.8 / 46.2 GB with 25 / 45% compression;
  45.5 GB at fill 0.9 without compression.
- indexes (`research/indexes/calc.py`, F-52): a name of 34 bytes; 42.0 GB with a 32-byte number version and 38.9 GB with a version ≤ 5 bytes
  (`output-version5-2026-10-27.txt`), a range of 36.9–41.5 GB for 25–60% compression; 29.8 GB after a rebuild. The discrepancy with the first
  calculation: the profile (a 34-byte versus 42.1-byte name) and the assumption that the word key in the W postings is compressed to ≈ 2 bytes at any
  compression (in `sizing.py` at zero compression it is stored whole).

The default is what holds under a conservative profile without relying on unproven compression: 61.2 GB fit into 64 GB
(a margin of 4.6%), into 50 GB only with compression ≥ 33.6% (not measured), into 40 GB only with dense loading and compression ≥ 25% or with
ordinal labels (Q-110). The variants (40 / 50 / 64 / 80 GB / no number) with consequences for disk, time and backup retention are in
ADR 0044 item 6 and `docs/open-questions.md`, D-21. Profile P0 is an assumption, there is no real data (Q-111).

### Left to choice or to implementation

Not decided, so as not to change the meaning of the subtasks' decisions: the page size (D-20) and the size target (D-21) — for the owner; `rust-version` versus `File::try_lock` (Q-118);
ordinal labels in W (Q-110); merging pages on deletes (Q-115); the "index error" boundary (Q-123).

## Requirements for other areas

Collected from the subtasks' requirements; the source is in parentheses.

### Server and API (area 3)

- The "read-only" and "storage error" states are distinguishable in responses; the source, `errno`, file, offset or page are available
  to the operator (ADR 0053 items 4–7; Q-44, Q-49, Q-66). Search under a "storage error" may refuse individual requests; when the
  generation counter location is unavailable, search works, and versions of an earlier generation are valid for reading only (ADR 0054 item 7).
- The number version is an opaque 32-byte value, the client compares it only for equality (ADR 0054 item 5; Q-32, Q-34).
- The refusal «база уже открыта другим процессом» ("the database is already open by another process") (ADR 0054 item 1).
- Word search (D-24): there is no search timeout or new error in storage, a valid query always completes with a result; a heavy query takes up to ≈ 14 s at 10^8 records, and
  if the API introduces a request timeout (10 s, Q-52), then its error is visible behavior of area 3 (ADR 0049 item 3, ADR 0048 item 6).
- The database state is checked before the operation checks: in "read-only" any write operation (including one with a stale version and an identical retry) gets a "read-only" refusal
  (ADR 0053 item 8; `write-operations`, "Check order on write"). A tree key above 65,535 bytes is a refusal of a single operation without a state change, the kind being an internal error (D-36).
- Snapshot lifetime (D-30): a short read — ceiling 30 s; a number stream has no general ceiling (client idle 120 s, the retention cap, last). The request timeout from brief §4 — 10 s
  (Q-52) — is shorter than the short-read ceiling and incompatible with a stream of up to 2,967,164 records (≈ 300 MB): either exempt the stream from the 10 s timeout or give it a different lifetime,
  otherwise the history of the maximal number is cut off by the timeout. A requirement on area 3 (ADR 0046 item 3, 0056, 0059).
- The "snapshot revoked" error is a separate reason (the kind and code belong to area 3); a number stream on snapshot revocation is cut off without a
  completion marker; a stream completion marker and a stream timeout are needed (ADR 0035 item 6, 0046 item 3, 0056 item 2).
- Acknowledgment of a write operation is sent after publication (ADR 0055 item 4).
- During an index rebuild at start, connections are not accepted or get a "server starting" refusal; the kind belongs to area 3
  (ADR 0058 item 2).
- The maximum page size of results by name: a write reads more pages than it seems (≈ 50 point reads of R per page of
  exact search), the page size maximum affects the D-23 targets (ADR 0047 item 4, 0049).

### Administration (area 4)

- The create command takes `LOCK` and creates `control` with overwrite forbidden (ADR 0054 item 2); the settings directory must be writable
  on a non-clean start; the `generation` file is not part of the backup and travels with the settings (ADR 0054 item 5, 0016 item 5).
- The clean shutdown timeout is at least 60 s (the estimate for steps (3)–(5) is ≲ 30 s), exit code 0 (ADR 0054 item 8; Q-108).
- Operator messages: the kind of the previous shutdown and the number of discarded WAL tail bytes; open refusals (foreign signature, newer
  version, truncation, `db_id`, "backup incomplete"); the reason, share and time estimate of an index rebuild, the rebuild is part of start, not a
  command (ADR 0040 items 4, 6, 0045 item 8, 0051 item 6, 0053, 0058 item 4; Q-66).
- Backup: the command goes through the server process (Q-60); it prints the commit number and the snapshot time; refusals — «копирование уже выполняется» ("backup already in progress"), target busy, out of space, source corrupted, server unavailable; document that a backup is a directory created by the
  command, and to start again from one archive the backup directory must be duplicated before the first start (the `backup` marker
  is cleared at start); start from backup is a new generation; a clean shutdown cancels copying (ADR 0057 items 1, 2, 8; Q-124).
- The disk estimate for the operator: up to 1.8 × the file size plus a backup (D-22, ADR 0044 item 10); a heavy format conversion shows progress
  (ADR 0040 item 4).

### UI (area 5)

- No requirements: the texts and the verbatim wording of CUJ16 (Q-65) do not depend on storage; the "read-only" and "storage error" states are distinguishable
  in the API (ADR 0053).

### Build and tests

- A build question, unresolved (Q-118): `File::try_lock` is stable since Rust 1.89 (F-75), the area 1 `rust-version` is 1.88 (an ICU4X
  requirement, not a ceiling, F-2, F-51); the variants are to raise `rust-version` to 1.89 or to take the lock via `libc::flock` with a separate
  ADR (ADR 0054 item 3, ADR 0017 item 3).
- Crates (ADR 0017 item 3, cargo deny): `crc32c` 0.6.8, fallback `crc` 3.4.0 (ADR 0043; RustSec not checked, F-55); the randomness
  source and locking (Q-116); the hash H64 (Q-112).
- Injected interfaces: file operations (`errno` injection, failure and "silent" flush), the randomness source, the generation
  counter storage (ADR 0053 item 9, 0054 item 10).
- Tests: CRC-32C reference vectors (0xE3069283, 0x94640B85, 0x1D167853); page capacity and keys from 0 to at least 16,000
  bytes long, names of "character × 200" and alternating pairs (ADR 0041 item 8); "all search methods give the same result"; the equality "index = function of
  records"; a break between `write` and flush, corruption of a WAL byte and of a page, the order inside a broken batch, a ghost of an old epoch;
  a 10^8 bench with query classes (Q-53, D-21, D-23, D-27, D-28); measurements on Linux (Q-119); a backup load test (D-25, D-27, D-32).

## Verified facts

External claims rest on `facts.md`: F-52…F-85 (new) and F-2, F-39, F-45, F-48, F-50, F-51 (earlier). The status of each is given in
the fact itself.

- From the primary source (URL and quote): F-53 (from a retelling of the page), F-55…F-57, F-59 (the interpretation of the table is not verified and is not used as an
  argument), F-64…F-78 (F-67 — from a retelling of the page; F-70, F-76 — the master branch, not a release; F-69 — the page does not name APFS), F-82, F-83.
- By running or measuring, the path in `research/`: F-52, F-54, F-58, F-60, F-79…F-81, F-84, F-85; F-63 — a `getconf` run, the output is not saved.
- By model calculation (not an engine measurement): F-61, F-62.
- Not verified (stated in the facts and ADRs): `cargo-deny` and RustSec for `crc32c`; flush on Linux (`fdatasync`, `sync_data`); `F_FULLFSYNC` on
  a macOS directory and on APFS (except the F-79 measurement); `F_BARRIERFSYNC`; `O_CLOEXEC` for `File::open`; POSIX `fcntl` locks; `ENOSPC` on
  copy-on-write file systems; a reliable volume identifier; prefix compression and the name distribution of a real corpus;
  the OS page on x86-64; parallel page-in during recovery (5–10 s is an estimate).

## Open questions

The unresolved items are moved to `docs/open-questions.md`: new Q-110…Q-125 (15 technical, 1 for the owner — Q-124; Q-121 and Q-125 closed by the fix pass after review); closed in area 2 — 22
(15 by [OURS] decisions, 7 by defaults — the section "Closed in area 2"); narrowed Q-52, Q-53, Q-104, Q-108. [DEFAULT] D-20…D-36
(17) go to the owner as a package at the end of the area: the page size, the size target, the disk margin, the search latency targets, the search work cap (without a timeout), the latency
growth during backup, the blocking rebuild, the write latency, the start time, exit from states, the snapshot timeout, the retention cap,
the backup speed and space margin, classifying `ENOSPC` and others as "read-only", the place of earlier page versions, refusal on a key above the cap. The count after area 2: (O) 36; (T) 20; [DEFAULT] 17.
They do not change the structure of the change.

## Fix pass after the Opus review (iteration 1)

The Opus review of area 2 (4 BLOCKERs, 6 contradictions) was handled in one fix pass; there is no repeat review. The accepted semantics of area 1 and the owner's decisions did not change, no MODIFIED delta.

- R-1: the writer checks an operation against the latest state, including the unflushed one; a response that depends on an unflushed operation waits for its flush (`storage-durability`, "Acknowledgement only after flush to stable storage"; ADR 0050 item 1, 0051 item 3).
- R-2: rollback of the writer's unpublished state when a batch fails (a new `storage-durability` requirement; ADR 0051 item 9).
- R-3: out of space is "read-only" everywhere (`storage-indexes`, `storage-snapshots`); moving `ENOSPC`, `EDQUOT`, `EACCES`, `EPERM`, `EROFS` out of "storage error" is recorded as [DEFAULT] D-34, a clarification of Q-20.
- R-4: the state is checked before the operation checks, exit from "read-only" is by a trial service WAL entry (`storage-durability`; ADR 0053 item 8, 0051 item 10; D-29 clarified).
- R-5: an exception was added to the start requirement for a start with an index rebuild or format conversion (the time is in "Rebuild time", up to 60 min).
- R-6: a number stream has no general lifetime ceiling; it is revoked on client idle and last at the retention cap; revocation is a storage read refusal (`storage-snapshots`; D-30, D-31).
- R-7: the search timeout and the new error are removed, a valid query always completes with a result (`storage-indexes`; D-24 replaced); `name-search` did not change.
- R-8: the backup composition and the backup's `control` are defined (a new `storage-snapshots` requirement; ADR 0057 item 13); Q-121 closed.
- R-9: the premise of ADR 0056 is fixed; earlier versions are in memory, beyond the budget in `retain`; the cap is counted over the sum (D-35, Q-125 closed).
- R-10: a single key cap of 65,535 bytes in `storage-format`; exceeding it is a refusal of a single operation without a "storage error" (D-36).
