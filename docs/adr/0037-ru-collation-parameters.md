# 0037. Name comparison under ru locale rules: what follows from the rules
- Status: Accepted
- Label: [OURS]; [USER] (OD-5: Q-93 — "ё" is a variant of "е")
- Date: 2026-10-27
- CUJ: CUJ5, CUJ11

## Context
The owner decided: "alphabetically under ru locale rules" (ADR 0009). Q-31: the role of case, the place of "ё" and the comparison
parameters are not chosen; the presence of ru data in ICU4X is not confirmed. This ADR records what follows from the ru rules themselves and
has been verified; the choice of "flavor" is ADR 0038 [USER] (OD-5).

## Decision
1. Comparison uses `icu_collator` from ICU4X at the version per ADR 0010 (exactly `=2.4.0`, Unicode 18, CLDR 49; pinning of the whole `icu_*` graph is ADR 0039 item 4), locale
   ru, `compiled_data` data.
   Ru data exists in 2.4.0: the ru collator sorts Cyrillic before Latin, and the root one does the opposite (experiment F-19, `research/ordering/`:
   "Яков" < "Ann" for ru; "Ann" < "Бобр" for root). This agrees with `[reorder Cyrl]` in CLDR ru.xml (F-13) and with the
   ICU4X documentation: the reordering of scripts is set by the locale, not by options (F-15).
2. Whole stored names are compared (in NFC, ADR 0003), not words and not the folded search form.
3. "ё": ru.xml has no rules of its own for "ё" (F-13), so the root order applies: "ё" is a variant of "е" with a secondary
   difference. Consequence (experiment): "Елка" < "Ёлка"; "Её" < "Еж" < "Ёж" < "Ежи" < "Жук" — "е/ё" is distinguished by the order only
   when the remaining letters match and does not outweigh the following letters. This does not contradict the search "ё" = "е" (ADR 0008):
   search finds both spellings, and the order distinguishes them.
4. The technical parameters that are not visible to the user and follow from the ru rules are the ICU4X/CLDR defaults:
   `case_level` Off, `max_variable` default, Identical is not used (ADR 0036: the order of alphabetically equal names is decided by the date). Everything the user
   sees in the order and that does not follow from the ru rules is the owner's decision in ADR 0038 (OD-5): comparison strength (`strength`), handling of punctuation and
   spaces (`alternate_handling`), `case_first` (ru default — lowercase before uppercase), numeric ordering
   (off: "Ann 10" before "Ann 2", D-15). One principle: what is visible and not derivable from the ru rules is decided by the owner (ADR 0038).
5. The sort locale is always ru — it does not depend on the client, Accept-Language or machine settings.

## Alternatives considered
- "ё" as a separate letter between "е" and "ж" (traditional alphabetic order): requires custom sort rules,
  which are not in the CLDR "ru locale rules" (F-13); support for custom rules in the ICU4X API was not verified. Not
  chosen: the owner tied the order to the ru locale rules, not to a separate setting; the owner confirmed "ё" as a variant of
  "е" (OD-5, Q-93): "Её" < "Еж" < "Ёж" < "Ежи", custom rules are not needed.
- "ё" = "е" in sorting (Primary): moved to ADR 0038 (D-11).
- The root locale or the client locale: rejected, the owner named ru.
- Numeric ordering ("Ann 2" before "Ann 10"): does not follow from the ru rules, is visible in the results; the default
  (off) and the alternative are in ADR 0038 (D-15).

## Consequences
- The ordering examples in the `result-ordering` capability were obtained by experiment on icu_collator =2.4.0 (the program and the reference
  output are in `research/ordering/`) and independently confirmed with ICU4C; they serve as the reproducibility reference (ADR 0039).
- The ICU4X data version that contains the ru data must be pinned (ADR 0039).
- Status and label: Accepted [OURS] — only items 1–3, 5 and the technical parameters of item 4, which follow from the ru rules or are not
  visible to the user; everything visible and not derivable from the rules (case, punctuation, digits) is moved to ADR 0038 (Accepted, [USER], OD-5).

## Sources
- `facts.md`: F-2, F-13 … F-19 (CLDR ru.xml; docs.rs icu_collator 2.4.0; UTS #10; experiment).
- `research/ordering/README.md` — programs, reference output, cross-check with ICU4C.
- ADR 0003, 0008, 0009, 0010.
