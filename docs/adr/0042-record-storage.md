# 0042. Record storage: number, date, name and number version
- Status: Accepted
- Label: [OURS]
- Date: 2026-10-27
- CUJ: CUJ1, CUJ2, CUJ3, CUJ4, CUJ6, CUJ9, CUJ12, CUJ14; X-5; items 1.3, 1.4 of the original wording

## Context
A record is (number, date) → name (ADR 0002, 0020); the version belongs to the number, not the record, and is stored "together with the records" in one
indivisible write to the medium (ADR 0027, 0028 item 6(a), 0029). Search by number reads all records of the number and the version as one
list from one consistent state, up to 2 967 164 records per number (ADR 0035 item 6); search by name takes the version
by point lookup for a results page (ADR 0029, "Consequences"). A copy of the version in every record of the name index is not needed.
The indexes (ADR 0045) have already accepted the keys of the record tree R: `NumKey ‖ DateKey`, the number header `NumKey ‖ 000000`; here are their
byte representation on the page, the values and the invariants.

## Decision
1. [OURS] Tree R (`kind` = 1 in META, ADR 0040) is the only place where the name is stored; the other trees (O, W, D) are
   derived from R (ADR 0045 item 1). The key of an R cell is 11 bytes: `NumKey` (8) ‖ `DateKey` (3).
2. [OURS] `NumKey` is 8 bytes (ADR 0045 item 4): the digits of the canonical number (without "+", 1–15 digits, ADR 0023 item 4), one per
   nibble, high nibble first, digit d → nibble d + 1 (values 1…10), the missing nibbles are 0. The encoding is
   one-to-one with "+" and 1–15 digits, and the byte order matches the order of the canonical strings (a short string before its
   continuation). Example: "+79991234567" (digits 7,9,9,9,1,2,3,4,5,6,7 → nibbles 8,A,A,A,2,3,4,5,6,7,8 and five zeros) →
   `8A AA 23 45 67 80 00 00`; "+1" → `20 00 00 00 00 00 00 00`; "+123456789012345" → `23 45 67 89 A1 23 45 60`.
3. [OURS] `DateKey` is 3 bytes big-endian (ADR 0045 item 4): `2 967 164 − d`, where `d` is the number of days from 1876-03-07 (the zero
   date, F-1): `d` = 0…2 967 163 for the dates 1876-03-07…9999-12-31 (F-7), the key 1…2 967 164. Ascending key is descending date, so
   reading the records of a number in ascending key order gives "date DESC" (CUJ4, ADR 0007 item 2) with no reverse traversal. 24 bits hold up to
   16 777 215. The value 0 is reserved for the number header (item 5). Converting a calendar date to `d` and back is a pure function of the
   Gregorian calendar (as in `records`, ADR 0024); storage does not know the calendar and checks only
   `1 ≤ DateKey ≤ 2 967 164`.
