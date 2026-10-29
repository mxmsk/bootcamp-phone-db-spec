# Review of the "data" group

Group: records, write-operations, number-search, name-search, result-ordering, user-journeys.
Result: 3 items, all contradictions. No BLOCKERs.

## 1. Contradiction: retrying an add without a date in CUJ1 and CUJ9
- Where: `user-journeys`, "CUJ1: Adding a new number" -> "Retry of an identical record"; "CUJ9: A network service changes data" -> "Retry of adding an identical record". Against: `write-operations`, "Retrying an operation after an unknown outcome" -> "Retry of an add without a date" and "Adding without a date for a known number" -> "A single record on 1876-03-07 with the same name"; `http-api`, "Adding a record" -> "Retry without a date"; OD-6, Q-160.
- Essence: in CUJ1 the UI adds a number without a date, and the scenario promises that on a retry (for example, after a connection drop) the result is "success, data and version unchanged". Under OD-6 and `write-operations` such a retry gets "number known" (409 `number_known`), and the UI goes to CUJ2 asking for a date for a number the user has just created. In CUJ9 the wording does not mention a date, so that scenario is also wrong for an add without a date.
- Fix: in both scenarios say "add with a date"; add a scenario "retry of an add without a date" with the outcome "number known", after which the client determines the outcome by a search by number. In CUJ1 describe what the UI does if the repeated add received "number known".

## 2. Contradiction: outcome of retrying a rename after an unknown outcome
- Where: `write-operations`, "Retrying an operation after an unknown outcome" (requirement text). Against: "Renaming a record" -> "The same name at the current version"; "When the number version changes"; `http-api`, "Retry after a connection drop".
- Essence: the requirement says without qualification that a retry of a rename with the old version yields "version conflict". If the first attempt replaced the name with the same one, the number version did not change, and the retry gets "unchanged". `http-api` explicitly calls such a retry safe. The second qualification of the same requirement ("if the number's records were changed, an add is decided by key occupancy") is not correct for an add without a date: that case checks whether the number has records, not whether the key is occupied.
- Fix: write "a rename that changed the name is a 'version conflict'; a rename to the same name is 'unchanged'". In the qualification write: "an add with a date is decided by key occupancy, an add without a date by whether the number has records".

## 3. Contradiction: the closed dictionary of value error reasons in `records` versus reasons in `http-api`
- Where: `records`, "Value error kind dictionary": "Other capabilities do not introduce other reason names for the fields number, date, name, query". Against: `http-api`, "Date representation": field `date`, reason `invalid_format`; "Number representation" -> "Number is not a string": field `number`, reason `wrong_type`.
- Essence: `http-api` introduces for the "date" and "number" fields reasons that are not in the `records` dictionary. These are date syntax (not `YYYY-MM-DD`, year not four digits) and a wrong JSON type. By the text of `records` they are forbidden, although the API cannot do without them. In addition, no specification in the group nor `http-api` says what error an add without the `name` or `number` field gets. The only decision is in ADR 0062, item 3: `number` is `invalid_format`, `name` is `empty`.
- Fix: in `records`, allow protocol-level refusals (syntax, type, absent member) with `http-api`'s own reasons, or add `invalid_format`/`wrong_type` to the dictionary for these fields. Record the ADR 0062, item 3 mapping for absent `number` and `name` as a requirement in `http-api`.
