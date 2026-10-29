# 0025. Write error kinds and check order
- Status: Accepted
- Label: [USER] (OD-5: D-6; Q-90 is the owner's alternative)
- Date: 2026-10-27
- CUJ: CUJ3, CUJ6, CUJ9, CUJ12, X-5
- Narrows ADR 0002 item 4 [USER] for rename and delete (OD-5, Q-90: the date is mandatory); clarifies ADR 0005 items 4,
  7 [USER] (at a stale version)

## Context
The brief and ADR 0005, 0006 set the kinds of outcomes (a conflict with the current name and version, a version
conflict, "not found") and one order rule: on rename to the same name the version is checked first (OD-4, item 7). Not
decided: what is returned if the record is missing and the version is stale, which version is "current" for a number
with no records, and in what order the remaining conditions are checked (Q-33). This is behavior visible to clients
(level (O)), so the value was confirmed by the owner (OD-5, D-6).

## Decision
1. Four kinds of write errors: "invalid input" (with the field: number, date, name, expected version, and the value
   error kind from capability `records`, ADR 0021–0024; the dictionary of kinds is not duplicated), "name conflict"
   (add only), "version conflict" (rename, delete), "not found" (rename, delete). Other refusals (permissions, limit,
   database state) belong to other areas. On any error the data and versions do not change.
2. Order for rename and delete: (a) input: number, date (mandatory, item 6), name (for rename), presence and kind of the
   expected version (absence or a wrong kind is "invalid input", field "expected version"; the kind boundary is item 7),
   the first wrong field in this order; (b) version: if the expected one is not equal to the current one, "version
   conflict" with the current version, regardless of whether the record exists; (c) existence of the record: "not
   found"; (d) for rename: the name equals the current one, a success "unchanged", otherwise a replacement.
3. Order for add: (a) input: number, date (optional: without it 1876-03-07 is taken, item 6), name; the expected version
   is not checked: a passed value of any kind, including a wrong one, is ignored, and there is no error for it; (b)
   occupancy of (number, date): a different name is "name conflict", the same name is a success "unchanged", free means
   creation.
4. Access and database state checks (CUJ8, CUJ16) come before the checks of this area; the order between them is defined
   by the "server and API" area and is not fixed here.
5. The current version of a number with no records is the initial version of the current generation (ADR 0027, 0028):
   rename or delete by it when the record is missing is "not found"; by any other, including the initial version of an
   earlier generation (before a crash restart or a start from backup), is "version conflict".
6. [USER, OD-5, Q-90 is the owner's alternative] The date in rename and delete is mandatory: a request without a date is
   "invalid input" with the pair (date, "missing") from the value error dictionary (`records`; ADR 0020 item 7). The
   order of checking the field "date": first presence ("missing"), then "nonexistent day", then "out of range" (ADR
   0024 item 6). Substituting the zero date 1876-03-07 (ADR 0002 item 4 [USER]) applies only on add; for rename and
   delete item 4 of ADR 0002 is narrowed by the owner's decision OD-5. A request with an explicit date 1876-03-07 is
   valid in all operations and addresses the record on that date.
7. The boundary between "invalid kind" and "version conflict" for the expected version. The kind of a version is syntax
   and format: whether the value belongs to the version type set by the API area (Q-32); the kind check does not depend
   on the database state. A value that does not belong to the version type is "invalid input" (field "expected
   version"). Any value that belongs to the type but is not equal to the number's current version is "version conflict"
   (including a value the server never issued, the version of another number, of an earlier generation, of another
   database). For add the version is not checked and has no effect (item 3).
8. **Changed by OD-6 (2026-10-28).** Q-160: a new add error kind "number known": add without a date when the number has
   at least one record on any date (including an identical one on 1876-03-07); the response is the current number
   version (as with a name conflict, without the current name). The add check order: input; without a date, whether the
   number has records; with a date, occupancy of (number, date); the check and the creation are one writer step (ADR
   0055); of two parallel adds without a date of a new number, one creates the record. There are five write error
   kinds. HTTP: 409 `number_known` (ADR 0062). The date value 1876-03-07 (item 6) is substituted only in this case.
9. **Changed by OD-8 (2026-10-28).** Q-180: an exception to item 8: add without a date when there is a record (number,
   1876-03-07) with the same name (compared as for an identical repeat of add with a date) gives "unchanged", not
   "number known"; otherwise the rule of item 8 is as before. The check order of add without a date: input; an
   identical record on 1876-03-07 is "unchanged"; otherwise the presence of records is "number known". There are still
   five error kinds.

## Clarification of owner decisions
- ADR 0005 items 4, 7 [USER] (brief §3, OD-3 Q-19): rename or delete of a missing record is "not found". Here this is
  done at the current number version; at a stale version the outcome is "version conflict" (item 2(b)), because the
  version check is first (OD-4, item 7 [USER]; ADR 0005 item 8). Two owner rules collide on a missing record with a
  stale version; the choice of order is the owner's decision OD-5, D-6, and it affects items 4, 7 of ADR 0005.

## Alternatives considered
- "Not found" before the version: rejected. A client with a stale version, whose record was deleted in parallel, would
  get a response without the current version; the rule would differ from the already accepted "version first" for the
  same name (OD-4, item 7).
- The version is checked before the input: rejected. There is no point checking the version in a request that cannot be
  executed at any version; the client would get "conflict" and, after re-reading, "invalid input".
- Returning a list of all input errors: rejected as a complication outside the brief; the first wrong field is
  reported.
- Substituting 1876-03-07 on rename and delete without a date too (the earlier value, the owner's decision ADR 0002 item
  4, Q-90): rejected by the owner in OD-5. The cost of the earlier value: a rename or delete sent without a date by a
  client error would address the record on 1876-03-07 and could change or delete it; the only protection is the
  expected version. The gain, uniformity with add and the reading of brief §3 "no date means substitute", did not
  outweigh the risk of an accidental change.

## Consequences
- A consequence of item 6: a client that does not specify a date on add (CUJ1, CUJ9) must pass the date explicitly
  (1876-03-07) for a later rename or delete of the same record. The UI (CUJ3: overwrite after a conflict; CUJ6: delete
  from results) and programmatic clients take the record's date from the results or from their own add request (without
  a date, 1876-03-07; the conflict response does not carry the date, ADR 0026 item 4) and pass it; there is no
  substitution in these operations. Expressing the reason "missing" in the protocol is the API area (Q-49, Q-67); how
  the UI knows a record's date is the UI area (Q-28, Q-37).
- The outcome on a parallel delete of the record the client was working with is "version conflict" (a delete changes the
  number's version): partly closes Q-68 on the server side; showing it to the user is the UI area.
- Q-33 is closed by this ADR; Q-90 is closed by the owner's decision (OD-5), the alternative of item 6.

## Sources
- `docs/source/brief.md`, section 3 "Operations"; section 2 (CUJ3, CUJ9, CUJ12).
- OD-4, item 7 (`docs/owner-decisions.md`): "a conflict with the current version (the version check first), a silent
  success only at the current version".
- ADR 0002, 0005, 0006, 0020–0024; `docs/open-questions.md`, Q-33, Q-68, Q-90; OD-5 (`docs/owner-decisions.md`).
