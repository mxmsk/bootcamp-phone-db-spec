# 0049. Targets for index size and search latency at 10^8 records
- Status: Accepted (OD-6)
- Label: [DEFAULT] (items 2–5: D-23, D-24, D-25, D-26; item 1 — the calculation for D-21)
- Date: 2026-10-27
- CUJ: CUJ4, CUJ5, CUJ8, CUJ15, X-1

## Context
Brief section 4: "file ≤ 40 GB at 10^8 records" and "search p99 ≤ 10 ms" were computed for the "name–phone pair" model; the owner
confirmed them when choosing word-prefix search; they must be recomputed for the model (number, date) → name (Q-47). Brief section 5: the latency
of multi-word search is not set (Q-59). Q-105: full keys in the order index — "41–63 GB"; Q-106: the work cap.
The calculation is `research/indexes/calc.py`, output `output-2026-10-27.txt`; the measurements are `research/indexes/keysize` (F-52).
All the numbers below are a model with the listed assumptions, not a measurement of the full engine: it is verified with a prototype (a requirement).

## Calculation assumptions (agreed with file-format)
- N = 10^8 records; the reference corpus: average name length 34 bytes of UTF-8 (measured for Russian full names: 33.7 bytes, sort
  key 30.2 bytes, 12 names — F-52; the sort key is ≈ 1.7 bytes per character for Cyrillic and Latin, 4.4 for Han
  characters), 2.5 words per name, 1.25 records per number, 10^7 distinct words; the number version in the header is the ordinal part S, a varint ≤ 5 bytes (ADR 0042 item 5; the generation (G, E) is not
  stored in the header, ADR 0051 item 7); the first edition of the calculation took 32 bytes (G, E and S together), the recalculation is
  `research/indexes/output-version5-2026-10-27.txt`.
- A 16 KiB page (or 8 KiB), a 64-byte header (ADR 0041: actually 40, with margin); fill 0.69 (random inserts) and 0.90 (after a rebuild);
  common-prefix compression of keys: a saving of 25 / 45 / 60 % of the sort key (an estimate from sorted neighbors, not measured
  on a real corpus — there is no real corpus of names).
- Hardware: an NVMe SSD, a random page read of ≈ 100 µs at queue depth 1, up to 8 in parallel, sequential read
  ≥ 2 GB/s, ≥ 8 cores; the inner levels of the trees (≈ 115 MiB) in the cache; merge/check ≈ 100 ns per record (measured on an arm64 dev machine:
  a heap merge 32–95 ns per key, sorting 35–95 ns per key for random 40-byte keys — `bench-output-2026-10-27.txt`;
  for real keys with common prefixes a comparison is more expensive, a margin was taken).
  These are assumptions about the test bench, not verified facts; no targets are set for HDD and SATA.

## Decision (defaults, put to the owner)
1. Index size. The `calc.py` calculation at 10^8 (16 KiB page; number version ≤ 5 bytes; the recalculation command:
   `sed 's/version_b=32/version_b=5/' calc.py | python3 -`, output `output-version5-2026-10-27.txt`): 38.9 GB with random inserts
   (a range of 36.9–41.5 GB by prefix compression of 25–60 %) and 29.8 GB (28.3–31.8) after a rebuild; R 8.1 + O 9.5 + W and exact
   search 20.9 + D 0.2 + inner 0.1 GB. The first edition with a 32-byte version gave 42.0 GB (40.0–44.6) and 32.2 GB. An 8 KiB
   page: +0.1 GB. Sensitivity (random inserts, compression 45 %): a name of 68 bytes — 62.7 GB; a name of 20 bytes — 29.2 GB;
   1.5 words per name — 33.2 GB, 4 words — 47.3 GB; 1 record per number — 39.3 GB. The file size target at 10^8 records is one
   [DEFAULT] D-21 (ADR 0044 item 6: no more than 64 GB for the whole `data` file under profile P0); the numbers of this item are the share of the indexes and
   the input for recomputation when the layout changes, not a separate target. The discrepancy with the ADR 0044 calculation (61.2 GB without compression for P0) is
   explained there: the profile (a name of 34 versus 42.1 bytes) and the assumption that the word key in the W postings compresses to ≈ 2 bytes at any
   compression level. The upper bound per record is the worst case (200 Han words) ≈ 180 KiB of indexes, 200 characters of U+FDFA ≈ 26 KiB:
   a database of such names is larger than the target, and with a full disk it is "read-only" (CUJ16; ADR 0053 item 5). Q-105: the claim "40 GB is unreachable without key
   compression" is wrong for typical names (a sort key of 30 bytes, not 410–630: those are for 200-character names), but right for long ones (68 bytes —
   62.7 GB). Levers if the target is not met on a prototype: ordinal labels instead of `OrderKey` in the postings (≈ −35 % W, Q-110),
   dropping the "word keys" value in O.
