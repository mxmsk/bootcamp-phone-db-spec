# 0040. Data file: directory contents, header, format version and compatibility in v1
- Status: Accepted
- Label: [OURS]
- Date: 2026-10-27
- CUJ: CUJ11, CUJ13, CUJ14, CUJ15, CUJ16; X-1, X-2; items 1.1, 1.6 of the original wording

## Context
The storage engine is written from scratch (ADR 0017). On start the server must find out what file it has, which version it is and whether it is the right
database, and a new program version within v1 must open databases of earlier versions on its own (ADR 0013 items 2–4, product
owner decision OD-4, item 8). The other subtasks of area 2 have already accepted the composition of the other database files and the shutdown markers:
`LOCK`, `control`, `dwb`, the `wal/` directory (ADR 0051 item 1, 0054 items 1, 4). Here we define the page file `data`, its header
and the format version; reconciliation with what was accepted is below.

## Decision
1. [OURS] The database directory. Files: `data` — the single page file, whose format is this ADR and ADR 0041–0044 (all records and all
   index trees R, O, W, D per ADR 0045); `control` — two 4 KiB slots with a number and a sum (the shutdown marker
   `run_state` ∈ {`clean`, `running`, `backup`}, `db_id`, `redo_start_lsn`, `next_S`, `G_run`, `E_run`, the WAL epoch;
   ADR 0051, 0054); `dwb` — a doublewrite buffer for 128 pages (ADR 0051); `wal/` — WAL segments (ADR 0051); `LOCK` —
   the owner lock (ADR 0054 item 1). The temporary file `retain` (earlier page versions for open snapshots beyond the memory budget, ADR 0056 item 1) is not part of the database, is not included
   in a backup and is deleted on start. The format of `control`, `dwb` and `wal/` is defined by durability. The `generation` counter,
   network tokens and settings live outside the database directory and are not included in a backup (ADR 0016 item 5; ADR 0054 item 5). One page
   file for all trees, not a file per index: a common free page map (ADR 0044), one checksum area
   (ADR 0043), and a rebuild of derived indexes (ADR 0045 item 8) is a swap of roots in the header, not a swap of files.
2. [OURS] The `data` header is page 0, type META (ADR 0041). It is an ordinary page: it changes by WAL entries
   (ADR 0051), carries `page_lsn`, and is protected by a checksum (ADR 0043) and the doublewrite buffer (ADR 0051 item 5 (b)). There is no second
   copy: a torn write of page 0 is repaired by `dwb` just like any page; unrepairable corruption is a start
   refusal (ADR 0053 item 1 (e), item 2). A file without a valid signature is "not a database", a refusal without reading the rest.
3. [OURS] The META fields (offsets from the start of the page; numbers are little-endian; the common page header is ADR 0041, 32 bytes):

   | Offset | Size | Field | Meaning |
   |---|---|---|---|
   | 32 | 8 | `signature` | `89 4E 50 44 42 0D 0A 1A` ("\x89NPDB\r\n\x1a"; `\r\n\x1a` catches line-ending translation on text-mode copying) |
   | 40 | 4 | `format_version` | the data format version, an integer; the first v1 version is 1 |
   | 44 | 4 | `page_size` | 16 384 (ADR 0041) |
   | 48 | 8 | `features_incompat` | feature bits; an unknown bit means refusal to open |
   | 56 | 8 | `features_compat` | feature bits; an unknown bit is ignored |
   | 64 | 16 | `db_id` | a random database identifier, equal to `db_id` in `control`, `dwb` and the segment headers (ADR 0053 item 1 (a)) |
   | 80 | 4 | `page_count` | the number of pages in use: the upper bound of allocation (ADR 0044) |
   | 84 | 4 | `freelist_head` | the first page of the free list, or `0xFFFFFFFF` (ADR 0044) |
   | 88 | 4 | `free_page_count` | the number of pages in the free list (ADR 0044) |
   | 92 | 2 | `tree_count` | the number of tree descriptors, at most 32 |
   | 94 | 2 | — | reserved, zeros |
   | 96 | 32 × `tree_count` | tree descriptors | `tree_id` u16 (@0), `kind` u16 (@2: 1 — R records, 2 — O order, 3 — W postings, 4 — D dictionary), `root_page` u32 (@4), `height` u32 (@8), `agg_len` u32 (@12: the length of the inner cell aggregates, ADR 0041; 0 for R, O, W; 16 for D), `entry_count` u64 (@16, at the time the page is written), `key_format_label` u64 (@24) |

   The rest of the page is zeros under the checksum. `key_format_label` is the format label of the tree keys (the key code version,
   the `icu_collator` version, the tables, the H64 hash; ADR 0045 item 7): a mismatch with the program's label triggers a rebuild of the
   derived trees (ADR 0045 item 8, ADR 0058); R and the number headers are not rebuilt.
