# 0039. Stability of the result order within v1
- Status: Accepted (item 10, part (T) of Q-100, — a test before the v1 release)
- Label: [OURS]; [USER] (item 6 (c) — OD-5, D-18)
- Date: 2026-10-27
- CUJ: CUJ5, CUJ11

## Context
ADR 0010 [USER]: Unicode 18 is fixed for all of v1, and a server update within v1 does not change matches or order. Per F-2,
ICU4X sort keys "should be presumed to be invalidated by a CLDR update, a new version of Unicode, or an update to the
ICU4X code"; in 2.4 the key format changed incompatibly. ICU4X does not promise that behavior stays unchanged between code versions, and the
order is set by the ICU4X comparison function itself, so "rebuild the index from the names" does not compensate for a change to it.
Q-74: how to preserve the visible order; Q-82 — the effect on start — is decided in the storage area.

## Decision
1. The visible order is defined by the comparison function of this specification (ADR 0036–0038), not by the bytes of stored keys.
   Stored sort keys are derived data, not the source of truth: the name is stored itself, and the key is computable from it.
2. An honest statement of the guarantee. "Order and matches do not change within v1" holds not because ICU4X
   is stable, but because within v1 the ICU4X code and data do not change: the `icu_*` version is pinned (item 4), and updating
   ICU4X in v1 is not a routine operation but an exception under the procedure of item 6. The specification does not promise "the order does not change when the ICU4X
   code is updated". The guarantee of OD-4 item 2 ([USER]) is unconditional; the default of item 6 keeps it
   unconditional (a backport without changing data or comparison), while allowing an update on a corpus match narrows the guarantee,
   so it was put to the owner with alternatives (item 6); the owner confirmed the default (OD-5, D-18).
3. For storage (a requirement for area 2): the prohibition of item 6 concerns changing the visible order, not rebuilding stored
   derived data. The order index is built from sort keys computed by code with the pinned ICU4X (item 4);
   the keys are derived data, and the index SHALL allow a rebuild from the stored names with the same comparison function: a change of the
   key record format in our code, a restore, an index error, and also an ICU4X update (alternative 1) of item 6 (c), rejected by the owner, OD-5, D-18; under it the ICU4X key
   format would change; ICU4X keys are "presumed invalidated" on a code update, F-2).
   In all these cases
   the result order before and after the rebuild is the same. A rebuild is not a means to change the order: an ICU4X version cannot be
   accepted "through a rebuild" if after it at least one reference of item 5 gives a different result (item 6 (d)). For this the stored
   key SHALL carry a label of its format (the format version of our code and the `icu_collator` version); if it does not match the labels of the
   running server, the index is rebuilt before requests are accepted. The effect on start and operator actions — Q-82 (storage).
