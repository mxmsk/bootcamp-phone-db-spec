# 0009. Result order of name search
- Status: Accepted
- Label: [USER]
- Date: 2026-10-27
- CUJ: CUJ5

## Context
CUJ5 requires sorting results alphabetically, then by date DESC. Cursor pagination needs a total order of records.

## Decision
1. Name-search results are sorted alphabetically under ru locale rules, then date DESC (brief §3/Search and sorting).
2. With equal name and date, then by number ascending (OD-3, Q-25), by the number's notation as a string ("+19…" before
   "+2…"), so numbers of one country are adjacent (OD-4, item 6).

## Alternatives considered
- Order by number "ascending" without specifying the comparison method (package 1 recommendation, Q-25): clarified by
  OD-4, item 6, comparison by the number's notation as a string.
- No other alternatives were considered in the sources.

## Consequences
- The comparison parameters (strength, role of case, placement of "ё", alternate_handling) were chosen by no one: the ru
  locale in CLDR sets only the order of scripts ("[reorder Cyrl]"), the presence of ru data in ICU4X is not directly
  confirmed, and the default strength is Tertiary (F-2). Open question Q-31.
- The mechanics of the cursor and the total order are Q-73 (T). Extending the order to exact whole-name search and
  pagination is Q-30.

## Sources
- `docs/source/brief.md`, section 3 "Search and sorting"; F-2 (facts.md).
- Owner decision of 2026-10-27 (OD-3), Q-25: "then by number ascending". Place of record: docs/owner-decisions.md, OD-3.
- Owner decision of 2026-10-27 (OD-4), item 6: "by the number's notation as a string ("+19…" before "+2…"), so numbers
  of one country are adjacent". Place of record: docs/owner-decisions.md, OD-4.
