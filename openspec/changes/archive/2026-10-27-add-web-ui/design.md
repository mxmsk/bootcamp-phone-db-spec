## Area boundaries

Area 5 defines the web interface: how the server serves the page, how the user signs in by link, what the page shows in the CUJ1–CUJ6 and CUJ12 journeys, how the refusals of CUJ13 and CUJ16 are displayed. This is the observable behavior of a browser page on top of the accepted HTTP API v1; the choice of UI build tool and framework is not part of the spec (ADR 0085 item 2). Data semantics (area 1), storage (2), API, access and the server process (3) and administration (4) are accepted and do not change, except for one `http-api` requirement ("API version in the path", MODIFIED delta): the paths `/` and `/ui/…` are the UI shell on the loopback listener and 404 on the network one. `api-access` does not change: the network listener accepts only the API and network tokens, and the response to `/` is defined by `http-api`. The requirements for area 5 accumulated by areas 3 and 4 are met: the UI address is one line of `serve` standard output, the token changes at every start, the shell without a token is at step 5 of the check order, the UI does not poll the server state.

## Map of capabilities, ADRs and CUJs

| Capability | ADR | CUJs and requirements |
|---|---|---|
| `web-ui-shell` (19 requirements, 57 scenarios) | 0085–0089 | CUJ13, CUJ16, CUJ7 (401 after restart), CUJ1–CUJ5 (value errors, date format) |
| `web-ui-flows` (30, 104) | 0090–0094 | CUJ1–CUJ6, CUJ12, CUJ13 (empty results), CUJ16 (stream texts) |
| `http-api` (MODIFIED: 1 requirement, 5 scenarios) | 0085 item 1 | CUJ13 |

In total 49 added requirements and 160 scenarios in two delta specs and one modified requirement of an accepted capability; ADR 0085–0094 are 10 decisions. Accepted [OURS] — 0087 entirely; 0085 (items 1, 3), 0086 (items 1, 3, 4), 0088 (items 1–4 except values, 6), 0090 (items 1–2), 0091 (items 1–4), 0092 (items 5, 7), 0093 (items 5–6 and confirmation before delete as an owner decision), 0094 (items 1, 3, 6); Proposed — the items with default values (D-76…D-94), ADR 0089 entirely. The owner's decisions ([USER]) are not narrowed by this area: the "delete" button on the left and confirmation before delete (CUJ6), the link with a token that changes at every start, and the closed-by-default refusal form (ADR 0011 items 2, 6) are carried out unchanged.

Boundaries between capabilities. `web-ui-shell` is everything that does not depend on the user journey: delivery, security, the token, the common request layer and the display of refusals by code and action, the date format, browsers. `web-ui-flows` is what the user does with records and what they see in response; it does not repeat the common refusals but passes the action to the layer ("Saving", "Deleting", "Search"). The single source of the value-error table and of the "<Action> failed: …" texts is `web-ui-shell`; the single source of the stream outcome texts (after save, «данные устарели, список обновлён» (data is stale, the list was refreshed), «Запись уже есть, данные не изменились», «Записи уже нет» (the record is already gone)) is `web-ui-flows`.

Dependence on owner decisions and the [DEFAULT] package: the values are collected in `docs/open-questions.md`, "Area 5: default values" (D-76…D-94).

## Subtask reconciliation

| Seam | Single decision | Where |
|---|---|---|
| Shell paths and the accepted `http-api` | MODIFIED "API version in the path": loopback — the shell, network — 404 `route_not_found` | `http-api` (delta), `web-ui-shell` ("UI page is served without a token only on loopback"), ADR 0085 item 1 |
| Sign-in by link in an open tab | the token is read at load and on fragment change; the entered data and the results stay | `web-ui-shell` ("Sign-in by link: token from the fragment", "Old link after server restart"), ADR 0086 item 1 |
| The form after 401 | "No token" is only a load without a token; after 401 the form and the results stay, there are no requests without a link | `web-ui-shell`, ADR 0088 items 1–2 |
| Number check and a broken stream | "unknown" is only `complete: true` and `count` 0; without the marker the check failed, the add is not sent | `web-ui-flows` ("Known number is determined by the server response"), ADR 0090 item 1 |
| The action in refusal texts | a write request was sent — "outcome unknown"; a read, including the number check, — «нет связи с сервером» ("no connection to the server") | `web-ui-shell` ("Server stopping" and connection drop), `web-ui-flows`, ADR 0088 item 4 |
| UI rules and value validity | there is exactly one own rule: the date is required in the CUJ2 step; the server decides the rest | `web-ui-flows` ("Date prompt for a known number (CUJ2)", "Date entry", "Value errors in the form"), ADR 0091 item 1 |
| One text per outcome | «Запись уже есть, данные не изменились»; «Записи уже нет» | `web-ui-flows`, ADR 0091 item 3, 0094 items 3–4 |

