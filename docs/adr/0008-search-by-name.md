# 0008. Search by name
- Status: Accepted
- Label: [USER]
- Date: 2026-10-27
- CUJ: CUJ1, CUJ5, CUJ7, CUJ8; item 1.5 of the original wording ("search Phones by Name")

## Context
The text of CUJ5 speaks of substring search. The brief clarifies that section 3 takes precedence over the CUJ text: not
a substring but the start of a word. We need to define what a word is, how the query words combine, what counts as an
empty query and how exact whole-name search works.

## Decision
1. Search by name is by the start of any word of the name (brief §3/Search and sorting; brief §2, last paragraph).
2. All words of the query must match (AND) (brief §3/Search and sorting). The order of query words does not matter; one
   word of the name may cover several query words (OD-3, Q-4).
3. The search is case-insensitive; "ё" = "е" (brief §3/Search and sorting).
4. Word boundaries are strictly per Unicode UAX #29, with no custom exceptions: a hyphen separates words, an apostrophe
   between letters does not ("O'Brien" is one word) (OD-4, item 1; replaces the wording of OD-3, Q-4).
5. The query is trimmed at the edges and converted to NFC, like a name on save (OD-3, Q-15; ADR 0003).
6. An empty query is an error: a query with no words after trimming, including one of only whitespace or only
   punctuation (OD-3, Q-5; OD-4, item 1).
7. "Next" pagination by cursor, without a total result count (brief §3/Search and sorting).
8. Exact search "all records for the whole name", case-insensitive, is kept separately (brief §3/Search and sorting).
   The rules are the same as for the name and word search: trimming, NFC, case-insensitive, "ё" = "е"; an empty query is
   an error (OD-3, Q-15).
9. **Changed by OD-6 (2026-10-28).** The scale is 10^7 records (ADR 0017); the numbers and estimates for 10^8 in this
   ADR are calculation history, the current values for 10^7 are in ADR 0044, 0049, 0052, 0058, 0059; non-linear
   estimates are recalculated on a prototype (T, `docs/open-questions.md`).

## Alternatives considered
- Substring search (the original CUJ5 text): rejected, "not a substring but the start of a word" (brief §2, last
  paragraph; brief §3/Search and sorting).
- An apostrophe as a word separator (package 1 recommendation, Q-4: "a hyphen and an apostrophe separate words"):
  replaced by OD-4, item 1; the boundary strictly per UAX #29 applies, and an apostrophe between letters does not
  separate a word.
- No other alternatives were considered in the sources.

## Consequences
- The word boundary implementation relies on ICU4X (ADR 0010); a fact about how the UAX #29 rules treat an apostrophe
  between letters is not recorded in facts.md (F-2 contains only "compatible with Unicode Standard Annex #29"): Fact
  Grounding before implementation (Q-76).
- Open questions: page size, cursor lifetime, pagination and order of exact search (Q-30); the composition of a row and
  empty results (Q-29); the latency of multi-word search (Q-59); the search target at 10^8 records (Q-47); the choice
  of ICU4X functions (Q-76); the set of "whitespace" characters (Q-84).

## Sources
- `docs/source/brief.md`, section 3 "Search and sorting"; section 2 (CUJ5 and the last paragraph); F-2 (facts.md).
- Owner decision of 2026-10-27 (OD-3): Q-4, Q-5, Q-15. Place of record: docs/owner-decisions.md, OD-3.
- Owner decision of 2026-10-27 (OD-4), item 1: "word boundaries strictly per UAX #29, with no custom exceptions: a
  hyphen separates words, an apostrophe between letters does not ("O'Brien" is one word); a query with no words at all
  (only punctuation or whitespace) is empty, an error". Place of record: docs/owner-decisions.md, OD-4.
