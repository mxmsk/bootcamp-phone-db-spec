## 1. Preparation for review

- [x] 1.1 Make sure `openspec validate add-data-model --strict` passes; check: the command exits with code 0.
- [x] 1.2 Make sure ADR 0020–0039, the delta-specs, `design.md`, `facts.md` and `docs/open-questions.md` contain no
  temporary subtask identifiers and no references to drafts; check: `grep -rE "F-[R]|F-or[d]|Д-[R]|Д-writ[e]|Д-name-searc[h]|Д-or[d]|Н-[R]|Н-or[d]|N-[F]|\bite[r]\b|draft[s]/|замет[к]" docs/adr openspec/changes/add-data-model facts.md docs/open-questions.md`
  finds nothing.
- [x] 1.3 Reconcile cross-subtask references: the names of the `records` requirements ("Name and query normalization",
  "Text length and length cap", "Value error kind dictionary") and the `write-operations` requirement ("Number version in
  search results") in `write-operations`, `name-search` and `number-search` match the headings; check: no headings are left in the old wording.

## 2. Review

- [x] 2.1 Opus review: consistency, conformance with `user-journeys` (including the changed X-5), ADRs,
  `docs/owner-decisions.md` (by sources and criteria, without history); check: a list of findings with classes
  BLOCKER/MAJOR/MINOR is obtained.
- [x] 2.2 Sonnet review: feasibility in Rust; check: a list of findings with classes is obtained.
- [x] 2.3 BLOCKER/MAJOR findings are worked through (next iteration, at most 5 iterations per area); check: no BLOCKER or
  MAJOR remains, or the remainder is recorded in `docs/open-questions.md`; after the 5th iteration a BLOCKER is raised as a
  question to the owner.
  Done: after the 5th iteration there is no BLOCKER; MAJOR (Opus R-1…R-4, Sonnet S-1…S-3) and MINOR were moved to the
  section "Remainder of the area 1 review" (Q-101…Q-109; R-3 — D-19, resolved by the owner in OD-5).
- [x] 2.4 Move the review remainder to `docs/open-questions.md`; the imprecise figures (800 bytes of input, 8 513 bytes of
  sort key) are marked in `design.md` and ADR 0022 with references to Q-101, Q-103; check: the section and the marks are
  present.

## 3. Owner decisions

- [x] 3.1 Hand the owner the package (27 items, prepared): all [DEFAULT] D-1…D-19 and the
  questions (O) that affect owner decisions: Q-85, Q-87, Q-88, Q-90 (whether to narrow zero-date substitution on rename and
  delete), Q-93, Q-97, Q-98, Q-100 (O) (a choice, if the order test on Unicode 18 characters shows a discrepancy) and the
  narrowed ones; technical (T) — Q-74, Q-76, Q-92, Q-95, Q-99 and Q-100 (T) — are not handed to the owner (Q-95: the owner
  is asked only if there is a vulnerability that changes the order, at the moment it arises; the fix policy is D-18);
  check: the list is handed over, the owner's answers are entered into the ADRs (status Accepted).
  Done: the package was handed over, the answer is `docs/owner-decisions.md`, OD-5 (entered: ADRs, delta-specs,
  `docs/open-questions.md`).
- [ ] 3.2 Perform (T) Q-100 before the v1 release: an `icu_collator` order test on Unicode 18 characters (list
  `research/name-search/out-uax29-diffage.txt`); check: the result is recorded; on a discrepancy the question (O) Q-100 is
  handed to the owner (the decision comes after the check result, OD-5). Open until the v1 release.
- [x] 3.3 Owner's answer to the package: the answers are entered into the ADRs (status
  Accepted); check: every D-1…D-19 and every (O) question of the package has an answer. Done (OD-5): D-1…D-18 and Q-85,
  Q-87, Q-88, Q-93, Q-97, Q-98 — package defaults; Q-90 and D-19 — the owner's alternatives (ADR 0025 item 6, ADR 0030
  item 3); Q-100 (O) — the order of decision is set (the choice follows the test result; on a discrepancy, to the owner),
  not closed; Q-100 (T) remains task 3.2.

## 4. Tests arising from the area (for implementation)

- [ ] 4.1 Record in the test plan: boundary values of `records`; the product functions `normalize_name`, `name_key`,
  `words`, `order_key` run `research/ordering/expected-order.txt`, the order corpus and the references
  `out-uax29-golden.txt`, `out-uax29-wbt.txt`, `out-uax29-data.txt` (`research/` holds prototypes that produce the
  snapshots; the "word" predicate of the `out-uax29-golden.txt` reference — per OD-5, D-19: not Z*/P*/Cc/Cf, regenerated
  2026-10-27); the control set `research/name-search` (`uax29 wbt`, `uax29 data`, `uax29 gc` on an `icu_properties`
  update, the `out-uax29-golden.txt` reference on the pinned build); the order corpus `research/ordering/order-corpus`
  with the snapshot `order-corpus.sha256` (`check-corpus.sh` in each release build checks both the hash and the byte
  order of the sort key against `compare()`); normative cursor vectors before the first release, then a cursor golden
  test; the version mechanism across generations (injected randomness source and generation counter storage; epoch
  capacity of at least 128 bits); size checks (input in bytes — Q-101; sort key, 0x00 and prefixes — Q-103, Q-109); a
  re-run of the `research/records` check (including the `sortkey-check` key sizes) on the build's data version; the ICU4X
  pin (direct `icu_*` per `research/ordering/icu-pin/expected-icu-tree.txt` — 15, without `icu_segmenter`, all `=2.4.0`,
  `default-features = false`, `compiled_data`, `Cargo.lock`, `--locked`; `check-pin.sh [directory]` is called on the
  product's root package, checks the features of all `icu_*` and the direct `=2.4.0`, warns on `[patch]`);
  `rust-version = "1.88"`; vendored UCD 18.0.0 files with SHA-256 and a build without network (`--locked --offline`);
  check: the items are present in the test plan of area 2/the build.

## 5. Completion

- [ ] 5.1 Archive the change (`openspec archive add-data-model`) after the owner's answer (received, OD-5); check: the
  delta-specs (including X-5 in `user-journeys`) are moved to `openspec/specs/`, the change is moved to the archive.
