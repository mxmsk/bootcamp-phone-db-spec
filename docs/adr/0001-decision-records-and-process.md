# 0001. ADR format, labels and the order for resolving technical items
- Status: Accepted
- Label: [USER] (items 1, 3, 4, 5); [OURS] (items 2, 6, 7)
- Date: 2026-10-27
- CUJ: not tied to a specific CUJ; the process of building the specification (brief §6, OD-1, OD-2)

## Context
Here and below, "the brief" means `docs/source/brief.md` (sections 1–6; an unchanged copy of the original source), "OD-x" means entries of the owner decision log
`docs/owner-decisions.md`, and "F-x" means the verified facts in `facts.md`.
The "CUJ breakdown stage" (OD-1) is the analysis of the CUJs into verifiable requirements and scenarios (change
`add-user-journeys`).

The brief deliberately contains no technical decisions: the new specification makes them, with rationale (brief,
introductory paragraph). Some items of the brief (sections 4 and 5) require "propose and ask" the owner, but without a
data model, storage and API they cannot be proposed on a sound basis. A single format for recording decisions is
needed, plus a rule for when and how such items are brought to the owner.

## Decision
1. ADRs are stored as `docs/adr/NNNN-slug.md`, MADR-like, in English (product UI and CLI messages are in Russian). Fields: context, decision, alternatives
   considered with the reason for rejection, consequences, trace to CUJ, sources, label, status (OD-2). Changed: the repository was translated into English on 2026-10-29 at the owner's request; product UI and CLI strings remain Russian.
2. Labels: [USER] is a decision of the product owner; [DEFAULT] is a default value that nobody has confirmed (brief
   §5; OD-1); [OURS] is a decision of the orchestrator/specification, not of the owner (`docs/owner-decisions.md`:
   recommendations that the owner accepted were drafted by the orchestrator [OURS]; category (T), item 6). Status
   Proposed means the decision has not yet been accepted by the owner (OD-1: "Proposed, to be decided in iteration X").
3. The technical items of brief sections 4–5 (for example: the storage page, file size and latency targets, the API
   error body, server limits, isolation level, write latency, the operator control channel, test data generation) are
   not decided at the CUJ breakdown stage. They are recorded as "Proposed, to be decided in iteration X" with the label
   [DEFAULT] and are brought to the owner with concrete numbers and options in the corresponding iteration: data
   model, storage, API, UI (OD-1).
4. This does not remove the brief's "propose and ask" requirement: each such item still goes to the owner, later
   (OD-1). At the CUJ breakdown stage the owner is asked only product questions whose answer does not depend on a data
   model, storage or API that has not been designed yet (OD-1).
5. Principles for building the specification (brief §6; the `process.md` the brief refers to is not part of the
   project, so below is how they are applied in the project): Critical User Journey is the user's path to a goal, to
   which every requirement traces (capability `user-journeys`); Elephant + Goldfish means everything needed is kept in
   project files (the "elephant" remembers) while the session reading it starts from scratch (the "goldfish" does not
   remember), so nothing may depend on session memory; (architectural) Decision Record means every decision is
   recorded as an ADR with alternatives; Fact Grounding means a claim about the outside world rests on a verified fact
   (`facts.md`) or a label; LLM-expert review is an independent review of the specification by a reviewer model,
   conducted once the whole specification is ready.
6. Classification of ambiguities, derived from OD-1: (P) product: a question to the owner right away; (O): to the owner
   in iteration X, status "Proposed, iteration X [DEFAULT]"; (T) technical: decided by the specification [OURS] with
   rationale, the owner is not needed (OD-1).
7. [OURS] Order of iterations: data model, then storage, then API, then UI. OD-1 lists the iterations but does not set
   an order. Rationale: storage depends on the data model (for example, the storage page Q-48 depends on the name
   length cap Q-55); the API contract depends on the model and storage; the UI is a client of the API. The order may be
   changed if the dependency is reversed.

## Alternatives considered
- Decide the technical items of sections 4–5 at the CUJ breakdown stage. Rejected: without a data model, storage and
  API they cannot be proposed on a sound basis (OD-1).
- A different order of iterations (for example, UI first): not chosen, because the UI depends on the API contract.
- No other alternatives were considered in the sources.

## Consequences
- Open (O) and (T) items are collected in `docs/open-questions.md`; the resolved parts are recorded as ADR 0002–0018,
  and the decisions of brief section 4 that have not yet been revised are in ADR 0019 (Proposed).
- The facts the ADRs rely on (F-1, F-2) are re-checked by the reviewer against the original source before being
  relied on (facts.md).
- An ADR that contains a part labelled [DEFAULT] has status Proposed for that part.

## Sources
- Owner decision of 2026-10-27 (OD-1): the technical items of brief sections 4–5 are not decided at the CUJ breakdown
  stage, are recorded as "Proposed, to be decided in iteration X" with the label [DEFAULT] and are brought to the
  owner later with numbers and options; place of record: `docs/owner-decisions.md`, OD-1.
- Owner decision of 2026-10-27 (OD-2): ADR format `docs/adr/NNNN-slug.md`, MADR-like, in Russian; fields: context,
  decision, alternatives with the reason for rejection, consequences, trace to CUJ, sources, label, status; place of
  record: `docs/owner-decisions.md`, OD-2.
- `docs/source/brief.md`, section 5 (unresolved), section 6 (principles for building the specification).
