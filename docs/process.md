# How the specification is built (owner's scheme, 2026-10-27)

Principles: Critical User Journey (everything traces to `user-journeys` or to the original wording), Elephant + Goldfish
(the orchestrator holds the memory; agents are fresh, the reviewer gets only sources, the thing under review and
criteria, with no history and no hints), Decision Record (`docs/adr/`), Fact Grounding (`facts.md`), LLM-expert review
once the whole spec is ready.

Areas in order; each is a separate OpenSpec change, decisions go to ADRs:
1. Data model and operations. 2. Storage. 3. Server and API. 4. Administration (operator CLI). 5. UI.

Area iteration (owner's scheme of 2026-10-27, second edition; the goal is to get the first complete version of the spec
quickly):
1. The area is split into subtasks; each is done in parallel by a fresh agent (Sonnet). The spec states observable
   behavior and guarantees; implementation mechanisms go to ADRs and design.md. Typical questions are closed with
   typical default behavior ([DEFAULT] with a rationale); work does not stop.
2. One Opus review per area: BLOCKERs and contradictions only (between capabilities, with accepted areas, with owner
   decisions); no MINORs.
3. One fix pass (Sonnet): BLOCKERs and contradictions, plus consolidation of shared documents (open-questions, facts,
   the ADR README). There is no repeat review; everything else goes to docs/open-questions.md as implementation
   questions. A BLOCKER that cannot be closed without the owner becomes an explicit question to the owner.
4. The change is archived; [DEFAULT]s accumulate in docs/open-questions.md and are brought to the owner in one package
   once the spec is ready.
5. Once the whole spec is ready: a Rust feasibility review (Sonnet) and an LLM-expert review (Opus) over the whole spec.

Finding classes: BLOCKER: a contradiction or underspecification that makes it impossible to implement or verify a
requirement, or a false statement on which a guarantee rests; MAJOR: a scenario that does not follow from the rules, a
wrong reference, unjustified scope, a distortion of an owner decision; MINOR: clarity.
