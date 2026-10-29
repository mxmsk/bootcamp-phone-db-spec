## Why

The owner answered a selection of important [DEFAULT] items of areas 2–5 (`docs/owner-decisions.md`, OD-6): a scale of 10^7 records instead of 10^8, a request body cap of 32 KiB instead of 256 KiB, the number version once per number in by-number results (Q-128), and add without a date only for a number with no records (Q-160); the remaining [DEFAULT] D-20…D-94 and questions (O) were accepted as defaults. The current spec (`openspec/specs/`) still contains the earlier values and behavior; this change brings it in line with OD-6.

## What Changes

- **Scale 10^7 (OD-6 item 1):** X-1 and all requirements with numbers derived from 10^8 are recalculated for 10^7; the `data` file size target is at most 8 × 10^9 bytes (D-21; the calculation without compression is about 6.1 GB); the search latency (D-23) and write latency (D-27) targets and the start time (D-28) are the same numbers, but at 10^7; quantities linear in the number of records are scaled proportionally (order-index scan about 14 s → about 1.4 s, index rebuild ≤ 60 min → ≤ 6 min, estimate 10–30 → 1–3 min, sizes 46–53 → 4.6–5.3 GB, 61 → 6.1 GB, index share about 39 → about 3.9 GB); `npdb-fill` fills 10^7 records by default. Non-linear estimates (the number of tree levels, the query class boundary by posting count, the 20 s read deadline) are marked (T) "recalculate on a prototype" in `docs/open-questions.md`.
- **Body cap 32 KiB (OD-6 item 2):** `http-api` "Body size cap", `server-runtime` "Request size cap", `operator-lifecycle` (the lower bound of `limits.body_bytes` is 32768), `records` ("Input size before normalization and request body cap"): the scenario "20 000 whitespace characters at the edges" is replaced — a body over 32 KiB gets 413 before parsing; the 200-character name cap does not change.
- **Version once per number (OD-6 item 3, Q-128):** in by-number results the version is only a result field (at the response root), and records do not have it (`write-operations`, `number-search`, `http-api`, `web-ui-flows`); in name results the version stays in every record (records of different numbers).
- **Add without a date (OD-6 item 4, Q-160):** allowed only if the number has no records; otherwise a "number known" conflict (HTTP 409 `number_known`, the body has `version`, the current number version) (`write-operations` — a new requirement and five error kinds, `records`, `http-api`, `web-ui-flows`, `web-ui-shell`, `user-journeys`). The UI sends an add without a date immediately, with no prior number query; the CUJ1/CUJ2 race is closed by the server.
- **The rest accepted (OD-6 item 5):** [DEFAULT] D-20…D-94 and questions (O) moved to "accepted by the owner, OD-6" in `docs/open-questions.md`; ADRs in status Proposed go to Accepted with a reference to OD-6; the changed ADRs gained a "Changed by OD-6" item.

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `user-journeys`: CUJ1, CUJ2, CUJ3, CUJ4, CUJ5, CUJ7, CUJ9, CUJ14, X-1 (renamed to "X-1: Scale of 10^7 records").
- `records`: "Zero date when no date is given"; "Input size before normalization and request body cap" (renamed).
- `write-operations`: "Adding a record", "Name conflict and add idempotency", "Write error kinds", "Check order on write", "Result of a successful operation", "Number version in search results", "Retrying an operation after an unknown outcome"; added "Adding without a date for a known number".
- `number-search`: "Number version in a by-number result: once per number".
- `http-api`: "Adding a record", "Error body", "Client-failure codes and statuses", "Number version is an opaque string", "Search by number: a stream with a completeness marker", "Retry after a connection drop", "Body size cap".
- `web-ui-flows`: the add form, determining a known number (renamed to "Known number is determined by the server response"), CUJ1–CUJ3, delete, flow texts.
- `web-ui-shell`: "Server stopping" and connection drop.
- `storage-format`: "File size at 10^7 records" (renamed).
- `storage-indexes`: search by number, the first page, word search, the label and rebuild, "Index size and latency targets at 10^7 records" (renamed).
- `storage-durability`: "Target write latency", "Start time at 10^7 records", the check at start, version without a number scan.
- `storage-snapshots`: snapshot open, latency growth during backup.
- `server-runtime`: "Request size cap", "Read processing deadline of 20 s", "Shutdown fits the deadline of area 4".
- `operator-lifecycle`: the shutdown deadline, `npdb-fill` (scale, profile), the settings error (body cap), the operator manual (disk estimate).

## Impact

- ADRs: 45 ADRs gained a "Changed by OD-6" item (values and recalculation for 10^7: 0017, 0041, 0044, 0045, 0046, 0048, 0049, 0052, 0056–0059, 0064, 0070, 0071, 0074, 0076, 0079, 0084; Q-128, Q-160: 0002, 0005, 0025, 0026, 0035, 0046, 0060, 0062, 0088, 0090, 0091; a general note on scale — 0007, 0008, 0014, 0018, 0019, 0028, 0029, 0032, 0033, 0039, 0040, 0042, 0047, 0051, 0053, 0054); the body-cap values in ADR 0071 and 0076 are corrected; statuses Proposed → Accepted (OD-6) in 37 ADRs; the index `docs/adr/README.md`.
- Documents: `docs/open-questions.md` (D-20…D-94 and (O) accepted, (T) for recalculation, counters), `docs/owner-decisions.md` does not change.
- Outside the delta (Purpose sections are not changed by deltas): `number-search` ("and in every record"), `storage-format`, `storage-indexes`, `operator-lifecycle` (10^8) — fix at archiving; the "[DEFAULT]/awaiting the owner" marks in requirements not affected by OD-6 in substance remain as a provenance label.
- No code: the change defines requirements, not implementation.
