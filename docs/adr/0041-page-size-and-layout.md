# 0041. 16 KiB page, page layout and spilling of large keys (overflow)
- Status: Accepted (OD-6)
- Label: [DEFAULT] (item 1, D-20; item 5 — refusal for a key above the cap, D-36); [OURS] (items 2–8)
- Date: 2026-10-27
- CUJ: CUJ1, CUJ2, CUJ3, CUJ5, CUJ7, CUJ14, CUJ16; X-1

## Context
Brief section 4 carried over a product owner decision made under the old constraints: "a 16 KiB storage page with a proven bound on
key length" (ADR 0019 item 2, Q-48). For the model (number, date) → name the bounds became: a name is up to 800 bytes of UTF-8; the comparison
key is up to 1 000 bytes (200 × 5, proven, F-39); the sort key is 8 513 bytes on homogeneous names (U+FDFA ×200,
an exhaustive search of all characters, F-39) and up to ≈ 9 200 by the additive estimate (200 × 46), and on mixed names the bound is not proven
(F-48, Q-103). Truncating a key is forbidden (ADR 0036 item 2: "alphabetically equal"), there is no "key does not fit" error kind in the
`records` dictionary, and adding one would change visible behavior. An additional measurement by this subtask
(`research/file-format/keysize`, F-60): for realistic names (22 characters) the sort key averages 35 bytes (1.58 bytes
per character), maximum 61; and for mixtures of "long" characters the key length is not monotonic: alternating two characters gave a key 398
bytes longer than either character alone (2 432 versus 2 034). So a proven upper bound cannot be relied on, and the capacity must be at least as large as any plausible one.

## Decision
1. [DEFAULT, D-20] The page size is 16 384 bytes, constant in v1, recorded in the header (`page_size`, ADR 0040). The value is the same
   as in brief section 4; the rationale changes: the "proven bound on key length" is replaced by item 5 (overflow), and the page
   stops limiting the length of names and keys. The question to the owner is in the area 2 package together with the size target (ADR 0044 item 6, D-21). By the index calculation an 8 KiB page
   gives a file at most 6 % larger (ADR 0045 item 9 (a): +0.1 GB on ≈ 39 GB of indexes, ADR 0049 item 1), and the height
   of the trees at 10^8 records is the same (4 levels).
2. [OURS] Page types (byte 4 of the common header): 0 — invalid (all zeros: an unallocated or zeroed
   region of the file; a reference to such a page is corruption), 1 — META (the header, ADR 0040), 2 — tree leaf, 3 — tree inner
   page, 4 — overflow page, 5 — free page list page (ADR 0044). A type outside the set is corruption.
3. [OURS] The common page header, 32 bytes, little-endian:

   | Offset | Size | Field |
   |---|---|---|
   | 0 | 4 | `checksum` — the page CRC-32C (ADR 0043) |
   | 4 | 1 | `page_type` |
   | 5 | 1 | `flags`, zeros in v1 |
   | 6 | 2 | `page_layout` — the layout version of this page (1 in v1.0; allows per-page upgrade, ADR 0040 item 4) |
   | 8 | 4 | `page_no` — its own number, equal to its position in the file (page N lies at offset N × 16 384) |
   | 12 | 4 | reserved, zeros |
   | 16 | 8 | `page_lsn` — the number of the change that last wrote the page (its meaning is durability) |
   | 24 | 2 | `cell_count` — the number of cells (for types 2, 3; otherwise 0) |
   | 26 | 2 | `cells_start` — the offset of the lower bound of the cell area |
   | 28 | 4 | reserved, zeros |

   The page number is u32: a file of up to 2^32 × 16 KiB = 64 TiB (arithmetic). A page whose `page_no` is not equal to its position
   is invalid (written to the wrong place).
