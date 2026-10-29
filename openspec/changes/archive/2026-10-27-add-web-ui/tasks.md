## 1. Review preparation

- [x] 1.1 Make sure `openspec validate add-web-ui --strict` passes with no errors or warnings; check: the command exits with code 0 and prints "is valid".
- [x] 1.2 Make sure ADR 0085–0094, the delta specs, `design.md`, `facts.md` and `docs/open-questions.md` contain no temporary subtask identifiers and no references to drafts; check: a search by the patterns of temporary subtask designations (local numbers of decisions, questions and facts, references to drafts) finds no matches.
- [x] 1.3 Reconcile the seams between subtasks: shell paths and `http-api`, sign-in and the form after 401, number check and a broken stream, the action in refusal texts, one text per outcome; check: the "Subtask reconciliation" table in `design.md`.
- [x] 1.4 Reconcile the numbering: facts F-116…F-119 in `facts.md`, questions Q-158…Q-164 and [DEFAULT] D-76…D-94 in `docs/open-questions.md`, rows 0085–0094 in `docs/adr/README.md`; check: every F-, Q-, D- reference from the new ADRs and specs points to an existing entry.

## 2. Review

- [x] 2.1 Opus review (done: R-1…R-9, 3 BLOCKERs and 6 contradictions): consistency, conformance with `user-journeys`, the ADRs, the accepted areas and `docs/owner-decisions.md`; check: a list of findings with classes is obtained.
- [ ] 2.2 Sonnet review (per the `docs/process.md` scheme, step 5 — when the whole spec is ready, not in this area; not done): feasibility of the UI in a browser (`hashchange` and `replaceState`, `sessionStorage`, reading a response stream, dialogs and focus); check: a list of findings with classes is obtained.
- [x] 2.3 Process the BLOCKER findings and contradictions (done in one fix pass per `docs/process.md`, second edition: R-1…R-9 fixed, see `design.md`, "Fix pass after the Opus review"; no repeat review); check: no BLOCKERs or contradictions remain, the remainder is recorded in `docs/open-questions.md`.
- [x] 2.4 Move the review remainder to `docs/open-questions.md` and recount the counters; check: the sections "Area 5: new questions", "Closed in area 5" and the totals are present.

## 3. Owner decisions

- [ ] 3.1 Hand the owner the [DEFAULT] package of area 5 as one list: D-76 (resources in the program), D-77 (token storage), D-78 (date format), D-79 (browsers), D-80 (startup auto-retry), D-81 (refusal texts), D-82 (no date field in the CUJ1 form), D-83 (after save), D-84 (result row), D-85 (empty results), D-86 (date 1876-03-07), D-87 (number owners), D-88 («Дальше»), D-89 (cancel delete, no undo), D-90 (re-query), D-91 (re-read), D-92 (record disappeared), D-93 (outcome texts), D-94 (delete dialog), as well as question (O) Q-160 (the CUJ1/CUJ2 race, an API parameter is needed); check: the list is handed over, the owner's answers are entered in the ADRs (status Accepted, label [USER]).
- [ ] 3.2 After the owner's answer, replace the values in the specs of areas 3 and 5 (date form, texts, date field) in a separate change if the owner chose otherwise; check: no values without the "default" mark remain.

## 4. Tests and checks arising from the area (for implementation)

- [ ] 4.1 Record in the test plan: the shell on loopback and 404 on the network listener, the security policy headers, absence of inline code; sign-in by link at load and at `hashchange` in an open tab; 401 after restart (the form and the results stay, there are no requests without a link); the number check on a broken stream (the add is not sent); connection drop on a read and on a write; `server_starting` with a 60 s auto-retry; version and name conflicts (CUJ3, CUJ12); delete with confirmation and re-query; a name with markup is output as text.
- [ ] 4.2 Run checks of external behaviors (Q-158): whether a link with a fragment lands in browser history after `replaceState`, restoration of the `sessionStorage` of a closed tab and after a browser restart in the supported browsers; check: the results are recorded in `research/`, the residual risk is confirmed or revised.

## Workflow follow-up

- Status: Opus review done, fix pass done, archive open.
- Archive (`openspec archive add-web-ui`) after the owner answers the [DEFAULT] package of area 5.
