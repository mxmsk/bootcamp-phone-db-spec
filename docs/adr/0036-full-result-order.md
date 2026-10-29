# 0036. Total result order of name search
- Status: Accepted
- Label: [OURS]
- Date: 2026-10-27
- CUJ: CUJ5, CUJ7, CUJ8

## Context
ADR 0009 [USER] sets the visible order: name alphabetically under ru, date DESC, and for equal name and date the number ascending as a
string. Cursor pagination (CUJ5) needs a total order: any two distinct records must compare unambiguously
(Q-73). Alphabetic comparison under ru can treat different strings as equal (verified, below); the question is what to do with
such names: in iteration 2 a code-point key stood between the alphabet and the date (this is not a literal
reading of ADR 0009).

## Decision
1. The record order key, exactly in the order of ADR 0009: (A) the alphabetic comparison key of the name per ADR 0037/0038; (B) the date in
   descending order; (C) the canonical number string by code points ascending, a shorter prefix string first.
   A record is identified unambiguously by the pair (number, date), so (A)–(C) define a total order with no equal records: two distinct records
   cannot match on both date and number at once.
2. There is no separate key "spelling of the name by code points" between the alphabet and the date (in iteration 2 it stood there; removed
   after a review comment). Names that alphabetic comparison treats as equal are "alphabetically equal" in the sense of ADR 0009 [USER],
   and the owner's next key applies between them, date DESC. Verified: "Ann" and "A­nn" (U+00AD) are equal under the parameters of
   ADR 0037 (`research/ordering/`, the "invisible" row). Scenario: the records "Ann" on 2024-01-01, "A­nn" on 2023-01-01 and "Ann"
   on 2022-01-01 go in exactly that order: dates descend, as seen by a user to whom both spellings look
   identical. A code-point key would give "Ann" 2024, "Ann" 2022, "A­nn" 2023: the dates visibly do not descend, and that would violate the
   owner's "then date DESC". The visible order does not change compared with ADR 0009.
3. The same applies when the ADR 0038 parameters change (D-11): Secondary and Primary were rejected by the owner (OD-5, D-11); had they been chosen, case (and "е/ё" under Primary)
   would stop distinguishing names, and such records would go by date DESC, not in groups by case; a separate order key is not
   needed for this.
4. The order does not depend on the query, the set of found records or the page size; there is no relevance.
5. The cursor is a position in this total order, not an index into the results (ADR 0032, [OURS], `name-search` capability): the position
   is given by the triple (name, date, number) of the last returned record, and continuation is strictly after it by keys (A)–(C).
   This defines the order the cursor walks; the lifetime, page size and cursor refusal are ADR 0032, 0033, 0034.
6. The same order applies to exact whole-name search (ADR 0031 item 3, [OURS], Accepted; pagination of exact search is item 4, owner decision OD-5, D-9): names equal under the exact search rules
   may differ in case and "ё"/"е", and keys (A)–(C) apply to them in the same way.

## Alternatives considered
- Key (B) "NFC string by code points" between the alphabet and the date (the iteration 2 decision): rejected, it changes the visible order
  relative to ADR 0009 [USER] for alphabetically equal names (see item 2), and under Secondary/Primary it would group cases by
  code points instead of by date. It is not needed for a total order: the pair (number, date) provides completeness.
- A code-point key after the date and number: does not change the order (the pair (number, date) is unique), so it is not introduced.
- The Identical level of the collator (NFD by code points as a fifth level; docs.rs Strength::Identical): rejected, docs.rs:
  "makes the comparison slower and should not be used in the general case"; it would likewise put code points before the date.
- Numeric comparison of numbers: rejected by ADR 0009 / OD-4 item 6 [USER] — by string.

## Consequences
- Storage (area 2) must be able to return records in the order (A)–(C) and continue the results from any position-triple.
  The index key is the name sort key, then date DESC, then number; the stored spelling of the name is not part of the key.
- Key (A) is computed per ADR 0037; its stability in v1 is ADR 0039.

## Sources
- ADR 0008, 0009; `docs/source/brief.md`, section 3.
- `facts.md`: F-14 (docs.rs icu_collator `Strength`), F-19 (experiment); the program and independent cross-check with ICU4C —
  `research/ordering/README.md`.
