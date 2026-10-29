# 0045. Indexes: the set, B+-trees, key encoding, format label and rebuild
- Status: Accepted ([OURS]); Accepted (OD-6) ([DEFAULT]: item 6, refusal for a key above the cap)
- Label: [OURS]; [DEFAULT] (item 6 — D-36)
- Date: 2026-10-27
- CUJ: CUJ4, CUJ5, CUJ7, CUJ8, CUJ11, CUJ14, X-1

## Context
Area 1 is accepted: search by number (ADR 0007, 0035), name results in the order "sort key, date DESC, number"
(ADR 0009, 0036) with continuation from any position (ADR 0032), word-prefix search (ADR 0008, 0030, up to 200 words per name),
exact whole-name search (ADR 0031). The engine is written from scratch (ADR 0017): off-the-shelf storages and search engines
are forbidden. We need: the index structure, the keys (Q-109 — prefix-freedom of the sort key; Q-48/Q-103 — a key up to
8 513 bytes and a 16 KiB page), and an index rebuild with the same comparison function (ADR 0039 item 3; Q-82).
The page format, the WAL and snapshots are the file-format subtask; here only the assumptions for it are set (item 9).

## Decision
1. The set of indexes. One source of truth and four derived trees:
   - R — records: the key `NumKey ‖ DateKey`, the value is the name (UTF-8, NFC); for each number with records there is one "header"
     `NumKey ‖ 000000` with the value "number version" (ADR 0027–0029). For a number without records there are no rows.
   - O — order: the key `OrderKey(record)`, the value is the keys of the words of the name (length + bytes, in word order).
   - W — postings: the space 0x01 "word" `0x01 ‖ esc(word key) ‖ 00 01 ‖ OrderKey` (one posting per
     distinct word of the name) and the space 0x02 "whole name" `0x02 ‖ H64(name comparison key) ‖ OrderKey`.
   - D — the word dictionary: `esc(word key)` → the number of postings (u32); inner pages store two subtree aggregates:
     the number of entries and the sum of the counters (item 3).
   O, W, D are derived data: each of their entries is a function of an R record (item 8); R stores the name whole, and the sort key
   and word keys are computable from it by code with the pinned ICU4X (ADR 0039 item 1).
2. The structure is our own B+-tree (one for all four indexes, differing only in keys and values): keys are
   byte strings, the order is bytewise (memcmp; a shorter prefix string comes first). A leaf is sorted entries
   with the key stored as "common prefix with the previous + suffix"; separators in inner nodes are the shortest prefix
   that distinguishes the neighbors; the search "first entry ≥ key" and a forward walk over leaves; everything is read from one snapshot
   (item 9). Insert and delete are in one atomic write together with the other indexes (CUJ14; the WAL is file-format).
   The reason for choosing a B+-tree: the result order (the sort key) and "continuation from any position" are exactly "find
   the first entry after the key and read forward"; a hash table, an LSM and a sorted array with appends do not give this without
   merging levels or rewriting (alternatives below).
3. Aggregates. In D, inner nodes store next to the child pointer: the number of entries in the subtree and the sum of the posting
   counters. Then for a word prefix p, `k(p)` — the number of distinct words — and `C(p)` — the number of postings in the range — are obtained
   by two descents (≈ 2 × depth reads), without reading the range. The aggregates are updated along the insert/delete path, which
   is rewritten anyway; there are no extra pages. D, rather than counters in W: a distinct word is determined simply
   (one D entry), while the "first/last occurrence" in W would have to be found among neighbors.
4. Key encoding (all numbers are big-endian, byte order = the needed order):
   - `NumKey` — 8 bytes: the digits of the number (after "+"), each as a nibble "digit + 1" (1…10), then nibbles 0 up to 16 nibbles.
     A number is ≤ 15 digits, so it always fits; the encoding is one-to-one (0 only in the tail), the byte order equals the
     order of canonical strings by code points, "+7495" before "+74951234567" (`result-ordering`, the number as a string).
     The fixed length removes the question "a number is a prefix of another number" in composite keys.
   - `DateKey` — 3 bytes: `2 967 164 − d`, where `d` is the number of days from 1876-03-07 (0…2 967 163): values 1…2 967 164
     (2^22 > 2 967 164 ≈ 2^21.5; ADR 0024), ascending key = descending date; 000000 is reserved for the number header.
   - `esc(b)` — the bytes of a sort key/word in which 0x00 is replaced by the pair `00 FF`; the terminator is `00 01`.
   - `OrderKey = esc(sort key) ‖ 00 01 ‖ DateKey ‖ NumKey` (the sort key is `write_sort_key_to`, ru,
     parameters of ADR 0037/0038). The bytewise order of `OrderKey` equals the order of the triple (name ru, date DESC, number as a string)
     (ADR 0036) for any sort keys, including when one key is a prefix of another: after the key comes `00 01`,
     and the byte of the long key that follows the prefix is either ≥ 01 (then `00` < the byte) or 0x00 (in `esc` this is `00 FF` > `00 01`).
   - `H64` — a 64-bit stable hash (the algorithm is fixed in v1; the crate is an implementation question, cargo deny); the hash only
     narrows the range, and equality of comparison keys is always checked against the record's name (ADR 0047 item 4).
