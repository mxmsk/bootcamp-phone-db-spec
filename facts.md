<!-- F-1, F-2 were established before the spec work started (2026-10-27). New verified facts are added here. -->
# Verified facts (Fact Grounding)

Each fact was checked by a subagent against the primary source on the verification date. Wordings "per summary" are marked. Before relying on a
fact in an ADR, the reviewer re-checks it via the link.

## F-1. Bell's telephone patent (verified 2026-10-27)
- US Patent 174,465, "Improvement in Telegraphy": application filed 1876-02-14, patent granted 1876-03-07.
  Source: https://patents.google.com/patent/US174465A/en (the wordings "Filed: February 14, 1876", "Granted:
  March 7, 1876" are per the page summary).
- H.Res.269 (107th Congress, 2002) on Meucci does not establish the date of the invention: "his work in the invention of the
  telephone should be acknowledged". Source: https://www.govinfo.gov/content/pkg/BILLS-107hres269eh/html/BILLS-107hres269eh.htm

## F-2. ICU4X 2.4.0 (verified 2026-10-27)
- icu_normalizer, icu_casemap, icu_properties, icu_segmenter, icu_collator, icu_locale, icu — version 2.4.0, published
  2026-10-22 as one release (crates.io API). rust_version 1.88.
- CHANGELOG 2.4: "Updated to Unicode 18", "Updated to CLDR 49".
  Source: https://github.com/unicode-org/icu4x/blob/main/CHANGELOG.md
- Sort keys: `CollatorBorrowed::write_sort_key_to` (added in 2.0: "Add `CollatorBorrowed::write_sort_key_to`; to
  support generating sort keys"); in 2.4 the key format changed: "This changes sort key output incompatibly!".
  Source: https://docs.rs/icu_collator/latest/icu_collator/struct.CollatorBorrowed.html and the CHANGELOG.
- Key stability: "Sort keys, if stored durably, should be presumed to be invalidated by a CLDR update, a new
  version of Unicode, or an update to the ICU4X code." (docs.rs icu_collator).
- CollatorOptions: strength (Primary/Secondary/Tertiary, Tertiary by default), alternate_handling, max_variable,
  case_level (Off by default) (docs.rs icu_collator).
- The ru locale in CLDR (common/collation/ru.xml): only "[reorder Cyrl]", no rules of its own; the presence of ru data in
  ICU4X is not directly confirmed.
- icu_casemap: `fold()` — "locale-independent and context-insensitive"; the Turkic rules are a separate
  `fold_turkic_string()` (docs.rs icu_casemap CaseMapperBorrowed). The wording "statuses C+F" was not found verbatim.
- icu_segmenter WordSegmenter: "compatible with Unicode Standard Annex #29"; `new_for_non_complex_scripts()` has no
  dictionaries for CJK/Southeast Asian scripts (docs.rs icu_segmenter).
- Data is embedded via the feature `compiled_data` (default). No public runtime constant for the Unicode/CLDR version was
  found (TESTED_UNICODE_TAG/TESTED_CLDR_TAG exist only in icu_provider_source — the data generation tool).
- Transitive dependencies (normal): icu_collections, icu_locale_core, icu_provider, smallvec, utf8_iter, utf16_iter,
  zerovec, yoke, zerofrom, tinystr, litemap, writeable, displaydoc, potential_utf, zerotrie, icu_*_data.

<!-- F-3…F-48 added 2026-10-27 from the subtasks of area 1 (add-data-model). The status of each fact is stated explicitly: "verified against the primary source", "per page summary" (WebFetch), "verified by running" (a program in research/), "not verified". -->

## F-3. The Unicode White_Space set (verified 2026-10-27; records subtask)
- Source: https://www.unicode.org/Public/18.0.0/ucd/PropList.txt, header "PropList-18.0.0.txt, Date: 2026-08-07".
- The White_Space section: "0009..000D; 0020; 0085; 00A0; 1680; 2000..200A; 2028; 2029; 202F; 205F; 3000", total "# Total code
  points: 25". U+200B, U+FEFF, U+180E are not in the list.
- Run of `research/records/unicode18-check`: the White_Space set from ICU4X 2.4.0 (`icu_properties`, Unicode 18.0.0 data,
  CLDR 49.0.0 per the README of `icu_properties_data-2.4.0`) equals the set from PropList-18.0.0 (25 = 25).
- Status: verified against the primary source and by running on Unicode 18 / ICU4X 2.4.0 (`research/records/unicode18-check/`, output —
  `output-2026-10-27.txt`).

## F-4. NFC and normalization stability (verified 2026-10-27; records subtask)
- Source: https://www.unicode.org/reports/tr15/ (Revision 58, Unicode 18.0.0). NFC — "Canonical Decomposition, followed by
  Canonical Composition". Stability: "once normalized, a string is normalized according to all future versions of Unicode".
- Run of `research/records/unicode18-check` (ICU4X `icu_normalizer` 2.4.0, Unicode 18.0.0 data): of the 25
  whitespace characters NFC changes only U+2000→U+2002 and U+2001→U+2003 (both whitespace); none of the 1 112 064 − 25 non-whitespace scalar
  values has a whitespace character in NFC or NFD; all whitespace characters have ccc = 0; `trim(NFC(s)) == NFC(trim(s))` on
  83 404 800 strings "w+x", "x+w", "w+x+w" (all w of the 25, all scalar x) and 2 000 000 pseudo-random strings of whitespace,
  combining marks and Hangul jamo — 0 mismatches; NFC is idempotent on all scalar values.
- Status: the definition and stability — against the primary source; the claims about whitespace characters and the equality of the orders "NFC,
  trim" and "trim, NFC" — verified by running on Unicode 18 / ICU4X 2.4.0 (`research/records/unicode18-check/`).
  The earlier check on Unicode 14.0.0 (Python) is replaced. The implementation test repeats the run on the build's data version.

## F-5. UTF-8 (verified 2026-10-27; records subtask)
- Source: https://www.rfc-editor.org/rfc/rfc3629, §3: "characters from the U+0000..U+10FFFF range … are encoded using
  sequences of 1 to 4 octets".
- Status: verified against the primary source.

## F-6. Names of the hyphen code points (verified 2026-10-27; records subtask)
- Source: https://www.unicode.org/Public/18.0.0/ucd/UnicodeData.txt: "2010;HYPHEN;Pd;…", "2011;NON-BREAKING HYPHEN;Pd;0;ON;<noBreak>
  2010;…" (a compatibility, not canonical, decomposition — NFC does not change it), "2012;FIGURE DASH;Pd", "2212;MINUS SIGN;Sm",
  "002D;HYPHEN-MINUS;Pd".
- Status: verified against the primary source and by running on Unicode 18 (`research/records/unicode18-check/`, output —
  `output-2026-10-27.txt`). The earlier local check on Unicode 14.0.0 is replaced.

## F-7. RFC 3339: date formats and the leap year (verified 2026-10-27; records subtask)
- Source: https://www.rfc-editor.org/rfc/rfc3339 §5.6: "date-fullyear = 4DIGIT", "date-month = 2DIGIT ; 01-12",
  "date-mday = 2DIGIT ; 01-28, 01-29, 01-30, 01-31 based on month/year"; app. C: a leap year is divisible by 4 and not divisible by
  100, or divisible by 400.
- The number of days 1876-03-07…9999-12-31 inclusive: 2 967 164 — computed as `(date(9999,12,31) − date(1876,3,7)).days + 1`
  (Python 3.11; the date difference is 2 967 163, plus 1 for including both bounds).
- Years 0 and 10000 are divisible by 400, so they are leap by the rule of app. C: the dates (0, 2, 29) and (10000, 2, 29) exist in the
  proleptic calendar (`calendar.isleap`, Python 3.11).
- Status: the quotes are verified against the primary source; the number of days and leap-year status are calculations (Python 3.11, independent of the Unicode
  version).

## F-8. UAX #29: word boundaries (verified 2026-10-27; name-search subtask)
- Source: https://www.unicode.org/reports/tr29/ (rev. 49, Unicode 18.0.0, 2026-09-01).
- Quotes: "WB6: AHLetter × (MidLetter | MidNumLetQ) AHLetter", "WB7: AHLetter (MidLetter | MidNumLetQ) × AHLetter",
  "MidNumLetQ = (MidNumLet | Single_Quote)", WB8 "Numeric × Numeric", WB9 "AHLetter × Numeric", WB10
  "Numeric × AHLetter", WB11/WB12 — "Numeric (MidNum | MidNumLetQ) × Numeric" and symmetrically, WB999 "Any ÷ Any".
  Single_Quote: U+0027. MidNumLet: U+002E, U+2018, U+2019, U+2024, U+FE52, U+FF07, U+FF0E. MidLetter: U+003A, U+00B7,
  U+0387, U+055F, U+05F4, U+2027, U+FE13, U+FE55, U+FF1A. U+002D (hyphen) is in none of these classes — a break by WB999.
  On the hyphen: "Hyphen-Minus (U+002D): Not classified as MidLetter by default, though implementations may tailor it".
- Conclusions: "O'Brien" (U+0027 between letters) is one word (WB6/WB7); "a1" is one word (WB9/WB10); "3.2" and "3,456" are one
  word (WB11/WB12); "ann-maria" is two words. An implementation may tailor the rules (C2-2), so "strict UAX #29" =
  the default profile, without tailoring.
- Status: verified via WebFetch (the model's retelling of the page; quotes checked against the extractions); the conclusion about the hyphen — also from the
  absence of U+002D in the listed classes.

## F-9. WordBreakProperty.txt (partially verified 2026-10-27; name-search subtask)
- Source: https://www.unicode.org/Public/UCD/latest/ucd/auxiliary/WordBreakProperty.txt ("WordBreakProperty-18.0.0.txt",
  Date: 2026-06-29): "0027 ; Single_Quote # Po APOSTROPHE".
- Status: only this line was verified; the other lines were not obtained via extraction (not verified line by line).

## F-10. CaseFolding.txt: statuses C, F, S (verified 2026-10-27; name-search subtask)
- Source: https://www.unicode.org/Public/UCD/latest/ucd/CaseFolding.txt (Date: 2026-02-03): "C: common… F: full case
  folding, mappings that cause strings to grow… S: simple…"; "To do a full case folding, use the mappings with status
  C + F"; "00DF; F; 0073 0073", "0401; C; 0451" (Ё → ё).
- Status: verified via WebFetch (the lines were obtained from the extraction).

## F-11. icu_casemap: `fold` (verified 2026-10-27; name-search subtask)
- Source: https://docs.rs/icu_casemap/latest/icu_casemap/struct.CaseMapperBorrowed.html: "Case-folds the characters in the
  given string… This function is locale-independent and context-insensitive"; `fold_turkic_string` — the Turkic variant.
- Run of `research/name-search` (mode `fold`, ICU4X 2.4.0, `compiled_data`): `CaseMapper::fold_string` — full
  folding (statuses C+F): zero mismatches with CaseFolding-18.0.0 on all 1 112 064 scalar values; examples:
  "ß" → "ss", "ẞ" → "ss", "İ" → "i̇", "Ё" → "ё".
- Status: verified by running on Unicode 18 / ICU4X 2.4.0 (`research/name-search/`, output — `out-fold.txt`).

## F-12. icu_segmenter: WordSegmenter and complex scripts (verified 2026-10-27; name-search subtask)
- Source: https://docs.rs/icu_segmenter/latest/icu_segmenter/struct.WordSegmenter.html: `new_for_non_complex_scripts` —
  "no support for scripts requiring complex context dependent word breaks"; `new_auto` — "best available compiled data for
  complex scripts". WordType (…/options/enum.WordType.html): "is_word_like: word-like segments include numbers, as well as
  segments made up of letters (including CJKV ideographs)".
- Status: quotes verified (WebFetch). Behavior on Han ideographs, hiragana and Thai script was verified by running — F-24,
  F-35; `is_word_like` is a rule tag, not a property of characters (the discrepancy with the predicate "has a letter or a digit" — F-37);
  Myanmar, Tibetan and Khmer were verified by the UAX #29 rules (F-36).

## F-13. CLDR: ru.xml (verified per summary 2026-10-27; result-ordering subtask)
- Source: https://raw.githubusercontent.com/unicode-org/cldr/main/common/collation/ru.xml (WebFetch, a retelling of the contents,
  not a verbatim copy): no rules of its own, the only directive is `[reorder Cyrl]`; the comment "The root collation
  already sorts й/Й as a base letter." The file has no rules for "ё". The chart
  https://www.unicode.org/cldr/charts/latest/collation/ru.html shows "ё Ё" in the standard order (per summary).
