# 0051. Write-ahead log, atomicity of the operation and version, crash recovery
- Status: Accepted
- Label: [OURS]
- Date: 2026-10-27
- CUJ: CUJ1, CUJ3, CUJ6, CUJ9, CUJ12, CUJ14, CUJ15, X-1, X-5

## Context
We need a mechanism under which a write operation (the record, its derivatives in the word, exact name and order indexes, the number
version) is durable with one flush, indivisible on failure, and start after a crash does not depend on the database size (10^8 records, Q-42).
Indexes and pages are the subject of other subtasks of area 2; here a page storage with a page header is assumed
(page number, type, LSN, checksum) — the page size of 16 KiB (ADR 0019 item 2, Proposed) is a parameter P here. The storage format is paged
(ADR 0041), the indexes are B+-trees on the same pages (ADR 0045): the page header carries the number, type, `page_lsn` and CRC-32C
(ADR 0041 item 3), as assumed here.

## Decision
1. [OURS] The mechanism is a write-ahead log (WAL) with replay (redo-only), a data file that
   changes only by checkpoint, and protection of pages from torn writes through a doublewrite buffer. The database files:
   `LOCK` (ADR 0054), `control` (two 4 KiB slots with a checksum and a number), `data` (pages), `dwb` (the doublewrite
   buffer, 128 pages), the `wal/` directory (16 MiB segments named by the starting LSN; segments are created zero-filled and
   flushed together with the directory, ADR 0050 item 3; used ones are renamed and reused).
2. [OURS] A transaction = one write operation = one WAL entry: a header (magic number, length, LSN, WAL epoch,
   `flushed_lsn`, the next value of the version counter `next_S`), a set of page changes (page number, offset, new bytes;
   physiological: the unit is the page, the content is a diff) and the CRC-32C of the whole entry. The LSN is an offset in the logical WAL
   stream. An entry is aligned to 8 bytes. The maximum entry size is 4 MiB (the calculation is in item 8). Changes do not touch the
   `checksum` field (bytes 0…3 of the page): it is computed when the page is written to the file (ADR 0043 item 3); the page's `page_lsn` after
   an entry is applied equals the entry's LSN (ADR 0041 item 3). Either the entry is entirely valid or it does not exist:
   a partial operation after a failure does not exist (ADR 0029 item 1). The number version, the record and the changes of all indexes are in one
   entry (ADR 0028 item 6 (a)).
3. [OURS] Execution: operations are executed sequentially by a single write executor (the version check and the change are one
   action, ADR 0029 items 2–3; reading the needed pages allows read-ahead before the queue); the executor collects changes in
   new page versions in memory, adds an entry to the WAL buffer and hands it to the flushing thread; that thread writes the accumulated
   batch with one `write`, flushes (ADR 0050 item 2) and only then: publishes the new page versions to readers (search reads
   the published LSN) and responds to clients. Pages with an LSN above the durable LSN are not written to the data file (the rule "log
   before data"). Visibility to readers and retention of snapshots for long reads (ADR 0035 item 6) belong to the indexes subtask; here the
   requirement is: the published state is always durable (ADR 0050 item 6). The operation's checks go against the executor's latest state, including the unflushed; a response
   that depends on an unflushed operation waits for its flush (ADR 0050 item 1). Published page versions are immutable: a page in memory is a chain of versions, and earlier ones are needed by snapshots
   (ADR 0056 item 1); the data file does not contain "the last durable version" (it holds what the last checkpoint wrote).
4. [OURS] A checkpoint (background, not on the acknowledgement path): choose dirty pages with durable versions; in batches of 128
   pages write them to `dwb` and flush; write the same pages to their places in `data`; flush `data`; then write into a `control`
   slot the new `redo_start_lsn` (the smallest LSN of the first change not yet reflected in `data`) and flush. The WAL segments
   before `redo_start_lsn` are freed for reuse. Triggered when: the WAL from `redo_start_lsn` is ≥ 256 MiB, or 5
   minutes have passed with dirty pages present, or on clean shutdown (ADR 0054). These two numbers are [OURS], the derivation is in ADR 0052.
5. [OURS] Recovery (every start; after a clean shutdown the WAL past `redo_start_lsn` is empty, and the pass (c)–(d) applies nothing): (a) choose the
   valid `control` slot (the larger number of the two with a valid sum); (b) for every occupied `dwb` slot where the page in
   `data` has a wrong sum and the copy in `dwb` is valid and belongs to the same page, put the copy back (a torn page
   write); (c) scan the WAL from `redo_start_lsn`, checking each entry (the magic number, LSN equals the offset, the epoch is not
   less than the previous one, CRC); (d) apply entries in order to pages whose `page_lsn` is less than the entry's LSN (replay
   is idempotent); (e) at the first invalid entry the scan stops, the tail rule is item 6; (f) restore
   `next_S` as the largest `next_S` among the applied entries and the values from `control`; (g) write into the WAL an
   "end of recovery" entry with a new WAL epoch (epoch = previous + 1; it is written to `control` before operations are accepted, the order with the version generation is ADR 0054 item 5), flush; if writing is impossible (ADR 0054 item 7), the entry is not made and there are no operations.
