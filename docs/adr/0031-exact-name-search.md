# 0031. Exact whole-name search: rules, order and pagination
- Status: Accepted
- Label: [OURS] (items 1, 3); [USER] (items 2, 4: OD-5, D-9)
- Date: 2026-10-27
- CUJ: CUJ7, CUJ8; item 1.5 of the original wording (closes Q-30 for exact search)

## Context
The brief keeps exact search "all records for the whole name", case-insensitive; ADR 0008 item 8 gives "the same rules",
but it is not said what an empty query means (punctuation without words?), how the result is sorted and whether there is
pagination (Q-30). One name may belong to many records (different numbers, dates), and the count is unbounded.

## Decision
1. A record matches if the comparison key of its name (ADR 0030) equals the key of the query after trimming at the edges.
   There are no words; whitespace inside is significant, as in ADR 0003.
2. An empty query is only one that is empty after trimming. Control characters in the query are not checked (ADR 0034 item
   5): the results are empty. The query "-" is valid and looks for the name "-" ([USER, OD-5, D-9]: exact search has no
   notion of a word, and such names are saved).
3. The order is the same as for word search (ADR 0009, capability `result-ordering`): name by ru, date DESC, number. Names
   with an equal key ("Ann", "ANN") may differ in spelling, so the order by name still applies.
4. Pagination is the same as for word search: a cursor, no total count, page size per ADR 0033 (50 by default, the client
   may set another) (ADR 0032). [USER, OD-5, D-9]

## Alternatives considered
- No pagination (a full list, like search by number): rejected, the number of records of a name is unbounded (common names),
  and the response may be huge; for a number it is bounded by the history of one number. If the owner wants a "full list",
  this is a change only in this search.
- Its own order (for example, only date DESC): rejected, a second order enlarges the contract and the indexes with no
  benefit.
- Treating a query with no words as empty (as in word search): rejected, a name of punctuation is saved and must be found.

## Consequences
- The owner confirmed item 4 (pagination of exact search) and item 2 (punctuation as a query) (OD-5, D-9).
- A requirement for the API: exact search has the same cursor and page as word search.

## Sources
- `docs/source/brief.md`, section 3 "Search and sorting"; ADR 0003, 0008 item 8, 0009; Q-30 (docs/open-questions.md).
