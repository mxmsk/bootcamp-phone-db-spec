# 0091. UI: date prompt (CUJ2), overwrite prompt (CUJ3), what to show after saving
- Status: Accepted ([OURS]: items 1–4); Accepted (OD-6) ([DEFAULT]: item 5)
- Label: [OURS] (items 1–4); [DEFAULT] (item 5 — D-83)
- Date: 2026-10-27
- CUJ: CUJ2, CUJ3, CUJ12

## Context
Q-37: what the overwrite prompt shows, what stays in the form, whether entering a date can be declined, how an invalid date is
shown. Q-61: what to show after saving (the value is not specified). Q-68: the "unchanged" outcome is not an error. UI strings are quoted in Russian, the language of the interface; English glosses are in parentheses.

## Decision
1. [OURS] CUJ2: the message «Номер {номер} уже есть в базе. Введите дату, с которой номер закрепляется за этим именем» ("Number {number} already exists in the database. Enter the date from which the number is assigned to this name"), the number
   history (already obtained by the check, ADR 0090) and the «Дата» ("Date") field; an empty date is not sent («Дата: укажите дату» ("Date: specify a date"), a text from the value-error table): the date being required in this step is the UI's own rule, the only exception to "the UI applies no rules of its own about validity"; the server does not require a date for adding. Declining is the «Отмена» ("Cancel") button
   or changing the number: nothing is saved, the number and name remain. Date errors are at the field, by the reasons `out_of_range`,
   `no_such_day`, `invalid_format`.
2. [OURS] CUJ3: on `409 name_conflict` the dialog «На дату {дата} у номера {номер} уже записано имя «{current_name}». Заменить его на
   «{имя}»? Старое имя будет стёрто» ("On date {date}, number {number} already has the name «{current_name}» recorded. Replace it with «{name}»? The old name will be erased") with the buttons «Да, заменить» ("Yes, replace") and «Нет» ("No"). The date is the one sent in the add (for an add without a date — 1876-03-07).
3. [OURS] "Yes" — `rename` with `number`, `date`, the entered `name`, `expected_version` from the conflict body; resending is blocked until the
   response. "No" — no request, the form (number, name, date) is kept, focus in the name field. `unchanged` is not an error: «Запись уже есть,
   данные не изменились» ("The record already exists, data unchanged") (one text for this outcome everywhere).
4. [OURS] What was entered in the form is kept on any failure; cleared only after success.
5. [DEFAULT] (D-83) After success: the message «Сохранено: {имя}, {номер}, с {дата}» ("Saved: {name}, {number}, from {date}") (values from the server response), the form is cleared, the date field
   is hidden, the number history is shown below (a search by number); the message stays until the next action. Alternatives: only the
   message (no confirmation "found by number", CUJ1); keep the entered values for the next input (risk of resubmission);
   hide the message automatically via a timer (the user may not have time to read it).
6. **Changed by OD-6 (2026-10-28).** Q-160: the overwrite dialog (item 1) is only for an add with a date; an add without a date for a known number gives 409 `number_known` and a date prompt (ADR 0090), so "for an add without a date — 1876-03-07" does not occur in the dialog.

## Alternatives considered
- Dialog text without the current name: the user does not see what exactly will be erased.
- Taking the version for "Yes" from a new read: an extra request, hides a concurrent change that CUJ12 requires to show.
- A «Сохранить без даты» ("Save without a date") button for a known number: CUJ2 requires a date; the user can enter a record on 1876-03-07 explicitly.

## Consequences
- The dialog opens only on a server response; the UI does not compare names itself (normalization is on the server).
- Spec: `web-ui-flows`, the requirements "Date prompt for a known number (CUJ2)", "Overwrite prompt (CUJ3)", ""Yes" answer to overwrite", ""No" answer to overwrite", "What to show after saving".

## Sources
- `docs/source/brief.md`, §2 CUJ2, CUJ3; §5. ADR 0005 item 9, 0006, 0025, 0026, 0060 (Q-34); `http-api`, "Renaming".