6. [OURS] A torn tail and corruption. On a cut-off the disk may have kept part of the last batch, and in it later entries without earlier
   ones. Therefore: an invalid entry at position p, after which there is no valid entry with `flushed_lsn` > p (a look-ahead
   of 32 MiB or to the end of the written area), is a torn tail: it is discarded, and the operator is told the number of discarded
   bytes. If such an entry exists, then position p was considered durable (`flushed_lsn` records only the durable boundary):
   this is WAL corruption in the middle, and **the database is not opened** (ADR 0053). An entry from an old epoch (a "ghost" from a torn
   batch of a previous run that landed at the needed offset) is not accepted because of the epoch rule in item 5 (c) and the "end of
   recovery" entry in item 5 (g). The boundary: corruption of the last durable WAL entry, with nothing after it, is indistinguishable from
   a torn tail — outside the guarantee (ADR 0014 item 3: media corruption).
7. [OURS] The number version. It is stored in the number record (in the same page change) as an ordinal value S (u64; in the number header it is a varint, ≤ 5 bytes for S < 2^35, ADR 0042 item 5). S is taken from a
   counter shared by all numbers, `next_S` is increased by 1 on each change of the number's records; S=0 is reserved for the
   initial state. The observable version is the triple (G, E, S), where (G, E) is the run generation (ADR 0054) — one per run, not
   stored with the number. Consequence: a change of generation on crash recovery start and start from backup instantly "devalues" the versions of all
   numbers without changing a single record (ADR 0028 item 6 (d): re-evaluating 10^8 numbers on start is not needed). Cost: ≤ 5 bytes per number, no
   more than 10^8 × 5 = 0.5 GB at 10^8 numbers (a contribution to Q-47; the upper estimate for u64 without compression is 0.8 GB).
8. [OURS] Sizes (an estimate, not a measurement): a typical operation — a record (~80 bytes), an order key (~100 bytes for a name of ~30 characters;
   up to 8 513 bytes for 200 characters, F-39), an exact-name key (~150 bytes), 3–5 word keys (~40 bytes), overhead ~150 bytes: about
   1 KiB; the design budget is 2 KiB. The worst case is a name of 200 characters in 200 words: the insert indexes ≈ 180 KiB (ADR 0049), and a rename is deleting the old and inserting
   the new keys, up to ≈ 204 leaves per write (ADR 0045 item 9 (d)); in "offset + bytes" changes — no more than ≈ 3.3 MiB of leaves (204 × 16
   KiB) plus the inner page paths; the upper limit of an entry is 4 MiB (the first edition took 128 KiB from an estimate of 16–30 KiB without
   the word indexes, and did not cover a 200-word name). At 100 operations per second the WAL grows by ~200 KiB/s (17 GB per
   day), but occupies no more than 256 MiB plus the segments in progress (freed by checkpoints).
