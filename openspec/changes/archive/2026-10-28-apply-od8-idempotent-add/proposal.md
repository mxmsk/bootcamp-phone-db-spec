## Why

Owner decision OD-8 (`docs/owner-decisions.md`, Q-180): retrying an add without a date after a lost response
produced "number known" (OD-6, Q-160) and confused the user. An add without a date, when the number has a record
(number, 1876-03-07) with the same name, is now "unchanged"; the retry is safe.

## What Changes

- Add without a date: a record on 1876-03-07 with the same name exists (names are compared as for an identical retry
  of an add with a date) — success "unchanged" (HTTP 200 `unchanged`, version the same), regardless of other records
  of the number; otherwise, if there are records — "number known" (409 `number_known`, OD-6 item 4 unchanged).
- `records`, `write-operations`, `http-api`, `web-ui-flows`, `user-journeys` (CUJ1, CUJ7, CUJ9) are aligned.
  `web-ui-shell` does not change: its texts ("outcome unknown", a drop during the number check) remain correct.
- Q-180 is closed in `docs/open-questions.md`; ADR 0002, 0005, 0025, 0026, 0060, 0062, 0090 gained a "Changed by OD-8"
  item; the ADR index is marked.

## Capabilities

### Modified Capabilities

- `records`: "Zero date when no date is given".
- `write-operations`: "Adding a record", "Write error kinds", "Check order on write",
  "Retrying an operation after an unknown outcome", "Adding without a date for a known number".
- `http-api`: "Adding a record", "Retry after a connection drop", "Absent member and `null`".
- `web-ui-flows`: "Adding a new number (CUJ1)", "Entered input is kept when a write fails",
  "Known number is determined by the server response".
- `user-journeys`: "CUJ1: Adding a new number", "CUJ7: A local program works with the database",
  "CUJ9: A network service changes data".

## Impact

Specification and documents only; there are no new error kinds or codes, and the API gains one more 200 `unchanged` case.
