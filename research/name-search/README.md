# research/name-search — ICU4X 2.4.0 probe for the name-search spec

## What here is the product reference and what is only a cross-check
- The product reference (variant B of ADR 0030) is the UAX #29 rules over `icu_properties`, `src/bin/uax29.rs`. The release regression reference:
  `out-uax29-golden.txt` (`golden` mode; scenarios of the name-search spec: Han, kana, Hangul, Thai, Lao, Myanmar,
  Khmer, Tibetan, apostrophes, colon, U+00B7, period/comma in numbers, digits, "_", "+", "$", U+202F, hyphen, emoji,
  U+200B, U+0000). The "word" predicate (`*` in the reference) is OD-5, D-19, ADR 0030 item 3: a segment does not consist only of characters of
  General_Category Z*, P*, Cc, Cf; the reference was regenerated on 2026-10-27 after the predicate changed (previously L*/N*). Besides it, the
  product references are `out-uax29-wbt.txt` (1944/1944), `out-uax29-data.txt` (0 data mismatches), `out-uax29-key.txt`.
- Only for cross-checking against variant A (`WordSegmenter`, does not match the product): `out-cases.txt`, `out-wordbreaktest.txt`,
  `out-locale.txt`, `out-uax29-diff.txt`, `out-uax29-diffage.txt`, `out-uax29-word.txt`, the "ICU4X" column in `out-uax29-points.txt`.
  The `name-search-probe` binary (src/main.rs) is built only with `--features variant-a` (enables `icu_segmenter/auto`).
- The build is pinned (like `research/ordering/icu-pin`): all `icu_*` at `=2.4.0`, `default-features = false`, `compiled_data`,
  without `auto`/`lstm`/`unstable`. Verified on 2026-10-27 (rustc/cargo 1.96): on this build `wbt`, `data`, `points`, `word` give the same
  outputs as the saved ones.

## Running the product reference
```
export CARGO_TARGET_DIR=$TMPDIR/target
cargo run --release --bin uax29 -- golden > $TMPDIR/golden.txt
diff $TMPDIR/golden.txt out-uax29-golden.txt
```

Variant A (cross-check only): checks how ICU4X 2.4.0 behaves (with `--features variant-a`, that is, with `auto`; not the pinned build; the outputs `out-cases.txt` and others were obtained with default features) and where it differs from default UAX #29.
Verified on 2026-10-27, rustc/cargo 1.96.

## Running
```
export CARGO_TARGET_DIR=$TMPDIR/target
curl -o $TMPDIR/WordBreakTest.txt https://www.unicode.org/Public/18.0.0/ucd/auxiliary/WordBreakTest.txt
curl -o $TMPDIR/CaseFolding.txt https://www.unicode.org/Public/18.0.0/ucd/CaseFolding.txt
curl -o $TMPDIR/WordBreakProperty.txt https://www.unicode.org/Public/18.0.0/ucd/auxiliary/WordBreakProperty.txt
cargo run --release --features variant-a --bin name-search-probe -- wbt $TMPDIR/WordBreakTest.txt
cargo run --release --features variant-a --bin name-search-probe -- cases
cargo run --release --features variant-a --bin name-search-probe -- locale
cargo run --release --features variant-a --bin name-search-probe -- fold $TMPDIR/CaseFolding.txt
python3 wbprop.py $TMPDIR/WordBreakProperty.txt $TMPDIR/WordBreakTest.txt
```

## Variant B (ADR 0030): UAX #29 rules over icu_properties — iteration 3
The `uax29` binary (`src/bin/uax29.rs`): the WB3–WB999 rules are written in code, the Word_Break and Extended_Pictographic classes are taken
from `icu_properties` 2.4.0 (`compiled_data`), and `WordSegmenter` does not take part in the implementation (in `diff` and `points` it is used only for
comparison). Verified on 2026-10-27, rustc/cargo 1.96.
```
export CARGO_TARGET_DIR=$TMPDIR/target
U=https://www.unicode.org/Public/18.0.0/ucd
curl -o $TMPDIR/WordBreakTest.txt $U/auxiliary/WordBreakTest.txt
curl -o $TMPDIR/WordBreakProperty.txt $U/auxiliary/WordBreakProperty.txt
curl -o $TMPDIR/emoji-data.txt $U/emoji/emoji-data.txt
curl -o $TMPDIR/DerivedAge.txt $U/DerivedAge.txt
cargo run --release --bin uax29 -- wbt $TMPDIR/WordBreakTest.txt
cargo run --release --bin uax29 -- data $TMPDIR/WordBreakProperty.txt $TMPDIR/emoji-data.txt
cargo run --release --bin uax29 -- points
cargo run --release --bin uax29 -- word
DIFF_LIST=1 cargo run --release --bin uax29 -- diff 2> $TMPDIR/difflist.tsv
python3 diffage.py $TMPDIR/difflist.tsv $TMPDIR/DerivedAge.txt
```
- `gc`: General_Category (all 30 values) and White_Space from `icu_properties` 2.4.0 against UnicodeData-18.0.0 and PropList-18.0.0 on all
  1 112 064 scalar values; 0 mismatches (including the "word" predicate: not Z*/P*/Cc/Cf, ADR 0030 item 3); Cc is 65 characters. Output: `out-uax29-gc.txt`.