9. [OURS] Rollback of a failed batch. The executor's unpublished state (new page versions, issued and freed pages, `next_S`, the commit number) is an overlay on the
   published state. On a write or flush error of the batch the overlay is discarded entirely; all operations executed on top of it after the first failed one are rejected with the same
   refusal and are not acknowledged; the WAL buffer is truncated to the last durable LSN. The next operation is built on the published state, and its entry is written from the last
   durable LSN. This way a checkpoint and recovery produce the same page content, and readers do not see the rejected one. After a failed flush the executor accepts no new operations
   (ADR 0053 item 6).
10. [OURS] A probe. A service WAL entry "probe" (a header and CRC, no page changes, no `next_S`): on recovery it is checked and skipped. It is written and flushed on the
   first write operation in the "read-only" state (ADR 0053 item 8).
11. **Changed by OD-6 (2026-10-28).** The scale is 10^7 records (ADR 0017); the numbers and estimates for 10^8 in this ADR are the history of the calculation, and the current values for 10^7 are in ADR 0044, 0049, 0052, 0058, 0059; nonlinear estimates are recomputed on a prototype (T, `docs/open-questions.md`).

## Alternatives considered
- Copy-on-write (shadow paging, like LMDB; the root version changes in one write): there is no WAL, start is instant, but
  each operation rewrites the path from root to leaf in every index: ~6 index paths × ~4 pages × 16 KiB ≈ 384 KiB per
  operation against ~2 KiB of WAL (~190 times more writing on the acknowledgement path) and two flushes per commit (data, then the root).
  Rejected on write latency (ADR 0052).
- Full page images in the WAL after a checkpoint (like PostgreSQL): they protect from a torn page without `dwb`, but the first
  change of a cold page puts 16 KiB into the WAL; an operation touching ~6 cold pages is ~96 KiB on the acknowledgement path
  (~48 times more than the designed 2 KiB). A doublewrite moves the same cost to the background (doubling the checkpoint volume).
- A logical WAL (the operation "put/delete a record") without page protection: on a torn page write during an index split
  the structure is inconsistent, and logical replay does not repair it. Rejected.
- A WAL without checkpoints (append only, rebuild on start): the start time grows with the size (10^8 records), Q-42.
- Data in place and a flush of every page on commit: random writes of dozens of pages per operation, torn pages. Rejected.
- A change log as a product feature ("who changed what"): out of scope (ADR 0018); this log is a mechanism, does not store
  history, and is freed by checkpoints.

## Consequences
- Q-78 is closed for the mechanics and cost (with ADR 0054): the version = (G, E, S); the cost is ≤ 5 bytes per number (S varint); 32 bytes (G 8 + E 16 + S 8) per number are not
  stored; the WAL write is the same ~2 KiB; on start there are two durable entries (ADR 0054).
- An addition to ADR 0028 item 6 (f): `next_S` is restored from the WAL and `control`, not by traversing numbers.
- For the page format and indexes subtask: a page header with number, type, LSN and CRC-32C is needed; operations on pages
  are expressed as "offset + bytes" changes; reading with publication by LSN (done: ADR 0041 items 3–4).
- Backup: a slice must include `control`, `data`, `dwb` and the WAL with `redo_start_lsn` at one LSN value, or
  be taken after a checkpoint (decided by the backup subtask, ADR 0016); a backup carries the "backup" marker in `control` (ADR 0054).
- CRC-32C: the implementation is a separate ADR about the crate (ADR 0017 item 3); the algorithm and crate are ADR 0043.

## Sources
- ADR 0006, 0014, 0016, 0019, 0027, 0028, 0029, 0035; `docs/open-questions.md`, Q-42, Q-47, Q-78.
- F-64…F-72, F-39 (`facts.md`); the estimates are in this ADR, the calculation in ADR 0052.

- **Changed by OD-7 (2026-10-28).** Item 4: to the WAL threshold and 5 min a third checkpoint trigger is added — the cap on modified pages of 1 GiB (ADR 0097); checkpoints are suspended for the duration of a backup (ADR 0096).