- Status: per WebFetch summary. Consistent with F-2 (ru.xml: only `[reorder Cyrl]`).

## F-14. icu_collator 2.4.0: CollatorOptions, Strength, AlternateHandling (verified 2026-10-27; result-ordering subtask)
- Source: https://docs.rs/icu_collator/latest/icu_collator/options/struct.CollatorOptions.html: fields `strength`
  (Tertiary by default), `alternate_handling` (NonIgnorable by default, except Thai — Shifted), `max_variable`,
  `case_level` (Off by default).
- `.../options/enum.Strength.html`: Primary — "Compare only on the level of base letters", Secondary — diacritics,
  Tertiary — "By default, if the separate case level is disabled, this corresponds to case for bicameral scripts",
  Identical — "This level makes the comparison slower and should not be used in the general case" (Identical — NFD by
  code points as the fifth level).
- `.../enum.AlternateHandling.html`: NonIgnorable — "Keep the characters whose level can be varied on the primary level",
  Shifted — "Shift the characters at or below `MaxVariable` to the quaternary level".
- Status: verified (WebFetch of the docs.rs pages). Supplements F-2.

## F-15. ICU4X: script reordering is set by the locale (verified 2026-10-27; result-ordering subtask)
- Source: https://unicode-org.github.io/icu4x/rustdoc/icu_collator/options/struct.CollatorOptions.html: "Reordering
  (BCP47 `kr`) currently cannot be set via the API and is implied by the locale of the collation."
- Status: verified (WebFetch).

## F-16. ICU4X: case_first and numeric are in CollatorPreferences (verified 2026-10-27; result-ordering subtask)
- Source: rustdoc icu_collator (index): "unlike `CollatorOptions`, this set of preferences can also be set implicitly by the
  locale"; in the sources of icu_collator-2.4.0 `src/options.rs` there are `CollationCaseFirst`, `CollationNumericOrdering`.
- Status: verified (WebFetch and reading the crate sources).

## F-17. UTS #10 (Unicode Collation Algorithm) (verified per summary 2026-10-27; result-ordering subtask)
- Source: https://www.unicode.org/reports/tr10/: conformance C1 — canonically equivalent strings compare as
  equal (step 1 — NFD); the minimal tertiary weight belongs to lowercase ("unmarked characters … MIN3"), i.e. lowercase comes before
  uppercase by default; Non-ignorable is the DUCET default; CLDR uses the Shifted variant with tailoring.
- Status: per WebFetch summary, verbatim quotes not checked.

## F-18. ICU4X sort keys: additional wordings (verified 2026-10-27; result-ordering subtask)
- Source: https://docs.rs/icu_collator/latest/icu_collator/struct.CollatorBorrowed.html: "you should not expect ICU4X to
  support your using an older, frozen copy of the sort key generation algorithm"; "If two sort keys generated at the same
  strength are compared bytewise, the result is the same as a collation comparison of the original strings at that
  strength." The quote "Sort keys, if stored durably, should be presumed to be invalidated by a CLDR update, a new version of
  Unicode, or an update to the ICU4X code." is already in F-2.
- Status: verified (WebFetch). Supplements F-2.

## F-19. Experiment: ru data in icu_collator 2.4.0 and the order (verified by running 2026-10-27; result-ordering subtask)
- Method: the program `research/ordering/icu4x-order` (a crate with `icu_collator =2.4.0`, `icu_locale =2.4.0`, compiled_data by
  default, `CollatorBorrowed::try_new(locale!("ru").into(), CollatorOptions::default())`). The published docs.rs and the
  ICU4X repository do not directly confirm the presence of ru data (there is no ru in the ICU4X tests), so the check was done by running.
  The reference output is `research/ordering/expected-order.txt` (11 sets of names x 6 parameter variants = 66 lines).
- Results (by default: Tertiary, NonIgnorable): ru and root differ (`Яков` < `Ann` in ru; `Ann` < `Бобр` in root) —
  ru data is loaded, the Cyrl reordering works; `Её` < `Еж` < `Ёж` < `Ежи` < `Жук`; `Елка` < `ЕЛКА` < `Ёлка`;
  `Еж` < `ёж` < `Ёж`; `анна` < `Анна` < `АННА` < `Антон`; `Ann Smith` < `Ann-Smith` < `AnnSmith`; `Ann Zed` < `Anna`;
  Shifted: `Anna` < `Ann Zed`; `Ann` and `A­nn` (U+00AD), `Ann` and `A​nn` — are equal (Equal); NFD "и+U+0306" equals
  "й"; the sort key `write_sort_key_to` for `Ёж` differs between ru and root in the initial byte.
- Status: verified by running on ICU4X 2.4.0 (Unicode 18 / CLDR 49 data from compiled_data), program and output in
  `research/ordering/`; not confirmed by documentation. Independent cross-check with ICU4C — F-25. Names equal under the collator are
  printed in the reference with " = " (key (B) by code points was dropped, ADR 0036 item 2). The Secondary/Primary/Shifted tables are in ADR 0038.
  Re-verify when the icu_collator version changes (Q-92).

## F-20. Sizes: text growth under NFC, NFD and case folding on Unicode 18 (verified by running 2026-10-27; records subtask)
- Program: `research/records/unicode18-check` (ICU4X `icu_normalizer`, `icu_casemap` 2.4.0, Unicode 18.0.0 data).
- Result (per single scalar value): the maximum of scalar values after NFC is 3; the maximum NFD length is 4 (U+1F82),
  that is, one character of NFC text may correspond to up to 4 input characters; the chain NFC → full case folding
  (`CaseMapper::fold_string`) → NFC gives up to 3 scalar values (U+FB03) and up to 3× bytes (U+1D160).
- Status: verified by running on Unicode 18 / ICU4X 2.4.0; measurements are per single character, the bound for a whole string was not
  proved. The sizes of derived name keys for a name of 200 scalar values were measured separately (F-39: name 800 bytes, comparison
  key 1 000, sort key 8 513). The numbers of F-20 (growth of one input character under NFC, NFD and folding) and F-39
  (the resulting keys for names of 200 identical NFC-stable characters) do not contradict each other, but neither
  proves the bound for mixed names.