4. [OURS] The record cell: the key from item 1 with `DateKey` ≥ 1; the value is the name as is: NFC, trimmed, UTF-8 (ADR 0003, 0021), from 1
   to 800 bytes, with no separate length field (the length is the cell's `value_len`, ADR 0041); an empty name is not stored. The name
   is stored and returned byte for byte: "the same name" is byte-for-byte equality (ADR 0021).
5. [OURS] The number header: the key `NumKey ‖ 00 00 00`; the value is the ordinal part of the version `S` (ADR 0051 item 7) as a u64 in varint form
   (LEB128, ADR 0041 item 4; 1–10 bytes, no more than 5 for `S` < 2^35). It stands immediately before the records of the number because
   `DateKey` ≥ 1 > 0. The header exists if and only if the number has at least one record; `S` = 0 never occurs in the
   header (0 is reserved for the initial state, ADR 0051 item 7); for a number with no records there is no version data
   (ADR 0027 item 2), and nothing is stored about deleted numbers.
6. [OURS] Indivisibility: add, rename, delete of a record, a change of the number's `S`, creating or deleting the header, and all changes of the
   derived trees are one WAL entry (ADR 0051 item 2; ADR 0029 item 1): "wrote the record, did not write the version"
   is impossible. Deleting the last record of a number also deletes the header in the same entry.
7. [OURS] The other components of the version (the generation `(G, E)`) and the counter `next_S` are not stored in `data`: `(G, E)` and `next_S`
   live in `control` and the WAL (ADR 0051 items 2, 5 (f), 7; ADR 0054 item 5). The observable version is `(G, E, S)`. Consequence: a change of
   generation on crash recovery start and start from backup changes the versions of all numbers without rewriting 10^8 headers
   (ADR 0028 item 6 (d)). The invariant `S` ≤ `next_S` is for the consistency check (`storage-durability`, ADR 0053) against `control` and the WAL.
8. [OURS] Reading. Search by number is one descent through R to `NumKey ‖ 00 00 00` and a sequential read of leaves to the end of the
   number: the header and records lie consecutively (ADR 0029 item 4); for empty results there is no header, and the initial version of the
   generation is returned. The version for a name results page is a point lookup of the header; it is in the same or an adjacent page as
   the record (`NumKey ‖ DateKey`) and is usually read together with it (ADR 0046).
9. [OURS] R invariants for the consistency check (`storage-durability`, ADR 0053): (a) every record has the header of its number; (b) every header has
   at least one record; (c) the header's `S` ≥ 1 and not more than `next_S`; (d) `1 ≤ DateKey ≤ 2 967 164`; (e) the name length is 1…800 bytes, and the name is
   valid UTF-8 in NFC; (f) keys ascend, `NumKey` is valid (nibbles 1…10, then zeros).
10. **Changed by OD-6 (2026-10-28).** The scale is 10^7 records (ADR 0017); the numbers and estimates for 10^8 in this ADR are the history of the calculation, and the current values for 10^7 are in ADR 0044, 0049, 0052, 0058, 0059; nonlinear estimates are recomputed on a prototype (T, `docs/open-questions.md`).

## Alternatives considered
- The version in every record (a copy): a write to a number with N records would rewrite all N on every change (up to 2 967 164 records per
  number, F-7; ADR 0029, "Consequences"). Rejected.
- A separate tree "number → version": two descents instead of one and a risk of desynchronization; a header lying next to the records is read
  by the same leaf (ADR 0045 item 1 accepted the header). Rejected.
- The number as an ASCII string (up to 16 bytes with "+"): twice as much per key in each of the trees (≈ 8 B × 10^8 × 4 trees ≈ 3 GB).
  Rejected (the same as ADR 0045 item 4).
- The number as 7 bytes (a rank in the numbering of strings of length 1…15, 1.11 × 10^15 < 2^50): saves 1 byte per key (≈ 0.4 GB per tree at
  10^8 records), but loses the "short before continuation" order. Rejected.
- The date as 2 or 4 bytes: 16 bits hold 65 535 days (≈ 179 years); a fourth byte is wasteful on every key. Rejected.
- The date ascending in R (`d + 1`, the first edition of this ADR) and a reverse traversal for "date DESC": ADR 0045 item 4 accepted a descending
  `DateKey`, and a direct read in ascending key order gives the needed order. Replaced.
- The full version `(G, E, S)` ≈ 32 bytes in the header: ≈ 24 extra bytes per number, ≈ 1.9 GB at 8 × 10^7 numbers, to no benefit:
  the generation is one per run (ADR 0051 item 7). Rejected.
- The name in separate value pages: a name ≤ 800 bytes is below the spill threshold (ADR 0041 item 5); an extra read per record. Rejected.

## Consequences
- Size estimate (ADR 0044): a record cell is ≈ 3 + 0.125 + 11 + the name length in bytes; a header is ≈ 3 + 0.125 + 11 + 5 bytes.
- A number with many records (up to 2 967 164): the header and records are consecutive in one tree, a direct read; at ≈ 70 B per record this is ≈ 200
  MB of stream from the leaves (ADR 0035 item 6); snapshot retention is ADR 0056.
- The encoding of the number, date, name and header is part of the v1 format: changing the encoding is a format upgrade (ADR 0040 item 4).
- Q-34 (the representation of the version in the API) is not affected: only `S` is stored in `data`.
- There is one representation of the number version: the number header holds `S` as a u64 varint (≤ 5 bytes for `S` < 2^35); the generation `(G, E)`
  (G is 8 bytes, E is 16 bytes) lives only in `control`; the observable version `(G, E, S)` — 32 bytes — exists only in an API response.
  ADR 0051 item 7 and ADR 0049 are aligned with this representation (the ADR 0049 calculation was redone with 5 bytes, ADR 0049 item 1).

## Sources
- ADR 0002, 0003, 0020, 0021, 0023, 0024, 0027, 0028, 0029, 0035 item 6, 0045 items 1, 4, 0051 items 2, 7, 0054 item 5; F-1, F-7 (`facts.md`).
- The size arithmetic is `research/file-format/sizing.py` (output `sizing-output-2026-10-27.txt`).
