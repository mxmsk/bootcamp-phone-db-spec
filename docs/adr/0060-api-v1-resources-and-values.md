# 0060. API v1: paths, methods, resources and value representation
- Status: Accepted ([OURS]: items 1–6, 8); Accepted (OD-6) ([DEFAULT]: item 7, D-37)
- Label: [OURS] (items 1–6, 8); [DEFAULT] (item 7 — unknown fields and parameters)
- Date: 2026-10-27
- CUJ: CUJ1–CUJ9, CUJ11; item 1.2 of the original wording

## Context
The brief and ADR 0011, 0013 set: HTTP + JSON, version in the path, the same operations as in the UI, plus exact whole-name search,
no administration. Not set: paths, methods, body shapes and value representation (Q-26, Q-32, Q-34, Q-51, Q-67).

## Decision
1. [OURS] Paths under `/v1/`: `POST /v1/records` (add), `POST /v1/records/rename`, `POST /v1/records/delete`,
   `GET /v1/records/by-number`, `GET /v1/records/by-words`, `GET /v1/records/by-name` (exact). The number and date go in the body
   (records) or parameters (search), not in the path: number input contains spaces, parentheses, "+" and "/", and encoding them in a path segment
   gives ambiguous errors; in the body "date missing" (ADR 0025 item 6) is expressible, in the path it is not.
2. [OURS] Rename and delete are `POST` to separate paths, not `PATCH` and not `DELETE`: a body on `DELETE` has no defined
   semantics (RFC 9110 §9.3.5, F-86), some clients and intermediaries drop it. Search is `GET` (a read, caching is not promised).
3. [OURS] Q-51: a path without a version or with another version — 404 `route_not_found`; no redirects and no default version:
   an implicit version would make behavior depend on an upgrade (CUJ11).
4. [OURS] Q-26: in responses the number is always in canonical form ("+79991234567"; ADR 0023); the entered form is not stored. On input
   any form from `records` is accepted. The number is only a JSON string (a JSON number — `wrong_type`: a number loses "+" and leading zeros).
   An unencoded "+" in the query string is read as a space; after cleaning this gives a number without "+" and the hint `missing_plus`.
5. [OURS] Q-67: the date is exactly `YYYY-MM-DD` (RFC 3339 `full-date`, F-7, but the year is strictly 4 digits). Any other syntax (time, zone, sign,
   a year not of 4 digits, spaces) — `invalid_input`, field `date`, reason `invalid_format`, at the protocol level before `records`; 4-digit years
   0000–9999 reach `records` (`no_such_day`, `out_of_range`). A 5-digit year is rejected by the protocol.
   A missing `date` field in rename and delete (or `null`) — reason `missing`; in add — 1876-03-07 is substituted and
   returned in the response.
6. [OURS] Q-32, Q-34: the version is a JSON string of 64 lowercase hexadecimal characters (32 bytes, ADR 0054 item 5); compared only for
   equality. The shape is checked without the database: any other shape (length, upper case, not a string) — `invalid_input`, field `expected_version`
   (`invalid_format` / `wrong_type`; missing — `missing`); a valid shape — comparison with the current one (`version_conflict`). The version is in
   every result record (a copy of the number version), at the root of the search by number, in a write success and in `version_conflict`/`name_conflict`;
   the client takes it from the conflict response or re-reads the number. Hexadecimal rather than base64url: the canonical form is unambiguous without
   a rule on unused bits.
7. [DEFAULT] Unknown members of the request JSON and unknown query string parameters are ignored; a repeat of a known parameter and a repeat of
   an object member name — 400 `invalid_request` (RFC 8259 §4: behavior on repeat is unpredictable, P-13). A known member whose type is not
   string — `invalid_input`, reason `wrong_type`. Rationale: request fields may be added in v1 (CUJ11), "extra" must not break a client,
   and required members are checked anyway. `expected_version` in add is ignored, as ADR 0025 item 3 requires. Alternative:
   strict refusal on unknown — catches typos, but makes any addition of a request field incompatible for old servers.
8. [OURS] Success bodies: add — `{outcome, record:{number,date,name}, version}` (201 on `changed`, 200 on `unchanged`);
   rename — the same, 200; delete — `{outcome:"changed", record:{number,date}, version}`, 200. `outcome` is `changed` or `unchanged`
   (ADR 0026 item 1). The date in the response is needed because otherwise the client would not learn the substituted date (ADR 0025, "Consequences"). Member names are snake_case.
   Language and order: the `Accept-Language` and `Accept` headers are ignored (ADR 0037 item 5).
9. **Changed by OD-6 (2026-10-28).** Q-128: a record in the by-number results has no version, it is only at the root (the record is `{number,date,name}`); in by-name results the version is in every record (item 6 corrected). Q-160: add without a date for a number with records — 409 `number_known` with `version` (ADR 0062). The cost in the "Alternatives considered" item (190 MB) is removed.
10. **Changed by OD-8 (2026-10-28).** Q-180: add without a date, when the number has a record on 1876-03-07 with the same name, — 200 `outcome: unchanged` with `record` on 1876-03-07 and the earlier `version`; 409 `number_known` — in other cases when records exist.

## Alternatives considered
- The record resource in the path (`/v1/numbers/{number}/records/{date}`): rejected — see item 1.
- `PATCH`/`DELETE` on `/v1/records`: rejected — see item 2.
- The version as a number: rejected — the version is opaque and 32 bytes; a number loses precision beyond 2^53 (RFC 8259 §6).
- The version as base64url: rejected, ambiguity of unused bits; hex is 21 characters longer, the cost is small.
- The number in results in the entered form: rejected — not stored, equal numbers would look different.
- A 5-digit and signed year (extended ISO 8601): rejected — the date range ends at 9999, an `out_of_range` response gives the client nothing.

## Consequences
- Q-26, Q-32, Q-34, Q-51, Q-67 are closed; item 7 — default D-37.
- The version is represented as 64 characters in every result record: ≈ 190 MB of extra JSON on the maximum number (2,967,164 × 64); the cost is accepted (ADR 0035 item 4).

## Sources
- `docs/source/brief.md` §3 "Interfaces", CUJ7–CUJ9; ADR 0011, 0013, 0023, 0024, 0025, 0026, 0054; facts F-7; RFC 9110 §9.3.5, RFC 8259 §4, §6 (F-86, F-87).