4. Pinning (the whole `icu_*` graph and features). Verified 2026-10-27 by running `cargo tree` (`research/ordering/icu-pin`):
   the full Unicode stack set of the product (`icu_collator` + `icu_locale` for order, `icu_normalizer`, `icu_casemap`,
   `icu_properties` for search) pulls in exactly 15 `icu_*` crates, all at version 2.4.0: `icu_casemap`,
   `icu_casemap_data`, `icu_collator`, `icu_collator_data`, `icu_collections`, `icu_locale`, `icu_locale_core`,
   `icu_locale_data`, `icu_locale_fallback`, `icu_locale_fallback_data`, `icu_normalizer`, `icu_normalizer_data`,
   `icu_properties`, `icu_properties_data`, `icu_provider` (the list is `research/ordering/icu-pin/expected-icu-tree.txt`;
   the number is set by that file, not by the text). `icu_segmenter` and `icu_segmenter_data` are not in the product list: word boundaries
   are set by UAX #29 over `icu_properties` (variant B, ADR 0030 items 3–4); the segmenter is allowed only in `research/` for
   cross-checking (F-35). Variant A of D-14 was rejected by the owner (OD-5), and the list stays as it was. Requirements:
   - (a) each of the 15 crates is a direct dependency in `Cargo.toml` with the exact version `=2.4.0`, including `*_data`,
     `icu_provider`, `icu_collections`, `icu_locale_core`, `icu_locale_fallback`. `=2.4.0` on the top crates alone is not enough:
     the `icu_collator` 2.4.0 crate itself requires `icu_collator_data = "~2.4.0"`, `icu_collections = "~2.4.0"`,
     `icu_provider = "2.4.0"` (that is, `^2.4.0`), and `cargo update` could raise them to 2.4.x or 2.x (verified from the
     `Cargo.toml` of the downloaded crate);
   - (b) `Cargo.lock` is committed, and all builds and tests run with `--locked`; this also pins the other transitive crates
     (zerovec, yoke, tinystr and others, 22 crates outside `icu_*` in the `icu-pin` verification project): changing them does not change Unicode data, but any
     edit to `Cargo.lock` must pass the reference tests (item 5);
   - (c) features are set explicitly on every direct `icu_*`: `default-features = false`, and for the data and core crates
     (`icu_collator`, `icu_locale`, `icu_locale_fallback`, `icu_normalizer`, `icu_properties`, `icu_casemap`)
     `compiled_data`; `auto`, `lstm`, `unstable`, `datagen`, `serde` are not enabled. Verified 2026-10-27: the
     `research/ordering/icu-pin` project builds and works with these features (the ru collator, `CaseMapper`,
     `DecomposingNormalizerBorrowed::new_nfd`, `CodePointMapData::<WordBreak>`, `CodePointSetData` Extended_Pictographic;
     F-30). `research/name-search` is built on the same pinned set (without `auto`; `icu_segmenter` only behind a cross-check
     feature): `uax29 wbt` 1944 of 1944, `uax29 data` 0 discrepancies, the other modes byte for byte as the saved outputs
     (F-43);
   - (d) a build check (`research/ordering/icu-pin/check-pin.sh`, exit 0 only when everything holds): (1) every
     direct `icu_*` dependency in `Cargo.toml` has req `=2.4.0` and `default-features = false`, only `compiled_data` is allowed among features,
     and the set of direct `icu_*` equals `expected-icu-tree.txt`; (2) the set and versions of `icu_*` in the whole
     `cargo tree --locked` tree equal `expected-icu-tree.txt`; (3) no `icu_*` crate in the tree has the features `auto`,
     `lstm`, `unstable`, `datagen`, `serde`, `experimental`, `export`. A discrepancy is a build error. Verified by a run on
     2026-10-27: on the current project, exit 0; negative checks give exit 1: `unstable` on `icu_collator`, `icu_provider`
     without `=`, `icu_collections` with default-features, an extra `icu_segmenter`; check (3) alone (without (1)) catches
     `icu_collator feature "unstable"` (F-40). The script takes the project directory as a parameter (`check-pin.sh [directory]`, not tied to `research/ordering/icu-pin`),
     compares tree crates by name and version, dropping the "(path)" suffix from `[patch]`/path (verified 2026-10-27 on a copy of the
     project with `[patch.crates-io]` pointing to a vendored `icu_collator`: exit 0 and a warning, F-46); limitations: the root
     package is checked, and the script does not check the contents of the vendored copy (these are covered by the references of item 5). Updating ICU4X without editing `Cargo.toml`, the expected list and the procedure of item 6
     is impossible;
   - (e) the CLDR data version (49, F-2) is part of the pinning: it is supplied by the `compiled_data` of the 2.4.0 crates, and CLDR is not
     updated separately.