## F-21. WordBreakTest-18.0.0 and ICU4X word boundaries (verified by running 2026-10-27; name-search subtask)
- `WordSegmenter::new_for_non_complex_scripts` and `new_auto` in ICU4X 2.4.0 pass WordBreakTest-18.0.0
  (https://www.unicode.org/Public/18.0.0/ucd/auxiliary/WordBreakTest.txt, Date 2026-06-12) on 1944 of 1944 lines
  (`research/name-search`, mode `wbt`, `out-wordbreaktest.txt`).
- The file has no Han ideographs, hiragana or Thai script (`wbprop.py`: of CJK only U+3031, katakana; output —
  `out-wbprop.txt`), so the test does not check the places where it diverges from UAX #29 — F-24.
- Not verified: that standard ICU4X does not change boundaries between minor versions 2.x (the segmenter data stability
  policy); consequence — version pinning (ADR 0039, Q-74, Q-82).
- Status: verified by running on ICU4X 2.4.0 and Unicode 18.

## F-22. A colon between letters does not split a word (verified by running 2026-10-27; name-search subtask)
- Run (`research/name-search`, mode `locale`, `out-locale.txt`): "Smith:John" and "Ann:Иван" are one segment under any
  `content_locale` (und, en, ru, sv, fi).
- The file `provider/source/data/segmenter/word.toml` of the tag icu@2.4.0 (raw.githubusercontent.com) excludes U+003A, U+FE55,
  U+FF1A from MidLetter and refers to Unicode 17, but the compiled data of 2.4.0 behaves differently (why — not established:
  the crate has two engines, v1 and v2). The basis is the behavior, not the text of the file.
- Status: verified by running on ICU4X 2.4.0.

## F-23. Characters inside a word and agreement with UAX #29 (verified by running 2026-10-27; name-search subtask)
- Run (`research/name-search`, mode `cases`, `out-cases.txt`): these do not split a word between letters: "_", U+00B7, U+2018,
  U+2019, U+0027, U+FF07, U+2027, U+0387 and the period; "3,456.7" is one word; the hyphen splits; "007b" is one word; Hebrew
  ("שלום", "ג'ינס", "צה"ל"), Hangul, Cyrillic and emoji agree with UAX #29.
- Status: verified by running on ICU4X 2.4.0. The earlier "exhaustive" list of characters inside a word is wrong — the list is illustrative.

## F-24. ICU4X discrepancies with UAX #29: Han ideographs, hiragana, Thai, Lao (verified by running 2026-10-27; name-search subtask)
- For "李", "雷", "こ", "世", "ส" Word_Break = Other (WordBreakProperty-18.0.0, F-32), so by UAX #29 WB999 gives a boundary
  between every pair; the "UAX" values below were obtained by running our own rules over the `icu_properties` tables (F-33, F-36).
- `new_for_non_complex_scripts`: "李雷" — 1 segment (UAX: 2); "こんにちは世界" — 1 (UAX: 7); Thai "สวัสดีครับ" — 1 word;
  Lao — 1; a boundary at a script change ("李雷Smith" → "李雷", "Smith"); "東京タワー" → "東京", "タワー".
- `new_auto`: "李", "雷" separately; "こんにちは", "世界" (dictionary); Thai "สวัส | ดี | ครับ" (three segments).
- Khmer "សួស្តី" — 1 segment in both ICU4X modes (by UAX #29 — three words, F-36). There is no warning "No segmentation
  model for complex script" (stderr is empty).
- Myanmar and Tibetan scripts were verified by the UAX #29 rules (F-36), not by `WordSegmenter`.
- Status: verified by running on ICU4X 2.4.0 (`research/name-search/`); the scale of the discrepancy — F-35; decision — ADR 0030 item 4
  (owner decision OD-5, D-14; option B: UAX #29 rules by default).

## F-25. Independent cross-check of name order: ICU4X and ICU4C (verified by running 2026-10-27; result-ordering subtask)
- Programs: `research/ordering/icu4x-order` (icu_collator =2.4.0, icu_locale =2.4.0, ru) and the independent
  `research/ordering/icu4c-check` (ICU4C 78.3, Unicode 17.0, locale ru; names equal under the collator are printed with " = ", ADR 0036 item 2).
  11 sets of names x 6 parameter variants = 66 lines gave line-by-line identical output (`research/ordering/expected-order.txt`); repeated 2026-10-27 after the change in printing equal names.
- Limitations: the CLDR version of ICU4C was not compared; the corpus is names from the scenarios and the tables of ADR 0038; there is no official CLDR data
  for ru (`CollationTest` is the root locale).
- Status: verified by running (two implementations).

## F-26. The `case_first` upper and numeric variants (verified by running 2026-10-27; result-ordering subtask)
- Tertiary + `case_first` upper: "АННА, Анна, анна, Антон". Tertiary + numeric: "Ann 2 < Ann 10", "Ann 1 < Ann 2 < Ann 10";
  by default "Ann 1 < Ann 10 < Ann 2". ICU4X 2.4.0 and ICU4C 78.3 agreed (`research/ordering/expected-order.txt`).
- Status: verified by running.

## F-27. Great Britain and its American colonies switched to the Gregorian calendar in 1752 (verified per summary 2026-10-27; records subtask)
- Source: https://en.wikipedia.org/wiki/Calendar_(New_Style)_Act_1750: "Section I of the act corrects this divergence by
  providing that Wednesday, 2 September 1752 be followed by Thursday, 14 September 1752."; "The Calendar (New Style) Act 1750
  applied to Britain's American colonies".
- Status: secondary source (verified via a WebFetch retelling of the article); sufficient for the claim "the 1876 date is Gregorian",
  but the primary source of the act was not opened. F-1 does not establish the calendar (only the grant date of the patent).

## F-28. The `icu_*` graph in the specification stack (verified by running `cargo tree --locked` 2026-10-27; result-ordering subtask)
- The project `research/ordering/icu-pin` (all needed crates as direct dependencies, without `icu_segmenter`) gives exactly 15 `icu_*`
  crates, all 2.4.0 (`research/ordering/icu-pin/expected-icu-tree.txt`): casemap, casemap_data, collator, collator_data,
  collections, locale, locale_core, locale_data, locale_fallback, locale_fallback_data, normalizer, normalizer_data,
  properties, properties_data, provider. Ordering (`icu4x-order`) pulls in 12 of them (collator, collator_data, collections,
  locale, locale_core, locale_data, locale_fallback, locale_fallback_data, normalizer, normalizer_data, properties,
  provider); name search (option B, ADR 0030) — casemap, casemap_data, properties, properties_data, normalizer,
  normalizer_data, collections, locale_core, locale_fallback*, provider. Outside `icu_*` the tree has 22 crates (zerovec, yoke,
  tinystr and others). With `icu_segmenter` and `icu_segmenter_data` (only `research/name-search` for cross-checking, F-35) there are 17 `icu_*` crates;
  they are not part of the product.
- Status: verified by running. `icu_segmenter` is not needed by the product under option B of D-14 (ADR 0030: `WordSegmenter` does not participate);
  it is needed for option A and for cross-checking (F-35). The number of crates is set by `expected-icu-tree.txt`, not by the text of the specification.

## F-29. Transitive version requirements of `icu_collator` 2.4.0 (verified from `Cargo.toml` of the downloaded crate 2026-10-27; result-ordering subtask)
- `icu_collator_data = "~2.4.0"`, `icu_collections = "~2.4.0"`, `icu_provider = "2.4.0"` (that is, `^2.4.0`). So `=2.4.0` only
  on the top crates does not pin this data and the core; direct `=2.4.0` on all `icu_*` is needed (ADR 0039 item 4 (a)).
- Status: verified from the source `Cargo.toml` (local cargo cache).

## F-30. Features: building without `auto`, `lstm`, `unstable` (verified by running 2026-10-27; result-ordering subtask)
- The `icu-pin` project with `default-features = false` on all direct `icu_*` and `compiled_data` on the core builds and works without
  `auto`/`lstm`/`unstable`: the ru collator, `CaseMapper::fold_string`, `DecomposingNormalizerBorrowed::new_nfd`,
  `CodePointMapData::<WordBreak>`, `CodePointSetData` (Extended_Pictographic); `rust-version = "1.88"`. `cargo update
  --dry-run`: "Locking 0 packages". A run on the same features — F-43 (`research/name-search`).
- Build check: `check-pin.sh` — F-40.

## F-31. UAX #29: texts of the rules on Southeast Asian scripts, ExtendNumLet, Extend, WSegSpace (verified against the primary source 2026-10-27; name-search subtask)
- https://www.unicode.org/reports/tr29/ (rev. 49, Unicode 18.0.0; the page was downloaded and checked against the text): "Otherwise, break
  everywhere (including around ideographs). WB999 Any ÷ Any"; "For scripts that use the South East Asian style of context
  analysis, neither the default word boundaries nor the default line breaks are adequate. Both need tailoring." (Thai,
  Myanmar, Khmer); ExtendNumLet — "General_Category = Connector_Punctuation, or U+202F NARROW NO-BREAK SPACE (NNBSP)";
  Extend (for Word_Break) — "Grapheme_Extend = Yes, or General_Category = Spacing_Mark, or Emoji_Modifier=Yes and not ZWJ";
  WSegSpace — "General_Category = Zs and not Linebreak = Glue"; on proximity search: "ignoring any words that contain only
  whitespace, punctuation, and similar characters" (UAX #29 does not define a "word" predicate).
- Status: verified against the primary source. Supplements F-8.

## F-32. WordBreakProperty-18.0.0: Thai, Lao, Myanmar, Han characters and katakana (verified against the primary source 2026-10-27; name-search subtask)
- WordBreakProperty-18.0.0 (Date 2026-06-29); the extract is `research/name-search/out-wbprop-sea.txt`. Thai letters U+0E01–0E30 and
  similar, Lao letters, Myanmar letters, Han ideographs, hiragana: no lines — Word_Break = Other. Extend: Thai U+0E31,
  U+0E34–0E3A, U+0E47–0E4E; Lao U+0EB1, U+0EB4–0EBC, U+0EC8–0ECE; Myanmar U+102B–103E and others. Thai, Lao and
  Myanmar digits are Numeric; katakana U+30A1–30FA, 30FC–30FE — Katakana; U+3005 — ALetter.
- Consequence: by default under UAX #29 a Thai, Lao or Myanmar word is cut into "a letter with Extend characters".
- Status: verified against the primary source (the file), the extract is `research/name-search/wbprop.py`. Supplements F-9.

## F-33. Our own WB3–WB999 rules pass WordBreakTest-18.0.0 (verified by running 2026-10-27; name-search subtask)
- The WB3–WB999 rules written in `research/name-search/src/bin/uax29.rs` over `icu_properties::CodePointMapData::<WordBreak>` and
  `CodePointSetData::<ExtendedPictographic>` (without `WordSegmenter`) match WordBreakTest-18.0.0 on 1944 of 1944 lines
  (`uax29 wbt`, `out-uax29-wbt.txt`).
- Status: verified by running on ICU4X 2.4.0 and Unicode 18. Supplements F-21.

## F-34. The `icu_properties` 2.4.0 data equals UCD 18 (verified by running 2026-10-27; name-search subtask)
- Word_Break equals WordBreakProperty-18.0.0, Extended_Pictographic equals emoji-data-18.0.0 on all 1 112 064 scalar values:
  mismatches 0 / 0 (`uax29 data`, `out-uax29-data.txt`).
- Not verified: that `icu_properties` does not change Word_Break for Unicode 18 characters between minor versions 2.x (the stability
  policy) — requirement: pin the version (ADR 0039) or a copy of the table (Q-74, Q-82).
- Status: verified by running.

## F-35. `WordSegmenter::new_for_non_complex_scripts` diverges from the UAX #29 rules on 105 047 scalar values (verified by running 2026-10-27; name-search subtask)
- Run `uax29 diff` (`out-uax29-diff.txt`, `out-uax29-diffage.txt`): strings "a"+c, c+"a", c+c, c+c+c, "1"+c+"1", "ー"+c;
  105 047 scalars diverge: 558 characters added in Unicode 18.0 (DerivedAge), and 103 346 Han (Other), 381 Hiragana,
  Myanmar 187, Khmer 87, Thai 73, Lao 73, Tai Tham/Tai Viet/New Tai Lue/Tai Le/Ahom, plus U+3005 (ALetter) and U+16FF0 (Extend).
  For other scripts and characters before Unicode 18 there are no mismatches.
- The cause for the 558 characters was not established (assumption: the segmenter rules are built from `word.toml` referring to Unicode 17;
  not verified). The claim "Unicode 18 data" is true for `icu_properties` but not for the segmenter.
- Status: verified by running on ICU4X 2.4.0.

## F-36. Point cases under the UAX #29 rules (verified by running 2026-10-27; name-search subtask)
- `uax29 points`, `out-uax29-points.txt`: "李雷" → "李"|"雷"; "こんにちは世界" — per character; "東京タワー" → "東"|"京"|"タワー";
  "コンピュータ" — one; "김철수" — one; "สวัสดีครับ" → "ส"|"วั"|"ส"|"ดี"|"ค"|"รั"|"บ"; Lao "ສະບາຍດີ" →
  "ສ"|"ະ"|"ບ"|"າ"|"ຍ"|"ດີ"; Myanmar "မြန်မာ" → "မြ"|"န်"|"မာ"; Khmer "សួស្តី" — three; Tibetan "བཀྲ་ཤིས" →
  "བཀྲ"|"་"|"ཤིས"; "_", "‿", a lone U+202F — a segment without letters; "Ann"+U+202F+"Smith" — one segment; "李雷Smith" →
  "李"|"雷"|"Smith"; "李雷2" — three.
- Status: verified by running our own rules (F-33). A correction to F-24: Khmer under UAX #29 is three words.

## F-37. The "word" predicate (has an L* or N* character) and `WordType::is_word_like` diverge (verified by running 2026-10-27; name-search subtask)
- `uax29 word`, `out-uax29-word.txt`: on single scalars about 22 500 characters diverge: 20 260 OtherLetter are not word-like; word-like
  are 598 OtherSymbol, 133 Mn, 60 Mc, 13 Cf, 10 Pc. `is_word_like` is a segmenter rule tag, not a property of characters.
- Addition (OD-5, 2026-10-27): the word predicate was changed to "the segment contains a character not in Z*, P*, Cc, Cf"; `uax29 word`
  was rerun with the new predicate (`out-uax29-word.txt`): the output still shows that `is_word_like` is a rule tag, not a property of
  characters (word-like are 10 Pc, 2 Pd, 13 Cf, 22 Po, 1 Zs — by our predicate they are not words; our words without word-like are letters, symbols, M* characters).
- Status: verified by running on ICU4X 2.4.0. The word predicate adopted by the specification is ADR 0030 item 3 (after OD-5 — not Z*/P*/Cc/Cf).

## F-38. Name comparison key: idempotence (verified by running 2026-10-27; name-search subtask)
- `uax29 key`, `out-uax29-key.txt`: the key (NFC, C+F folding, NFC, "ё"→"е") on single scalars — 0 non-idempotent; on pairs
  "character + U+0308/U+0301/U+0323" — 2 (U+0401 and U+0451 + U+0308: the key "е"+U+0308 is not in NFC).
- Status: verified by running. The key is the result of exactly four steps and is not normalized again (ADR 0030 item 1).

## F-39. Sizes of derived name keys for a name of 200 scalar values (verified by running 2026-10-27; records subtask)
- Program: `research/records/sortkey-check/` (ICU4X 2.4.0: `icu_collator`, `icu_normalizer`, `icu_casemap`; ru,
  `CollatorOptions::default()`, `write_sort_key_to`); output — `research/records/sortkey-check/output-2026-10-27.txt`.
- Result: names "x×200" for each of the 1 110 876 scalar values x (not Cc, NFC-stable): the maximum name is 800
  bytes; comparison key (NFC, full case folding, NFC) 1 000 bytes (U+1FAD…U+1FAF); sort key 8 513 bytes
  (U+FDFA; next are U+FDFB 3 652, U+321D 3 645); the minimum sort key is 2 bytes; typical 410–630. The maximum matched an
  independent check by the reviewer.
- Status: verified by running. Uniform names were measured; the bound for mixed ones is not proved (ADR 0022 item 5). Consistent with
  F-20 (there — growth of one input character under NFC/NFD/folding).

## F-40. `check-pin.sh` checks the pinning and catches violations (verified by running 2026-10-27; result-ordering subtask)
- Program: `research/ordering/icu-pin/check-pin.sh` (needs `jq`, `cargo`). On the current project code 0; code 1 with `unstable`
  on `icu_collator`, with `icu_provider = "2.4.0"` (without `=`), with `icu_collections` with default-features, with an extra
  `icu_segmenter`; the features check over the tree separately (without checking direct dependencies) catches `icu_collator feature
  "unstable"`.
- Status: verified by running (positive and negative runs). The check rules are ADR 0039 item 4 (d).

## F-41. The order corpus and its SHA-256 (verified by running 2026-10-27; result-ordering subtask)
- Program: `research/ordering/order-corpus` (`check-corpus.sh`, snapshot `order-corpus.sha256`): 19 628 names (single
  characters of the given ranges, a base with combining marks, 20 000 pseudo-random xorshift64* strings, seed
  `0x5EED0001`; NFC, no duplicates), the order is by `icu_collator` ru; SHA-256 of the output
  `b66cffd5be83a028c05bc1e3329169ebcd4cca50be78a086b9683e5a4cc7084c`. A rerun gives the same; changing the seed gives a
  mismatch (code 1).
- Status: verified by running. The corpus is a snapshot of the implementation, not an independent oracle (ICU4C 78.3 is Unicode 17, F-25). The definition of the
  corpus is ADR 0039 item 5 (b).

## F-42. Collator data: declared Unicode and CLDR versions (verified from the crate README 2026-10-27; result-ordering subtask)
- README of `icu_collator_data` 2.4.0 (local cargo cache): "This data was generated with CLDR version 49.0.0, Unicode version
  18.0.0".
- Status: the crate's declaration; behavior on characters added in Unicode 18 was not verified (Q-100). ICU4C 78.3 (Unicode 17)
  does not distinguish on the names of scenarios 17 and 18 (F-25).

## F-43. `research/name-search` on the pinned build (verified by running 2026-10-27; name-search subtask)
- `research/name-search/Cargo.toml`: all `icu_*` at `=2.4.0`, `default-features = false`, `compiled_data`, without `auto`, `lstm`,
  `unstable`; `auto` is enabled only by the feature `variant-a` of the cross-check binary `name-search-probe` (option A). Build — rustc/cargo
  1.96.
- Result: on the pinned build `uax29 wbt` — 1944 of 1944 lines of WordBreakTest-18.0.0, `uax29 data` — 0 / 0 mismatches,
  `uax29 points`, `uax29 word` — byte-for-byte as the saved outputs (F-33, F-34, F-36, F-37). The outputs of option A (`out-cases.txt`
  and others) were obtained on default features, were not regenerated and are not a reference.
- Status: verified by running. Closes the "not verified" of F-30 (Q-96).

## F-44. The product's word-boundary reference: `out-uax29-golden.txt` (verified by running 2026-10-27; name-search subtask)
- Program: `research/name-search`, `cargo run --release --bin uax29 -- golden` (the pinned build without `auto`); the saved
  output `research/name-search/out-uax29-golden.txt`, 35 lines: Han, kana, Hangul, Thai, Lao, Myanmar, Khmer,
  Tibetan, apostrophes U+0027/U+2018/U+2019, colon, U+00B7, period and comma in numbers, digits, "_", U+203F, U+202F, TAB,
  hyphen, emoji, "Annиван".
- Result (name-search scenarios): "Smith:John", "Ann·Bob", "O’Brien", "O‘Brien", "Smith.John", "3.14", "1,000",
  "john_smith" — one word each; "Anna-Maria" — two; "o'" — "o" and "'"; "A1 2B" — "A1", "2B".
- Status: verified by running; the snapshot is the release's regression reference (ADR 0030 item 3, ADR 0039 item 5 (a)).

## F-45. The sort key byte order matches `compare()` on the order corpus (verified by running 2026-10-27; result-ordering subtask)
- Program: `research/ordering/order-corpus` (`check-corpus.sh`): for each adjacent pair of the 19 628 names the byte order of
  `write_sort_key_to` is checked against the result of `compare()`; 0 mismatches; the SHA-256 of the output is unchanged (`b66cffd5…084c`, F-41).
  On a mismatch it exits 1.
- Status: verified by running. The corpus is a snapshot of the implementation, not an independent oracle (F-41). The order index is built on the consistency of the key bytes with
  `compare()` (ADR 0039 item 9 (d)).

## F-46. `check-pin.sh [directory]`: path as a parameter and robustness to `[patch]` (verified by running 2026-10-27; result-ordering subtask)
- Program: `research/ordering/icu-pin/check-pin.sh [directory]` (needs `jq`, `cargo`). The suffix "(path)" in the `cargo tree` output
  from `[patch]`/path is dropped; on `icu-pin` code 0; on a copy of the project with
  `[patch.crates-io] icu_collator = { path = "vendor/icu_collator" }` code 0 and a warning about path/`[patch]`.
- Limitation: the script does not check the contents of the vendored copy (they are covered by the references of ADR 0039 item 5 and review of the diff).
- Status: verified by running (supplements F-40).

## F-47. `icu_properties` 2.4.0: General_Category, the L*/N* predicate, White_Space and Cc equal UCD 18 (verified by running 2026-10-27; name-search subtask)
- Program: `research/name-search`, mode `uax29 gc` (`out-uax29-gc.txt`): `icu_properties` 2.4.0 against
  UnicodeData-18.0.0 and PropList-18.0.0 on all 1 112 064 scalar values: General_Category — 0 mismatches (all 30
  values); the predicate "has an L* or N* character" — 0 (this was before OD-5); after OD-5 the "word" predicate — "has a character not in Z*, P*, Cc,
  Cf" — was rerun 2026-10-27 on the same files: 0 mismatches; White_Space — 0; Cc — 65 characters.
- Status: verified by running. The source of the White_Space and Cc properties for `records` and of the "word" predicate of `name-search` is
  `icu_properties` 2.4.0 (ADR 0021 item 1, ADR 0030 item 3).

## F-48. Key sizes of mixed names: a sample, not a proof (verified by the reviewer's run 2026-10-27; records subtask)
- Review by running: alternating U+FDFA with 8 other characters, random mixes of the 40 "longest" characters, hill climbing;
  the maximum sort key is 8 513 (as for the uniform U+FDFA, F-39), mixes at most 7 014.
- Status: verified by running on a sample; the bound for mixed names is not proved (ADR 0022 item 5), the implementation re-verifies it on
  mixed names and on a data version change.

## F-49. `icu_normalizer` 2.4.0 equals NormalizationTest-18.0.0 (verified by the reviewer's run 2026-10-27; the program is not saved in `research/`; records subtask)
- Feasibility review by running: `icu_normalizer` 2.4.0 (`compiled_data`, NFC and NFD) against `NormalizationTest-18.0.0.txt`
  (Date 2026-06-24, unicode.org): 20 171 lines, 0 mismatches.
- Status: verified by the reviewer's run, the program is not saved in `research/`; repeat when building the product. Confirms
  the assumption "normalization data is Unicode 18" (before that F-4 verified NFC only on whitespace characters and idempotence).

## F-50. ICU4X sort keys: no 0x00 byte, no own prefixes — on a sample (verified by the reviewer's run 2026-10-27; the program is not saved in `research/`; result-ordering subtask)
- Feasibility review by running: `write_sort_key_to` (ru, `CollatorOptions::default()`) on 346 201 strings (all strings of length
  ≤ 4 from a pool of 28 characters: Latin, Cyrillic, "ё", space, "-", "_", ".", U+00AD, U+0301, U+0308, digits, U+200B, U+3000):
  keys with a 0x00 byte — 0; keys that are a proper prefix of another distinct key (adjacent ones in the
  sort were checked) — 0; empty keys — 0 (the empty string has a non-empty key). Example: "Ann" → `2b 45 45 01 07 01 dc 06`.
- Status: verified by the reviewer's run on a sample, the program is not saved in `research/`; not a proof (Q-109). Supplements
  F-45 (there only adjacent pairs of the order corpus were checked).

## F-51. MSRV of the `icu_*` graph — Rust 1.88 (verified by the reviewer from the crates' `Cargo.toml` 2026-10-27; the program is not saved in `research/`; result-ordering subtask)
- Feasibility review: by the `Cargo.toml` of all crates of the `icu_*` graph (the `registry/src` cache) the maximum `rust-version` is 1.88;
  consistent with `rust-version = "1.88"` in design.md. The reviewer's environment: rustc 1.96.0, `icu_*` 2.4.0.
- Status: verified by the reviewer (running and reading manifests), the program is not saved in `research/`. Supplements F-29.

## F-52. Key sizes of typical names and the structure of the ICU4X sort key (verified by running 2026-10-27; indexes subtask)
- The program `research/indexes/keysize` (ICU4X 2.4.0, ru, `CollatorOptions::default()`, `write_sort_key_to`; the build is pinned as in
  `research/records/sortkey-check`; output — `research/indexes/keysize/output-2026-10-27.txt`): 28 names (12 Russian full names and titles,
  8 Latin, 4 Han/kana/Hangul, Arabic, Thai, a mix). Russian: on average 17.7 characters, 33.7 bytes of UTF-8, comparison
  key 33.7, sort key 30.2 bytes (1.71 bytes per character); Latin 14.4 characters, key 23.5 (1.63); Han/kana/Hangul
  3 characters, key 13.2 (4.42 per character). Example: "Ann" → `2b 45 45 01 07 01 dc 06`.
- Structure: on all 1 112 064 scalar values and on all 160 000 pairs from a pool of 400 characters (U+0000–024F, U+0300–036F,
  U+0400–04FF) the sort key contains no 0x00 byte and contains exactly two 0x01 bytes.
- Status: verified by running. The sample of names is small (28) and hand-picked; there is no real corpus of names; the figures are for calculation, not
  an estimate over a real distribution. Not a proof for all strings; the encoding of ADR 0045 does not rely on this property (Q-109
  closed).

## F-53. SQLite index B-trees spill large keys to overflow (verified per page summary 2026-10-27; indexes subtask)
- https://www.sqlite.org/fileformat.html, §1.6: "Large keys on index b-trees are split up into overflow pages so that no single
  key uses more than one fourth of the available storage space on the page and hence every internal page is able to store at
  least 4 keys." The formula `X = ((U-12)*64/255)-23` is the maximum local part of an index record.
- Status: verified against the primary source (a WebFetch tool summary). Used only as a precedent for the technique (ADR 0045 item 6;
  the threshold of our format is different — ADR 0041 item 5); SQLite code is not used.
## F-54. Microbenchmark of key sorting and merging (verified by running 2026-10-27; indexes subtask)
- `research/indexes/keysize/src/bin/bench.rs`, output `research/indexes/keysize/bench-output-2026-10-27.txt` (arm64, dev machine):
  sorting 131 072 random 40-byte keys — 54 ns/key; heap merge of k = 16/256/4096 runs — 54/75/95 ns/key; for
  2^20 keys — 32/51/89 ns.
- Status: verified by running. The keys are random (real ones share a prefix — comparison takes longer), the machine is not a reference; the latency calculation
  uses 100 ns (ADR 0049).

## F-55. The `crc32c` crate 0.6.8 (verified against the primary source 2026-10-27; file-format subtask)
- https://crates.io/api/v1/crates/crc32c and the contents of `crc32c-0.6.8.crate` (static.crates.io). Description: "Safe implementation for
  hardware accelerated CRC32C instructions with software fallback"; license `Apache-2.0/MIT`; released 2024-06-09 (the latest). README:
  "Rust implementation of the CRC-32-Castagnoli algorithm with hardware acceleration where possible"; x86-64 SSE 4.2; aarch64 "Only
  available on rust version >= 1.80.0"; otherwise a software variant. `cargo tree`: no runtime dependencies (build dependency
  `rustc_version`).
- Status: verified against the primary source and by running `cargo tree`. `cargo-deny` is not installed in the environment: the license and RustSec
  checks were not performed.

## F-56. The `crc32fast` crate 1.5.3 (verified against the primary source 2026-10-27; file-format subtask)
- crates.io: "Fast, SIMD-accelerated CRC32 (IEEE) checksum computation"; `MIT OR Apache-2.0`; released 2026-10-20. Computes IEEE
  (check `0xCBF43926` on "123456789", run of `research/file-format/crc-bench`), not Castagnoli.
- Status: verified against the primary source and by running.

## F-57. The `crc` crate 3.4.0 (verified against the primary source 2026-10-27; file-format subtask)
- crates.io: "Rust implementation of CRC with support of various standards"; `MIT OR Apache-2.0`; MSRV 1.83; implementations NoTable /
  Table<1> / Table<16> (README). `CRC_32_ISCSI` gives `0xE3069283` on "123456789" (run of `research/file-format/crc-bench`).
- Status: verified against the primary source and by running.

## F-58. Measurement and CRC-32C vectors for 16 KiB (verified by running 2026-10-27; file-format subtask)
- `research/file-format/crc-bench`, output `research/file-format/crc-bench-output-2026-10-27.txt` (macOS arm64, Darwin 25.6.0; the CPU
  model was not obtained — `sysctl` is forbidden by the sandbox). A 16 KiB page: `crc32c` ≈ 0.67 µs (≈ 22.7 GiB/s), `crc32fast` ≈ 0.82 µs
  (≈ 18.7 GiB/s), `crc` Table<16> ISCSI ≈ 3.3 µs (≈ 4.6 GiB/s); on 4 KiB `crc32c` — 12.2 GiB/s. Vectors: CRC-32C("123456789") =
  0xE3069283; 16 384 zeros = 0x94640B85; a page where byte i = i mod 251, bytes 0…3 = 0, = 0x1D167853; all three were checked with an independent
  implementation in Python.
- Status: verified by running; one machine, not a reference. The measurement of the snapshots subtask (F-85) for `crc32c` on 16 KiB pages gave 8.4–12.2
  GiB/s on a different build and method (the discrepancy was not investigated; the smaller value was taken for the backup calculation).

## F-59. Koopman's table for CRC-32C (read 2026-10-27; file-format subtask)
- https://users.ece.cmu.edu/~koopman/crc/crc32.html, entry `0x8f6e37a0`: `{2147483615, 2147483615, 5243, 5243, 177, 177, 47, 47, 20,
  20, 8, 8, 6, 6, 1, 1}`, "iSCSI; CRC-32C".
- Status: the interpretation ("HD=4 up to 2 147 483 615 bits, HD=6 up to 5 243 bits") is by the order of entries; no explanation of the list format was found on the page:
  not verified, not used as an argument in the ADR.

## F-60. Key sizes of realistic and mixed names (verified by running 2026-10-27; file-format subtask)
- `research/file-format/keysize` (ICU4X 2.4.0, ru), output `research/file-format/keysize-output-2026-10-27.txt`: 2 106 synthetic
  Russian and Latin full names and titles (the set is embedded in the program, not real data): on average 22.2 characters, 42.1 bytes, comparison key
  42.1, sort key 35.2 (1.58 bytes per character), maxima 45 characters, 62, 62, 61. Mixed names: 19 200 names from pairs of the 80 "longest"
  characters: the maximum 8 513 only for U+FDFA/U+FDFA (uniform); with different characters — 7 297 (U+FDFA/U+FDFB); for alternating pairs
  the key can exceed the maximum of the components' keys by 398 bytes (2 432 versus 2 034: U+3347/U+327C), that is, key length is not monotonic.
- Status: verified by running; not a proof (Q-113). Supplements F-39, F-48.

## F-61. B+-tree leaf fill under random inserts (model calculation 2026-10-27; file-format subtask)
- `research/file-format/sizing.py`, part A (output `sizing-output-2026-10-27.txt`): a 16 KiB page, split in the middle by bytes;
  fill 0.669 / 0.679 / 0.693 / 0.712 with average cells of 58 / 65 / 120 / 49 bytes.
- Status: a model, not a measurement of the engine.

## F-62. Calculation of the file size for 10^8 records under the layout of ADR 0045 (calculation 2026-10-27; file-format subtask)
- `research/file-format/sizing.py`, parts B–D (output `sizing-output-2026-10-27.txt`): profile P0 (ADR 0044 item 7), Σ = 404 B per record;
  61.2 GB without compression at fill 0.67; 52.8 / 46.2 GB with prefix compression of 25 / 45 %; 45.5 GB at fill 0.9 without compression;
  the compression at which the file is exactly 50 GB (fill 0.67) is 0.336. Sensitivity to name length: 40 characters — 112.6 GB without compression.
- Status: arithmetic on the assumptions of profile P0; prefix compression was not measured (no corpus); the recalculation of ADR 0049 with a number version of
  ≤ 5 bytes — `research/indexes/output-version5-2026-10-27.txt` (the command is in ADR 0049 item 1).

## F-63. OS page size on the development machine (verified by running 2026-10-27; file-format subtask)
- `getconf PAGESIZE` = 16384 (arm64 macOS); the output was not saved to a file.
- Status: verified by running on one machine; on x86-64 the OS page is 4 KiB, not checked (16 KiB is a multiple of both).

## F-64. Linux `fsync(2)`: flush to the device (verified against the primary source 2026-10-27; durability subtask)
- https://man7.org/linux/man-pages/man2/fsync.2.html: "transfers ("flushes") all modified in-core data of the file referred to by the
  file descriptor fd to the disk device ... This includes writing through or flushing a disk cache if present. The call blocks until
  the device reports that the transfer has completed."
- Status: verified against the primary source.

## F-65. Linux `fsync(2)`: the directory entry needs a separate flush (verified against the primary source 2026-10-27; durability subtask)
- Same page: "Calling fsync() does not necessarily ensure that the entry in the directory containing the file has also reached disk. For
  that an explicit fsync() on a file descriptor for the directory is also needed."
- Status: verified against the primary source.

## F-66. Linux `fdatasync` and size metadata (verified against the primary source 2026-10-27; durability subtask)
- Same page: `fdatasync` "similar to fsync(), but does not flush modified metadata unless that metadata is needed in order to allow a
  subsequent data retrieval to be correctly handled ... a change to the file size (st_size, as made by say ftruncate(2)), would
  require a metadata flush." For this reason the WAL is pre-filled segments of constant size (ADR 0050 item 2).
- Status: verified against the primary source.

## F-67. Linux `fsync(2)`: errors and their delivery (verified per page summary 2026-10-27; durability subtask)
- Same page, errors: EBADF, EINTR, EIO, ENOSPC, EROFS, EINVAL, EDQUOT; "Since Linux 4.13, errors from write-back will be reported to all
  file descriptors that might have written the data which triggered the error".
- Status: verified per a page summary by the tool (WebFetch).

## F-68. macOS `fsync(2)`: data may not reach the platters (verified against the primary source 2026-10-27; durability subtask)
- https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/fsync.2.html: "while fsync()
  will flush all data from the host to the drive ..., the drive itself may not physically write the data to the platters for quite
  some time and it may be written in an out-of-order sequence ... if the drive loses power or the OS crashes, the application may
  find that only some or none of their data was written."
- Status: verified against the primary source (a page from the documentation archive).

## F-69. macOS `F_FULLFSYNC` (verified against the primary source 2026-10-27; durability subtask)
- Same page: "Mac OS X provides the F_FULLFSYNC fcntl. The F_FULLFSYNC fcntl asks the drive to flush all buffered data to permanent
  storage. Applications, such as databases, that require a strict ordering of writes should use F_FULLFSYNC." `fcntl(2)`
  (https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/fcntl.2.html): "Does the same
  thing as fsync(2) then asks the drive to flush all buffered data to the permanent storage device ... currently implemented on HFS,
  MS-DOS (FAT), and Universal Disk Format (UDF) file systems. The operation may take quite a while to complete."
- Status: verified against the primary source; a page from the documentation archive, it does not name APFS (the operation of `F_FULLFSYNC` on APFS is confirmed
  only by the measurement F-79: 4–5 ms versus 0.02 ms; this is not proof of a drive cache flush). `F_BARRIERFSYNC` is not described on the page:
  not verified, not used.

## F-70. Rust std: `fsync` and `datasync` on Apple — `F_FULLFSYNC` (verified against the primary source 2026-10-27; durability subtask)
- Branch master, `library/std/src/sys/fs/unix.rs` (https://raw.githubusercontent.com/rust-lang/rust/master/library/std/src/sys/fs/unix.rs):
  `fsync` — `libc::fcntl(fd, libc::F_FULLFSYNC)` for `target_vendor = "apple"`, otherwise `libc::fsync(fd)`; `datasync` on Apple — also
  `F_FULLFSYNC`.
- Status: verified against the primary source. For Linux `sync_data` = `fdatasync`: the branch was not shown by the tool — not verified.
## F-71. PostgreSQL: retrying `fsync` after an error is unreliable (verified against the primary source 2026-10-27; durability subtask)
- https://wiki.postgresql.org/wiki/Fsync_Errors: "buffers are marked clean after errors, so retrying fsync() can falsely report success
  and the modified buffer can be thrown away at any time due to memory pressure"; "PostgreSQL will now PANIC on fsync() failure"
  (PostgreSQL 12, backported to 9.4).
- Status: verified against the primary source.

## F-72. LWN: kernel behavior after a write error (verified against the primary source 2026-10-27; durability subtask)
- https://lwn.net/Articles/752063/ (2018-04-18, "PostgreSQL's fsync() surprise"): "filesystems will respond differently, but that
  behavior usually includes discarding the data in the affected pages and marking them as being clean"; "PostgreSQL assumes that a
  successful call to fsync() indicates that all data written since the last successful call made it safely to persistent storage.
  But that is not what the kernel actually does."
- Status: verified against the primary source.

## F-73. Linux `flock(2)` (verified against the primary source 2026-10-27; durability subtask)
- https://man7.org/linux/man-pages/man2/flock.2.html: "flock() places advisory locks only"; the lock is "associated with an open file
  description" and is released "when all such file descriptors have been closed"; `LOCK_NB` — error `EWOULDBLOCK`; duplicate
  descriptors (fork/dup) refer to the same lock; since 2.6.12 NFS clients emulate it with byte-range locks.
- Status: verified against the primary source.

## F-74. macOS `flock(2)` (verified against the primary source 2026-10-27; durability subtask)
- https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/flock.2.html: "Advisory locks
  allow cooperating processes to perform consistent operations on files, but do not guarantee consistency"; "Locks are on files, not
  file descriptors ... descriptors duplicated through dup(2) or fork(2) ... multiple references to a single lock"; `LOCK_NB` —
  `EWOULDBLOCK`.
- Status: verified against the primary source; the page says nothing about release on process exit or about NFS (verified by running, F-81).

## F-75. Rust 1.89.0 stabilized `File::try_lock` (verified against the primary source 2026-10-27; durability subtask)
- https://blog.rust-lang.org/2025/08/07/Rust-1.89.0/ (2025-08-07): stabilized `File::lock`, `lock_shared`, `try_lock`,
  `try_lock_shared`, `unlock`. The MSRV of area 1 is 1.88 (an ICU4X requirement, F-2, F-51): `File::try_lock` requires raising the MSRV or
  using `libc::flock` (Q-118).
- Status: verified against the primary source.

## F-76. Rust std: `try_lock` = `flock(LOCK_EX | LOCK_NB)` (verified against the primary source 2026-10-27; durability subtask)
- The same file as in F-70: `try_lock` = `libc::flock(fd, LOCK_EX | LOCK_NB)`, `WouldBlock` on `ErrorKind::WouldBlock`.
- Status: verified against the primary source (branch master; not compared against release 1.89).

## F-77. Linux `getrandom(2)` (verified against the primary source 2026-10-27; durability subtask)
- https://man7.org/linux/man-pages/man2/getrandom.2.html: "If the urandom source has not yet been initialized, then getrandom() will
  block, unless GRND_NONBLOCK is specified"; "reads of up to 256 bytes will always return as many bytes as requested and will not be
  interrupted by signals" (if the source is initialized).
- Status: verified against the primary source.

## F-78. The `getrandom` crate 0.4.3 (verified against the primary source 2026-10-27; durability subtask)
- https://docs.rs/getrandom/latest/getrandom/ (as of 2026-10-27): Linux — the `getrandom` system call, otherwise `/dev/urandom` after polling
  `/dev/random`; macOS — `getentropy`.
- Status: verified against the primary source; the crate is not chosen for use (ADR 0017 item 3, Q-116).

## F-79. Measurement of flush on macOS: `fsync` and `F_FULLFSYNC` (verified by running 2026-10-27; durability subtask)
- `research/durability/fsync-bench.py`, output `research/durability/fsync-bench-macos.out`: macOS 26.6.2, arm64, data volume (APFS),
  a pre-filled 64 MiB file, append + flush, 1000 repetitions. `fsync` 4 KiB — p50 0.018 ms, p99 0.027 ms; `F_FULLFSYNC` 4 KiB —
  p50 4.783 ms, p90 5.238, p99 8.297, max 16.043; 64 KiB: `fsync` p50 0.036, p99 0.067; `F_FULLFSYNC` p50 4.120, p99 10.691, max
  17.178.
- Status: verified by running on one machine; the conclusion — `fsync` on macOS does not wait for the drive; Linux was not measured (Q-119).

## F-80. Measurement of 16 KiB random reads (verified by running 2026-10-27; durability subtask)
- `research/durability/read-bench.py`, output `research/durability/read-bench-macos.out`: the same bench, a 6 GiB file (`F_NOCACHE` before
  writing), random 16 KiB reads, queue depth 1, 5000 reads — p50 0.108 ms, p99 0.165 ms, max 1.713 ms; sequential
  read 4016 MB/s.
- Status: verified by running. The first attempt with 2 GiB and `F_NOCACHE` after writing gave p50 0.002 ms (the file was in cache) — discarded.

## F-81. Measurement of `flock` locking on macOS (verified by running 2026-10-27; durability subtask)
- `research/durability/flock-test.py`, output `research/durability/flock-test-macos.out`: a second process while the owner is alive — `EAGAIN`;
  after `kill -9` of the owner the lock is acquired; while a child with an inherited descriptor is alive — `EAGAIN`; after its death it is acquired.
- Status: verified by running on macOS; Linux was not run.

## F-82. SQLite WAL: readers and the writer, the "end mark", a long reader (verified against the primary source 2026-10-27; snapshots subtask)
- https://www.sqlite.org/wal.html: "WAL provides more concurrency as readers do not block writers and a writer does not block
  readers."; "there can only be one writer at a time."; "When a read operation begins on a WAL-mode database, it first remembers the
  location of the last valid commit record in the WAL. Call this point the "end mark"."; "Thus a long-running read transaction can
  prevent a checkpointer from making progress."; "then no checkpoints will be able to complete and hence the WAL file will grow
  without bound."
- Status: verified against the primary source (WebFetch). A precedent for the model (ADR 0055) and the retention risk (ADR 0056); the SQLite mechanism is not
  copied.

## F-83. SQLite online backup: restarting the copy on an external write (verified against the primary source 2026-10-27; snapshots subtask)
- https://www.sqlite.org/backup.html: "SQLite detects this and usually restarts the backup process when sqlite3_backup_step() is next
  called."; "If the backup process is restarted frequently enough it may never run to completion".
- Status: verified against the primary source (WebFetch). The basis for rejecting copying with restart (ADR 0057).

## F-84. Measurement of sort key speed (verified by running 2026-10-27; snapshots subtask)
- `research/snapshots/keybench` (`icu_collator` =2.4.0, pinned versions, Apple M3 Pro, synthetic two-word names of 4–10
  letters, 3/4 Cyrillic): 1.96 / 8.42 / 14.8 million sort keys per second on 1 / 4 / 8 threads, average key 24.2 bytes
  (`research/snapshots/README.md`).
- Status: verified by running. Only the order index; the word and exact-search indexes were not measured.

## F-85. Measurement of `crc32c` over 16 KiB pages (verified by running 2026-10-27; snapshots subtask)
- `research/snapshots/crcbench` (`crc32c` =0.6.8, 1 thread, 16 KiB pages): 8.4–12.2 GiB/s (`research/snapshots/README.md`).
- Status: verified by running; an order of magnitude, the crate is not chosen here. Diverges from F-58 (22.7 GiB/s on 16 KiB); for the backup
  calculation (ADR 0057 item 4) the smaller value was taken.

## F-86. RFC 9110: statuses 405, 409, 413–415, 503 and DELETE with a body (verified against the primary source 2026-10-27; protocol subtask)
- https://www.rfc-editor.org/rfc/rfc9110 (WebFetch). §15.5.6: "An origin server MUST generate an Allow header field in a 405 response"; §15.5.10:
  "409 (Conflict): The request conflicts with the current state of the target resource."; §15.5.14: "413 (Payload Too Large): The server is
  refusing to process a request because the request payload is larger…"; §15.5.15 "414 (URI Too Long)…"; §15.5.16 "415 (Unsupported Media Type)…";
  §9.3.5: "A payload within a DELETE request message has no defined semantics; sending a payload body on a DELETE request might cause some
  implementations to reject the request."; §15.6.4: "503 (Service Unavailable): The server is currently unable to handle the request due to a
  temporary overload or scheduled maintenance…".
- Status: verified against the primary source (WebFetch). The page gave the wording "Payload Too Large" (an early edition); in the final RFC 9110 the name of 413 is
  "Content Too Large"; this does not affect the numbers, not re-verified. The basis of ADR 0060 item 2, ADR 0062 item 4.

## F-87. RFC 8259: repeated member names and UTF-8 (verified against the primary source 2026-10-27; protocol subtask)
- https://www.rfc-editor.org/rfc/rfc8259: §4 "When the names within an object are not unique, the behavior of software that receives such an object
  is unpredictable."; §8.1 "JSON text exchanged between systems that are not part of a closed ecosystem MUST be encoded using UTF-8.".
- Status: verified against the primary source (WebFetch). §6 (number precision beyond 2^53) is from the list of pitfalls, not opened. Not verified: that `serde_json`
  rejects a repeated member name when parsing into a struct — to be verified in the implementation.

## F-88. RFC 4648: base64url and zero padding bits (verified against the primary source 2026-10-27; protocol subtask)
- https://www.rfc-editor.org/rfc/rfc4648: §5 — the base64 alphabet with URL-safe characters; §3.5 "These pad bits MUST be set to zero by conforming
  encoders".
- Status: verified against the primary source (WebFetch). The basis of the canonical cursor encoding (ADR 0061 item 1).

## F-89. SHA-256 and CRC-32C on reference inputs; cursor size (verified by running 2026-10-27; protocol subtask)
- Python 3, `hashlib`: SHA-256("abc") = ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad (matches the FIPS 180-4 reference; the
  standard itself was not opened); our own CRC-32C gives 0xE3069283 on "123456789" (reference — F-57). Maximum cursor: 844 bytes → 1126 characters of
  base64url without padding; maximum date — 2 967 163 days from 1876-03-07. Cursor vectors — ADR 0061 item 3.
- Status: verified by running; the script is not saved in `research/`. Not verified: the "query → key" step for the vector "ё" + U+0308 (ICU4X was not run).

## F-90. RFC 6750: bearer token in a header, TLS, 401 (verified against the primary source 2026-10-27; access subtask)
- https://www.rfc-editor.org/rfc/rfc6750: §2.3 query string "SHOULD NOT be used unless it is impossible to transport the access token in the
  "Authorization" request header field or the HTTP request entity-body"; §5.3 "Clients MUST always use TLS (https) or equivalent transport security
  when making requests with bearer tokens"; §3: for a request without authentication — 401 with `WWW-Authenticate: Bearer realm="…"` without an error code ("SHOULD
  NOT include an error code").
- Status: verified against the primary source (WebFetch). The basis of ADR 0065 item 6, ADR 0066 item 2, ADR 0067.

## F-91. RFC 6265 §8.5: cookies are not isolated by port (verified against the primary source 2026-10-27; access subtask)
- https://www.rfc-editor.org/rfc/rfc6265 §8.5: "Cookies do not provide isolation by port. If a cookie is readable by a service running on one port,
  the cookie is also readable by a service running on another port of the same server."
- Status: verified against the primary source (WebFetch). The basis for rejecting cookies (ADR 0065, 0067).

## F-92. RFC 6585 §4: 429 and Retry-After (verified against the primary source 2026-10-27; access subtask)
- https://www.rfc-editor.org/rfc/rfc6585: "The 429 status code indicates that the user has sent too many requests in a given amount of time ("rate
  limiting")"; the response "MAY include a Retry-After header"; "This specification does not define how the origin server identifies the user, nor how it
  counts requests".
- Status: verified against the primary source (WebFetch). The basis of ADR 0068 item 3, ADR 0062 item 8.

## F-93. rustls 0.23.46: TLS 1.2 and 1.3, providers, key pair check (verified against documentation 2026-10-27; access subtask)
- https://docs.rs/rustls/latest/rustls/, https://docs.rs/crate/rustls/latest: "Rustls implements TLS1.2 and TLS1.3 for both clients and servers"; providers:
  "aws-lc-rs - enabled by default", "ring - available with the ring crate feature enabled"; license Apache-2.0 OR ISC OR MIT; MSRV 1.71.
- Status: verified against documentation (WebFetch). Not verified: the behavior of `ServerConfig::with_single_cert` on a mismatched key and certificate pair
  (the documentation page did not open, 404) — to be checked at implementation (ADR 0066 item 3, Q-131).

## F-94. tokio-rustls 0.26.6 and rustls-pki-types 1.15.1: reading PEM (verified against documentation 2026-10-27; access subtask)
- `tokio-rustls` 0.26.6 (2026-09-28), Apache-2.0/MIT, dependencies `rustls ^0.23.27`, `tokio ^1` (https://docs.rs/crate/tokio-rustls/latest);
  `rustls-pki-types` 1.15.1 (2026-10-02), MIT OR Apache-2.0, trait `PemObject`: "Decode the first section of this type from the PEM contents of the
  named file" (`from_pem_file`), `pem_file_iter`, the `std` feature; key formats PKCS#1, SEC1, PKCS#8
  (https://docs.rs/rustls-pki-types/latest/rustls_pki_types/pem/trait.PemObject.html).
- Status: verified against documentation (WebFetch). `cargo-deny` was not run (not installed), `cargo tree` was not obtained: network access to crates.io from the sandbox is closed (403).

## F-95. W3C Secure Contexts §3.1: loopback is potentially trustworthy (verified against the primary source 2026-10-27; access subtask)
- https://www.w3.org/TR/secure-contexts/: a host from `127.0.0.0/8` or `::1/128` is "Potentially Trustworthy"; `localhost` and `*.localhost` too, if
  the agent guarantees resolution to loopback.
- Status: verified against the primary source (WebFetch). The basis for no TLS on loopback (ADR 0066 item 4).

## F-96. DNS rebinding: protection by checking Host (verified against a secondary source 2026-10-27; access subtask)
- https://en.wikipedia.org/wiki/DNS_rebinding, section Protection: "Web servers can reject HTTP requests with an unrecognized Host header".
- Status: verified against a secondary source (WebFetch). The basis of ADR 0067 item 1.

## F-97. RFC 9110: 401, 403, missing Host, fragment (verified against the primary source 2026-10-27; access subtask)
- https://www.rfc-editor.org/rfc/rfc9110: §15.5.2 "A server generating a 401 response MUST send at least one WWW-Authenticate header field"; §15.5.4 "The
  403 (Forbidden) status code indicates that the server understood the request but refuses to fulfill it"; §7.2 "A recipient of a request without a Host
  header field MUST respond with a 400 (Bad Request) status code"; §4.2.5 — the fragment identifier is not part of the definition of the http/https schemes (summary).
- Status: verified against the primary source (WebFetch). The basis of ADR 0062 item 8, ADR 0067 items 1, 5.

## F-98. Browsers: CORS preflight for a request with Authorization (not verified; access subtask)
- The claim "a request with `Authorization` from a page of another origin requires a preflight and is not sent without the server's consent" is knowledge outside the
  subtask's sources.
- Status: not verified. The requirement "the server does not answer with CORS consent" does not depend on this (ADR 0067 item 4).

## F-99. Referrer Policy: the fragment is stripped from Referer (verified against the primary source 2026-10-27; access subtask)
- https://www.w3.org/TR/referrer-policy/, "strip url for use as a referrer": "Set url's fragment to `null`"; username is empty, password `null`.
- Status: verified against the primary source (WebFetch). The basis for delivering the UI token in the fragment (ADR 0067 item 5).

## F-100. The `sha2` 0.11.1 and `subtle` 2.6.1 crates (verified against documentation 2026-10-27; access subtask)
- `sha2` 0.11.1 (2026-10-12), Apache-2.0 or MIT, pure Rust (https://docs.rs/crate/sha2/latest); `subtle` 2.6.1, `ConstantTimeEq::ct_eq` — "should execute in
  constant time" (a goal, not a hard guarantee), for slices of different length — "short-circuits if the lengths of the input slices are different"
  (https://docs.rs/subtle/latest/subtle/trait.ConstantTimeEq.html).
- Status: verified against documentation (WebFetch). The `subtle` license is not shown by the source — not verified; the release date differs between two pages — not to be
  used; `cargo-deny` was not run (ADR 0069 item 5).

## F-101. hyper 1.12.0: deadlines and limits of http1::Builder (verified against documentation 2026-10-27; server-runtime subtask)
- https://docs.rs/hyper/latest/hyper/server/conn/http1/struct.Builder.html: "Set a timeout for reading client request headers. If a client does not
  transmit the entire header within this time, the connection is closed." — `header_read_timeout`, 30 s by default, requires a `Timer`, otherwise
  `serve_connection` panics; `keep_alive` is true by default; `max_headers` — "431 Request Header Fields Too Large", 100 by default. `Builder` methods
  (16): `new, half_close, keep_alive, title_case_headers, allow_multiple_spaces_in_request_line_delimiters, ignore_invalid_headers, preserve_header_case,
  max_headers, max_header_size, header_read_timeout, writev, max_buf_size, auto_date_header, pipeline_flush, timer, serve_connection`.
- Status: verified against documentation (WebFetch). There is only one deadline (`header_read_timeout`): there are no deadlines for idle, body, response write or the whole request;
  the conclusion "deadlines are server code" is drawn from the method list (a summary; at implementation compare with the source text).

## F-102. hyper-util 0.1.21: GracefulShutdown (verified against documentation 2026-10-27; server-runtime subtask)
- https://docs.rs/hyper-util/latest/hyper_util/server/graceful/struct.GracefulShutdown.html: "A graceful shutdown utility"; `watch` — "Wrap a future for
  graceful shutdown watching."; `shutdown` — "Signal shutdown for all watched connections. This returns a `Future` which will complete once all watched
  connections have shutdown."
- Status: verified against documentation (WebFetch).

## F-103. hyper: Connection::graceful_shutdown (verified against documentation 2026-10-27; server-runtime subtask)
- https://docs.rs/hyper/latest/hyper/server/conn/http1/struct.Connection.html: `graceful_shutdown` — "Start a graceful shutdown process for this
  connection. This `Connection` should continue to be polled until shutdown can finish."; "This should only be called while the `Connection` future is
  still pending." There are no deadlines in the description.
- Status: verified against documentation (WebFetch).

## F-104. hyper: the graceful shutdown guide (verified against the primary source 2026-10-27; server-runtime subtask)
- https://hyper.rs/guides/1/server/graceful-shutdown/: "A graceful shutdown is when a connection stops allowing new requests, while allowing currently
  in-flight requests to complete." The guide shows its own accept loop on `TcpListener`; it says nothing about idle keep-alive connections.
- Status: verified against the primary source (WebFetch).

## F-105. tokio 1.54.1: TcpListener::accept (verified against documentation 2026-10-27; server-runtime subtask)
- https://docs.rs/tokio/latest/tokio/net/struct.TcpListener.html: "A TCP socket server, listening for connections"; `accept` — "Accepts a new incoming
  connection from this listener."; cancel-safe with `tokio::select!`.
- Status: verified against documentation (WebFetch). From F-104 and F-105: accepting connections is done by the server code, not by hyper.

## F-106. http-body-util 0.1.5: Limited (verified against documentation 2026-10-27; server-runtime subtask)
- https://docs.rs/http-body-util/latest/http_body_util/struct.Limited.html: "A length limited body. This body will return an error if more than the
  configured number of bytes are returned on polling the wrapped body."
- Status: verified against documentation (WebFetch). Suitable for counting a body without `Content-Length` (ADR 0071); the error type is not named in the description.

## F-107. RFC 9112: Transfer-Encoding with Content-Length and closing the connection (verified against the primary source 2026-10-27; server-runtime subtask)
- https://www.rfc-editor.org/rfc/rfc9112 §6.3: "If a message is received with both a Transfer-Encoding and a Content-Length header field, the
  Transfer-Encoding overrides the Content-Length. Such a message might indicate an attempt to perform request smuggling or response splitting and ought
  to be handled as an error."; §6.2: "A sender MUST NOT send a Content-Length header field in any message that contains a Transfer-Encoding header
  field."; §9.6: "If a server performs an immediate close of a TCP connection, there is a significant risk that the client will not be able to read the
  last HTTP response…"; "servers typically close a connection in stages. First, the server performs a half-close by closing only the write side of the
  read/write connection. The server then continues to read from the connection until it receives a corresponding close by the client…".
- Status: verified against the primary source (WebFetch). WebFetch named this §6.2, not §6.1 as in the list of pitfalls; the section number for the first sentence was not
  clarified. The basis of ADR 0072 items 5, 8.

## F-108. hyper 1.x: behaviors not confirmed by documentation (not verified; server-runtime subtask)
- Not verified: (a) behavior with `Transfer-Encoding` and `Content-Length` together; (b) `Expect: 100-continue` with `Content-Length` above the limit; (c) a response from the
  service without reading the body (reads it to the end, closes, RST); (d) whether the connection is cut without a final chunk if the body stream returned an error (the basis of
  "a cut-off stream does not look complete", ADR 0063 item 5); (e) whether a client's connection drop cancels the service future and with what delay (ADR 0070 item 7);
  (f) from which moment `header_read_timeout` is counted on a keep-alive connection after idling; (g) whether `tokio::net::TcpListener::bind` sets
  `SO_REUSEADDR`/`SO_REUSEPORT` (ADR 0070 item 3 requires not sharing the port).
- Status: not verified; checked by implementation tests (Q-138). The behavior of `SO_SNDTIMEO` on Linux and macOS is taken from the list of pitfalls without re-verification: the decision of
  ADR 0072 item 3 does not depend on it. The numbers 14 s, 2 967 164 records, 300 MB, 30 s and 120 s are estimates and defaults of areas 1 and 2, not measured.

## F-109. signal(7): SIGHUP, SIGINT, SIGTERM, SIGKILL (verified against the primary source 2026-10-27; lifecycle subtask)
- https://man7.org/linux/man-pages/man7/signal.7.html: "SIGHUP: Hangup detected on controlling terminal or death of controlling process"; "SIGINT: Interrupt from keyboard"; "SIGTERM: Termination signal";
  "The signals SIGKILL and SIGSTOP cannot be caught, blocked, or ignored." The default action "Term" for SIGHUP, SIGINT, SIGTERM is from the page table as summarized by WebFetch (a verbatim quote of the "Action" column was not obtained).
- Status: verified against the primary source; macOS behavior was not verified. The basis of ADR 0077 item 5 (SIGHUP is ignored).

## F-110. tokio::signal::unix (verified against documentation 2026-10-27; lifecycle subtask)
- https://docs.rs/tokio/latest/tokio/signal/unix/index.html: "Creates a new listener which will receive notifications when the current process receives the specified signal `kind`"; the module is "Available on Unix and crate feature `signal` only".
- Status: verified against documentation. Not verified: `SignalKind::hangup`, overriding the default action (the module page does not cover it) — to be checked at implementation (ignoring SIGHUP, ADR 0077 item 5; Q-142).

## F-111. systemd `TimeoutStopSec=` (verified against the primary source 2026-10-27; lifecycle subtask)
- https://www.freedesktop.org/software/systemd/man/latest/systemd.service.html: "it configures the time to wait for the service itself to stop. If it does not terminate in the specified time, it will be forcibly terminated by SIGKILL".
- Status: verified against the primary source. Not verified: the default value — `DefaultTimeoutStopSec=` from the manager configuration (the number was not obtained from the page); the launchd analogue (`ExitTimeOut`) was not checked (Q-144). The basis of ADR 0077 item 4.

## F-112. The `toml` crate (verified against documentation 2026-10-27; lifecycle subtask)
- https://docs.rs/toml/latest/toml/: "A serde-compatible TOML-parsing library"; version 1.1.8+spec-1.1.0; license MIT OR Apache-2.0.
- Status: verified against documentation. Not verified: cargo deny, RustSec, MSRV — at build time (ADR 0017 item 3). The basis of ADR 0076 item 1.
- Not checked and marked in the ADR: `sysexits.h` codes (ADR 0077, alternative); platform default directories (ADR 0075); SIGPIPE behavior in a Rust program (ADR 0078 item 8 requires that it not terminate the program; to be checked by a test, Q-142).

## F-113. unix(7): socket path, permissions, SO_PEERCRED, an orphaned file (verified against the primary source 2026-10-27; control subtask)
- https://man7.org/linux/man-pages/man7/unix.7.html (WebFetch). Quotes: "The `sun_path` field has a maximum of 108 bytes on Linux"; for portable code the field can be short — down to 92 bytes; "connecting to a stream socket object requires write permission on that socket";
  "POSIX does not make any statement about the effect of the permissions on a socket file, and on some systems (e.g., older BSDs), the socket permissions are ignored. Portable programs should not rely on this feature for security";
  SO_PEERCRED — "returns the credentials of the peer process connected to this socket", values as of `connect`/`listen`/`socketpair`; "The socket file has all permissions enabled, other than those that are turned off by the process umask(2)";
  EADDRINUSE — "the filesystem socket object already exists" (an `unlink` is needed).
- Status: verified (WebFetch), the quotation marks convey the gist of the tool's response, not the page verbatim; the quotes were not compared with the source character by character. The basis of ADR 0080 items 3, 4, ADR 0081 item 2.

## F-114. getpeereid(3) on macOS and BSD (verified against the primary source 2026-10-27; control subtask)
- https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man3/getpeereid.3.html (WebFetch): "The getpeereid() function returns the effective user and group IDs of the peer connected to a UNIX-domain socket";
  the socket is a `SOCK_STREAM` on which `connect(2)` or `listen(2)` was called; "This mechanism is reliable; there is no way for either side to influence the credentials returned to its peer except by calling the appropriate system call … under different effective credentials";
  "One common use of this routine is for a UNIX-domain server to verify the credentials of its client". Also https://man.freebsd.org/cgi/man.cgi?query=getpeereid&sektion=3 (FreeBSD 4.6).
- Status: verified against the Apple primary source (the documentation archive, the iPhoneOS version of the manual); the page for the macOS build (Darwin 25.6) was not opened; applicability to macOS 26 is based on tokio listing macOS among the platforms of `UCred` (F-115), not verified by running. The basis of ADR 0080 item 4.

## F-115. tokio: `UnixStream::peer_cred` and `UCred` (verified against documentation 2026-10-27; control subtask)
- https://docs.rs/tokio/latest/tokio/net/struct.UnixStream.html: `peer_cred` — "Returns effective credentials of the process which called `connect` or `pair`."
  https://docs.rs/tokio/latest/tokio/net/unix/struct.UCred.html: methods `uid()`, `gid()`, `pid()`; `pid()` — "implemented under Linux, Android, OpenBSD, FreeBSD (since FreeBSD 13), NetBSD, NTO, iOS, macOS, tvOS, watchOS, visionOS, Solaris, Illumos, Cygwin, Haiku, and Redox. On other platforms this will always return `None`";
  the struct is available "only on Unix systems".
- Status: verified against documentation (WebFetch). The tokio version was not specified (F-105 — 1.54.1).
- Not verified (an implementation decision or a test): `bind` of a Unix socket on macOS with `umask 0177` (file permissions) and on volumes where permissions are ignored (Q-148); whether macOS ignores the socket file permissions (ADR 0080 does not rely on this: the peer check layer is independent);
  the order of `umask` in a multi-threaded process — "set before creating threads" — is a calculation, not a fact about the outside world. From `facts.md` F-81 (`flock` on macOS: release on process exit) and F-78 (`getrandom`) are used without re-verification.

## F-116. history.replaceState (verified against documentation 2026-10-27; area 5, ui-shell)
- https://developer.mozilla.org/en-US/docs/Web/API/History/replaceState: "modifies the current history entry, replacing it with the state object and URL passed"; "The new URL must be of the same origin as the current URL"; "Replaces the current history entry rather than creating a new one".
- Status: a secondary source (MDN); the HTML specification (nav-history-apis) was not compared. Not verified: whether the URL with a fragment written before `replaceState` stays in the browser's general history of visited addresses (ADR 0086 item 2 accepts this as a residual risk, Q-158); whether `hashchange` fires on `replaceState` itself (ADR 0086 item 1 does not depend on this).

## F-117. sessionStorage: scope (verified against the primary source 2026-10-27; area 5, ui-shell)
- https://html.spec.whatwg.org/multipage/webstorage.html: "If the user had multiple windows opened on the site, each one would have its own individual copy of the session storage object"; when an auxiliary context is created, session storage is copied (as summarized by the tool, not fully verbatim).
- Status: partially verified. Not verified: the moment of clearing when the tab is closed and whether browsers persist `sessionStorage` to disk and return it on "reopen closed tab" and session restore — this is implementation behavior, the specification does not rely on it (ADR 0086 item 2 accepts the return as possible and does not promise "the token is not on disk" and "disappears on close").

## F-118. Content Security Policy Level 3 (verified against the primary source 2026-10-27; area 5, ui-shell)
- https://www.w3.org/TR/CSP3/: `'self'` — "matches the current URL's origin"; `'none'` — a source that matches nothing; `base-uri` defines the allowed base URLs; `form-action` restricts form submission targets; `frame-ancestors` controls embedding.
- Status: verified against documentation. Not verified: that `frame-ancestors` has no effect in `<meta>` (ADR 0087 chooses a header without relying on this).

## F-119. Retry-After (verified against documentation 2026-10-27; area 5, ui-shell)
- https://developer.mozilla.org/en-US/docs/Web/HTTP/Reference/Headers/Retry-After: formats `<http-date>` and `<delay-seconds>` ("a non-negative decimal integer indicating seconds"); applied with 503, 429 and redirects.
- Status: a secondary source (MDN); RFC 9110 §10.2.3 was not compared. The UI parses only whole seconds; the server value per `http-api` is `Retry-After: 5` (ADR 0088 item 3).

## F-120. RFC 9112 §6.1 and hyper 1.12.0: Transfer-Encoding together with Content-Length (verified 2026-10-28; OD-7)
- https://www.rfc-editor.org/rfc/rfc9112#section-6.1: "A server MAY reject a request that contains both Content-Length and Transfer-Encoding"; the server "MUST close the connection after responding to such a request to avoid the potential attacks".
- hyper 1.12.0, `proto/h1/role.rs` ≈ lines 284–360: `Content-Length` is removed, the body is read as chunked, `keep_alive = false`, the request is passed to the service; the service does not see that there were two length headers (from the sources read by the expert, `docs/reviews/rust-expert.md`, item 1).
- Status: the RFC — verified by WebFetch; hyper — from the expert's sources, not re-read by me. Closes F-108 (a). The basis of ADR 0098 item 1.

## F-121. hyper 1.12.0: `header_read_timeout` starts after the previous response (from the expert's sources; OD-7, expert's item 5)
- `proto/h1/conn.rs` ≈ 216–260: the single `header_read_timeout` timer counts keep-alive idle and header reading together. Closes F-108 (f).
- Status: from the sources read by the expert (`docs/reviews/rust-expert.md`, item 5); not re-verified. The deadlines of ADR 0071/0072 item 3 are owner decisions and are not affected; the question — Q-172 (T).

## F-122. hyper 1.12.0: `graceful_shutdown` immediately closes idle keep-alive connections (from the expert's sources; OD-7)
- `proto/h1/dispatch.rs` ≈ 91–99; the same for `GracefulShutdown` in hyper-util (clarifies F-102, F-103).
- Status: from the expert's sources (item 15); not re-verified. The basis of ADR 0098 item 3.

## F-123. hyper 1.12.0: the stream is bounded by the write buffer (from the expert's sources; OD-7)
- `proto/h1/io.rs:586`: writing the body runs into `max_buf_size`, not into "socket acceptance".
- Status: from the expert's sources (item 12); not re-verified. The basis of ADR 0100 item 2.

## F-124. tokio 1.54.1 / mio 1.2.4: `TcpListener::bind` sets `SO_REUSEADDR` (from the expert's sources; OD-7)
- `mio/src/net/tcp/listener.rs:82`: `SO_REUSEADDR` is always set. Closes F-108 (g) as regards `SO_REUSEADDR`.
- Not verified: on macOS/BSD, binding 127.0.0.1:P while a foreign *:P exists (the sandbox forbade bind). The basis of ADR 0101 item 2.

## F-125. socket2 0.6.5: `Socket::bind`, `set_reuse_address`, `listen` (verified against documentation 2026-10-28; OD-7)
- https://docs.rs/socket2/latest/socket2/struct.Socket.html: `bind` — `bind(2)`; `set_reuse_address` — the `SO_REUSEADDR` option; `listen` — `listen(2)`.
- Status: verified against documentation (WebFetch). The basis of ADR 0101 item 2.

## F-126. rustix 1.1.6: `fs::statvfs`; std 1.96 has no `statvfs`, I/O priority, `posix_fadvise`/`F_NOCACHE` (OD-7)
- https://docs.rs/rustix/latest/rustix/fs/fn.statvfs.html: `pub fn statvfs<P: Arg>(path: P) -> Result<StatVfs>` — "Queries filesystem metadata, POSIX version" (verified by WebFetch 2026-10-28).
- The absence in std 1.96 is from the expert's `std` sources (`docs/reviews/rust-expert.md`, item 8); not re-verified. The basis of ADR 0101 items 1, 4.

## F-127. rustls 0.23.46: a client offering only `h2` gets `no_application_protocol` (from the expert's sources; OD-7)
- `rustls server/hs.rs:114`: with ALPN `["http/1.1"]` and a client with only `h2` — a fatal `no_application_protocol`.
- Status: from the expert's sources (minor remarks); not re-verified. The question — Q-173 (T).