4. [OURS] Format version and compatibility in v1. (a) `format_version` is an integer; a v1 program knows all data format versions from 1
   up to its current one and opens a database of any of them on its own, without an operator command (ADR 0013 items 2–3). (b) A database with
   a `format_version` greater than the program knows, or with an unknown `features_incompat` bit, is not opened: a start refusal with
   an operator message ("database was created by a newer version"; an earlier program is not required to open a database after a newer one,
   ADR 0013 item 4); `data` is not modified (ADR 0053 item 2). (c) Upgrade to the current version happens on start before the first
   operation is accepted, by ordinary WAL entries (ADR 0051), so it survives a failure: an interrupted upgrade is
   continued or repeated on the next start. A light upgrade is per page: each page carries the version of its layout
   (`page_layout`, ADR 0041), the new program reads all earlier layouts and rewrites the page in the new one on its next
   change; `format_version` in META is raised by a WAL entry before operations are accepted. A heavy upgrade (a rewrite of all
   pages or trees) is allowed, its time is part of the start time (Q-42) and progress is visible to the operator. (d) A backup made before an
   update is started by the new program under the same rules (ADR 0013 item 4, ADR 0057 item 10). (e) Rolling back the program version
   is not supported: the only rollback is a restore from a backup made before the update.
5. [OURS] The shutdown and backup markers are not stored in `data`: `run_state` lives in `control` (ADR 0054 item 4; `backup` is set by
   the backup command in the `control` of the backup, ADR 0057 item 8), the rules for setting and clearing them are ADR 0028 item 10, ADR 0054 items 5–6, 9.
   The data format does not duplicate them, to avoid two sources of truth and a window of inconsistency between two files;
   `data` and `control` share only `db_id`.
6. [OURS] Opening `data` checks: the signature, `format_version`, `features_incompat`, `page_size` = 16 384, `db_id` matches
   `control`, and the file length is not less than `page_count` × 16 384 as of the last checkpoint, before WAL replay (otherwise
   "truncated"); a tail beyond `page_count` is allowed (growth headroom, ADR 0044). Applying the WAL to a page beyond the end of the file (file
   growth was not flushed before the failure) proceeds as for a zero page and extends the file (ADR 0044 item 3). Everything else is checked on start by ADR 0053 item 1.
7. [OURS] Creating a database writes `data`: page 0 with `format_version` = current, a random `db_id` (the OS randomness source),
   and empty tree roots (the root is an empty leaf; ADR 0054 item 2, the order of creating files and flushes is also there).
8. **Changed by OD-6 (2026-10-28).** The scale is 10^7 records (ADR 0017); the numbers and estimates for 10^8 in this ADR are the history of the calculation, and the current values for 10^7 are in ADR 0044, 0049, 0052, 0058, 0059; nonlinear estimates are recomputed on a prototype (T, `docs/open-questions.md`).

## Alternatives considered
- Two copies of the header (pages 0 and 1), written alternately: needed if a page is written in place without protection;
  here every page is protected by the WAL and `dwb`, and a second copy duplicates them and adds "which is newer". Rejected.
- The markers `CLEAN_SHUTDOWN` and `BACKUP_COPY` in the `data` header (the first edition of this ADR): contradict the accepted ADR 0054
  item 4 (`run_state` in `control`) and create two sources of truth. Rejected.
- A file per index (records, order, words, dictionary): independent growth and rebuild per file, but four free space maps
  and four headers; a rebuild of derived trees is a swap of roots anyway (ADR 0045 item 8). Rejected.
- The `data` header in a separate file: a `cp` of `data` alone would lose it. Rejected.
- The format version as a pair (major, minor): v1 needs one axis — "the new program reads everything earlier, an earlier one is not required to read the new"
  (ADR 0013); for compatible additions agreed in advance there is `features_compat`.
- A separate format upgrade command: cancelled by the owner (ADR 0013 item 3).

## Consequences
- The data format version lives only in META: the single source, with no second copy in `control`. `control` and the WAL keep
  their own layout labels, and ADR 0053 item 1 (a) checks the layout label of `control`, not `format_version`. Two labels are two
  checks on open; both must be no newer than the program.
- Writing META is an ordinary WAL entry; the write frequency is no higher than the frequency of changes to the roots, `page_count`, the free list.
- A format upgrade on start can be long at 10^8 records (Q-42): the time estimate is for the release that changes the format.
- Q-99 (binding the shutdown marker to the medium) is not introduced by ADR 0054 item 4; there is no room for it in META.

## Sources
- ADR 0013, 0014, 0015, 0016, 0028 (items 5–7, 10), 0045, 0051, 0053, 0054, 0057; `docs/source/brief.md`, section 2 (CUJ13–CUJ16),
  section 3 ("Scale and reliability", "Interfaces").
- The use of a signature with `\r\n\x1a` is a well-known technique of the PNG format [not verified against the primary source, does not affect the decision].