5. Reference sets and corpus. Reproducibility is verified by: (a) `research/ordering/expected-order.txt` (the program
   `research/ordering/icu4x-order`: 11 sets x 6 parameter variants) and the name reference for search —
   `research/name-search/out-uax29-golden.txt` (F-44; the run `cargo run --release --bin uax29 -- golden` on the pinned
   build of `research/name-search/Cargo.toml`), together with `out-uax29-wbt.txt` and `out-uax29-data.txt` (F-43); the outputs of
   `WordSegmenter` (`out-cases.txt` and others) do not serve as references;
   (b) the order corpus (defined unambiguously, the program `research/ordering/order-corpus`):
   - names: (A) single characters from the ranges U+0020–007E, U+00A0–024F, U+0300–036F, U+0400–052F, U+2000–206F,
     U+3040–30FF, U+4E00–4E7F; (B) a base (Cyrillic А–я, Ё, ё, Latin, digits, space, "-", "_", ".", U+00AD) + one of
     7 combining marks, and a base (а, е, и, a, e, o) + two marks in all orders; (C) 20 000 pseudo-random strings of length
     1…8 from a fixed pool, the generator xorshift64* with seed `0x5EED0001`. All names are converted to NFC and deduplicated;
     this gives 19 628 names (verified by a run, F-41). Pairwise combinations and combining marks are included deliberately: that is where
     keys and contractions change;
   - order: names are sorted by the comparison function (`icu_collator`, ru, parameters of ADR 0037/0038); names equal under it are
     ordered by code points only in this program, for the sake of reproducible output;
   - hash: the program output (one line per name: the code points in hex and the relation to the previous name, `<` or `=`) is passed
     through SHA-256 (`shasum -a 256`); the result together with the number of names is the snapshot;
   - snapshot: the file `research/ordering/order-corpus/order-corpus.sha256`, committed to the repository; it is created once
     (`check-corpus.sh --write`) on the pinned ICU4X 2.4.0 when preparing the reference (done 2026-10-27:
     `b66cffd5be83a028c05bc1e3329169ebcd4cca50be78a086b9683e5a4cc7084c`, 19 628 names) and is not regenerated by the build;
   - check: `check-corpus.sh` in the build of every v1 release computes the hash and compares it with the snapshot; any difference in the hash or
     in the number of names counts as a discrepancy (one name in a different place changes the hash); on a discrepancy the build fails, the release
     is blocked, and the snapshot is not edited in v1. Verified: changing the seed gives exit 1.
   (c) A run by product code (S-1). `research/ordering/icu4x-order`, `order-corpus` and `research/name-search` are prototypes
   that generate the reference (snapshots), not a check of the product: they have their own collator options and their own comparison call. The product must
   run the same references through its own functions (a build and test requirement, item 9 (d)).
   The corpus is a snapshot of the implementation, not an independent oracle (ICU4C 78.3 is Unicode 17 and cannot cross-check characters of Unicode 18);
   it catches an unexpected change, not a wrong order. A discrepancy with (a) or (b) is a release-blocking defect.
   The reference changes only outside v1.
6. A vulnerability or bug in `icu_*` within v1. The procedure:
   - (a) assess reachability: ICU4X is used only for comparison, folding and character properties of text that has already been accepted and is
     length-capped (ADR 0022); if the defect is not reachable or does not affect the functions in use, the version is not changed;
   - (b) a fix without a behavior change: a backport of the fix into a vendored copy of the pinned version
     (`[patch.crates-io]` pointing to a 2.4.0 copy in the repository) without changing data (`*_data`, CLDR 49) and without editing the comparison,
     normalization and property code beyond the fix; item 5 is re-run; `check-pin.sh` accepts such a copy by name and version
     (the path in `cargo tree` is dropped) and warns that it exists, while the contents of the copy are checked by the references of item 5 and by review of the diff;
   - (c) [USER, OD-5, D-18] If a backport (b) is impossible (the fix cannot be separated from a change to comparison or data),
     the visible order and matches in v1 do not change: the path of item 6 (d). The decision is only (b), without a version update.
     Rationale: OD-4 item 2 ([USER], ADR 0010 item 2) is unconditional ("a server update within v1 does not change what search finds
     or in what order"); a match on a finite corpus does not prove it and leaves a residual risk of the order being unchanged
     on unverified strings. Alternatives (considered; each except the chosen one narrows or changes the owner's decision; 1) and 3) were rejected by the owner in OD-5):
     1) allow updating `icu_*` to a new version if all reference sets and the corpus (item 5) give a byte-for-byte identical result
        and the cross-check with ICU4C (`research/ordering/icu4c-check`) matches; the cost is residual risk: the order or matches may
        change on strings outside the corpus (19 628 names), and the guarantee of OD-4 item 2 becomes a guarantee "verified on a sample";
        the gain is security without maintaining our own copy; if the ICU4X key format changes, the index is rebuilt
        from the names (item 3);
     2) chosen: backport only (b); the cost is maintaining a vendored copy and manually analyzing each fix;
        the guarantee is kept unconditionally at the code level (comparison and data do not change) and is verified by item 5;
     3) a ban on any updates and backports in v1; the cost is that a known vulnerability stays until v2, and the reachability assessment (a)
        remains the only protection;
   - (d) if a fix changes the order or matches (any set or the corpus changed, or the fix cannot be
     separated from a change to comparison): it is not released in v1 (item 8); the fix goes into v2 with a described migration, or the owner
     revisits OD-4 item 2 (a question to the owner at the moment it arises, Q-95). A release with such a change without a revision is not
     made.
   A match of the reference sets does not prove that nothing changed across all of Unicode (the corpus is finite); for variants (b) and 1) it is
   a necessary condition, not a proof; the residual risk of variant 1) is accepted only if the owner chooses it and is
   stated in "Consequences".
