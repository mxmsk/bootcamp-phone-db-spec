# 0048. Word-prefix search: postings, dictionary, planner, work cap
- Status: Accepted
- Label: [OURS]
- Date: 2026-10-27
- CUJ: CUJ5, CUJ8, CUJ12, X-1

## Context
A record is found if for each word of the query there is a word of the name whose key starts with the query word key (ADR 0008,
0030); one word of the name may cover several query words; up to 200 words per name and per query (ADR 0022 item 4). The results are in the order of
ADR 0036, a page is full while records remain (ADR 0033), the first page without full materialization of matches, and continuation
without reading earlier pages. Q-59, Q-106: targets and the work cap; the worst cases are two frequent words with a distant intersection,
and a short prefix.

## Decision
1. Indexes: W (space 0x01: the posting `word key ‖ OrderKey` for each distinct word of the name; the postings of a word are ordered
   by `OrderKey`), D (word → number of postings; `k(p)`, `C(p)` for a prefix — per ADR 0045 item 3), O (the value is the word keys of
   the name). Words and word keys are functions of the name per ADR 0030; a repeated word in a name gives one posting.
2. Query → the set of word keys → removal of redundant ones: a key `a` that is a prefix of another query key `b` is redundant
   (a word starting with `b` also starts with `a`); what remains is the antichain `P = {p_1…p_m}`. For each `p_i`, `k_i` and
   `C_i` are taken from D (≈ 2 × 3 reads, with the inner levels cached — 2 reads). A record matches ⇔ for every `i` the name has a word with prefix
   `p_i`. From here everything is in the order of `OrderKey`, strictly after the cursor position (ADR 0047 item 2); there is no state between pages.
