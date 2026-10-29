# 0038. Role of case, punctuation and digits in name sorting
- Status: Accepted
- Label: [USER] (OD-5: D-11, D-12, D-15; Q-93)
- Date: 2026-10-27
- CUJ: CUJ5

## Context
The ru rules define the alphabet but do not say how to order "Анна" and "анна" or how to treat spaces, hyphens and apostrophes.
This is visible in the results (search ignores case, but in the results names of different spelling sit side by side) and the owner did not choose it
(Q-31). It is equally visible how numbers inside names are ordered ("Ann 2", "Ann 10"). The values below are the CLDR/ICU4X
defaults for ru; this is an owner-level question, confirmed by the owner (OD-5). Principle: everything the user sees in the order
and that does not follow from the ru rules themselves is decided by the owner.

## Decision
The values are confirmed by the owner (OD-5):
- **D-11. Role of case.** `strength` = Tertiary (the default for ru): names equal by letters are distinguished by
  case, lowercase before uppercase; case does not outweigh a difference in letters: "анна" < "Анна" < "АННА" < "Антон".
  "е/ё" are distinguished secondarily ("Еж" < "Ёж"), case tertiarily. `case_first` — the ru default (lowercase first).
- **D-12. Spaces and punctuation.** `alternate_handling` = NonIgnorable (the default): space, hyphen and apostrophe are ordinary
  characters that sort before letters and digits: "Ann Zed" < "Ann-Zed" < "Anna" < "AnnZed".
- **D-15. Digits in names.** Numeric ordering is off (the default): digits are compared by characters,
  "Ann 1" < "Ann 10" < "Ann 2". The alternative is on (numeric): "Ann 1" < "Ann 2" < "Ann 10". It affects only the order.

## Alternatives considered
Order on the test sets (experiment F-19, ICU4X 2.4.0, ru, the program and the full output are in `research/ordering/`; all
rows matched ICU4C, F-25; names equal under the collator stand side by side in the table and are not distinguished by order, their order in the
results is decided by the date, ADR 0036 item 2):

| Variant | "Анна", "анна", "АННА", "Антон" | "Ann Zed", "Ann-Zed", "Anna", "AnnZed" | "Её", "Еж", "Ёж", "ёж", "Ежи" |
|---|---|---|---|
| Default: Tertiary, NonIgnorable | анна, Анна, АННА, Антон | Ann Zed, Ann-Zed, Anna, AnnZed | Её, Еж, ёж, Ёж, Ежи |
| Secondary: case is not distinguished alphabetically | АННА = Анна = анна, Антон | same as default | Её, Еж, Ёж = ёж, Ежи |
| Primary: "ё" = "е", case is not distinguished | АННА = Анна = анна, Антон | same as default | Её, Ёж = Еж = ёж, Ежи |
| Tertiary + Shifted: spaces and punctuation are ignored | анна, Анна, АННА, Антон | Anna, Ann Zed, Ann-Zed, AnnZed | Её, Еж, ёж, Ёж, Ежи |
| Tertiary + `case_first` upper | АННА, Анна, анна, Антон | same as default | Её, Еж, Ёж, ёж, Ежи |
| Tertiary + numeric (D-15) | same as default | same as default | same as default |

D-15: "Ann 10, Ann 2" by default; with numeric — "Ann 2, Ann 10" (the "numeric" set in `research/ordering/expected-order.txt`).

Under Secondary and Primary, alphabetically equal names (in the table, "=" in `expected-order.txt`: "АННА = Анна = анна"; under Primary
also "Ёж = Еж") go by date DESC intermixed, not in groups by spelling (ADR 0036 item 3). Rejected as the default:
Secondary and Primary (under Primary the "е/ё" distinction is lost; under both, the order within a name cannot be predicted by case),
`case_first` upper (departs from the CLDR default for no reason), numeric (departs from the CLDR default with no reason in a CUJ), Shifted
("Anna" before "Ann Zed" — words run together, less expected for names of several words).

## Consequences
- The choice affects only the order of the results, not what search finds. Changing the value after v1 is released changes the order
  (ADR 0010: the order is unchanged within v1); the values were confirmed by the owner before v1 (OD-5).
- The scenarios "Role of case and punctuation in name comparison" in `result-ordering` depend on these values; the owner confirmed
  the defaults (OD-5), so no replacement per the table is required.

## Sources
- `facts.md`: F-14 (CollatorOptions, Strength, AlternateHandling), F-16 (case_first, numeric), F-17 (UTS #10),
  F-19 (experiment), F-25 (cross-check with ICU4C), F-26 (results for `case_first` upper and numeric); `research/ordering/README.md`. The decision was made by the owner (OD-5).
