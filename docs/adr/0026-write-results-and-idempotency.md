# 0026. Results of successful write operations and idempotency
- Status: Accepted
- Label: [USER] (OD-5, D-7)
- Date: 2026-10-27
- CUJ: CUJ1, CUJ3, CUJ6, CUJ9, CUJ12, X-5

## Context
The brief requires a "silent success" for a repeat of an identical add and for rename to the same name, but does not say
what a successful response contains: whether the version is returned and what an identical repeat answers (Q-32). It was
accepted that a repeated rename after a connection drop gives "not found" (brief §4), with a note that this is now
probably closed by versioning (Q-54).

## Decision
1. The success of add, rename and delete returns the outcome, "changed" or "unchanged", and the number version current
   after the operation. The version is opaque: it is compared only for equality; the representation (number, string) is
   defined by the API.
2. "Unchanged": add of an identical record; rename to the same name at the current version. The version stays the same
   (OD-3, Q-80).
3. Deleting the last record of a number returns the initial version of the current generation (ADR 0027).
4. "Name conflict" returns the record's current name and the number version; "version conflict" returns the number's
   current version.
5. Idempotency when the number's records are unchanged: add and rename to the same name are idempotent. A repeat of
   rename and delete after a first success, with the earlier version, gives "version conflict": the number's version has
   changed. The client re-reads the number with a search by number (ADR 0029 item 4) and sees its own result; there is
   no second effect. If another client changed the number between write attempts, the promise does not hold: a repeat of
   add after someone else's delete creates the record again (add without a version is an owner decision), after
   someone else's rename it is "name conflict"; a repeat of rename and delete is "version conflict".
6. For Q-54: the message "not found" on a repeat after a connection drop does not arise while the version is stale (the
   version is checked first, ADR 0025); one's own executed request cannot be told from someone else's identical one by
   the version alone, only by the content of the re-read history.
7. **Changed by OD-6 (2026-10-28).** Q-160: only add with a date is idempotent; a repeat of add without a date after a
   success gives "number known" (not "unchanged"), and the client finds the outcome by search by number.
8. **Changed by OD-8 (2026-10-28).** Q-180: add without a date is also idempotent if the number has a record on
   1876-03-07 with the same name: a repeat after a lost response gives "unchanged" (HTTP 200 `unchanged`, the version is
   the same); otherwise, when records exist, it is "number known". Replaces "a repeat of add without a date after a
   success gives 'number known'" from item 7.

## Alternatives considered
- A success without a version: rejected. After each operation the number would have to be re-read to continue a chain of
  changes (for example, the overwrite confirmation in CUJ3 after a conflict).
- Returning the number history in the response: rejected as an extension outside the brief.
- Not distinguishing "changed" and "unchanged": rejected. The client would not see whether a record was created or
  already existed.
- A repeat of rename and delete as a success "unchanged": rejected. It would require issuing idempotency keys or keeping
  an operation log (the log is out of scope, ADR 0018).

## Consequences
- Q-32 is closed in substance (what is returned); the representation of the version and the response format: Q-32 stays
  with the API (area 3).
- Q-54 is narrowed in semantics (it stays with the API: representation); D-7 is confirmed by the owner together with ADR
  0025 (OD-5, D-6, D-7).

## Sources
- `docs/source/brief.md`, section 3 "Operations"; section 4 (retrying a rename after a connection drop); section 2
  (CUJ9).
- OD-3, Q-80; OD-4, item 7 (`docs/owner-decisions.md`).
- `docs/open-questions.md`, Q-32, Q-54; ADR 0005, 0006, 0018, 0019.