## Fix pass after the Opus review

One fix pass per `docs/process.md` (second edition); there is no repeat review.

- R-1 (BLOCKER). The token storage guarantee is stated honestly: the token is not in a cookie, `localStorage`, request addresses, texts, the console; it lives until the server restarts; it is accepted only on loopback. "Not on disk" and "disappears on close" are not promised; the residual risks (restoration of `sessionStorage` by the browser, tab duplication, the link in browser history and the terminal) are listed in the requirement and in ADR 0086 item 2; verification is Q-158. The owner decision is not narrowed (ADR 0011 item 2 is met), so the default value remains D-77.
- R-2 (BLOCKER). The fragment is read at load and at `hashchange` with the same address cleaning; the scenario "A new link in the same tab" (CUJ13).
- R-3 (BLOCKER). Known number: an add without a date goes out only at `complete: true` and `count` 0; a response without a completeness marker is a failed check; the "The stream is broken…" scenarios are added.
- R-4. The MODIFIED requirement of `http-api` "API version in the path"; the shell's network listener scenario refers to it, `api-access` does not change.
- R-5. "No token" is only a load without a token; after 401 the form and the results stay until sign-in by a new link.
- R-6. The mandatory date in the CUJ2 step is the only UI rule, stated in "Value errors in the form" and "Date entry".
- R-7. A drop during a write request is "outcome unknown"; during a read (including the number check) it is «нет связи с сервером» ("no connection to the server").
- R-8. One text each: «Запись уже есть, данные не изменились» and «Записи уже нет» (requirements, scenarios, ADR 0091, 0094).
- R-9. Confirmation before delete is [USER]; the dialog text, buttons, focus and Esc are [DEFAULT] D-94 (ADR 0093 item 2).

## Requirements for other areas

### Server and build (areas 3 and 4)

- `http-api`: the change made by this change is "API version in the path". The shell is served at step 5 of the check order without a token; shell responses carry the security policy headers (`web-ui-shell`); `/favicon.ico` is 404; the server sets no cookies. During startup `/` gets 503 `server_starting` (JSON), with no page: the link is printed after the switch to working mode.
- Build: the UI resources (plain HTML, CSS, JavaScript with no external runtime dependencies) are embedded in the program; there are no separate settings (D-76).
- Recommendation to the server (Q-159): `X-Content-Type-Options: nosniff` on API responses.

### The page's common request layer (for implementation)

- All API requests go through one layer: it adds `Authorization`, parses refusals by `code`, `field`, `reason`, `hint` (and also `current_name` and `version` in conflicts), retries `server_starting`; it returns the status and the error body to the flow.
- The layer is passed the action ("Saving" — number check, add, replacement; "Deleting"; "Search") and the "write request sent" marker; the common refusals (401, startup, shutdown, drop, `read_only`, `storage_error`, others) are shown by the shell, the flows do not duplicate their texts.
- Results by number are a stream: the layer marks a response without `complete: true` as incomplete; the flow does not take it for complete.
- All data from responses is output as text; the UI introduces no state requests.

### API (area 3)

- There are no operation changes. Observations: `name_conflict` does not contain a date (the UI takes it from its own request); `by-number` returns `version` at the root and in the records; there is no "add if the number is unknown" mode (Q-160).

## Verified facts

F-116 (`history.replaceState`), F-117 (`sessionStorage`), F-118 (CSP Level 3), F-119 (`Retry-After`) in `facts.md`; F-1, F-90, F-91, F-95, F-97, F-99 were used. Not verified and not put in the spec as a claim: browser history and `sessionStorage` restoration (Q-158), the behavior of specific browser versions (dialogs, focus, Esc).

## Open questions

In `docs/open-questions.md`: Q-158…Q-164 (section "Area 5: new questions"; (O) is Q-160, the CUJ1/CUJ2 race, the rest are (T)), D-76…D-94 (section "Area 5: default values"), the closed ones in "Closed in area 5" (Q-1, Q-2, Q-28, Q-29, Q-35, Q-36, Q-37, Q-39, Q-61, Q-62, Q-63, Q-65, Q-68; Q-38 narrowed).

## Workflow follow-up

- Status: Opus review done, fix pass done, archive open. Archive the change (`openspec archive add-web-ui`) after the owner answers the [DEFAULT] package; the delta specs `web-ui-shell`, `web-ui-flows` are moved to `openspec/specs/`, the MODIFIED requirement "API version in the path" replaces the accepted one in `http-api`.
- Check the archived result: `openspec validate --specs` with no errors.
