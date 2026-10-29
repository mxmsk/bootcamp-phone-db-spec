# 0030. Name comparison key and word boundaries
- Status: Accepted
- Label: [OURS] (items 1, 2, 7; item 3 except the "word" predicate); [USER] (OD-5: item 3, the "word" predicate: D-19
  with the clarification (OD-5, "Clarification of D-19": Z*, P*, Cc, Cf); item 4: D-14; item 5: Q-85; item 6: Q-87; item
  8: Q-98; Q-97)
- Date: 2026-10-27
- CUJ: CUJ5, CUJ7, CUJ8, CUJ11; item 1.5 of the original wording

## Context
ADR 0008 sets the rules in words ("case-insensitive", "ё" = "е", word boundaries strictly per UAX #29), but does not
define them precisely: which case folding, in what order with NFC and "ё", what counts as a word (a UAX #29 segment
includes whitespace and punctuation), and what to do with digits and complex scripts. The owner's decisions: "word
boundaries strictly per UAX #29, with no custom exceptions" (OD-4 item 1; the same in the accepted CUJ5) and "the whole
Unicode stack is ICU4X" (ADR 0010). These decisions do not contradict each other: the Word_Break and
Extended_Pictographic data are taken from ICU4X (`icu_properties`), while the UAX #29 rules (WB3–WB999) are a
specification that can be executed by code on top of the data. Running the programs from `research/name-search/`
(reproduction: `research/name-search/README.md`) showed that the ready-made ICU4X 2.4.0 `WordSegmenter` differs from
UAX #29 by default: it glues Han ideographs, hiragana, Thai, Lao, Myanmar, Khmer and several more scripts into a word and
treats differently 558 characters added in Unicode 18.

## Decision
1. The text comparison key: NFC → Unicode full case folding (statuses C+F; ICU4X `CaseMapper::new().fold_string(&str)`,
   a method of `CaseMapperBorrowed`, locale-independent) → NFC → replace U+0451 with U+0435. The order matters: "ё" is
   replaced after folding and NFC, so that "Ё" and "Е"+U+0308 give "е". Other accents and "й" are not affected. That
   `fold` equals C+F was verified by a run (F-11): zero discrepancies with CaseFolding-18.0.0 on all 1,112,064 scalar
   values. The key is the result of exactly these steps and is not normalized again: on "ё"+U+0308 it gives "е"+U+0308
   (not NFC, on repeated application "е"; run `uax29 key`: on single scalars 0 non-idempotent keys, on pairs "character
   + U+0308/U+0301/U+0323" 2, both "Ё"/"ё" + U+0308). The order of steps does not change: otherwise "Ё" and "Е"+U+0308
   would stop giving one key. The key is a computed function of the text: the index stores it, but is rebuilt from the
   name text, not from the earlier key.
2. Order for words: the name (query) is brought to NFC and segmented; then the key is computed for each word separately.
   Segmentation of folded text is not applied. Exact search (ADR 0031) computes the key of the whole name.
3. Word boundaries are the default UAX #29 rules (WB3–WB999, revision 49, Unicode 18; F-8) on the Word_Break and
   Extended_Pictographic data from ICU4X `icu_properties` 2.4.0, with no dictionaries and no locale. [USER, OD-5, D-19 and
   "Clarification of D-19" (Cc and Cf: whitespace control characters and invisible formatting characters do not form
   words); the earlier value was rejected, see alternatives] A word is a segment that does not consist only of whitespace
   characters and punctuation. Precisely: a segment is a word if it has at least one scalar value whose General_Category is
   not Z* (Zs, Zl, Zp), not P* (Pc, Pd, Ps, Pe, Pi, Pf, Po), not Cc and not Cf. Words: letters L*, numbers N*, marks M*,
   symbols S* (Sm, Sc, Sk, So, including emoji), and also Co and Cn. Not words: segments only of whitespace (Z*;
   White_Space lies entirely in Z* and Cc), punctuation (including "_" and U+203F, which are Pc, the hyphen, which is Pd,
   the apostrophe and the dot, which are Po), control characters (Cc: TAB, LF, U+0000…) and invisible formatting characters
   (Cf: U+200B, U+200D, U+FEFF, etc.). A query with no words is empty, an error (OD-4 item 1; the predicate was clarified
   by the owner's decision OD-5). Rationale for the boundary: the owner called empty a query "only of whitespace and
   punctuation" (OD-4 item 1; OD-5, D-19: symbols and emoji are words); hence Z* (whitespace) and P* (punctuation) are not
   words, and symbols S* are words; Cc is included because TAB, LF, CR, U+0085 are whitespace characters (White_Space) of
   category Cc, and the other Cc (U+0000…) give no content and are forbidden in a name (ADR 0021 item 5); Cf is included
   because these are invisible characters with no content of their own: a query of a single U+200B is, from the user's point
   of view, empty (such a character is valid in a name, Q-88, but there is no word in it); marks M* are not excluded, since
   the owner did not name them. Cn and Co are not excluded: the category does not say the character is absent.
   Consequences (scenarios in `name-search`): "😀", "+" (Sm), "$" (Sc) are words, and the query "😀" finds "Ann 😀"; "_"
   (Pc) and "‿" without letters are not words, a query of them is "empty" (the example "_" from OD-5 was checked by
   category: Pc is punctuation, not a word); a name of only emoji or symbols is found by word search, a name of only
   punctuation ("-", "_") is not. Cost: search by symbols and emoji gives noisy matches; "C++" is the words "C", "+", "+"
   (the UAX #29 boundaries do not change, only the predicate does). Basis for the predicate: UAX #29 (the section on
   proximity search) suggests ignoring segments "only of whitespace, punctuation and the like", but does not set a
   predicate; the segmenter's `WordType::is_word_like()` is a rule tag, not a property of characters (discrepancies with
   the accepted predicate on single scalars: `uax29 word`, `out-uax29-word.txt`): it recognizes "_", U+203F, hyphens, Cf,
   22 Po characters and U+202F as words, while letters (20,260 OtherLetter, including Tibetan syllables), symbols and marks
   M* it does not. Consequences (default UAX #29, scenarios in `name-search`): "Ann" + U+202F + "Smith" is one word (U+202F
   is ExtendNumLet, WB13a/b), "smith" does not find it; "_" and "‿" without letters are not words (punctuation);
   "john_smith" is one word. Verified by a run: "Smith:John", "Ann·Bob", "O’Brien", "O'Brien", "Smith.John", "3,456.7" are
   one word each; "Anna-Maria", "R2-D2" are two each; "007b" is one. How it is implemented: the boundary function is
   written by the rules (about 100 lines of Rust) over `CodePointMapData::<WordBreak>` and
   `CodePointSetData::<ExtendedPictographic>`; verified 2026-10-27: 1,944 of 1,944 lines of WordBreakTest-18.0.0 and zero
   discrepancies of the icu_properties data with WordBreakProperty-18.0.0 and emoji-data-18.0.0 on all 1,112,064 scalar
   values (`research/name-search/`, `uax29 wbt`, `uax29 data`). `WordSegmenter` takes no part in the implementation; on
   every update of `icu_properties` the same two checks are mandatory.
   The product's regression reference is the saved output of variant B `research/name-search/out-uax29-golden.txt` (F-44;
   mode `uax29 golden`; regenerated 2026-10-27 with the OD-5 predicate, "*" is a word: Han, kana, Hangul, Thai, Lao,
   Myanmar, Khmer, Tibetan, the apostrophes U+0027/U+2018/U+2019, the colon, U+00B7, a dot and comma between digits, "_",
   "+", "$", U+203F, U+202F, U+200B, U+0000, the hyphen, emoji ("😀", "Ann 😀") and others; the command is in
   `research/name-search/README.md`); a build with pinned features (`default-features = false`, `compiled_data`, without
   `auto`/`lstm`/`unstable`; verified by a run, F-43). The output of `WordSegmenter` (`out-cases.txt`,
   `out-wordbreaktest.txt`, `out-uax29-diff*.txt`, the "ICU4X" column in `out-uax29-points.txt`) is only for comparison
   with variant A and does not serve as a release reference.
   The reference is produced by the program `research/name-search` (a prototype); a requirement for the product: the
   product runs the same set of scenarios (`out-uax29-golden.txt`, WordBreakTest-18.0.0, keys) through its own functions
   (word segmentation, the comparison key and the "word" predicate) in a release test; the output of `research/*` does not
   replace checking the product's code (requirement "Word boundary and key references are run through the product's
   code"; design: tests, "Required of others").
   The General_Category on which the "word" predicate rests (not Z*/P*/Cc/Cf) was checked against UCD data:
   `icu_properties` 2.4.0 equals UnicodeData-18.0.0 on all 1,112,064 scalar values (all 30 values, 0 discrepancies; the
   "word" predicate, 0; White_Space against PropList-18.0.0, 0; Cc, 65 characters), verified by running `uax29 gc`
   (`research/name-search/out-uax29-gc.txt`, 2026-10-27; F-47).
4. [USER, OD-5, D-14] For scripts without spaces, the default UAX #29 behavior is accepted, with no dictionaries and no
   custom exceptions: "李雷" is the words "李" and "雷" (found also by "雷"); "こんにちは世界" is by character, found by "世界";
   katakana "タワー" is one word ("東京タワー" is "東", "京", "タワー"); Thai, Lao, Myanmar are "a letter with Extend marks"
   (the Word_Break of the letters themselves is Other: WordBreakProperty-18.0.0 has no lines for U+0E01–0E30 and the like,
   the marks U+0E31, U+0E34–0E3A, U+0E47–0E4E and analogous Lao and Myanmar ones are Extend;
   `research/name-search/out-wbprop-sea.txt`): "สวัสดี" → "ส", "วั", "ส", "ดี". UAX #29 itself notes that for these scripts
   the default rules are insufficient and require "tailoring" (dictionaries). Cost: the query "李" finds all names with the
   character "李" and "雷" in any position (word-prefix search degenerates into search by character); for Thai the query
   "ชาย" turns into three letter-words. This is a literal execution of OD-4 item 1; the owner confirmed (OD-5, D-14; Q-97:
   there is no protection against a query of a single ideograph or a single Thai letter; a minimum query word length and a
   UI warning are not introduced).
5. [USER, OD-5, Q-85] Different spellings of the apostrophe (U+0027, U+2019, U+2018, U+FF07) are not identified: "o’brien"
   (U+2019) does not find "O'Brien" (U+0027).
6. [USER, OD-5, Q-87] A word match is a prefix by code points of the keys. There is no restriction on the grapheme
   boundary: "q" finds "q̣" (there is no precomposed form), "e" does not find "é".
7. [OURS] The composition of the words of the query and the name is per item 3 regardless of script; there is no special
   rule for a change of script: "李雷Smith" → "李", "雷", "Smith" (Han and Latin are a boundary, WB999); "Annиван" is one
   word (both ALetter, WB5); "李雷2" is three words (WB999 between a character and a digit). Any wording like "a boundary at
   a change of script" is wrong.
8. [USER, OD-5, Q-98] U+202F inside a name is part of a word (item 3): "Ann" + U+202F + "Smith" is one word, "smith" does
   not find it (UAX #29), although at the edges of a name U+202F is trimmed as whitespace (ADR 0021); we do not treat it as
   a separator.

## Alternatives considered
- (B, chosen, item 4) The UAX #29 rules over `icu_properties` data: a literal execution of OD-4 item 1, unified Unicode 18
  data from ICU4X (checked on all scalar values), no dictionaries and no dependence on `WordSegmenter`. Cost: our own code
  of about 100 lines (not an "exception" to UAX #29 but its execution; checked by WordBreakTest-18) and a "character by
  character" behavior for Chinese, Japanese, Thai, Lao, Myanmar writing.
- (A, rejected) ICU4X `WordSegmenter::new_for_non_complex_scripts` as is: "李雷" is one word, "雷" does not find "李雷",
  "こんにちは世界" does not find "世界". Violates OD-4 item 1 and the accepted CUJ5 ("word boundaries strictly per UAX #29",
  with no custom exceptions): ICU4X differs from UAX #29 for 105,047 scalars (including 558 Unicode 18 characters;
  `research/name-search/out-uax29-diff.txt`, `out-uax29-diffage.txt`), that is, the segmenter's data and the Unicode 18
  from `icu_properties` do not coincide. Acceptable only if the owner explicitly lifts OD-4 item 1 for these scripts.
- (B2) The ICU4X segmenter for everything except a list of scripts, and a cut by character for the list: rejected; the
  list of scripts and characters (Han 103,346, Myanmar 187, Khmer 87, … plus 558 Unicode 18 characters) would have to be
  maintained by hand, these are "custom exceptions" and two data tables instead of one.
- (C) Dictionary segmentation (`new_auto`, `new_dictionary`): "李" and "雷" separately, Japanese by dictionary, Thai into
  words; the result depends on dictionaries and their updates (OD-4 item 2: v1 behavior does not change), "not strictly
  UAX #29". Rejected.
- The word predicate "`is_word_like`" of the segmenter: rejected, a rule tag, not a property of characters (see item 3).
- (D-19, earlier value; rejected by the owner in OD-5) A word is a segment with a letter or a number (L*/N*); a query with
  no such segments, including of symbols and emoji ("😀", "+", "$"), is the error "empty"; the name "Ann 😀" is not found by
  "😀". Rejected: wider than the owner's wording (OD-4 item 1: empty is only punctuation and whitespace); chosen: symbols
  and emoji are words.
- (D-19, option "not White_Space and not P*") A word is any segment except whitespace and punctuation: close to the chosen
  one, but U+0000 and U+200B would become words; chosen is the definition through Z*, P*, Cc, Cf (item 3).
- Simple folding (C+S): rejected, the standard way of case-insensitive comparison is full folding; the difference is visible
  only for "ß", "İ" and the like. The Turkic variant (`fold_turkic_string`) is not used: the key does not depend on locale.
- Removing all accents (NFD and deleting Mn): rejected, the brief names only "ё" = "е".
- Identifying apostrophes: rejected, "no custom exceptions" (OD-4, item 1). Question Q-85 was closed by the owner (OD-5).
- A prefix by graphemes: rejected as a complication; the difference is visible only for combinations with no precomposed
  form.
- A key idempotent on "ё"+U+0308 (a different order of steps or a repeated NFC): rejected, "Ё" and "Е"+U+0308 would stop
  giving one key, or the key would stop equalling a predictable function of the text; there is no practical harm (2 pairs
  out of 1,112,064 × 3).

## Consequences
- Storage builds the indexes by word keys under Unicode 18; a change of rules or tables is an index migration (Q-74, Q-82);
  the Word_Break/Extended_Pictographic tables are fixed for all of v1 (OD-4 item 2): pinning the `icu_properties` version or
  an embedded copy of the table with a checksum. A change of matches is not released in v1; fixing dependencies is only
  without changing results (ADR 0039 items 6, 8).
- Names only of punctuation ("-", "_") are valid on save but are not found by word search (ADR 0031); names of symbols and
  emoji are found (OD-5, D-19).
- The owner's decision (OD-5, D-14): item 4 (B) is confirmed; the observable consequence of B is that the query "雷" finds
  "李雷", and a query of two ideographs finds any name where they stand in any words.
- Q-86 is closed by this ADR for behavior (B): segmentation without dictionaries, the default UAX #29 rules for all
  scripts; Q-94 is closed by a run for Myanmar (Extend marks attach, letters one by one; `uax29 points`), Tibetan (a
  syllable is a word, tsheg is a separator) and Khmer; the other scripts are governed by the same rule, the class of
  characters was checked in full (`uax29 data`).
- Q-85, Q-87, Q-97, Q-98 are closed by the owner's decision (OD-5) at the default value.

## Sources
- UAX #29 (rev. 49, Unicode 18), rules WB3–WB999, the values Extend/ExtendNumLet/WSegSpace, the caveat about Thai,
  Myanmar, Khmer, Chinese, Japanese; WordBreakTest-18.0.0, WordBreakProperty-18.0.0, emoji-data-18.0.0,
  CaseFolding-18.0.0, DerivedAge-18.0.0: facts.md F-8…F-12, F-21…F-24, F-31…F-38, F-43, F-44, F-47 and the run of
  `research/name-search/` (README.md there).
- ICU4X 2.4.0: facts.md F-2, F-11, F-12; `icu_properties::props::WordBreak`, `ExtendedPictographic`.
- ADR 0008, 0010, 0031; `docs/owner-decisions.md` OD-4 items 1, 2; OD-5 (D-14, D-19, Q-85, Q-87, Q-97, Q-98).
