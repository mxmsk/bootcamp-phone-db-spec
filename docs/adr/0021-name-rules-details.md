# 0021. Name: normalization, whitespace character set, validity
- Status: Accepted
- Label: [OURS] (items 1–4, 6); [USER] (item 5: OD-5, D-2; Q-88: OD-5)
- Date: 2026-10-27
- CUJ: CUJ1, CUJ2, CUJ3, CUJ9; item 1.3 of the original wording

## Context
ADR 0003 (an owner decision) says to trim the name at the edges of "whitespace" and convert to NFC, but does not name the
whitespace set (Q-84, category (T)) or the order of the two actions, and does not say which characters are valid in a
name besides "not empty" and "not longer than the cap". These details change visible behavior, so explicit rules are
needed.

## Decision
1. [OURS, Q-84] "Whitespace characters" for trimming a name (and cleaning a number, ADR 0023) are the code points with
   the Unicode property White_Space, 25 of them: U+0009–U+000D, U+0020, U+0085, U+00A0, U+1680, U+2000–U+200A, U+2028,
   U+2029, U+202F, U+205F, U+3000 (F-3). The source of the White_Space and Cc properties is the `icu_properties` 2.4.0
   data, equal to UCD 18.0.0 (F-47). Zero-width space U+200B, U+FEFF, U+180E and control characters outside the list are
   not whitespace.
2. [OURS] Name normalization: first NFC, then removing whitespace characters from both edges. Inside the name the
   characters are not changed (apart from the effect of NFC). The result is the stored name. The owner's wording "trim,
   convert to NFC" (OD-3, Q-3) gives the same result on any input: NFC maps whitespace characters only to whitespace
   (U+2000→U+2002, U+2001→U+2003), no non-whitespace character decomposes into a whitespace one, and whitespace
   characters have class 0 and do not take part in composition. Checked by a run on Unicode 18.0.0 data (F-4;
   `research/records/`): no discrepancies on 85,404,800 checked strings, including all pairs and triples "whitespace +
   any scalar value". So the order is not visible, and the owner's decision is not changed. The same function normalizes
   the name-search query (ADR 0008, item 5).
3. [OURS] "The same name" is a byte-for-byte (character-for-character) match of normalized names: case is
   distinguished, "ё" and "е" are distinguished, whitespace inside is distinguished (ADR 0003, items 1–2).
4. [OURS] A name is invalid if after normalization it is empty (ADR 0003, item 3; reason "empty") or exceeds the cap
   (ADR 0022; reason "too long"). A name that looks empty but contains a non-whitespace invisible character (U+200B and
   the like) is non-empty and valid.
5. [USER, OD-5, D-2] The normalized name must not contain control characters (General_Category Cc: U+0000–U+001F,
   U+007F–U+009F). Consequences: "Ann<TAB>Smith" and "Ann<LF>Smith" are rejected; a tab or a line feed at the edges is
   trimmed as whitespace and does not interfere. Other characters (including formatting, rare and unassigned ones) are
   valid. The refusal reason is "control character".
6. [OURS] The order of reasons for a name: "empty", then "control character", then "too long"; the first is reported
   (ADR 0020, item 5). A name of 300 whitespace characters gives "empty"; "Ann<TAB>Smith" of length 201 gives "control
   character".

## Alternatives considered
- The set "only U+0020": rejected, copying from documents brings U+00A0, U+3000 and the like; trimming "Ann " against
  "Ann" would be visible and inexplicable to the user (brief §3: "after whitespace trimming").
- Python's `str.isspace()` set or other library sets: rejected as a tie to an implementation; White_Space is a Unicode
  property, and the data version is fixed by ADR 0010 (Unicode 18).
- Treating U+200B, U+FEFF as whitespace: rejected, they are not whitespace by Unicode (U+200B is a format character, Cf;
  they are not in PropList, F-3); a name made of such characters is non-empty, a conscious risk, see Consequences.
- D-2, rejected by the owner (OD-5): allow any characters, including control ones (simpler, but NUL and a line feed in a
  name break output and logs); also forbid formatting characters Cf (stricter, but spoils names with ZWJ/ZWNJ in emoji
  and a number of scripts).
- Conversion to NFKC instead of NFC: rejected, the owner chose NFC (ADR 0003).

## Consequences
- A name of a single U+200B is non-empty and valid; searching for it gives nothing useful. Accepted so as not to
  introduce our own list of "invisible" characters; the owner confirmed: there is no "has visible content" check (OD-5,
  Q-88). The alternative (require at least one visible character) is rejected: "visible" has to be defined, and the
  decision depends on the Unicode version.
- Name search (ADR 0008) normalizes the query in the same way. `records` does not set the validity of control characters
  in a query (the query is not stored): `name-search` does not reject them (ADR 0034, item 5), so a query has no "control
  character" reason; D-2 (item 5) applies only to the name.
- Normalization is performed by the ICU4X library (ADR 0010; the choice of functions is Q-76). The implementation test
  must repeat the check of `research/records/` (equality of the orders "NFC, trim" and "trim, NFC") on the data version
  in the build.

## Sources
- ADR 0003, 0010; brief §3/Data; OD-3, Q-3, Q-81 (`docs/owner-decisions.md`).
- F-3, F-4 (`facts.md`): PropList-18.0.0 White_Space; UAX #15, NFC; the Unicode 18 check is in `research/records/`.
