# 0002. Record model: (number, date) → name
- Status: Accepted
- Label: [USER]
- Date: 2026-10-27
- CUJ: CUJ1, CUJ2, CUJ3, CUJ4, CUJ9, CUJ12; item 1.3 of the original wording ("store Names and Phones")

## Context
The original wording requires storing names and phone numbers and searching in both directions. CUJ2 speaks of "owners
by date": one number may belong to different people in different periods. The brief fixes the record model and the role
of the date.

## Decision
1. Record = (number, date) → name. For one date a number has exactly one owner (brief §3/Data).
2. The same name may own the same number on another date; that is a separate record (brief §3/Data).
3. The date is a calendar day; any date not earlier than 1876-03-07 is valid, future dates are allowed (brief
   §3/Data). There is no product upper bound (a consequence of the wording "future dates are allowed", brief §3/Data).
4. The date in a CUJ is UI behavior. If the request to the server has no date, the server substitutes the zero date
   1876-03-07 (brief §3/Data). The zero date is the issue date of Bell's patent US 174,465 (F-1).
   Note (OD-5, Q-90; the [USER] text above was not changed): item 4 is narrowed for rename and delete, where the date
   is mandatory; substituting the zero date remains only for add (ADR 0025 item 6).
5. Consequence of items 3–4: the minimum valid date equals the zero date, so an explicitly entered 1876-03-07 and one
   substituted by the server are the same record (brief §3/Data; CUJ1 does not ask for a date, CUJ2 does).
6. **Changed by OD-6 (2026-10-28).** Add without a date substitutes 1876-03-07 only if the number has no records;
   otherwise it is a "number known" conflict (Q-160, ADR 0025 item 3). Rename and delete are unchanged (OD-5, Q-90).
   The text of item 4 ([USER]) was not rewritten.
7. **Changed by OD-8 (2026-10-28).** Add without a date when writing to 1876-03-07 with the same name is "unchanged"
   (item 6 is otherwise unchanged; ADR 0025 item 9).

## Alternatives considered
- An interval ownership model ("who owned on date D", "current owner"): the brief has no such operation, and it is not
  introduced; the result is a list of records by date DESC (brief §3/Data, §3/Search and sorting).
- A different zero date: F-1 notes that H.Res.269 (2002) on Meucci does not establish the date the telephone was
  invented, so the only basis is the patent issue date.
- No other alternatives were considered in the sources.

## Consequences
- The version, search and write operations are built around the key (number, date): ADR 0005, 0006, 0007.
- Open questions: showing the date 1876-03-07 in results (Q-1), showing past/current/future owners (Q-2), the date
  format in the UI (Q-62) and in the API (Q-67), the technical limit of the date's upper bound (Q-64), how the UI
  determines that a number is already known (Q-28); see `docs/open-questions.md`.

## Sources
- `docs/source/brief.md`, section 3 "Data" (owner decisions [USER]); section 2 (CUJ1, CUJ2).
- F-1 (facts.md, verified 2026-10-27): US Patent 174,465, patent issued 1876-03-07 (wording "Granted: March 7, 1876",
  from the page summary at https://patents.google.com/patent/US174465A/en); H.Res.269 does not establish the date.