3. Three sources of candidates; the planner chooses by estimates (the constants are tuning, not contract):
   - G1 "bounded posting merge". Take `p*` with the smallest `C*`. If `C* ≤ M1` (default 2^17 = 131 072),
     the W range of this prefix is read entirely (the postings of several words of the prefix are `k*` sorted runs;
     a heap merge, with postings having `OrderKey` ≤ the cursor position and record repeats discarded). The other `p_i` are checked:
     if `C_i ≤ M1`, the same way in memory (an intersection of sorted streams); otherwise by the record's word keys from
     the O value (one read by the posting's `OrderKey`). Memory is at most M1 postings, not the number of matches.
   - G2 "O walk". If `C* > M1`: read O forward from the cursor position and on each record check all `p_i` against the word
     keys in the value (without reading other pages and without parsing the name). Expected to scan `50 × N / C(intersection)`
     records. The budget is S_max = 2^20 scanned records; if the page is not filled, the search continues with source G3 from
     the position of the last scanned record.
   - G3 "posting intersection". For prefixes with `k_i ≤ K1` (default 1 024) — a merge cursor over `k_i` W runs;
     a leapfrog intersection: take the largest current `OrderKey`, advance the other cursors to it
     (the search "first ≥" or a forward read along the current leaf, whichever is cheaper); prefixes with `k_i > K1` are checked against the O value.
     The cost is ≤ the sum of the list lengths `C_i` (less with leapfrogging).
   The planner estimates the cost in page reads: G1 — the pages of the range `C*/289` plus the number of O checks (at most
   `C*`, one random read each, if there is a prefix with `C_i > M1`); G2 — `50/ρ` records, where `ρ = ∏(C_i/N)` (word
   independence is a rough estimate, needed only to choose the first source); G3 — `Σ C_i` postings. The cheaper one is started; when
   the budget is exceeded it switches. The result does not depend on the choice, only the cost changes.
4. The first page without full materialization: G2 and G3 are streaming; G1 materializes at most M1 postings of one prefix and
   stops as soon as a page and the "more records exist" marker are collected (one more is read). Continuation is the same
   algorithm with the cursor position, without reading earlier pages; repeated G1 work on every page is up to M1 postings
   (up to ≈ 5 MiB) — the price of a stateless cursor.
5. Checking a record before returning it is a read from R (name, version, date) under the same snapshot (ADR 0046 item 4); the returned name is taken
   from R, and a posting only points to the record.
6. The work cap (Q-106). The number of query words is not limited, either visibly or internally: redundant prefixes are dropped
   (item 2), the planning cost is ≈ 2 × depth reads per distinct prefix (200 prefixes — up to ≈ 400 D leaf reads with
   a cold cache), and checking a candidate is a comparison against ≤ 200 word keys in memory. The work cap per query is not the number of words but
   the walk: there is no search deadline upon exhausting which it answers with an error — the accepted `name-search` has no such error, and X-1 requires "no refusals caused by volume";
   a valid query always ends with a result (a full page or the end of the results), and the work is bounded by the walk over O (≈ 14 s at 10^8 records, item 8). A request timeout, if the API introduces one, is
   area 3 (ADR 0049 item 3, D-24).
7. A short prefix. 1–2 characters ("а"): `C` is large (the share of records ≈ 10 % and higher), and under G2 500–1 000 O records are expected — 2–6 leaves;
   the heaviest single prefix is at the G1/G2 boundary (`C ≈ M1`): ≈ 450 pages sequentially (G1) or ≈ 215 (G2),
   ≈ 10–20 ms (ADR 0049). A rare short prefix ("ъ") is G1.
8. Q-59 worst cases (the calculation is `research/indexes/output-2026-10-27.txt`): two frequent words with an empty intersection
   (`C = 3·10^6` each): G3 ≈ 2·10^4 pages and ≈ 6·10^6 steps, ≈ 0.8 s; all prefixes with `k > K1`, an empty intersection: a full
   walk over O ≈ 5.6·10^5 pages and ≈ 14 s — the largest work of one query (item 6); reachable only with several very frequent and mutually
   exclusive prefixes, and the residual risk is stated in ADR 0049.
9. Scripts without spaces (up to 200 words per name, ADR 0022 item 4): 200 words, 200 postings per record; writing such a name is
   ≈ 180 KiB of indexes (an order key of ≈ 900 bytes in every posting, calculation); a query of 200 distinct Han characters is 200 prefixes
   of length 1, planning ≈ 450 reads, G1 on the rarest character (the others checked against the word keys in the O value).
   The size per record is the price of variant B (ADR 0030, D-14, confirmed by the owner); the lever is ordinal labels (ADR 0045, Q-110).
10. **Changed by OD-6 (2026-10-28).** The work is bounded by the walk over O: ≈ 1.4 s at 10^7 records (recomputed proportionally from ≈ 14 s at 10^8); the planner constants (M1, S_max, K1) are verified at 10^7.

## Alternatives considered
- Full materialization and sorting of all matches: rejected (the requirement "without full materialization", ADR 0032).
- Only a walk over O with checking: reasonable for frequent prefixes, but for rare words it would scan all of O (10^8 records, ≈ 14 s).
- Only posting merge: for 1–2 character prefixes `k` is on the order of 10^4–10^5 runs; reads of `k` leaves instead of 2–6.
- Fixed-length prefix postings (1–3 characters) for frequent prefixes: multiply W by 3 (≈ +26 GB), rejected on size.
- A cap on the number of query words (for example, 32, the Q-106 proposal): a visible change (ADR 0022 [USER]); not needed — the work
  is set by dropping redundant prefixes and by the walk over O.
- A Bloom signature of the other words in the posting, cutting candidates without reading O: +0.5 GB, deferred as an optimization (an implementation question).
- A dictionary D with exact record lists (a record id instead of `OrderKey`): loses the order without additional reads.

## Consequences
- Latency and size: ADR 0049. There is no search deadline and no new error; a heavy query may exceed the API request timeout (area 3).
- The planner and its constants (M1, S_max, K1) are implementation tuning, verified by a load test at 10^8 records.
- The search result does not depend on the choice of source: one test "all sources give the same result" (a requirement).

## Sources
- ADR 0008, 0022 item 4, 0030, 0032, 0033, 0036, 0045–0047; `openspec/specs/name-search`; `research/indexes/{calc.py,keysize}`.

- **Changed by OD-7 (2026-10-28).** Items 1–3, 10: D without aggregates (`k`, `C` — a bounded scan up to the thresholds K1, M1); sources G2 and G3 are introduced first, G1 by profile; the result does not depend on the source, and the strategies are cross-checked by a differential test (ADR 0099).
