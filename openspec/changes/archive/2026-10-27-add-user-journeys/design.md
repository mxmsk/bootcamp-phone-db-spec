## Context

The `user-journeys` capability is the traceability root of the project: the original task statement (7 items) and the user journeys
CUJ1–CUJ16. CUJ1–CUJ6 are the owner's original text; CUJ7–CUJ16 were formulated from the owner's answers and confirmed by the owner.
All other artifacts (ADRs, future capabilities for the data model, storage, API and UI) refer to the CUJs.

## Sources

The project is self-contained: `docs/source/brief.md` (the brief, sections 1–6),
`docs/owner-decisions.md` (the owner's decisions OD-1…OD-4 after the brief), `facts.md` (F-x), `docs/adr/`,
`docs/open-questions.md`. The specification refers only to them.

## Goals / Non-Goals

**Goals:**
- A session that reads only the project sees what is being built and why.
- Every CUJ behavior is a verifiable WHEN/THEN scenario.
- Every CUJ is linked to decisions (ADRs) and open questions.

**Non-Goals:**
- There are no technical decisions here: formats, protocols, numbers and mechanisms are settled in ADRs and later capabilities.
- There is no code and no planning of it.

## Decisions

### Traceability CUJ — ADR — open questions

- CUJ → ADR: every requirement has a line "Decisions: ADR NNNN, …" (files `docs/adr/NNNN-slug.md`, the list 0001–0019).
  An ADR lists in its "CUJ" section which journeys it serves.
- CUJ → open questions: a parameter that the owner's decisions leave open is written in the scenario as
  "<not decided: Q-x>"; Q-x is described in `docs/open-questions.md` (iteration, options, status). A requirement does not invent
  a value in place of an open one.
- Cross-cutting requirements X-1…X-5 exist only if they trace to a CUJ or to an item of the original statement; an owner
  decision to limit scope is a valid trace for a requirement that limits scope (X-3). The rule is the same as in
  `config.yaml`; the trace is given in the requirement text.
- Reverse check (the project rule from `config.yaml`): a requirement without a trace is extra scope; a CUJ step without a requirement
  is a gap.

### Decision labels

- [USER] is an owner decision; [OURS] is a project decision with a rationale; [DEFAULT] is a value that the owner has not
  confirmed and that is put to the owner. The label sits in the ADR; in scenarios [DEFAULT] is marked with the words "[DEFAULT] D-x, ADR NNNN"
  (for example, the first digit of a number 1–9: [DEFAULT] D-1, ADR 0004).

### Order of closing open questions

Iterations: data model, storage, API, UI (owner decision OD-1, `docs/owner-decisions.md`; OD-1 does not
set an order). The order is [OURS]: data model, then storage, then API, then UI; the rationale and alternatives are in ADR 0001,
item 7. The decision of each iteration is recorded as an ADR (or an update of an existing one), after which "<not decided: Q-x>" in
scenarios is replaced with a concrete value by a separate change.

## Risks / Trade-offs

- [A scenario with "<not decided: Q-x>" cannot be verified until the question is closed] → such scenarios verify the rest of the behavior;
  the parameter is verified after the iteration.
- [A discrepancy between the CUJ text and the owner's decisions] → the owner's decision prevails (for example, CUJ5: word start, not
  substring); it is recorded in the requirement text.

## Open Questions

The list is in `docs/open-questions.md`.