7. The reference and its independence. The reference output was obtained from the same implementation (ICU4X 2.4.0), so by itself it
   verifies only that the order is unchanged, not that it is correct. Correctness was verified once independently: ICU4C 78.3
   (`research/ordering/icu4c-check`) gave the same output line by line (F-25; repeated 2026-10-27 after a fix to the printing of equal
   names, 66 lines). There is no official CLDR test data for ru (`CollationTest` is for the root locale), so the corpus
   is limited to names from the scenarios, the table of ADR 0038 and the corpus of item 5 (b).
8. A single statement. A change to search results (matches) or to the result order is not released in v1; anything else is possible only by
   the owner revisiting OD-4 item 2 (Q-95), not by a v1 procedure. This statement is applied by `result-ordering` and
   `name-search`; a permission "by owner decision within v1" does not exist.
9. Build and test requirements (outside behavior, `design.md`): (a) `rust-version = "1.88"` at minimum (F-2: the MSRV of ICU4X 2.4.0);
   (b) tests do not require a network: the UCD 18.0.0 files against which the data is checked (PropList, UnicodeData, WordBreakTest,
   CaseFolding, emoji-data and others needed by the `records` and `name-search` tests) are in the repository with SHA-256 checksums,
   and the crates are in `cargo vendor` or the build cache (`--locked --offline`); the exact list of files and the sums are fixed by the build
   at vendoring time; (c) only the first download of crates and the UCD update remain network-dependent;
   (d) the product, not only `research/`, checks the references: the product library has the functions `normalize_name`, `name_key`, `words` and `order_key` (the sort key bytes,
   `write_sort_key_to`; the collator parameters are set by one shared product construction: [OURS] ADR 0037 — `=2.4.0`, ru,
   `case_level` Off, `max_variable` default; [USER] ADR 0038 — `strength`, `alternate_handling`, `case_first`,
   `numeric`, D-11, D-12, D-15); the build test of every release runs the names of `expected-order.txt` (the default
   variant), the corpus of item 5 (b) (hash check), `out-uax29-golden.txt`, `out-uax29-wbt.txt`, `out-uax29-data.txt` through them and compares with
   the files; for the corpus it checks both the order by `compare()` and the order by the sort key bytes (the index is built on this:
   for each adjacent pair the byte order of the key matches `compare()`; `order-corpus` checks this,
   0 discrepancies on 19 628 names, 2026-10-27; F-45). A change to the collator options or to the comparison key steps in the product is caught by this
   test, not by the `research/` programs, which remain generating prototypes.
10. The Unicode version in the collator data. The README of `icu_collator_data` 2.4.0: "This data was generated with CLDR version
   49.0.0, Unicode version 18.0.0" (verified 2026-10-27 from the README of the `icu_collator_data` 2.4.0 crate; F-42).
   This is a claim of the crate; it has not been verified by behavior: ICU4C 78.3 (Unicode 17) does not distinguish Unicode 17 and 18 on the names of the scenarios.
   Verification status: NOT VERIFIED BY BEHAVIOR. A test on characters added in Unicode 18 has not been made for the collator (Q-100,
   part (T): the test is mandatory before the v1 release). The statement "the whole stack is Unicode 18" (ADR 0010 item 1, [USER]) for order
   rests only on this claim. If the test shows that collator 2.4.0 does not distinguish or wrongly orders characters of
   Unicode 18, then ADR 0010 item 1 does not hold for order, and the choice (weaken the order to "per the CLDR 49 collator data",
   change the collator implementation, or something else) belongs to the owner, not to (T): weakening a [USER] decision is only the owner's decision
   (Q-100, part (O): the decision on a discrepancy belongs to the owner — the owner confirmed this procedure, OD-5; the options and cost are
   in `docs/open-questions.md`, section "Closed (OD-5)"). The text of ADR 0010 does not change.