5. Q-109 is closed: the correctness of composite keys does not rely on "the sort key does not contain 0x00 and is not a
   prefix of another" — this is ensured by `esc` and the terminator (item 4). Measured (F-52): for all 1 112 064 scalar values
   and for all 160 000 pairs from a pool of 400 characters the sort key contains no 0x00 and contains exactly two bytes 0x01 (level
   separators), that is, `esc` is in practice the identity; the property is not promised and is used only as a check in the
   reference test (item 7). The key structure (the position of byte 0x01) is not parsed by the product.
6. A key of any length (Q-48, Q-103). A tree cell payload (the own part of the key and the value) that does not fit the 2 048-byte threshold
   (1/8 of a page) keeps the first 256 bytes in the page and the rest in a chain of overflow pages (the threshold and layout are
   ADR 0041 items 5–6, one threshold for all trees);
   key comparison reads the tail only when the local parts match. The precedent for the approach is SQLite index B-trees (F-53; ours has a stricter threshold — 1/8 of a page, at least 7 cells per page, ADR 0041 item 7):
   "no single key uses more than one fourth of the available storage space on the page and hence every internal page is
   able to store at least 4 keys". Therefore: a sort key up to 8 513 bytes and any mixture of it (the bound for mixed names is not
   proven, Q-103) is stored without truncation and without refusal; the upper bound of a tree key is 65 535 bytes; exceeding it (unreachable
   for valid names: the additive bound of 9 200 per sort key, ×2 after `esc` = 18 400 plus a word ≤ 2 000)
   is a refusal of one operation without changing data and without the server going into "storage error" (a state of the whole server, cleared only by a restart, which user input does not
   cause; the kind of refusal for the client is area 3, D-36), verified by a test on mixtures of long characters on every release and every data change. A typical
   key (a name of 34 bytes, a sort key of 30 bytes, F-52) is 60+ times smaller than the 2 KiB threshold; the overflow is the path of pathological names
   and does not affect the speed of ordinary queries.
7. The index format label. At the root of the indexes (the file-format meta page) lies the label `(the version of the key and value format
   of our code, the icu_collator version, the version of the Unicode/word property tables, the id of the H64 hash)`. On start the server compares the
   label with its own. A mismatch means O, W, D are rebuilt from R before requests are accepted (item 8); R and the version headers do not change.
   A matching label does not cancel the references of ADR 0039 item 5 and the test "byte order of `OrderKey` = the result of `compare()`"
   (ADR 0039 item 9 (d)); the test includes a check that there is no 0x00 in the corpus keys (informational, item 5).
8. A rebuild with the same comparison function (ADR 0039 item 3). Reasons: a change of the key record format in our code,
   a restore, an index error found by a check. The procedure: (a) read R sequentially; (b) for each record
   compute the sort key, the word keys, the comparison key; (c) external batch sorting by the keys of O, W (and D —
   by counting); (d) bulk-load the new trees into free pages; (e) replace the roots and the
   label in one atomic commit. A failure before (e) leaves the previous state; on the next start the server begins again. The result order after
   a rebuild is the same if the comparison function is the same (ADR 0039 item 3); a rebuild is not a way to accept a different ICU4X:
   if after it at least one reference of ADR 0039 item 5 gives a different result, the version is not accepted (ADR 0039 item 6 (d)).
   Estimate at 10^8 records (research/indexes/calc.py, assumptions there): CPU ≈ 1 min on 8 cores, external
   sorting ≈ 1 min at 2 GB/s (both by model, not a measurement); with a margin for real efficiency — on the order of 10–30 minutes
   (an assumption, to be verified on a prototype); during this time the server does not accept requests and writes progress to the operator (Q-82; the message is area 4).
