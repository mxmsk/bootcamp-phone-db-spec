# 0010. Unicode stack: one version of ICU4X, Unicode 18 for all of v1
- Status: Accepted
- Label: [USER]
- Date: 2026-10-27
- CUJ: CUJ5, CUJ11, X-4; item 1.7 of the original wording

## Context
Word-prefix search, case, "ё" = "е", NFC and ru sorting depend on the Unicode/CLDR version. Per F-2, stored sort keys
must be considered invalid when CLDR, Unicode or the ICU4X code is updated.

## Decision
1. The whole Unicode stack is ICU4X of one version, Unicode 18 (brief §3/Implementation; F-2).
2. Unicode 18 is fixed for all of v1; a server update within v1 does not change what search finds or in what order
   (OD-4, item 2).

## Alternatives considered
- Updating the Unicode/CLDR version within v1 with a note in the release description (package 1 recommendation, Q-21:
  "yes, only when the Unicode/CLDR version changes"): cancelled by OD-4, item 2; fixing Unicode 18 for all of v1
  applies.

## Consequences
- Facts F-2 (verified 2026-10-27): ICU4X components version 2.4.0, published 2026-10-22; CHANGELOG 2.4: "Updated to
  Unicode 18", "Updated to CLDR 49"; the sort key format changed incompatibly in 2.4; stored keys must be considered
  invalid when CLDR, Unicode or the ICU4X code is updated; no public runtime constant for the Unicode/CLDR version was
  found.
- Updating the ICU4X code within v1 must not change the visible matches and the search order; how to reconcile this with
  stored data is Q-74 (T); the effect on start and operator actions is Q-82; the choice of ICU4X functions and
  configuration is Q-76 (T); the comparison parameters are Q-31.

## Sources
- `docs/source/brief.md`, section 3 "Implementation"; F-2 (facts.md; primary sources: crates.io API, ICU4X CHANGELOG,
  docs.rs icu_collator).
- Owner decision of 2026-10-27 (OD-4), item 2: "Unicode 18 is fixed for all of v1; a server update within v1 does not
  change what search finds or in what order". Place of record: docs/owner-decisions.md, OD-4.
- Owner decision of 2026-10-27 (OD-3), Q-21 (cancelled). Place of record: docs/owner-decisions.md, OD-3.
