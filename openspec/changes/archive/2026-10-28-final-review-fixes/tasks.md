## 1. Delta specs

- [x] 1.1 Findings of the data group (`docs/reviews/review-data.md`, 1–3): `user-journeys`, `write-operations`, `records`, `http-api`; check: retrying an add without a date gives "number known", a rename to the same name gives "unchanged".
- [x] 1.2 Findings of the storage group (1–9): `storage-format`, `storage-durability`, `storage-snapshots`, `storage-indexes`; check: `next_S` is strictly greater than the largest S, a zero page after growth is accepted, the commit number is defined.
- [x] 1.3 Findings of the server group (1–9): `http-api`, `api-access`, `server-runtime`, `records`; check: 401 closes the connection, `null` is equivalent to absence, shutdown rejects records not taken by the writer.
- [x] 1.4 Findings of the admin-ui group (1–6): `operator-control`, `operator-lifecycle`, `server-runtime`, `web-ui-shell`, `web-ui-flows`; check: no contradictions between the texts "no connection" / «ответ получен не полностью» ("response received incompletely").
- [x] 1.5 Open-question placeholders and "Open: Q-x" in `user-journeys` replaced with the accepted values; marks of waiting for the owner removed.
- [x] 1.6 Purpose (direct edit): 10^8 → 10^7 in `storage-format`, `storage-indexes`, `operator-lifecycle`; the placeholder mechanism removed from the `user-journeys` Purpose.
- [x] 1.7 `openspec validate final-review-fixes --strict`; check: "is valid".

## 2. ADRs and open questions

- [x] 2.1 ADR 0095; "Changed by the final fix pass" in ADR 0053, 0054, 0057, 0062, 0077, 0080; index `docs/adr/README.md`.
- [x] 2.2 `docs/open-questions.md`: closed Q-166 (a)–(c) marked; Q-167 (T) added; the totals line updated.
- [x] 2.3 grep for placeholders and waiting marks in `openspec/specs` and in the change; check: leftovers are listed in the report.

## 3. For the orchestrator

- [ ] 3.1 Do not archive until the orchestrator decides; do not change `journal.md`.
