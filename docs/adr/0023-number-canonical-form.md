# 0023. Number: cleaning, canonical form, comparison
- Status: Accepted
- Label: [OURS] (items 1–4, 6, 7); [USER] (item 5: OD-5, D-4)
- Date: 2026-10-27
- CUJ: CUJ1, CUJ2, CUJ4, CUJ9; items 1.3, 1.4 of the original wording

## Context
ADR 0004 fixes the input policy (an owner decision): "+", then 1–15 digits; spaces, hyphens, parentheses and dots are
removed; numbers written differently are one number after cleaning. Not defined: what exactly counts as a "space", a
"hyphen" and a "digit", what remains after cleaning (Q-75), how numbers are compared and which number counts as
"another". D-1 (first digit 1–9, no list of country codes) is confirmed by the owner (OD-5), ADR 0004, item 6, and is
not changed here.

## Decision
1. [OURS] Order: (a) remove all separators from the input string (items 2, 5); (b) check the result: the first character
   is "+", then 1–15 digits (ADR 0004, item 2; first digit 1–9: D-1); (c) the result is the canonical form.
2. [OURS] Separators removed at any position: the whitespace characters of ADR 0021, item 1; parentheses "(" U+0028 and
   ")" U+0029; the dot "." U+002E; hyphens per item 5. Parentheses are not checked for pairing. Any other character
   (letters, "+" not in first place, "/", "x", "#", etc.) makes the number invalid: it is not removed and not replaced.
3. [OURS] Digits are only ASCII "0"–"9" (U+0030–U+0039). Arabic-Indic, fullwidth and other digits of other scripts are
   invalid (not removed, not converted). "+" is only ASCII U+002B; the fullwidth "＋" is invalid.
4. [OURS] The canonical form of a number: the string "+" and from 1 to 15 ASCII digits with no separators, for example
   "+79991234567". Two numbers are the same if and only if their canonical forms match character by character. There are
   no other equivalences: "+7999…" and "+7999…0" are different numbers, national prefixes ("8", "0") are not converted,
   zeros are not dropped.
5. [USER, OD-5, D-4] Hyphens are U+002D (HYPHEN-MINUS), U+2010 (HYPHEN) and U+2011 (NON-BREAKING HYPHEN) (F-6). Dashes
   U+2012–U+2015 and the minus sign U+2212 are not hyphens, a number with them is invalid.
6. [OURS] The same rules apply to a number in all operations where it is entered: add, rename, delete and search by
   number (ADR 0004, item 5; ADR 0007). Empty input, input of only separators and "+" without digits are invalid.
7. [OURS] There is one kind of number value error, "invalid shape" (ADR 0020, item 7); the "missing plus" flag (a hint
   per ADR 0004, item 4) is set if after cleaning there is no "+", at least one digit remains, there are no other
   characters, and all digits are ASCII: "79991234567", "0079991234567". Other invalid inputs have no flag ("7+999",
   "abc"), including a string empty after cleaning (the empty string, "  - () . "): there are no digits.

## Alternatives considered
- Removing any non-digit character: rejected, "+7-abc" would turn into a valid number, while the owner named specific
  separators (ADR 0004, item 3).
- Accepting digits of any scripts with conversion to ASCII: rejected, it needs Unicode digit tables and extends the
  owner's policy without his request; invalid input is rejected with a clear error.
- Treating all characters of category Pd (dashes) as hyphens: rejected, "+7–999" with an en dash is more often a typing
  error than a separator; D-4, rejected by the owner (OD-5): only ASCII "-" (stricter); all of Pd (softer).
- Normalizing by the dialing rules of the country (national prefix, "+7" ↔ "8"): rejected, it needs a database of
  country codes, and D-1 excludes it; no CUJ requires it.
- Storing the entered spelling together with the canonical one: not introduced, the owner defined "one number"; how a
  number looks in results is Q-26.

## Consequences
- Number comparison is exact string comparison (or any one-to-one representation); the internal representation in
  storage belongs to the storage area, provided it is one-to-one and supports 1–15 digits (a requirement for storage).
- How a number looks in results and responses is Q-26; the hint on rejection without "+" is Q-49, Q-65.
- The scope of "whitespace" for name search: ADR 0021, 0008.

## Sources
- ADR 0004, 0007, 0021; OD-3, Q-23, Q-6; brief §3/Data, §4.
- F-3, F-6 (`facts.md`): PropList White_Space; names U+2010/U+2011 (checked against UnicodeData.txt of Unicode 18.0.0,
  `research/records/`).
