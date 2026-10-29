# 0007. Search by number
- Status: Accepted
- Label: [USER]
- Date: 2026-10-27
- CUJ: CUJ1, CUJ2, CUJ4, CUJ6, CUJ7, CUJ8; item 1.4 of the original wording ("search Names by Phone number")

## Context
CUJ4 requires that search by number returns only an exact match and a list of everyone to whom the number was added, by
date DESC.

## Decision
1. Search by number is exact match only; the result is all records of the number, date DESC; a full list without
   pagination (brief §3/Search and sorting).
2. A number not found is an empty list, not an error (OD-3, Q-6).
3. A number of invalid shape is a format error; the rules are the same as on save (OD-3, Q-6; ADR 0004).
4. A number written differently is one number after cleaning, so searching for it finds the same records (OD-3, Q-23;
   ADR 0004).
5. **Changed by OD-6 (2026-10-28).** The scale is 10^7 records (ADR 0017); the numbers and estimates for 10^8 in this
   ADR are calculation history, the current values for 10^7 are in ADR 0044, 0049, 0052, 0058, 0059; non-linear
   estimates are recalculated on a prototype (T, `docs/open-questions.md`).

## Alternatives considered
- Search by part (a fragment) of a number: rejected, "for a number we return only an exact match" (brief §2, CUJ4; brief
  §3/Search and sorting).
- Pagination of by-number results: rejected, "a full list without pagination" (brief §3/Search and sorting).
- No other alternatives were considered in the sources.

## Consequences
- Open questions: showing past, current and future owners (Q-2); how a number looks in results (Q-26); the composition
  of a result row (Q-29); the search latency target at 10^8 records (Q-47, ADR 0019); the internal representation of a
  number (Q-75).

## Sources
- `docs/source/brief.md`, section 3 "Search and sorting"; section 2 (CUJ4).
- Owner decision of 2026-10-27 (OD-3), Q-6: "a number not found is an empty list, not an error; a number of invalid
  shape is a format error (the same rules as on save)". Place of record: docs/owner-decisions.md, OD-3.
