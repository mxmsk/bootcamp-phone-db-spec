# 0089. Date format in the UI and supported browsers
- Status: Accepted (OD-6)
- Label: [DEFAULT] (item 1 — D-78; item 2 — D-79)
- Date: 2026-10-27
- CUJ: CUJ1, CUJ2, CUJ13
- Closes Q-62 for the format (the date field in the CUJ1 form is `web-ui-flows`)

## Context
Brief §5: the date format in the UI is not decided. A date is a calendar day, valid from 1876-03-07, future dates are allowed (ADR 0024); the API accepts only `YYYY-MM-DD` (`http-api`, "Date representation"). Supported browsers are not specified by the brief.

## Decision
1. [DEFAULT] (D-78) The UI shows and accepts a date as `YYYY-MM-DD` (for example, 2024-05-01), without conversions: independent of the browser's locale and time zone. Rationale: the format is unambiguous (unlike 01.05.2024 / 05/01/2024), matches the API and what programs see; the JavaScript date type is tied to an instant and a time zone and can shift the day. The native `type=date` field is not used: its display depends on the browser's locale, and choosing a year before 1900 is inconvenient. Alternatives: `DD.MM.YYYY` (more familiar to a Russian user, requires parsing and reverse conversion, risk of errors), the native date picker.
2. [DEFAULT] (D-79) The two latest stable versions of Chrome, Edge, Firefox and Safari at the time of release are supported. Rationale: the UI is needed on the same machine, the user has an ordinary modern browser; old versions are outside the brief. Required capabilities: `fetch`, reading a response stream (the search-by-number results are a stream with a completeness marker, ADR 0061), `sessionStorage` (or fallback memory), CSP. Actual support across browser versions was not verified (not verified). Without them the page shows «Браузер не поддерживается» ("Browser not supported"); without JavaScript — «Нужен включённый JavaScript» ("JavaScript must be enabled") (`noscript`). Alternative: supporting old versions — requires transpilation and polyfills, outside the goal.

## Alternatives considered
- See items 1, 2.

## Consequences
- `web-ui-shell`: the requirements "Date format in the UI", "Supported browsers".
- `web-ui-flows`: the date form and hints use this format.
- D-78, D-79 — to the owner as a package.

## Sources
- ADR 0024, 0061; `http-api`; `docs/source/brief.md` §5.