4. [OURS] The layout of a tree page (types 2 and 3), followed by an 8-byte tree header: `level` u8 (0 — leaf), `reserved` u8,
   `restart_count` u16, `link` u32 (leaf: the next leaf or `0xFFFFFFFF`; inner: the rightmost child). Then
   an array of `u16` slots — offsets of the restart-point cells, ascending by key; the cells lie from the end of the page downward.
   Usable capacity is 16 384 − 40 = 16 344 bytes. A cell:
   `prefix_len` (varint) — the number of key bytes shared with the key of the previous cell;
   `tail_len << 1 | spill` (varint) — the length of the own part of the key and the spill flag;
   for an inner page `child` u32 and `agg` — `agg_len` bytes of subtree aggregates (the length from the tree descriptor, ADR 0040;
   0 for R, O, W; 16 for the dictionary D, ADR 0045 item 3); for a leaf `value_len` (varint); then the own part of the key and the value.
   The key = the first `prefix_len` bytes of the previous key + the own part. A restart point is a cell with
   `prefix_len` = 0; every 16th cell must be one, as well as a cell with a spill and the one following it (so that restoring
   a key does not require reading the overflow). Search is binary over the slots, then linear over at most 16 cells.
   The varint type is LEB128 (7 bits per byte, the high bit means "continues"), no longer than 5 bytes for u32 and 10 for u64.
5. [OURS] Moving large values out. The payload of a cell is the own part of the key and the value together (in the order tree O the value is the keys of the words of the name,
   up to 200 words, ADR 0045 item 1). If it is no
   larger than 2 048 bytes (1/8 of a page), the cell lies in the page entirely. Otherwise (`spill` = 1) the page holds the first 256
   bytes of the payload, `total_len` u32 and `first_overflow` u32 — the number of the first overflow page; the rest is in a chain of
   pages of type 4. A key and value of any length up to 2^32 − 1 bytes are stored entirely, without truncation (the tree key ceiling is set by ADR 0045 item 6:
   65 535 bytes; exceeding it is unreachable for valid names; a key above the ceiling rejects one operation and does not put the server into "storage error"). Key comparison
   goes first over the bytes lying in the page and goes to the overflow only if they did not decide the order.
6. [OURS] The overflow page: the common header, `next` u32 (the next page of the chain or `0xFFFFFFFF`), `data_len`
   u32, data — up to 16 344 − 8 = 16 336 bytes. A chain belongs to one cell; when a cell is deleted, the chain's pages
   are returned to the free list (ADR 0044).
7. [OURS] The capacity invariant: any tree page fits at least 7 cells of the largest size without a spill
   (7 × (2 048 + up to 10 bytes of cell header) = 14 406 < 16 344; an 8th is not required to fit), so splitting a page is always
   possible, and any tree has at least two cells per page. A consequence for the data: a name (≤ 800 B) and a comparison key
   (≤ 1 000 B) with an 11 B tail always lie in the page; the sort key of realistic names (35–630 B) does too; only
   keys longer than ≈ 2 000 B go to the overflow (names where the weight of a character is ≳ 10 bytes, like U+FDFA, 42.6 bytes per
   character).
8. [OURS] Verification: capacity tests are mandatory in every release — names "x × 200" for every character x (F-39, maximum
   8 513 bytes), names of alternating pairs of "long" characters and random mixtures (F-48; `research/file-format/keysize`);
   insertion, search, deletion and key order (the equality "byte order of the key = order of names": F-45) are checked on
   keys of length from 0 to at least 16 000 bytes. Proving an upper bound of the sort key is not needed for the storage to work
   (Q-103, the storage side is closed by this item).
9. **Changed by OD-6 (2026-10-28).** At 10^7 records the tree height is expected to be one level lower (3 at 16 KiB; T — confirm with a prototype); the choice of the 16 KiB page size (D-20) does not change.

## Alternatives considered
- A 32 KiB page without overflow: the largest observed key (8 513 + tail) fits 3 times, but the additive estimate
  of 9 200 is on the edge, and there is no proof; the cost is reading and writing 32 KiB for every point lookup and twice the
  write per change; the bound still remains "not proven" (Q-103). Rejected.
