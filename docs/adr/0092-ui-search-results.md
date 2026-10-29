# 0092. UI: results by number and by name: row, empty results, "Next"
- Status: Accepted ([OURS]: items 5, 7); Accepted (OD-6) ([DEFAULT]: items 1–4, 6)
- Label: [OURS] (items 5, 7); [DEFAULT] (item 1 — D-84; item 2 — D-85; item 3 — D-86; item 4 — D-87; item 6 — D-88)
- Date: 2026-10-27
- CUJ: CUJ4, CUJ5, CUJ6, CUJ13

## Context
The brief specifies a "list" and a button on the left; the row composition, empty results, the display of the date 1876-03-07 and of owners are not specified (Q-29, Q-1,
Q-2). Results by number are all records without pagination; by name — pages with a cursor (ADR 0061). UI strings are quoted in Russian, the language of the interface; English glosses are in parentheses.

## Decision
1. [DEFAULT] (D-84) Row: on the left a "delete" button with a trash can icon, then the name, the number (canonical, "+" and digits, ADR 0060 item 4), the date
   "YYYY-MM-DD"; the same in both result sets; the full name, as text only. Alternatives: the number once in the header of the by-number results
   (fewer repeats, two row kinds); the date as the first column (more convenient for history, worse for by-name results).
2. [DEFAULT] (D-85) Empty results are neutral text («Записей с номером … нет» ("There are no records with number …") / «По запросу «…» ничего не найдено» ("Nothing found for the query «…»")), not an error (ADR 0034).
   Alternative: only an empty list without text (the user cannot tell it from loading).
3. [DEFAULT] (D-86) The date 1876-03-07 is like any date, with an optional hint «дата по умолчанию для записей, добавленных без даты» ("the default date for records added without a date").
   An explicitly entered one and a substituted one are indistinguishable (ADR 0002). Alternative: show «без даты» ("no date") instead of the date — wrong for explicit input.
4. [DEFAULT] (D-87) The results do not distinguish past, current and future owners and do not show a "current owner": there is no "who owned on date D" operation
   and it is not introduced (ADR 0007). Alternative: a "in the future" mark by the browser clock — introduces a meaning the owner did not specify,
   and depends on the time zone.
5. [OURS] The UI has only word-prefix search (`by-words`); the exact whole-name search is API only: the brief's CUJ7 calls it
   an API addition to "the same operations as the UI" (ADR 0011 item 4).
6. [DEFAULT] (D-88) «Дальше» ("Next") appends the next page below those shown, without page numbers and a total count; when `next_cursor` is `null` there is no
   button. Alternative: page replacement with a «Назад» ("Back") button (a cursor is a position, not a page number, ADR 0061; "Back" would require storing cursors).
7. [OURS] The row order is as in the server response; the UI does not re-sort (ADR 0036) and does not hide repeats of a record across pages (ADR 0032):
   on a stale row, deletion will reach CUJ12.

## Alternatives considered
- A single search with auto-detection (number or name by a first "+" character): a name query may start with "+" after punctuation —
  parsing is ambiguous; an explicit choice of search kind was selected (element placement — `web-ui-shell`).

## Consequences
- A number with a very large number of records (up to 2,967,164, `number-search`) is returned without pagination: receiving and showing without truncation is the question Q-163.
- Spec: `web-ui-flows`, the requirements "Search by number (CUJ4)", "Result row", "Date 1876-03-07 and number owners in the results", "Empty results",
  "Name search: by word prefix, no exact search in the UI", ""Next" pagination", "Row order matches the server response", "Name-search request errors" (error texts — ADR 0088 item 6).

## Sources
- `docs/source/brief.md`, §2 CUJ4–CUJ6, CUJ7; §3 "Search and sorting". ADR 0002, 0007, 0011, 0032, 0034, 0036, 0060, 0061.

- **Changed by OD-7 (2026-10-28).** "Consequences" (Q-163 closed): the UI guarantees responsiveness of the number history up to ≈ 10^4 records, beyond that it is not guaranteed (ADR 0100).
