## Why

Area 1 per `docs/process.md` ("Data model and operations") defines the observable semantics of the data, write operations
and searches. Storage (area 2), the server and API (area 3), administration (area 4) and the UI (area 5) build on it:
without it none of those areas can be implemented or verified. It is the first area in order; the decisions for it are
recorded in ADR 0020–0039, and what remains unresolved is moved to `docs/open-questions.md`.

## What Changes

- The record "(number, date) → name": record key and identity, rules for name, number and date (normalization, validity,
  bounds, the zero date 1876-03-07).
- Write operations: add, rename, delete; number version, value error kinds and check order, results and
  idempotency, atomicity and visibility.
- Name search: by word prefix and exact whole-name search, comparison key, words per UAX #29, "next" pagination by
  cursor without a total count, empty results and errors.
- Number search: match by canonical form, all records of a number by date DESC without pagination (up to 2 967 164 records),
  number version as a result field.
- Result order for name search: alphabet under ru locale rules, then date DESC, then number; total order for the cursor;
  stability within v1 (no change to ordering or search is released in v1; fixing dependencies is allowed only without
  changing results, ADR 0039).
- X-5 in `user-journeys` is aligned with the version generation and with a number that has no records: the guarantee "the version
  differs from all previously obtained ones" holds for a number with records, while a number without records has the initial
  version (ADR 0027 item 4; clarification of ADR 0006 item 6(a)); "never" across generations is replaced by a guarantee boundary:
  deterministic on an installation with saved settings (the generation counter lives outside the backup), otherwise with a
  collision probability of at most 2^-64 per event, deterministic within a generation
  (owner decision OD-5, D-17, option (c) of ADR 0028 item 9; narrowing of ADR 0006 item 3); the exception for files copied
  by something other than the backup command is owner decision OD-5, D-13 (ADR 0028 item 7). The same narrowing affects CUJ1, CUJ7, CUJ14, CUJ15.
- In CUJ1, CUJ2, CUJ3, CUJ5, CUJ6, CUJ7, CUJ8, CUJ9, CUJ12, CUJ14, CUJ15 the "<not decided: Q-x>" placeholders for questions
  closed in area 1 (Q-30, Q-33, Q-55, Q-64, Q-73, Q-84) are replaced with references to the decisions; the meaning of the
  CUJs does not change; the CUJ5 scenario about an invalid cursor is aligned with ADR 0032.
- New and updated verified facts `F-3…F-51` in `facts.md`; new questions Q-85…Q-109 in
  `docs/open-questions.md`; closed questions (including by owner decisions OD-5) and narrowed questions are marked there too.
- Owner decisions OD-5 (`docs/owner-decisions.md`) are incorporated: [DEFAULT] D-1…D-18 confirmed (ADRs moved to Accepted,
  label [USER]); Q-90 — the owner's alternative: the date is required for rename and delete, substituting 1876-03-07 only on
  add (narrowing of ADR 0002 item 4; affects CUJ1, CUJ3, CUJ6, CUJ7, CUJ9); D-19 — the owner's alternative: a word is a UAX #29
  segment that does not consist only of whitespace characters and punctuation (symbols and emoji are words; ADR 0030 item 3).
- No BREAKING changes; the narrowing of X-5 and CUJ1, 7, 14, 15 (D-13, D-17) and the date in rename and delete (Q-90) are owner
  decisions (OD-5); v1 has not been released yet, so there is no previous contract.

## Capabilities

### New Capabilities

- `records`: the record (number, date) → name, uniqueness and identity; normalization and validity of name, number and date.
- `write-operations`: add, rename, delete; number version; errors, check order, idempotency,
  atomicity.
- `name-search`: word-prefix search and exact whole-name search; comparison key; cursor and page; errors.
- `number-search`: search by phone number by canonical form; all records of a number, date DESC, no pagination.
- `result-ordering`: result order for name search (ru, then date DESC, then number); completeness and stability of the order.

### Modified Capabilities

- `user-journeys`: requirement X-5 "Number version" — wording aligned with the version generation and with a number that has no
  records (closes Q-91), with a guarantee boundary across generations (D-17) and the D-13 exception (owner decisions, OD-5);
  CUJ1, CUJ2, CUJ3, CUJ5, CUJ6, CUJ7, CUJ8, CUJ9, CUJ12 — references to decisions instead of placeholders for closed questions;
  CUJ1, CUJ3, CUJ6, CUJ7, CUJ9 — the date is required in rename and delete (OD-5, Q-90); CUJ1, CUJ7, CUJ14, CUJ15 — a caveat to
  the version guarantee; the other requirements do not change.

## Impact

- ADR 0020–0039 (new); they build on ADR 0002–0010, 0014.
- Requirements for other areas (storage, API, UI, administration, build) are collected in `design.md`, section "Requirements
  for other areas".
- Documents: `facts.md` (F-3…F-51), `docs/open-questions.md` (13 closed, 10 narrowed, 25 new Q-x; then OD-5: 8 closed (O) and 19 [DEFAULT]),
  `docs/adr/README.md`; ADR 0006 (only the notes block with references: clarification of item 6(a),
  caveats for D-13, D-17; the decision text was not changed), ADR 0002 and 0005 (only notes on narrowing per Q-90; the [USER] text was not changed).
- No code: the change defines requirements, not implementation.
