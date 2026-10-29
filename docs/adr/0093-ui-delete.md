# 0093. UI: delete, confirmation, cancel, re-query
- Status: Accepted ([USER]: item 1, item 2 — confirmation before deletion; [OURS]: items 5–6); Accepted (OD-6) ([DEFAULT]: item 2 — dialog, item 3, item 4)
- Label: [USER] (item 1; item 2 — confirmation before deletion); [OURS] (items 5–6); [DEFAULT] (item 2 — dialog text, buttons, focus — D-94; item 3 — D-89; item 4 — D-90)
- Date: 2026-10-27
- CUJ: CUJ6, CUJ12

## Context
CUJ6 specifies a button on the left with a trash can icon, confirmation and a re-query of the list. Not specified: what exactly is re-queried (Q-35),
what "cancel deletion" means (Q-63, brief §5), how the date and version are passed (OD-5, Q-90). UI strings are quoted in Russian, the language of the interface; English glosses are in parentheses.

## Decision
1. [USER] On the left of the result row (by number and by name) there is a «удалить» ("delete") button with a trash can icon (brief §2, CUJ6).
2. [USER] Confirmation is requested before deletion; until then the record is not deleted (brief §2, CUJ6).
   [DEFAULT] (D-94) The form of the confirmation is the author's choice, not the owner's decision: a page dialog «Удалить запись: {имя}, {номер}, {дата}?» ("Delete the record: {name}, {number}, {date}?") with the buttons «Удалить» ("Delete") and «Отмена» ("Cancel");
   initial focus on «Отмена» ("Cancel"), Esc equals «Отмена». Rationale: named buttons and a safe focus prevent deleting by an accidental Enter. Alternatives:
   focus on «Удалить» ("Delete") (faster, risk of accidental deletion); confirmation by a second click without a dialog (rejected below).
3. [DEFAULT] (D-89) "Cancel deletion" is declining in the confirmation dialog: the dialog closes without requests or messages, the list does not change. Undo
   of a performed deletion is not introduced: there is no such operation (brief §3, "Out of scope"; ADR 0005), it would require restoring the record
   with a new version and rules for concurrent changes. Alternative: undo for a limited time (a new API operation, a question for the owner).
4. [DEFAULT] (D-90) After `200` the UI repeats the original results: by number — the same `by-number`; by words — the same query from the first page
   for as many pages as were shown, then «Запись удалена» ("Record deleted"). Alternatives: only from the first page (the place in a
   long result set is lost); remove the row locally without a request (the brief requires a re-query).
5. [OURS] The delete request carries the row's `number`, `date` and `expected_version`; the UI does not substitute dates (OD-5, Q-90; ADR 0025 item 6).
6. [OURS] Repeated clicks are blocked until the response; the unavailability of the re-query does not cancel the deletion: the row is removed locally,
   «Запись удалена, но список обновить не удалось» ("Record deleted, but the list could not be refreshed") and «Обновить» ("Refresh") are shown.

## Alternatives considered
- `window.confirm`: the look depends on the browser, no control of focus and button names; a page dialog is needed.
- Confirmation by "clicking twice" without a dialog: does not match "we request confirmation".

## Consequences
- A by-words re-query costs up to N page requests (N is the number of pages shown; a page is ≤ 100 records, ADR 0033).
- Spec: `web-ui-flows`, the requirements "Row "delete" button (CUJ6)", "Delete confirmation", "Cancel in the delete dialog",
  "Delete execution", "Re-query after delete".

## Sources
- `docs/source/brief.md`, §2 CUJ6; §3 "Operations"; §5. ADR 0005, 0025, 0033, 0061; OD-5 (Q-90).
