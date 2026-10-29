# 0020. Record: the key (number, date), uniqueness and record identity
- Status: Accepted
- Label: [OURS]
- Date: 2026-10-27
- CUJ: CUJ1, CUJ2, CUJ3, CUJ4, CUJ9; item 1.3 of the original wording

## Context
ADR 0002 sets "record = (number, date) → name" and "for one date a number has exactly one owner", but does not say what
exactly is compared in the key ("the same number" after separator cleaning), what an "identical record" means and in
what order the fields are checked if several are wrong. Without this the scenarios of CUJ1–CUJ3 and CUJ9 cannot be
verified.

## Decision
1. A record consists of three values: number, date, name. The record key is the pair (canonical number, date); the name
   is the value of the record, not part of the key (ADR 0002, item 1; the canonical form: ADR 0023).
2. Only the key is unique: two records with one key do not exist. The name is not unique: one name may stand at
   different numbers and at one number on different dates (ADR 0002, item 2).
3. A number written differently but with one canonical form is the same number, so one key for the same date (ADR 0004,
   item 3).
4. An identical record: the same key and the same name after normalization (ADR 0021). Identity is the "same name" of
   ADR 0003; a difference only in case is not identity (ADR 0003, item 1).
5. Values are validated independently of the database state and of the clock: the validity of a number, date and name is
   a function of the value alone (and the kind of operation: "date missing" is an error for rename and delete, but not
   for add, ADR 0025 item 6). If several fields are wrong, the first in order is reported: number, date, name; the order
   of reasons within a field is in ADR 0021 (name), 0024 (date).
6. The date does not take part in comparing names, and the name does not take part in comparing dates: the order and the
   searches are defined by other capabilities.
7. A single dictionary of value error kinds: an error is a pair (field, reason). Number: "invalid shape" (with the
   "missing plus" flag, ADR 0023); date: "nonexistent day", "out of range" (ADR 0024), "missing" (only rename and
   delete, ADR 0025 item 6); name: "empty", "control character", "too long" (ADR 0021, 0022); name-search query: "too
   long" (ADR 0022), "empty" (ADR 0008, 0031, 0034); the query does not reject control characters (ADR 0034, item 5),
   so a query has no "control character" reason. The order of query reasons: "too long", then "empty" (ADR 0034, item
   4). Other capabilities refer to this dictionary and do not introduce other names: "number format error" is (number,
   "invalid shape"). "Invalid input" on write is any such pair.

## Alternatives considered
- The name as part of the key: rejected, "for one date a number has exactly one owner" (brief §3/Data); otherwise there
  would be several names for one date.
- Reporting only the field without the reason: rejected, CUJ1–CUJ3 and CUJ9 require distinguishable messages, and
  different names for one reason in neighboring capabilities (there were "invalid input" and "name value error") caused
  discrepancies in tests.
- Reporting all wrong fields at once: rejected as an extension of the error contract (the shape of an error is the API
  area, Q-49); one first violation in a fixed order gives deterministic validation.
- The check order "name, number, date" (the order of form fields): rejected in favor of the key order; the difference is
  visible only with several errors at once, and there is no substantive argument for another order.

## Consequences
- Storage must distinguish records exactly by the key (canonical number, date); the requirements for storage are in
  `design.md` ("Requirements for other areas", section "Storage").
- The representation of value errors in the protocol (Q-49) must preserve both the field and the reason from the
  dictionary (item 7).
- The order of value validation relative to the version check and the record lookup (Q-33) is defined by the operations
  capability.

## Sources
- ADR 0002, 0003, 0004; `docs/source/brief.md`, section 3 "Data"; `docs/open-questions.md`, Q-49.