```
curl -o $TMPDIR/UnicodeData.txt $U/UnicodeData.txt
curl -o $TMPDIR/PropList.txt $U/PropList.txt
cargo run --release --bin uax29 -- gc $TMPDIR/UnicodeData.txt $TMPDIR/PropList.txt
```
Saved results: `out-uax29-wbt.txt`, `out-uax29-data.txt`, `out-uax29-points.txt`, `out-uax29-word.txt`,
`out-uax29-diff.txt`, `out-uax29-diffage.txt`, `out-wbprop-sea.txt`.
- `wbt`: our own rules on icu_properties data — 1944 of 1944 lines of WordBreakTest-18.0.0.
- `data`: `icu_properties` equals WordBreakProperty-18.0.0 and emoji-data-18.0.0 (Extended_Pictographic) on all 1 112 064
  scalar values, 0 mismatches. Together with `wbt` this checks both the rules and the data; for Han characters, Hiragana,
  Thai, Lao, Myanmar the rules are the same and their class is Other/Extend (see `out-wbprop-sea.txt`) — WordBreakTest has
  no such characters, but those places are covered by `data` (class) and `points` (segments).
- `diff`: where `WordSegmenter::new_for_non_complex_scripts` diverges from the UAX #29 rules (the strings "a"+c, c+"a", c+c,
  c+c+c, "1"+c+"1", "ー"+c for each scalar c): 105 047 scalars. Of them 558 are characters added in Unicode 18.0
  (the segmenter handles them differently from WordBreakProperty-18.0.0; the cause was not established, possibly stale rule data), the rest are
  Word_Break Other/Extend of the scripts Han, Hiragana, Myanmar, Thai, Lao, Khmer, Tai Tham, Tai Viet, New Tai Lue, Tai Le,
  Ahom (ICU4X glues them into a word; UAX #29 splits them), plus U+3005 (ALetter) and U+16FF0 (Extend) in Han (`out-uax29-diffage.txt`).
- `word`: the predicate "the segment contains a character whose General_Category is not Z*, not P*, not Cc and not Cf" against
  `WordType::is_word_like` on single scalars (mismatches by category: `out-uax29-word.txt`; regenerated after the predicate
  changed in OD-5).

Saved results: `out-wordbreaktest.txt`, `out-cases.txt`, `out-locale.txt`, `out-fold.txt`.

## Results
- `wbt`: both constructors (`new_for_non_complex_scripts`, `new_auto`) match WordBreakTest-18.0.0 on 1944 of 1944
  lines. But the file has no Han characters, Hiragana or Thai script (`wbprop.py`: of CJK only U+3031, Katakana), so the
  test does not check exactly the places where ICU4X differs from UAX #29.
- `cases`: colon, "_", U+00B7, U+2018, U+2019, U+0027, U+FF07, U+2027, U+0387, a period between letters do not split a word
  (like default UAX #29); a hyphen splits. "李雷" is one segment in `new_for_non_complex_scripts` (default UAX #29:
  "李" ÷ "雷", both Other, WB999) and two in `new_auto`; "こんにちは世界" is one / two (こんにちは, 世界); Thai is one
  segment / three; "東京タワー" is "東京" + "タワー" in both. Hebrew, Hangul, Cyrillic, emoji match UAX #29.
- `locale`: `content_locale` und/en/ru/sv/fi does not change the boundaries for "Smith:John", "k:a". The ICU4X `word.toml` file (tag
  icu@2.4.0) excludes U+003A from MidLetter, but in the compiled data of 2.4.0 the colon does not split a word.
- `fold`: `CaseMapper::fold_string` equals CaseFolding-18.0.0 (statuses C+F) for all 1 112 064 scalar values.
- The output has no "No segmentation model for complex script" message (stderr is empty).
