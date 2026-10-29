# Proposal

## Why

A session started from scratch and reading only the project must see what is being built and why: the original task statement,
the user journeys (CUJ1–CUJ16) and the owner's decisions. Without this, the requirements, ADRs and later iterations (data model,
storage, API, UI) have nothing to trace to, and the open questions are not visible.

## What Changes

- A new capability `user-journeys`: the original task statement (7 items), CUJ1–CUJ16 as separate requirements with
  WHEN/THEN scenarios, cross-cutting constraints X-1…X-5 with a trace to the original statement and the owner's decisions.
- ADRs in `docs/adr/` (0001–0019): one per coherent topic of owner decisions; CUJs refer to them with a "Decisions" line.
- Open questions and [DEFAULT] values go in `docs/open-questions.md`; in scenarios they are marked "<not decided: Q-x>".
- `openspec/config.yaml`: context with the original statement and the specification rules.
- `docs/source/` (the brief), `docs/owner-decisions.md` (the owner decision log OD-1…OD-4),
  `facts.md`: the sources that the ADRs and the specification refer to.
- There is no code implementation in this change.

## Capabilities

### New Capabilities

- `user-journeys`: user journeys (UI, programmatic API, operator) to which all project requirements trace.

### Modified Capabilities

(none)

## Impact

- New files: `openspec/changes/add-user-journeys/`, then (on completion) `openspec/specs/user-journeys/spec.md`.
- Related artifacts outside the change: `docs/adr/*.md`, `docs/open-questions.md`, `docs/source/*`, `docs/owner-decisions.md`,
  `facts.md`.
- Code and dependencies are not affected.
