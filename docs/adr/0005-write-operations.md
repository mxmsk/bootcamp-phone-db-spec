# 0005. Write operations: add, rename, delete
- Status: Accepted
- Label: [USER]
- Date: 2026-10-27
- CUJ: CUJ1, CUJ2, CUJ3, CUJ6, CUJ7, CUJ9; item 1.3 of the original wording

## Context
CUJ3 says "we change the data, completely erasing the old", after an overwrite confirmation, while the brief excludes a
silent overwrite. Section 3 of the brief takes precedence over the CUJs and clarifies the mechanism: overwrite is an
explicit rename operation.

## Decision
1. Add only creates a record. If (number, date) already has a different name, it is a conflict error, and the response
   carries the current name and the number version; there is no silent overwrite (brief §3/Operations).
2. Repeating the add of an identical record is a silent success, unchanged (brief §3/Operations).
3. Renaming an existing record is a separate explicit operation; the UI calls it after the confirmation in CUJ3 (brief
   §3/Operations).
4. Renaming a missing record is "not found" (brief §3/Operations).
5. Fixing a name (a typo) in the UI goes only through CUJ3; there is no separate "rename" button in the UI (brief
   §3/Operations).
6. Delete removes one record (number, date) (brief §3/Operations).
7. Deleting a missing record is "not found", symmetric to rename (OD-3, Q-19).
8. Renaming to the same name at the current number version is a silent success, the number version does not change, as
   with a repeated add (OD-3, Q-80). At a stale version it is a conflict with the current version; the version check
   comes first (OD-4, item 7, docs/owner-decisions.md).
9. Overwrite in CUJ3: on "yes" the data is changed and the old name of this record is not kept; on "no" the data is not
   changed and the UI waits for new input (brief §2, CUJ3).
10. **Changed by OD-6 (2026-10-28).** Q-160: add without a date is valid only for a number without records; otherwise
    it is a "number known" conflict with the current number version (ADR 0025). The decision text ([USER]) was not
    rewritten.
11. **Changed by OD-8 (2026-10-28).** Q-180: add without a date, when the number has a record on 1876-03-07 with the
    same name (compared as for an identical repeat of add with a date), is a success "unchanged"; otherwise, when
    records exist, it is "number known" (item 10 is otherwise unchanged).

## Clarification notes (do not change the decision text above)
- Items 3, 6: the date in rename and delete is mandatory, a request without a date is a value error (OD-5, Q-90; ADR
  0025 item 6, ADR 0002 item 4 is narrowed).

## Alternatives considered
- Silent overwrite on add: rejected, "there is no silent overwrite" (brief §3/Operations).
- A "rename" button in the UI: rejected, fixing goes only through CUJ3 (brief §3/Operations).
- A silent success on rename to the same name regardless of version (the wording of OD-3, Q-80 before clarification):
  replaced by OD-4, item 7; the current version is item 8: the version check comes first, a silent success only at the
  current version.
- No other alternatives were considered in the sources.

## Consequences
- A conflict returns the current name and the number version, which the overwrite dialog uses (see ADR 0006).
- Open questions: the check order for a missing record and a stale version (Q-33); responses to successful operations
  (Q-32); the CUJ2/CUJ3 dialogs and declining to enter a date (Q-37); the definition of a "known number" in the UI
  (Q-28); undoing a delete (Q-63); the UI reaction when a record disappears (Q-68); what is re-queried after delete
  (Q-35); what to show after save (Q-61); see `docs/open-questions.md`.

## Sources
- `docs/source/brief.md`, section 3 "Operations"; section 2 (CUJ3, CUJ6, CUJ9).
- Owner decision of 2026-10-27 (OD-3): Q-19 ("not found" for delete), Q-80 ("silent success, the number version does
  not change"). Place of record: docs/owner-decisions.md, OD-3.
- Owner decision of 2026-10-27 (OD-4), item 7: "a conflict with the current version (the version check first), a silent
  success only at the current version". Place of record: docs/owner-decisions.md, OD-4.
