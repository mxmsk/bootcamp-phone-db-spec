# 0085. UI page delivery: shell routes, embedded resources, language
- Status: Accepted ([OURS]: items 1, 3); Accepted (OD-6) ([DEFAULT]: item 2 — D-76)
- Label: [OURS] (items 1, 3); [DEFAULT] (item 2 — D-76)
- Date: 2026-10-27
- CUJ: CUJ13, CUJ16; item 1.2 of the original wording

## Context
The UI is a web page in a browser on the same machine (ADR 0011 item 1). ADR 0067 item 5 requires serving the shell without a token and without data; `http-api` ("Request check order", scenario "UI shell") leaves it to area 5 to define which paths belong to the shell. Not specified: the paths, how resources are stored, the language.

## Decision
1. [OURS] The shell is the page `GET /` and the resources under `/ui/` (script, styles); methods GET and HEAD; other methods get 405, an unknown path under `/ui/` gets 404 (`http-api`). Only on the loopback listener (ADR 0066 item 1); on the network listener `/` and `/ui/…` are 404 `route_not_found`, like any path without a version. Paths outside `/`, `/ui/` and `/v1/` are still 404. The shell contains no database data or tokens, so it can be served before the token check, at step 5 of the check order; during startup the shell, like any path, gets `server_starting` (ADR 0074 item 2).
2. [DEFAULT] (D-76) The UI resources are embedded in the server binary (at build time), not kept as files alongside. Rationale: one program without a resource directory — no mismatch between script and server versions after an upgrade, no path from which an extra file could be served, no setting for the operator; the UI is small. Alternative: a resource directory in the settings — allows changing the UI without a rebuild, but adds "file not found" errors, permissions, and version mixing. The choice of the UI build (which tool, whether a framework is needed) is not part of the spec: the guarantee is a page without external resources and without inline scripts (ADR 0087); plain HTML, CSS and JavaScript with no external dependencies, built into the binary, will do; runtime dependencies in the browser are not allowed.
3. [OURS] The interface language is Russian, all texts and `lang="ru"`: the brief's texts (CUJ12, CUJ16) are Russian, the operator and the spec are Russian. There is no language switch (not in the brief).

## Alternatives considered
- Resources from disk: see item 2.
- Serving the shell after the token check: impossible, the token is taken by the page script from the fragment, which the server does not see (ADR 0067 item 5).
- The page at another path (`/ui/`) with a redirect from `/`: rejected, the CUJ13 link (`http://127.0.0.1:<port>/#<token>`) points at `/`, no redirect is needed.
- A bilingual interface: rejected, outside the brief.

## Consequences
- A change to the accepted `http-api`: a MODIFIED requirement "API version in the path" (delta of this change) — the paths `/` and `/ui/…` on the loopback listener serve the shell, on the network listener 404 `route_not_found`; `api-access` does not change.
- Requirement for area 4: the build includes the UI in the program; there is no separate setting.
- A program upgrade replaces the UI together with the server.

## Sources
- ADR 0011, 0066, 0067, 0074; `http-api`, `api-access`, `server-runtime`; `docs/source/brief.md` §2, §3 (Interfaces).
