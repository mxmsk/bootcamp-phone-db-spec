## 1. Change artifacts

- [x] 1.1 proposal.md, design.md, specs/user-journeys/spec.md are written in Russian
- [x] 1.2 spec.md has a Purpose with the original statement (7 items) and the requirements CUJ1–CUJ16 and X-1…X-5

## 2. Related project artifacts

- [x] 2.1 ADRs `docs/adr/0001`…`0019` are created and each has the sections: context, decision, alternatives, consequences, CUJ,
  sources, label, status
- [x] 2.2 `docs/open-questions.md` (open questions Q-x and [DEFAULT]) and `docs/owner-decisions.md` are created; `docs/source/brief.md`
  and `facts.md` are moved into the project
- [x] 2.3 `openspec/config.yaml`: context is filled in (the original statement and the specification rules)

## 3. Verification

- [x] 3.1 Every reference "ADR NNNN" and "Q-x" in spec.md points to an existing ADR and an existing question; the "Decisions" lists
  in spec.md and the "CUJ" fields in the ADRs are consistent in both directions
- [x] 3.2 Every requirement traces to a CUJ or an item of the original statement; there are no CUJ steps without a requirement
- [x] 3.3 `openspec validate add-user-journeys --strict` passes
- [ ] 3.4 The specification review is passed

## 4. Completion

- [ ] 4.1 The change is archived (`openspec archive add-user-journeys`)