9. Assumptions for file-format (to be agreed): (a) page size P = 16 KiB (brief section 4) or 8 KiB — the ADR 0049 calculations
   are given for both, the size difference ≤ 6 %; the indexes need P ≥ 8 KiB and usable space U = P − 64 bytes of header and
   checksum (ADR 0041: the page and tree header is 40 bytes, the calculation has a margin); (b) overflow pages are a shared file-format capability; (c) a reader sees a consistent snapshot of all trees as of
   the start of the operation, not blocking writes (needed for ADR 0035 item 6 and for the consistency of a results page with the number
   versions, ADR 0029); (d) the write of an operation (R + O + W + D) is atomic, up to ≈ 204 changed leaves per write in the worst
   case (a 200-word name) and ≈ 7 on average (the R record, the header, O, ~2.5 W postings, space 0x02, D); (e) the inner
   pages of all trees are ≈ 115 MiB at 10^8 records (research/indexes/output-2026-10-27.txt) — a page cache of at least
   256 MiB gives "inner levels in memory" (an assumption of the latency calculations); (f) a page carries a checksum, and a page
   read error is a "storage error" (ADR 0014, CUJ14).
10. **Changed by OD-6 (2026-10-28).** The estimates at 10^8 (item 8: CPU ≈ 1 min on 8 cores; item 9 (e): inner pages ≈ 115 MiB) are recomputed proportionally for 10^7: ≈ 6 s and ≈ 11.5 MiB; the number of tree levels is T, to be confirmed with a prototype.

## Alternatives considered
- A composite key without `esc` and a terminator, relying on "the sort key is prefix-free" (the Q-109 proposal): rejected;
  F-50 is a sample, F-52 is an exhaustive search of single characters and pairs, not a proof for all strings, and an error would silently corrupt the
  order; `esc` costs nothing when there is no 0x00.
- Truncating the sort key to a fixed length: rejected by ADR 0036 item 2 and ADR 0022 item 5 (alphabetic equality
  breaks).
- Capping the key length and rejecting names ("too long" by key): changes the `records` contract (ADR 0022 [USER]) and needs
  a question to the owner; page overflow costs only on rare names and creates no question.
- Only a 16 KiB page "with a proven bound on key length" (brief section 4): the bound is unproven (Q-103), and with a key ≤ 8 513
  bytes only 1–2 keys per node would fit in 16 KiB; overflow with a 2 KiB threshold gives at least 7 cells per page without a proof (ADR 0041 item 7).
- An LSM tree or hash indexes: an LSM requires reads of several levels for every "next after the key" and background
  merging, and complicates a single snapshot and the backup slice (CUJ15); a hash is not ordered. A sorted array: insert is O(N).
- A format label only for O, with W and D always rebuilt: rejected, W contains `OrderKey` and depends on the same format.
- Ordinal record labels instead of `OrderKey` in the W postings (a monotonic u64): halves W and the worst case per record from
  ~180 to ~4 KiB, but when the gap is exhausted it requires relabeling neighbors and rewriting their postings in one transaction;
  deferred as a lever if the size target (ADR 0049) is not met on a prototype (new question Q-110).

## Consequences
- Each index is a function of R: the consistency check (the integrity subtask) can recompute the expected set of keys of a record
  and compare it with the tree; restoring an index is per item 8.
- Size and latency are ADR 0049; the algorithms are ADR 0046–0048.
- Q-109 is closed; Q-48 and Q-103 (for storage) are closed: a key of any length, a refusal of one operation only above 65 535.
- Area 4 gets a requirement to show the operator the rebuild progress.

## Sources
- ADR 0007–0009, 0017, 0022, 0024, 0027–0029, 0031, 0032, 0035–0039; `docs/open-questions.md`: Q-48, Q-82, Q-103, Q-109.
- `facts.md`: F-2 (ICU4X keys "presumed invalidated"), F-39, F-45, F-50; F-52, F-53, F-54.
- `research/indexes/keysize` (measurements), `research/indexes/calc.py`.

- **Changed by OD-7 (2026-10-28).** Item 3 (aggregates in the inner nodes of D) does not apply: D is an ordinary tree, and `k(p)` and `C(p)` are computed by a bounded scan that stops at a threshold (ADR 0099).
