# 0019. Decisions of brief section 4 that have not yet been revised
- Status: Accepted (OD-6)
- Label: [DEFAULT]
- Date: 2026-10-27
- CUJ: CUJ1, CUJ2, CUJ4, CUJ5, CUJ7, CUJ8, CUJ9, CUJ11, CUJ12, CUJ16 (by item below)

## Context
Brief §4 lists owner decisions made under the old constraints (a ban on crates, the "name–phone pair" model); they must
be proposed anew or replaced, with a question to the owner. By decision OD-1, technical items are not decided at the CUJ
breakdown stage, but are brought to the owner in the corresponding iteration with numbers and options (ADR 0001).

## Decision
The status of each item of brief section 4 as of 2026-10-27:

Not yet revised (Proposed [DEFAULT]; brought to the owner in the stated iteration, OD-1):
1. The targets "file ≤ 40 GB for 10^8 records" and "search p99 ≤ 10 ms": calculated for the "name–phone pair" model; the
   owner confirmed keeping them when choosing word-prefix search (brief §4); recalculate for the new model. Iteration:
   storage (Q-47).
2. A 16 KiB storage page with a proven key length bound (brief §4). Iteration: storage (Q-48; linked to the name length
   cap Q-55).
3. The API error body `{code, field}`, HTTP status per code (brief §4). Iteration: API (Q-49).
4. Server limits: 256 connections, request body 32 KiB, request timeout 10 s, connection idle 60 s; accepted under the
   "server on loopback only" model, before network clients appeared (brief §4). Iteration: API (Q-52).
5. A test tool for populating the database to check scale 10^8 is not a product operation (the option the owner chose
   when giving up batch load) (brief §4). Iteration: storage (Q-53).
6. A repeated rename after a connection drop gives "not found" (accepted before the number version appeared); a note of
   the previous orchestrator: probably now closed by versioning, to be checked (brief §4). Iteration: API (Q-54).
   Revised per the semantics of ADR 0026 (Accepted, owner decision OD-5, D-7): Q-54 is narrowed.

Items of section 4 that are already closed and not part of this ADR:
7. There is no rollback to an old version of the program (and format): confirmed by the owner (OD-3, Q-22; OD-4, item 8),
   ADR 0013.
8. Strict number format: the policy is confirmed (OD-3, Q-23), ADR 0004; the interpretation of "(E.164)" (D-1) is
   confirmed by the owner (OD-5), ADR 0004.

The unresolved items of brief section 5 (name length, isolation, write acknowledgment, latencies, request limit, TLS,
operator command channel, interface behavior) are collected in `docs/open-questions.md`.
9. **Changed by OD-6 (2026-10-28).** The scale is 10^7 records (ADR 0017); the numbers and estimates for 10^8 in this ADR
   are calculation history, the current values for 10^7 are in ADR 0044, 0049, 0052, 0058, 0059; non-linear estimates
   are recalculated on a prototype (T, `docs/open-questions.md`).

## Alternatives considered
- Carry over the decisions of section 4 without revision: rejected by the brief, "the new specification must propose
  them anew or replace them, with a question to the owner" (brief §4).
- No other alternatives were considered in the sources.

## Consequences
- Until items 1–6 are brought to the owner, the values from section 4 are not considered current requirements.
- Items 1, 2, 5 affect the scale (ADR 0017); items 3, 4, 6 affect the v1 API contract (ADR 0013).

## Sources
- `docs/source/brief.md`, section 4 "Owner decisions made under the old constraints: to be revised before carrying over".
- Owner decision of 2026-10-27 (OD-1): technical items are recorded as "Proposed, to be decided in iteration X" with the
  label [DEFAULT] and are brought to the owner later with numbers and options. Place of record:
  `docs/owner-decisions.md`, OD-1.
- Owner decision of 2026-10-27 (OD-3), Q-22, Q-23; OD-4, item 8. Places of record: `docs/owner-decisions.md`, OD-3 and
  OD-4.
