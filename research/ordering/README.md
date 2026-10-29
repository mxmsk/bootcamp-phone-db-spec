# research/ordering: reproducible reference for name order

Purpose: confirm fact F-19 (`facts.md`) and the tables of ADR 0037, 0038 on name order under ru rules, and provide a reference
for the v1 release tests (ADR 0039). It contains two independent programs and a file with the expected output.

## Contents
- `icu4x-order/` — Cargo project: `icu_collator =2.4.0`, `icu_locale =2.4.0` (compiled_data), locale ru. Prints the order
  of 11 name sets for 6 parameter variants (Tertiary/Secondary/Primary + NonIgnorable, Tertiary + Shifted,
  Tertiary + `case_first` upper, Tertiary + numeric). Names that are equal
  under the collator are printed joined by " = " (there is no code-point key, ADR 0036 item 2); their order within a group exists only for
  output reproducibility.
- `icu4c-check/` — independent cross-check of the same sets through ICU4C (a different code base, not ICU4X).
- `icu-pin/` — ICU4X pinning project (ADR 0039 item 4): 15 direct dependencies `=2.4.0` (without `icu_segmenter`, ADR 0039 item 4 (c)) with explicit features,
  `expected-icu-tree.txt` (expected `icu_*` crates in the tree), `check-pin.sh [directory]` (checks direct dependencies `=2.4.0`, `default-features = false`, the tree, and the ban on features for all `icu_*`).
  Run: `CARGO_TARGET_DIR=$TMPDIR/pin-target ./icu-pin/check-pin.sh icu-pin` (the project path is a parameter, the current directory by default; network required; `jq` and `cargo` required). Crates from `[patch]`/path are compared by name and version; the script does not check the contents of such a copy.
- `order-corpus/` — the order corpus (19,628 names) and the hash snapshot `order-corpus.sha256` (ADR 0039 item 5 (b)). Run:
  `cd order-corpus && CARGO_TARGET_DIR=$TMPDIR/corpus-target ./check-corpus.sh`; `--write` only when preparing the reference. The program additionally checks the order by sort key bytes (`write_sort_key_to`) against `compare()` for every adjacent pair (exit 1 on a mismatch; 0 mismatches on 19,628 names). This is a prototype that produces the reference; the check in the product build is done by product functions (ADR 0039 item 9 (d)).
- `expected-order.txt` — expected output (lines `set | variant | names`), also the reference for tests.

## Running
ICU4X (network required to download crates; keep the build outside the repository):
```
cd research/ordering/icu4x-order
CARGO_TARGET_DIR=$TMPDIR/ordering-target cargo run --quiet | grep ' | ' | grep -v '^ru:\|^equal' > $TMPDIR/x.txt
diff $TMPDIR/x.txt ../expected-order.txt
```
The last three lines of output (without the filter) are checks: ru and the root locale differ; «Ann» = «A­nn»; NFD «и+U+0306» = «й».

ICU4C (requires ICU4C with headers and a C++ compiler; macOS: `brew install icu4c`, prefix — `ICU_PREFIX`):
```
cd research/ordering/icu4c-check
./run.sh > $TMPDIR/c.txt
diff $TMPDIR/c.txt ../expected-order.txt
```

## Check result (2026-10-27)
- ICU4X 2.4.0 (Unicode 18, CLDR 49) and ICU4C 78.3 (Unicode 17.0 per the `run.sh` output; the CLDR version was not checked) produced
  line-for-line identical output for all 66 lines (rechecked 2026-10-27 after changing the printing of equal names to " = ") (set x variant). This is independent confirmation of the order (the Unicode versions differ, with no mismatches on these lines): two different
  UCA/CLDR implementations agree on the specification examples.
- What this does not prove: only the names from the `result-ordering` scenarios and the ADR 0038 table are covered, not all of Unicode.
  The official CLDR `CollationTest` files are for the root locale; there is no data for ru, so an ru corpus cannot be obtained from them.
- The check does not replace the reference: after an ICU4X version change, the order in the scenarios is checked by this program, and a mismatch with
  `expected-order.txt` is a release-blocking defect (ADR 0039 item 5). `expected-order.txt` may only be changed deliberately,
  outside v1.
