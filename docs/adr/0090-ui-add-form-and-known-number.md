# 0090. UI: add form, detecting a known number, the date field and format
- Status: Accepted ([OURS]: items 1–2); Accepted (OD-6) ([DEFAULT]: item 3)
- Label: [OURS] (items 1–2); [DEFAULT] (item 3 — D-82)
- Closes Q-28; Q-62 — for the date field (format — ADR 0089, `web-ui-shell`)
- Date: 2026-10-27
- CUJ: CUJ1, CUJ2, CUJ3

## Context
CUJ2 requires showing on save that the number already exists and asking for a date. Adding through the API only creates a record and, without
a date, sets 1876-03-07 (ADR 0005, 0002), so adding without a check would silently create a record on 1876-03-07 (Q-28). The
operation has no "add only if the number is unknown" mode; the API cannot be changed. It must also be decided whether the CUJ1 form
has a date field (Q-62; date format — ADR 0089, D-78). UI strings are quoted in Russian, the language of the interface; English glosses are in parentheses.

## Decision
1. [OURS] Before adding without a date, the UI requests `GET /v1/records/by-number` with the entered number: no records — add
   without a date (CUJ1); records exist — a date prompt with the number history (CUJ2). A response is taken as complete only with `complete: true` (`http-api`: a broken stream is 200 without `count` and `complete`): "number unknown" is only `complete: true` and `count` 0; a 200 response without the completeness marker, with records or without, is a check failure, the add-without-date is not sent, and the refusal «Сохранение не выполнено: ошибка хранилища, обратитесь к оператору» ("Saving failed: storage error, contact the operator") is shown (otherwise a broken stream would silently create a record on 1876-03-07 for a known number); a server refusal and a connection drop are also a check failure (texts — ADR 0088 item 4). The check also gives the invalid-shape number error before
   adding. The version for CUJ3 is taken from `name_conflict` (ADR 0060, Q-34), not from this check.
2. [OURS] The race "the number became known between the check and the add" is accepted: the server will perform the add without a date by its
   own rules (a record on 1876-03-07, `unchanged` or a name conflict), the UI will show the outcome with the date from the response. The API gives no atomic
   "check and add"; the record is neither lost nor overwritten (there is no silent overwrite). The residual risk is Q-160.
3. [DEFAULT] (D-82) The CUJ1 form has no date field: it appears only when the number is known (CUJ2) — "CUJ1 without a date". Alternative:
   always show an optional «Дата» ("Date") field (simpler, but CUJ1 and CUJ2 merge; CUJ2 "ask to enter a date").
4. The date format is "YYYY-MM-DD" (ADR 0089 item 1, D-78); date validity is decided by the server, the UI introduces no bounds of its own.
5. **Changed by OD-6 (2026-10-28).** Q-160: item 1 is replaced — the UI sends the add without a date immediately, without a `by-number` request; whether the number is known is determined by the server with a 409 `number_known` response, after which the UI requests the history (accepted only with `complete: true`) and shows the date prompt; item 2 — the race is closed by the server, there is no residual risk Q-160. The extra read is now only for a known number. The original items 1–2 are not rewritten (decision history).
6. **Changed by OD-8 (2026-10-28).** Q-180: retrying an add without a date after an unknown outcome — if the number has a record on 1876-03-07 with the same name, the server replies 200 `unchanged`: the UI shows «Запись уже есть, данные не изменились» ("The record already exists, data unchanged") without a date prompt and without `by-number`; otherwise, if records exist — 409 `number_known`, as in item 5.

## Alternatives considered
- Add immediately and determine "known" from the response: does not work, a `201` response on 1876-03-07 does not distinguish "the number was known"
  (Q-28).
- Always add with an explicit date: breaks CUJ1 ("without a date"), requires guessing a date.
- A new API parameter "only if the number is unknown": the API does not change (area 5 boundaries); it stays an open question Q-160.

## Consequences
- Every add without a date costs one extra read (a short read, ADR 0056).
- Name errors for a known number are visible only after the date is entered (the add checks the name) — acceptable.
- Spec: `web-ui-flows`, the requirements "Add form: fields and the date field", "Known number is determined by the server response",
  "Date entry".

## Sources
- `docs/source/brief.md`, §2 CUJ1, CUJ2; §5 ("date format in the UI"). ADR 0002, 0005, 0060; `http-api`, "Adding a record".
- facts.md F-1 (the date 1876-03-07).
