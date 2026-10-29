## 1. Delta specs

- [x] 1.1 Scale 10^7 (OD-6 item 1): `user-journeys` (X-1, CUJ1, CUJ4, CUJ5, CUJ14), `storage-format`, `storage-indexes`, `storage-durability`, `storage-snapshots`, `server-runtime`, `operator-lifecycle`; check: the delta has no active requirements with 10^8.
- [x] 1.2 Body cap of 32 KiB (OD-6 item 2): `http-api`, `server-runtime`, `operator-lifecycle`, `records`; check: the scenario "20,000 whitespace characters at the edges" is replaced by a 413 refusal before parsing, the 200-character cap is unchanged.
- [x] 1.3 One version per number (OD-6 item 3, Q-128): `write-operations`, `number-search`, `http-api`, `web-ui-flows`; check: in number-search results the version appears only at the root, in name-search results it appears in every record.
- [x] 1.4 Add without a date (OD-6 item 4, Q-160): `records`, `write-operations`, `http-api`, `web-ui-flows`, `web-ui-shell`, `user-journeys`; check: `number_known` 409 with `version`, the CUJ1/CUJ2 race is closed by the server.
- [x] 1.5 `openspec validate apply-owner-decisions-od6 --strict`; check: "is valid".

## 2. ADRs and open questions

- [x] 2.1 ADRs: a "Changed by OD-6" item on the changed ones, statuses Proposed → Accepted (OD-6), index `docs/adr/README.md`; check: no Proposed in the index.
- [x] 2.2 `docs/open-questions.md`: D-20…D-94 and (O) accepted (OD-6), new values D-21…D-24, D-26…D-28, D-32, D-35, D-43, D-52, D-61, D-82; Q-128, Q-160 resolved; Q-165, Q-166 (T); counters "Total after OD-6"; check: 75 marks "Accepted by the owner, OD-6".
- [x] 2.3 `grep -rn "10^8\|10⁸\|256 КиБ" openspec/specs docs/adr`; check: after the delta is applied, the remaining matches are history (ADR context, "Changed by OD-6" notes), listed in the report.

## 3. For the orchestrator

- [ ] 3.1 Archive the change (`openspec archive apply-owner-decisions-od6`) after verification.
- [ ] 3.2 After archiving, fix the Purpose sections of `number-search` ("and in every record"), `storage-format`, `storage-indexes`, `operator-lifecycle` (10^8) and, if desired, the "not decided" placeholders in `user-journeys` (Q-166).
- [ ] 3.3 Recompute the nonlinear quantities on the prototype (Q-165).
