# 0004. Phone number format on input
- Status: Accepted
- Label: [USER] (items 1–5; item 6: OD-5, D-1)
- Date: 2026-10-27
- CUJ: CUJ1, CUJ2, CUJ4, CUJ9; items 1.3, 1.4 of the original wording

## Context
CUJ1 requires entering a number in the international format with "+<country code>". Brief §4 counts the strict number
format among the decisions made under the old constraints, and says the specific list of separators and other
processing details are not owner decisions. The owner confirmed the input policy, including the list of separators,
when answering ambiguity Q-23 (OD-3).

## Decision
1. A number is entered in the international format with "+" and a country code (brief §3/Data).
2. "+" is mandatory, followed by 1–15 digits (OD-3, Q-23; the source has the note "(E.164)").
3. Spaces, hyphens, parentheses and dots are removed; numbers written differently are one number after cleaning (OD-3,
   Q-23).
4. A number without "+" is rejected with a hint (OD-3, Q-23).
5. The same rules apply to search by number: a number of invalid shape is a format error (OD-3, Q-6; see ADR 0007).
6. [USER, OD-5, D-1] Interpretation of the note "(E.164)": "+", then 1–15 digits, the first digit 1–9; there is no
   check against a list of country codes. Consequences: "+0123…" is rejected; a number of valid shape with a code not
   assigned to any country is accepted (`docs/open-questions.md`, D-1). Rationale: [OURS], with no verified source
   (`facts.md` has no fact about the structure of E.164 codes); the claim that a country code does not start with 0 is
   not verified. The value narrows the owner's decision (OD-3, Q-23: "1–15 digits"); the owner confirmed it (OD-5, D-1).

## Alternatives considered
- Length 1–15 only, with no restriction on the first digit (exactly the owner's decision, OD-3, Q-23): rejected by the
  owner (OD-5, D-1): "+0…" is clearly not like a number in the international format (this is not confirmed by a fact).
- A full E.164 check with assigned country codes: not accepted, because it needs an external list of codes that is not
  among the verified facts, and the owner's decision does not require such a check (OD-3, Q-23); rejected by the owner
  (OD-5, D-1).
- A number without "+" is accepted: rejected (OD-3, Q-23).
- No other alternatives were considered in the sources.

## Consequences
- The list of separators (spaces, hyphens, parentheses, dots) is confirmed by the owner (OD-3, Q-23). Other processing
  details are not owner decisions (brief §4): the set of "whitespace" characters is defined by the specification (Q-84,
  category (T)), the internal representation of a number is Q-75 (T).
- Open questions: how a number looks in UI results and in API responses (Q-26); how to express a "hint" for an API
  client (Q-49); D-1 is confirmed by the owner (OD-5).

## Sources
- `docs/source/brief.md`, section 3 "Data"; section 4 (strict number format).
- Owner decision of 2026-10-27 (OD-3), Q-23: "confirm: '+' is mandatory, followed by 1–15 digits (E.164); spaces,
  hyphens, parentheses and dots are removed; a number without '+' is rejected with a hint; numbers written differently
  are one number after cleaning"; Q-6 (format in search). Place of record: docs/owner-decisions.md, OD-3.
- Value of D-1: a project proposal, confirmed by the owner (OD-5, `docs/owner-decisions.md`).
