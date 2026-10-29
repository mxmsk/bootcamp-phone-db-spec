# 0033. Page size of name-search results
- Status: Accepted
- Label: [USER] (OD-5, D-10)
- Date: 2026-10-27
- CUJ: CUJ5, CUJ7, CUJ8

## Context
The page size is not set (Q-30). The value is visible to clients in both the UI and the API (whether the client may choose
is the API area).

## Decision
The default page size is 50 records. If the interface lets the client set a size, the page contains the specified number
of records (valid values and the maximum are the API area). A page is full (contains as many records as specified, or 50 by
default) if there are more records after it; shorter only for the last one. The promise "a page of 50" is not
unconditional: it applies when the size is not set. Wordings "up to 50 records" in other documents are read as "up to the
page size (50 by default; the maximum is the API area)".
- **Changed by OD-6 (2026-10-28).** The scale is 10^7 records (ADR 0017); the numbers and estimates for 10^8 in this ADR
  are calculation history, the current values for 10^7 are in ADR 0044, 0049, 0052, 0058, 0059; non-linear estimates are
  recalculated on a prototype (T, `docs/open-questions.md`).

## Alternatives considered
- 20: rejected, more requests when browsing common names.
- 100 and more: rejected, a long list in the UI; for programs the size can still be chosen through the API (if the API area
  allows it).
- No limit: rejected, the number of matches for a short prefix can reach 10^8 (CUJ5, X-1).

## Consequences
- The response of one page is bounded. The speed of the first page is not the invariant "does not grow with the number of
  matches" (for two frequent words with a distant intersection and for a short prefix the cost is determined by the number
  of elements traversed, not by the number of matches), but a latency target (p99, Q-59) that the storage area must
  describe together with the worst case.
- If the API lets the client choose the size, 50 is the default; the maximum is a decision of the API area. The cursor does
  not depend on the size (ADR 0032 item 6).
- The way to get the first page of a short prefix (for example, "a", up to 30% of records) is not semantics but a
  requirement for storage: the result order (ADR 0009) does not coincide with the order of the word index, so storage must
  have a strategy by selectivity (traversing the name index in result order with a match check, or selecting a small set
  with sorting).

## Sources
- `docs/source/brief.md`, section 3 "Search and sorting"; Q-30, Q-59.