2. Latency [D-23] (p99, one query on an unstressed database, 10^8 records, the test bench above; "model" is the `calc.py` calculation):
   - Search by number up to 100 records: ≤ 10 ms (brief section 4 kept; model ≈ 0.2 ms).
   - Any page of exact search and word search where the rarest word has ≤ 10^4 postings (the first and following):
     ≤ 20 ms (model 2–7 ms; dominated by ≈ 50 reads of R, with parallel fetching of 8 — ≈ 1 ms).
   - Any page where the planner chooses G1/G2 at `C ≤ 10^7`, or a single prefix of any frequency: ≤ 50 ms (model 1–22 ms;
     the heaviest is the G1/G2 boundary, `C ≈ 1.3·10^5`).
   - Heavy cases — two or three frequent words with a distant or empty intersection (G3): ≤ 2 s (model ≈ 0.8 s at `C = 3·10^6`
     each); a direct 10 ms promise for them is infeasible.
   - A number with 2 967 164 records: a stream of ≈ 12 000 pages, ≈ 0.4 s on the storage side (model), and there is no target for the whole results (the network determines it).
   - A page with `k > K1` for all prefixes and an empty intersection: a full walk over O ≈ 14 s — the largest work of a query, beyond the target of any class; there is no deadline (item 3).
3. The work cap [D-24] (Q-106): the number of query words is not limited; there is no search deadline upon exhausting which it ends with an error (the first edition introduced a 5 s deadline and a new
   error; this would change the accepted `name-search` and X-1 "no refusals caused by volume"). A valid query always ends with a result; the work is bounded by the walk over O (up to ≈ 14 s at 10^8 records;
   a heavy query does not block writes or other readers). The request timeout from brief section 4 (10 s), if the API introduces one, is area 3 and its visible behavior. Alternatives: a search deadline with the error "search
   deadline exhausted" (MODIFIED `name-search`, a question to the owner); a cap of 32 on the number of words (changes ADR 0022 [USER]). A default, awaiting the owner.
4. Backup [D-25] (Q-59, CUJ15): no refusals; the allowed latency growth is up to 2 × the target of item 2 (a default; the backup snapshot is
   the backup subtask).
5. Index rebuild on a label change [D-26] (ADR 0045 item 8, ADR 0058): blocks start, and the server does not accept requests until it finishes. The estimate of a full rebuild of O, W, D at 10^8 records is ≈ 10–30 min (the `calc.py` model: ≈ 1 min of CPU on 8 cores and
   ≈ 1 min of external sorting at 2 GB/s, that is, ≈ 2 min, with a margin of 5–15 times for real efficiency; the order index
   alone — from 20 s to 2.5 min, ADR 0058 item 5); the target is no longer than 60 minutes. The frequency is only on a change of the key record format in
   our code (not on an ordinary update, ADR 0058). Alternatives: a background rebuild without downtime (two key code formats,
   duplicate writes during the build; more complex, deferred); a target without a number. A question to the owner: is a start downtime of up to
   60 min acceptable on a format change.

## Alternatives considered
- Keeping "p99 ≤ 10 ms" for all search: feasible for number search and for a lookup with ≤ 10^4 postings with parallel reads and
  cache, but not for frequent words and heavy intersections (hundreds of thousands of pages); the promise would be false.
- Setting targets only in page reads: independent of the test bench, but unclear to the owner; both are given (the page count is in ADR 0047, 0048 and in `output`).
- Targets for HDD: not set; random reads of ≈ 10 ms make ≈ 50 R reads per page ≈ 0.5 s.

## Consequences
- X-1 and CUJ5 "Name search latency" (Q-47, Q-59) get verifiable numbers after the owner's decision; the scale test
  (Q-53) must measure the query classes of item 2, not a single "p99".
- A prototype on 10^8 synthetic records (keys with a real word distribution, prefix compression) confirms or changes the sizes.

## Sources
- `docs/source/brief.md` sections 4–5; ADR 0017, 0019, 0022, 0045–0048; `docs/open-questions.md` Q-47, Q-59, Q-105, Q-106.
- `research/indexes/calc.py`, `output-2026-10-27.txt`, `output-version5-2026-10-27.txt`, `keysize/` (F-52, F-54).

- **Changed by OD-6 (2026-10-28).** N = 10^7 (OD-6). The index sizes (item 1) are recomputed proportionally: 3.9 GB with random inserts (a range of 3.7–4.2 GB by compression of 25–60 %) and 3.0 GB (2.8–3.2) after a rebuild; R 0.8 + O 0.95 + W and exact search 2.1 + D 0.02 + inner 0.01 GB; sensitivity — the same ratios. The D-23 latency targets are the same numbers (10 ms, 20 ms, 50 ms, 2 s) at 10^7; the class boundaries (10^4 postings, C ≤ 10^7, G1/G2 at C ≈ 1.3·10^5, 3·10^6 per word) were not recomputed — T, to be confirmed with a prototype. The O walk is ≈ 1.4 s (was ≈ 14 s). The rebuild (item 5): estimate 1–3 min, target ≤ 6 min (was 10–30 and ≤ 60 min). D-23…D-26 accepted by the owner (OD-6).
