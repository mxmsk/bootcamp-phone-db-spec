# 0044. Free space, file growth and size at 10^8 records
- Status: Accepted (OD-6)
- Label: [OURS] (items 1–5, 7–9); [DEFAULT] (item 6 — D-21; item 10 — D-22)
- Date: 2026-10-27
- CUJ: CUJ14, CUJ15, CUJ16; X-1; item 1.6 of the original wording

## Context
Brief section 4: the target "file ≤ 40 GB at 10^8 records" was computed for the "name–phone pair" model; the owner confirmed it when choosing
word-prefix search; it must be recomputed for the new model (Q-47, ADR 0019 item 1). The reviewer (Q-105) estimated the order index at
10^8 records at 41–63 GB from sort keys of 410–630 bytes (F-39). The measurement by this subtask (F-60): 410–630 bytes is the key of names
of 200 characters; for realistic names of 22 characters the key averages 35 bytes (1.58 bytes per character). The index layout is set by ADR 0045
(trees R, O, W, D); this ADR computes the file size from it and from the page format (ADR 0041–0042) and is parametrized
(`research/file-format/sizing.py`, F-61, F-62). ADR 0049 item 1 (indexes) gives the index size calculation and refers
to the target of this ADR (item 6): for the owner there is one number (D-21).

## Decision
1. [OURS] Free pages are a chain of list pages (type 5, ADR 0041); the head and the count are in META (`freelist_head`,
   `free_page_count`, ADR 0040). A list page: the common header, `next` u32 (@32; the next list page or `0xFFFFFFFF`),
   `count` u32 (@36), then 12-byte elements (@40): `page_no` u32, `freed_at` u64 — the LSN of the WAL entry of the operation that freed
   the page (ADR 0051 item 2). The capacity of a list page is 1 362 elements ((16 384 − 40) / 12).
2. [OURS] Issuing a page. A page from the list is issued for new data if there is no open snapshot (reading a number history,
   ADR 0035 item 6; a backup slice, ADR 0057; reading a results page) whose published LSN (the LSN of the last
   WAL entry visible to the snapshot, ADR 0055 item 3) is less than `freed_at`: such a snapshot still sees its contents. Otherwise the page
   waits and the file grows (item 3). The comparison is by LSN, not by commit number N: the LSN continues across restarts, so
   the `freed_at` of an earlier run is less than the published LSN of any new snapshot (there are no snapshots from an earlier run, ADR 0055 item 8) and
   such pages are available immediately. The list covers pages taken out of the trees by an operation (a merge, deleting an overflow
   chain, a swap of roots on rebuild); earlier versions of pages changed in place are not counted by the list
   (ADR 0056 item 1). The durability of the freeing
   operation is not checked separately: an operation is published only after it is durable (ADR 0055 item 4). Changes to the free
   list go into the same WAL entry as the tree change that caused them (ADR 0051 item 2): a page
   is never both in a tree and in the list, nor in neither.
3. [OURS] File growth is in chunks of 4 096 pages (64 MiB): when there is no suitable free page, the file is extended by one
   chunk and `page_count` grows by the issued pages. The extended area reads as zeros (type 0, invalid) and cannot
   be addressed until the page is written with a valid sum. A 40 GB file is about 600 extensions over its whole life
   (arithmetic), not one per operation; an extension is flushed together with the checkpoint data (ADR 0050).
4. [OURS] Out of space. Being unable to extend the file or write a page — `ENOSPC` and the like — leads to "read-only" or
   "storage error" per ADR 0053 items 5–6 (the operation is not performed, there is no partial result, ADR 0029 item 1; search works
   as long as it can, ADR 0014 items 6–7). The client does not learn in advance that space is running out (CUJ16); the operator gets a message (Q-66).
5. [OURS] There is no file shrinking: freed space is reused, the file does not shrink automatically,
   and there is no "shrink the file" operation in v1 (X-3). It is allowed to return the free tail of the file to the OS if `page_count` is reduced by a WAL
   entry; it is not required. The file size is a "high-water mark", not the current volume of data.