- An 8 KiB page: a spill threshold of 1 KiB leaves the comparison key (≤ 1 000 + 14) only 10 bytes of headroom, and every insert
  of a record with a spilled key goes through the overflow more often; there are twice as many pages (4.7 million versus 2.3 million per 38 GB),
  twice as many inner nodes; the gain is reading 8 KiB. The choice between 8, 16 and 32 KiB is not decided by calculation
  (the difference is moderate: the tree height at 10^8 records is 4 levels at 8 and 16 KiB, 3 at 32 KiB; rough arithmetic from the branching
  width 263 / 527 / 1 054 with a separator of ≈ 24 B); without a measurement on the target medium no choice other than keeping the owner's
  number is justified. The OS page here is 16 KiB (`getconf PAGESIZE`, arm64 macOS; on x86-64 it is 4 KiB), and 16 KiB is a multiple of both.
- The key always outside the page (the pages hold only a reference and a hash): every key comparison is a visit to another page;
  a double read on every descent step. Rejected; spilling only for keys larger than 2 KiB.
- Truncating the sort key to the page size: violates ADR 0036 item 2. Rejected.
- A "name does not fit" error: changes visible behavior, is absent from the `records` dictionary; the owner confirmed the cap of 200
  (ADR 0022). Rejected.
- Reducing the name length cap to a "provable" one: a question to the owner (ADR 0022); not needed while storage fits any length.
- Constant pages of variable size (by tree type): complicates the free page map and copying; there is no gain.

## Consequences
- A sort key of any length is stored entirely; the load on a page does not depend on the name, and the storage restriction is lifted;
  on exceeding any bound the storage does not create a "too long" error.
- A tree on a 16 KiB page: leaf fill under random inserts in the model is 0.67–0.71 (`sizing.py`, part A,
  F-61); the size calculation uses this value (ADR 0044).
- The rule "no more than 2 KiB per cell in a page" does not limit indexes: a cell larger than 2 KiB goes to the overflow. There is one threshold
  for all trees: 2 KiB with 256 bytes in the page (ADR 0045 item 6 refers to it). The "quarter of a page" (≈ 4 KiB) with which
  ADR 0045 began is replaced by the 2 KiB threshold: it gives at least 7 cells per page (with a quarter threshold, 4) and better
  fill with large cells; the SQLite precedent (F-53) is kept as a precedent of the approach, and its guarantee of "at least 4 keys on
  an inner page" is met with margin. The tree key ceiling is 65 535 bytes (ADR 0045 item 6).
- The 32-byte page header together with the 8-byte tree header is 40 bytes: the index calculations took 64 bytes (ADR 0045 item 9 (a),
  ADR 0049), that is, with margin. The `checksum` field is computed when the page is written and does not take part in WAL changes, and `page_lsn`
  after a WAL entry is applied equals its LSN (ADR 0051 item 2).
- Writing a 16 KiB page is not atomic on the medium (a sector of 512 or 4 096 bytes; not verified for the target medium); protection
  against a torn page is provided by `dwb` (ADR 0051 item 5 (b)); the format provides detection (the checksum, ADR 0043) and `page_lsn`
  (WAL replay only onto pages with a smaller `page_lsn`, ADR 0051 item 5 (d)).
- For indexes (ADR 0045): the format provides prefix compression and restart points; the rules of splitting, merging and choosing
  separators are ADR 0045 item 2. The minimum page fill under deletes is not part of the format (Q-115).
- Q-48 is closed by this ADR; Q-103 is closed on the storage side.

## Sources
- ADR 0019 item 2, 0022 item 5, 0036 item 2, 0039; `docs/open-questions.md`, Q-48, Q-103.
- F-39, F-45, F-48, F-60, F-61, F-63 (`facts.md`); `research/file-format/keysize/`,
  `research/file-format/sizing.py`, output `research/file-format/sizing-output-2026-10-27.txt`.
