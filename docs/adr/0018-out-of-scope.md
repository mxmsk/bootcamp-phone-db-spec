# 0018. Out of scope
- Status: Accepted
- Label: [USER]
- Date: 2026-10-27
- CUJ: CUJ10, CUJ11, CUJ15, X-3 (describes the absence of functions)

## Context
The brief explicitly excludes a number of functions so that they do not enter the specification and the v1 API.

## Decision
1. Out of scope: batch load and import (the typical story is a person typing a number in by hand); a "who changed what"
   log; administration through the API; the policy for the v1 → v2 transition (brief §3/Out of scope).
2. Tokens and backup are managed only by an operator command on the server machine (brief §3/Interfaces; brief §2,
   CUJ10, CUJ15).
3. There is no "read-only" mode on operator command (OD-3, Q-17).
4. **Changed by OD-6 (2026-10-28).** The scale is 10^7 records (ADR 0017); the numbers and estimates for 10^8 in this
   ADR are calculation history, the current values for 10^7 are in ADR 0044, 0049, 0052, 0058, 0059; non-linear
   estimates are recalculated on a prototype (T, `docs/open-questions.md`).

## Alternatives considered
- Batch load and import: rejected, "out of scope" (brief §3/Out of scope). A test load-generation tool for checking
  10^8 is a separate question (Q-53, ADR 0019).
- No other alternatives were considered in the sources.

## Consequences
- There is no change log (brief §3/Out of scope), so an overwrite does not keep the earlier name (CUJ3, brief §2; a
  consequence, not a separate owner decision).
- A test tool for populating the database to check 10^8 is not a product operation; its composition is not yet
  confirmed (Q-53, ADR 0019).

## Sources
- `docs/source/brief.md`, section 3 "Out of scope", "Interfaces".
- Owner decision of 2026-10-27 (OD-3), Q-17. Place of record: docs/owner-decisions.md, OD-3.
