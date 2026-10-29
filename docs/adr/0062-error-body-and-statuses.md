# 0062. Error body and HTTP statuses
- Status: Accepted ([OURS]: items 1–4, 6); Accepted (OD-6) ([DEFAULT]: items 5, 7 (Q-79, D-39, D-40), item 8 (D-41))
- Label: [OURS] (items 1–4, 6); [DEFAULT] (items 5, 7, 8)
- Date: 2026-10-27
- CUJ: CUJ7, CUJ8, CUJ9, CUJ11, CUJ16

## Context
Brief §4: the error body `{code, field}`, status by code, accepted before (field, reason) pairs, versions and storage states appeared. A
revision is required (Q-49, Q-50, Q-79).

## Decision
1. [OURS] The body is a flat JSON object `{"code","field","reason","hint","message", …}`: `code` is a string (the contract, snake case), `field`, `reason`,
   `hint` — as circumstances require, `message` is Russian text. The `{code, field}` form from the brief is preserved as a subset: extension is
   only by adding members. Special members: `name_conflict` — `current_name`, `version`; `version_conflict` — `version`.
2. [OURS] Q-50: the contract is `code` and status; `message` is outside the contract and may change (it holds the brief's texts: «не авторизован», «нет прав»,
   «слишком много запросов, повторите позже», «сервер запускается», «база доступна только для чтения», «ошибка хранилища, обратитесь к оператору»).
3. [OURS] Value error: `code` = `invalid_input`, `field` ∈ {`number`, `date`, `name`, `query`, `expected_version`, `page_size`}, `reason` ∈
   `invalid_format` (with `hint:"missing_plus"` for a number without "+", ADR 0023 item 7), `missing`, `no_such_day`, `out_of_range`, `empty`,
   `control_character`, `too_long`, `wrong_type`. Mapping to the `records` dictionary: "invalid shape" = `invalid_format`; "missing" = `missing`;
   "nonexistent day" = `no_such_day`; "out of range" = `out_of_range`; "empty" = `empty`; "control character" = `control_character`;
   "too long" = `too_long`. A missing or `null` member: `number` — `invalid_format`, `name`, `query` — `empty`, `date` in add — not an error,
   otherwise `missing`, `expected_version` — `missing`. The field check order — ADR 0025 item 2.
4. [OURS] Statuses: `invalid_request`, `invalid_input`, `invalid_cursor` — 400 (all client errors without a state conflict); `record_not_found`,
   `route_not_found` — 404 (distinguishable by `code`); `method_not_allowed` — 405 with `Allow` (RFC 9110 §15.5.6: "MUST generate an Allow header");
   `name_conflict`, `version_conflict` — 409 (RFC 9110 §15.5.10); `payload_too_large` — 413, `unsupported_media_type` — 415;
   `storage_error`, `internal_error` — 500; `read_only`, `server_starting`, `snapshot_revoked` — 503. Codes for access refusals, limits, time and
   server state — item 8. 422 is not used: a single 400 for all input errors is simpler for clients.
5. [DEFAULT] Q-79: in a situation where a v1 release answered with a code and status, all following v1 releases answer the same; a new "more precise" code is not introduced in place of the
   old one. Rationale: the client branches on `code`; a replacement would break it just as a removal does. Alternative: allow new
   codes with the same status — clients that enumerated the known codes break; the price of the guarantee is less precise codes in the future.
6. [OURS] Client rule: an unknown `code` is handled by status class; unknown members are ignored.
7. [DEFAULT] Error body: the brief §4 owner decision `{code, field}` is revised by an extension (item 1); the value is put to the owner as a package.
   Alternative: only `{code, field}` (the field is not enough for a reason: the `records` dictionary requires a pair).

8. [DEFAULT] (D-41) Codes and statuses of refusals not in items 1–4 — a single table for all capabilities (spec `http-api`, "Client-failure codes and statuses", "Server-failure codes and statuses"; `api-access` and `server-runtime` refer to it): `invalid_request` — 400 also for unparseable HTTP/1.1, a conflict of
   `Transfer-Encoding` and `Content-Length`, and a missing Host (RFC 9110 §7.2, F-97; ADR 0072 item 8); `unauthorized` — 401 with `WWW-Authenticate: Bearer`
   without an error code (RFC 9110 §15.5.2, RFC 6750 §3, F-90, F-97), one response for any unrecognized token; `forbidden` — 403 («нет прав»);
   `host_not_allowed` — 403 (ADR 0067); `rate_limited` — 429 (RFC 6585 §4, F-92); `headers_too_large` — 431 (the request line with headers exceeds
   the cap; one measure and one refusal, 414 is not used; RFC 6585 §5); `server_stopping`, `too_many_connections`, `request_timeout` — 503
   (not executed, retry is safe); `write_outcome_unknown` — 504 (the operation may have executed; 504 rather than 503 so that a client that does not know the code
   does not blindly retry by status class). Closing on the request read deadline and response idle carries no code. Rationale: the client branches on
   `code`, and "not executed" and "outcome unknown" must be distinguished (ADR 0071 item 4, P-12). Alternatives: 408 for `request_timeout` (RFC 9110:
   the client did not send the request in time — a different condition); 500 or 503 for `write_outcome_unknown`; 421 for `host_not_allowed` (does not reflect a refusal).
9. **Changed by OD-6 (2026-10-28).** Q-160: code 409 `number_known` — add without a date for a number that has records; the body — `code`, `message`, `version` (the number's current version), no `current_name`; the status class is conflict. The table of item 4 is extended with this code.
10. **Changed by the final fix pass (2026-10-28, ADR 0095).** Refusals produced by HTTP parsing carry only a status; `null` is equivalent to an absent member; the reasons `invalid_format`, `wrong_type`, `missing` belong to the protocol.
11. **Changed by OD-8 (2026-10-28).** Q-180: 409 `number_known` is not returned if the number has a record on 1876-03-07 with the same name (add without a date — 200 `unchanged`, ADR 0060 item 9); otherwise item 9 is as before.

## Alternatives considered
- A nested object `{"error":{…}}`: rejected — it changes the brief's shape instead of extending it.
- Status 422 for value errors: rejected — gives clients no distinctions beyond 400, increases the number of branches.
- Text only without a code (Q-50): rejected — texts change, the code is the contract.
- 409 for `record_not_found`: rejected — this is a missing resource (CUJ9 distinguishes conflict and "not found").

## Consequences
- Q-49, Q-50 (codes 401/403/429 and other refusals — item 8), Q-79 (default) are closed.
- `reason` values may be added (ADR 0064).

## Sources
- ADR 0013, 0019 item 3, 0020, 0023, 0025, 0034; `records` "Value error kind dictionary"; RFC 9110 §15.5.2, §15.5.6, §15.5.10, §15.5.14–16, §15.6.4 (F-86, F-97); RFC 6585 §4–5 (F-92); RFC 6750 §3 (F-90).

- **Changed by OD-7 (2026-10-28).** the code `too_many_connections` is not used: on slot overflow the connection is closed without a response (ADR 0098).