6. [DEFAULT, D-21] The file size target at 10^8 records: the `data` file (without `wal/`, `dwb` and backups) is no more than **64 GB**
   (64 × 10^9 bytes) for a database of 10^8 records of the reference profile P0 (item 7), instead of "≤ 40 GB" from brief section 4. The default is
   what the calculation sustains under a conservative profile: without relying on unproven key prefix compression and on dense
   loading; at a leaf fill of 0.67 (the lowest in the random insert model) and zero compression, the ADR 0045 layout gives
   61.2 GB (headroom 4.6 %), at 25–45 % compression — 52.8–46.2 GB (headroom 21–38 %, which includes freed, not yet reused
   pages). The calculations of two subtasks: (1) file-format (`sizing.py`, item 8): 61.2 GB without compression; 52.8 / 46.2 GB at 25 / 45 % compression;
   45.5 GB at fill 0.9 without compression; (2) indexes (`calc.py`, ADR 0049 item 1): 38.9 GB at fill 0.69 and 45 % compression
   (36.9–41.5 GB at 25–60 % compression), 29.8 GB after a rebuild. The calculations differ because of the profile (a name of 34 bytes versus 42.1 for P0)
   and the compression assumptions: `calc.py` compresses the word key in the W postings to ≈ 2 bytes at any compression level, while `sizing.py` at zero
   compression stores it whole (16 bytes). 64 GB was chosen by the calculation that does not use these assumptions. The options for the owner and their
   consequences (disk and time are computed for a database at the limit of the target; the backup at 64 MiB/s, D-32; retention at 100 records per
   second for the duration of the backup, ADR 0056 item 7; disk headroom is item 10):

   | Target | What is needed to meet it | Disk: rebuild (up to 1.8 ×) + backup | Backup | Retention during backup |
   |---|---|---|---|---|
   | 40 GB (brief section 4) | fill 0.9 and compression ≥ 25 % (39.3 GB), or ordinal labels instead of `OrderKey` in W (Q-110, −35 % W; not recomputed); with random inserts without this — 52.8 GB | 72 + 40 = 112 GB | 10 min | 7.3 GiB |
   | 50 GB (ADR 0049, first edition) | prefix compression ≥ 33.6 % at fill 0.67, or dense loading (45.5 GB without compression); compression is not measured (Q-111) | 90 + 50 = 140 GB | 12.4 min | 9.1 GiB |
   | **64 GB (default)** | nothing beyond the layout: 61.2 GB without compression at fill 0.67 | 115 + 64 = 179 GB | 15.9 min | 11.6 GiB |
   | 80 GB | headroom for names of about 30 characters at 25 % compression (interpolating the table of item 8) | 144 + 80 = 224 GB | 19.9 min | 14.6 GiB |
   | No number | only the "bytes per record" formula (item 8: 404 B for P0 without compression); X-1 remains without a measurable threshold (Q-104) | not defined | not defined | not defined |

   Retention during a backup within the cap max(1 GiB, 25 % of the data) (ADR 0059 item 2: 14.9 GiB at 64 GB) at a write rate up to ≈ 100 per
   second; at 1 000 per second the backup hits the cap and is revoked for any target (ADR 0056 item 7).

7. [OURS] The reference profile P0 (an assumption, not a measurement on real data; there is no real data): an average name of 22 characters (42.1
   bytes of UTF-8; a set of 2 106 Russian and Latin full names and titles, F-60); a sort key of 35.2 bytes; 2.5 words per name, a word
   is 16 bytes of key (ADR 0022 item 4 estimates 2–4 × 10^8 word occurrences per 10^8 records); 10^7 distinct words (an ADR 0049 assumption);
   1.25 records per number (8 × 10^7 numbers); leaf fill 0.67 (the lowest in the random insert model on a 16 KiB page,
   F-61); inner nodes 1.5 % (branching width ≈ 527). Prefix compression is a parameter, 0 in the estimate without assumptions.
8. [OURS] The calculation (`sizing.py`; a cell = a 3 B header + a 0.125 B slot + the key + the value). Per ADR 0045: R — a record cell
   of 56.2 B + a number header of 15.3 B per record; O — `OrderKey` (sort key, 00 01, date, number) and the word keys in the value, 93.8 B; W —
   a word posting (`0x01`, word key, `00 01`, `OrderKey`) for each distinct word, 175.8 B at 2.5 words, and a whole-name
   posting (`0x02`, H64, `OrderKey`) 60.3 B; D — the word dictionary 2.3 B. Σ = 404 B per record; file = Σ / 0.67 × 1.015 × 10^8.

   | Leaf fill | Without compression, GB | Compression 25 %, GB | Compression 45 %, GB |
   |---|---|---|---|
   | 0.67 (random inserts, the lowest in the model) | 61.2 | 52.8 | 46.2 |
   | 0.71 | 57.7 | 49.9 | 43.6 |
   | 0.90 (dense loading) | 45.5 | 39.3 | 34.4 |

   Sensitivity to name length (characters: GB without compression / compression 25 % / 45 %): 12 — 34.4 / 30.4 / 27.2; 22 — 61.2 / 52.8 / 46.2;
   40 — 112.6 / 94.9 / 80.8; 100 — 352.1 / 285.4 / 232.0; 200 — 1 028 / 812 / 638. The size grows almost linearly with name length and
   word count; the number of records per number has almost no effect (1.0 → 61.8; 10 → 59.1 GB). The order index (Q-105): `OrderKey` in O and W takes ≈ 217 of 404 B (54 %), that is, O with W together, not "41–63 GB" for one
   order index: those values are right for names of 200 characters, and a typical name has a key of 35 bytes.
   The pages at 61.2 GB are 3.7 million; the inner nodes are about 7 thousand pages (≈ 115 MB, as in ADR 0045 item 9 (e)); the upper levels
   fit in the cache.
