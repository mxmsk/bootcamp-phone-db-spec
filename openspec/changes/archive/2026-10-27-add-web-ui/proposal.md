## Why

Area 5 per `docs/process.md` ("UI") defines the web interface without which CUJ1–CUJ6, CUJ12 and the sign-in by link of CUJ13 with the refusals of CUJ16 cannot be completed: the server serves a page, the user signs in by a link with a token, enters and searches records, deletes them and sees stale data and server refusals. Data semantics (area 1), storage (2), API and access (3) and administration (4) are accepted; the UI is their client (ADR 0011). The area's decisions are recorded as ADR 0085–0094; values not set by the brief and the owner are put to the owner as [DEFAULT] (D-76…D-94).

## What Changes

- The UI shell: the page `/` and the resources under `/ui/` are served without a token and without database data only on the loopback listener (on the network listener — 404 `route_not_found`), embedded in the program, Russian language, a strict page security policy, no external resources and no cookies, database data only as text (ADR 0085, 0087).
- Sign-in and refusals: the token is taken from the link fragment at load and on fragment change in an open tab, kept in the tab's `sessionStorage` or in memory with an explicit list of residual risks, and passed only in the `Authorization` header; the texts and behavior for a missing token, 401 after a server restart (the form and the results stay), startup, shutdown, connection drop, "read-only", "storage error", value errors; the date format and browsers (ADR 0086, 0088, 0089).
- User journeys: the add form and detection of a known number by a number search (a response without a completeness marker is a failed check), the date prompt and overwrite, the message after save, results by number and by word prefix of the name, «Дальше» (Next), delete with confirmation and re-query, stale data, the retry offer, preservation of what was entered, message texts (ADR 0090–0094).
- Fix pass after the Opus review (R-1…R-9): an honest token storage guarantee, sign-in by a new link in the same tab, incomplete results in the number check, the MODIFIED `http-api` requirement, the form after 401, the mandatory date in CUJ2 as a UI rule, distinguishing a write request from a read on a drop, one text per outcome, the choice of agent in the delete dialog — [DEFAULT]. Details are in `design.md`, "Fix pass after the Opus review".

## Capabilities

### New Capabilities

- `web-ui-shell`: page delivery, embedded resources, language, security policy, no cookies and no external resources, text instead of markup, sign-in by link, token storage and passing, refusals (no token, 401, startup, shutdown and drop, "read-only", "storage error", value errors, others), date format, browsers.
- `web-ui-flows`: the add form and the date field, known number, date prompts and overwrite, the message after save, value errors in the form, search by number and by word prefix of the name, the result row, pagination, delete, stale data, retry offers, stream texts, preservation of what was entered.

### Modified Capabilities

- `http-api`: the requirement "API version in the path" — the paths `/` and `/ui/…` on the loopback listener serve the UI shell, on the network listener they get 404 `route_not_found`. The other requirements of areas 1–4, `api-access` and `user-journeys` do not change.

## Impact

- ADR 0085–0094 (new, 10); they build on ADR 0011, 0060–0067, 0074; the index is `docs/adr/README.md`.
- The build and server requirements are collected in `design.md`, "Requirements for other areas".
- Documents: `facts.md` (F-116…F-119), `docs/open-questions.md` (13 closed, 1 narrowed, 7 new: Q-158…Q-164, [DEFAULT] 19: D-76…D-94), `docs/adr/README.md`.
- No code: the change defines requirements, not implementation.