11. **Changed by OD-6 (2026-10-28).** The scale is 10^7 records (ADR 0017); the numbers and estimates for 10^8 in this ADR are the history of the calculation, and the current values for 10^7 are in ADR 0044, 0049, 0052, 0058, 0059; nonlinear estimates are recomputed on a prototype (T, `docs/open-questions.md`).

## Alternatives considered
- Storing only keys, without a rebuild: rejected, F-2: on an ICU4X code update the keys are invalid.
- Promising order stability "including an ICU4X code update" (iteration 2): rejected, ICU4X does not guarantee this (F-2,
  F-18), and an index rebuild does not help: the comparison function itself changes.
- Pinning by `Cargo.lock` alone without `=` in `Cargo.toml`: rejected, it depends on the discipline of `--locked` and is lost on
  `cargo update`; direct exact versions make the pinning visible in `Cargo.toml`, and `Cargo.lock` and the tree check duplicate
  it.
- A fork of ICU4X with frozen comparison and segmentation: rejected as the default (maintenance cost), and remains
  a backport option of item 6 (b) for a specific fix.
- Always sorting on the fly by names without an index: rejected for the scale of 10^8 records (X-1), search latency (Q-47).
- Allowing an ICU4X update on a corpus match as the default (iterations 3–4): rejected, it narrows the guarantee of OD-4 item 2
  [USER] without an owner decision; it remains alternative 1) of item 6 (c), rejected by the owner (OD-5, D-18).
- Allowing an ICU4X update with an order change and announcing it in the release notes (recommendation of package 1, Q-21):
  cancelled by OD-4 item 2 [USER].

## Consequences
- Requirement for storage: the name index serves the order of ADR 0036; the key carries a format label (our code, the `icu_collator` version);
  a rebuild with the same comparison function (a format change in our code; an ICU4X update under alternative 1) of item 6 (c), rejected
  by the owner, OD-5, D-18) is not visible as an order change (start and operator — Q-82).
- Build: 15 direct dependencies at `=2.4.0` with explicit features, `Cargo.lock`, `--locked`, `check-pin.sh` (with a path parameter),
  the reference sets and corpus of item 5 (`check-corpus.sh`, checking both `compare()` and the key bytes), a run of the references by product functions (item 9 (d)), `rust-version = "1.88"`, vendored UCD (item 9) in every v1 release.
- Residual risk: an urgent vulnerability in `icu_*` whose fix cannot be separated from an order change puts the owner before a
  choice between security and order stability (item 6 (d)). The default (backport only) is worth the maintenance of the
  copy but does not narrow OD-4 item 2; allowing an update on the corpus (alternative 1) would add residual risk on unverified strings.
- The test "order from the scenarios" is a check that nothing changed (a snapshot of the implementation), not an independent oracle; an ICU4X update
  that changes the order would pass it only together with an edit of the scenarios, which is not allowed in v1.

## Sources
- `facts.md`: F-2 (sort keys, CLDR 49), F-18 (the sort keys docstring, 2.4.0), F-19, F-25, F-28…F-30 (the `icu_*` graph,
  version requirements, features), F-40 (`check-pin.sh`), F-41, F-45 (the order corpus and its cross-check with `compare()`), F-42 (collator data), F-43, F-44 (`research/name-search`), F-46 (`check-pin.sh [directory]`).
- `research/ordering/README.md`, `research/ordering/icu-pin/` — reproducibility, tree check, cross-check with ICU4C.
- `Cargo.toml` of the `icu_collator` 2.4.0 crate (verified 2026-10-27 in the local cargo cache): the dependencies `~2.4.0` and `2.4.0`.
- The README of `icu_collator_data` 2.4.0 (item 10), verified 2026-10-27 from the local cargo cache.
- ADR 0010; ADR 0030 items 3–4; OD-4 item 2.
