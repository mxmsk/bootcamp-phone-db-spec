# 0094. UI: stale data (CUJ12), re-read, vanished record, message texts
- Status: Accepted ([OURS]: items 1, 3, 6); Accepted (OD-6) ([DEFAULT]: items 2, 4, 5)
- Closes Q-36, Q-68; Q-65 — for CUJ12 (CUJ16 and value errors — ADR 0088)
- Label: [OURS] (items 1, 3, 6); [DEFAULT] (item 2 — D-91; item 4 — D-92; item 5 — D-93)
- Date: 2026-10-27
- CUJ: CUJ12, CUJ3, CUJ6, CUJ16

## Context
CUJ12: an action on stale data is not performed; the UI reports «данные устарели, список обновлён» ("data is stale, the list has been refreshed"), re-reads the number
history, offers to retry; the entered name is kept. Not specified: what is re-read on a conflict from by-name results
(Q-36), the reaction to a vanished record and to "unchanged" (Q-68), the verbatim wording of texts (Q-65). UI strings are quoted in Russian, the language of the interface; English glosses are in parentheses.

## Decision
1. [OURS] On `409 version_conflict` (delete, replace) the UI does not consider the action done, shows «данные устарели, список
   обновлён», requests `by-number` and shows the current number history and an offer to retry. A change to any date of the number makes the version stale (ADR 0006 item 5).
2. [DEFAULT] (D-91) From by-number results, the results are replaced by the re-read. From by-words results, the number history is shown in a dialog; in the rows
   of this number deleted records are removed, names and versions are updated, the rest of the results, the order and the pages do not change (the order
   updates on a new search). Alternative: repeat all the results (the position is lost; a new record of the number may not match the query).
3. [OURS] Retrying a delete — the button «Повторить удаление» ("Retry deletion") with the version from the re-read, if the record (number, date) still exists; otherwise «Записи
   уже нет» ("The record is already gone"). Retrying a replace — «Повторить сохранение» ("Retry saving"): a new add with the entered number, name and date; the outcome is decided by the server
   (created, unchanged, a new overwrite prompt with the current name and version). The UI does not compare names itself.
4. [DEFAULT] (D-92) `404 record_not_found` with a current version (practically unreachable from the results) is handled as stale data; `unchanged` is success
   («Запись уже есть, данные не изменились» ("The record already exists, data unchanged")). Alternative: show "not found" — the user has nothing to do with it.
5. [DEFAULT] (D-93) Q-65 texts for the flows: «данные устарели, список обновлён» verbatim (without changing the case of the first letter); the CUJ2/CUJ3 dialog texts
   and the message after saving are in `web-ui-flows`. The CUJ16 refusals and value errors are in ADR 0088 items 5–6 (the scheme "<Action> failed:
   <the brief's reason>", D-81): here only the actions — "Saving" (number check, add, replace), "Deletion", "Search". Texts
   are selected by `code`/`field`/`reason`/`hint`, not by `message`. Alternatives: free-form CUJ12 wording (violates the brief's verbatim wording).
6. [OURS] On `write_outcome_unknown` and a drop after sending, the entered input is kept, the UI reports that the outcome is unknown (text — ADR 0088 item 4), and re-reads the
   number; a retry is safe (ADR 0026 items 5–6).

## Alternatives considered
- Automatic retry of the action after a conflict: violates "offers to retry"; the user does not see the new data.
- Showing the server's `message`: outside the contract and may change (ADR 0062).

## Consequences
- The user sees the current record before retrying, before answering again.
- Q-71: the sources of and exit from the "read-only" and "storage error" states are outside the UI; the UI neither polls them nor shows them in advance.
- Spec: `web-ui-flows`, the requirements "Stale data (CUJ12)", "What is re-read on conflict", "Offer to retry the delete", "Offer to retry the replacement",
  "Record vanished between read and action", "Flow message texts", "Entered input is kept when a write fails".

## Sources
- `docs/source/brief.md`, §2 CUJ12, CUJ16; §3 "Interfaces". ADR 0006, 0011 item 6, 0025, 0026, 0062.
