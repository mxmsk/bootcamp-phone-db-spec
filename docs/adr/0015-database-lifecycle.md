# 0015. Creating the database and the single server process
- Status: Accepted
- Label: [USER]
- Date: 2026-10-27
- CUJ: CUJ13, X-2

## Context
CUJ13: first start. An empty database is created by a separate command; the server does not start without it.

## Decision
1. An empty database is created by an explicit operator command; without it the server does not start on a directory
   with no database and says so (brief §3/Scale and reliability; brief §2, CUJ13).
2. The create command on a directory where a database already exists is a refusal; it never overwrites (OD-3, Q-7,
   docs/owner-decisions.md).
3. A second server on the same database is a start refusal, «база уже открыта другим процессом» ("the database is already open by another process") (OD-3,
   Q-7). This agrees with the decision "one server process owns the database" (brief §3/Interfaces; ADR 0011).

## Alternatives considered
- Creating the database automatically at server start: excluded, "the server does not start without it" (brief
  §3/Scale and reliability).
- No other alternatives were considered in the sources.

## Consequences
- Open question: the channel through which the server tells the operator the reason for a start refusal (Q-66); see
  `docs/open-questions.md`.
- A corrupted database at start: ADR 0014.

## Sources
- `docs/source/brief.md`, section 3 "Scale and reliability", "Interfaces"; section 2 (CUJ13).
- Owner decision of 2026-10-27 (OD-3), Q-7: "the create command on a directory where a database already exists is a
  refusal, it never overwrites; a second server on the same database is a start refusal 'the database is already open
  by another process'". Place of record: docs/owner-decisions.md, OD-3.