9. [OURS] The worst case is not bounded by the target: both a name of 800 bytes and a sort key of 8 513 bytes are allowed. The calculation per ADR 0045:
   200 characters of U+FDFA, one word — ≈ 27 KiB of indexes per record; 200 Han words of one character — ≈ 179 KiB (ADR 0049: 26 and ≈ 180
   KiB — consistent). A database of such names is larger than the target, and with a full disk it is "read-only" (CUJ16; ADR 0053 item 5). Consequence for
   the indexes: the size of a record in the indexes depends linearly on the length of its name and the number of words; the lever for compressing W is ordinal labels
   (ADR 0045, Q-110 of the indexes); storage does not impose it.
10. [DEFAULT, D-22] Disk for operations beyond `data`: a backup (CUJ15, ADR 0057) is another `data` size; a rebuild of derived trees on
    a change of the format label (ADR 0045 item 8, ADR 0058) goes into free pages: temporarily another size of O, W, D (P0: ≈ 80 % of the file
    without compression). The disk headroom for operation at 10^8 records of P0 is up to 1.8 × the file size plus the backup. These numbers are an estimate for the operator,
    not an acceptance requirement.
11. **Changed by OD-6 (2026-10-28).** D-21: the `data` file size target is no more than 8 GB (8 × 10^9 bytes) at 10^7 records of the reference profile P0 (instead of 64 GB at 10^8). The calculation of item 8 is recomputed proportionally for 10^7 (without compression): 6.1 GB at fill 0.67 (headroom 23 %); at 25 % compression — 5.3, at 45 % — 4.6 GB; at fill 0.9 without compression — 4.55 GB; by name length without compression (characters: GB) — 12: 3.4; 22: 6.1; 40: 11.3; 100: 35.2; 200: 102.8. Number versions — up to 0.05 GB. D-22: disk for a rebuild and a backup — up to 1.8 × the file size plus the backup: at a cap of 8 GB — up to ≈ 22 GB (14.4 + 8); a backup at 64 MiB/s — ≈ 2 min. The table of target options (item 6) is the history of the choice; profile P0 at 10^7: 8 × 10^6 numbers, 0.37 million pages. D-21, D-22 accepted by the owner (OD-6).

## Alternatives considered
- A free page bitmap (1 bit per page; 64 GB — ≈ 480 KiB): easier to verify "every page is either in use or
  free"; but it does not carry the `freed_at` needed for snapshots (item 2). The list with `freed_at` was chosen; the bitmap is built in memory by the full consistency check.
- Not reusing free pages (only append, shrink by an operation): requires a "shrink" operation, which does not exist (X-3).
  Rejected.
- Finding free pages by a traversal on start: incompatible with the start time (Q-42; ADR 0052). Rejected.
- File growth one page at a time: frequent changes of the file length; in 64 MiB chunks — about 600 extensions per 40 GB.
- A 40 GB target with random inserts: unreachable without measured compression (the calculation of item 8). Left as alternative (b).

## Consequences
- Q-47 (the "file" part) and Q-105 are closed by calculation with assumptions; the part of Q-47 "search p99 ≤ 10 ms" is ADR 0049 and Q-59.
- The target is verified with a test load-generation tool (Q-53, not a product operation): inserts in random order (otherwise
  fill 0.9 understates the result) and the P0 profile is reproduced; prefix compression must be measured on a synthetic corpus.
- The calculations of ADR 0049 item 1 (38.9 GB at fill 0.69 and 45 % compression; with a 32-byte number version it was 42.0) and of this ADR (49.9–43.6
  GB at 0.71 and 25–45 % compression) are consistent within the assumptions (item 6): the profile, the compression of the word key in W. The figure for the owner is
  one — D-21.
- If the index layout changes (ordinal labels, ADR 0045), recompute `sizing.py`.
- The file size limit: 64 TiB with a u32 page number (arithmetic).

## Sources
- `docs/source/brief.md`, section 4 ("Targets file ≤ 40 GB…"), ADR 0017, 0019 item 1, 0022 items 4–5, 0045, 0049, 0051, 0053, 0055, 0057;
  Q-47, Q-105 (`docs/open-questions.md`).
- F-39, F-48, F-60, F-61, F-62 (`facts.md`); `research/file-format/sizing.py`,
  `sizing-output-2026-10-27.txt`, `keysize/`, `keysize-output-2026-10-27.txt`.
