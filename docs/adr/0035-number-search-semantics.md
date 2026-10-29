# 0035. Search by number: match on the canonical form
- Status: Accepted
- Label: [OURS] (items 1–5); [USER] (item 6 — OD-5, D-16)
- Date: 2026-10-27
- CUJ: CUJ4, CUJ6, CUJ8, CUJ9; item 1.4 of the original wording ("search Names by Phone number")

## Context
ADR 0007 [USER] defines search by number: exact match only, all records of the number by date DESC without pagination, an empty
list for a number not found, a format error for an invalid one. It is not defined what exactly the query is compared with (the entered
string or the canonical form), how the search relates to the cleaning done on save, and what makes the result order unambiguous.

## Decision
1. The query is converted to the canonical form by the same cleaning and the same format rules as a number on save
   (the `records` capability; ADR 0004). There are no separate normalization rules for search. "Exact match" is equality of
   canonical forms; comparison of the entered strings is not used.
2. The results are all records of the found number (number, date, name) by date DESC. The date is unique per number (ADR 0002), so the
   order is unambiguous and no additional keys are needed; there is no pagination or cursor.
3. A number not found, in a valid format, is a success with an empty list of records and the number version (the initial version of the current
   generation, ADR 0027); an invalid format is a value error (number, "invalid shape", `records` dictionary) with no results,
   distinguishable from an empty list.
4. The number version is a field of the search-by-number result (present even with empty results, where there are no records) and is duplicated in each record
   of the results, consistent with the number's records in the same results (ADR 0006,
   0029 item 4; the general requirement is set by `write-operations`, and here it is applied to search by number). The client needs the version
   for rename and delete (CUJ6, CUJ9); the representation of the version belongs to the API area (Q-32, Q-34).
5. Search by number is a read: it does not create a number, does not change the version, is available to a "read" token (CUJ8), and reflects all
   changes acknowledged to the client before the search began (CUJ6: the list after a delete). The read isolation level (Q-56)
   is not decided by this ADR.
6. [USER, OD-5, D-16] The upper bound on the number of records of one number is 2 967 164 (the number of valid dates, ADR 0024, F-7: the date
   is unique per number); no separate cap on the number of records per number is introduced on write, and the results for a number are the full
   list without truncation up to this bound. Requirement for storage and the API: return up to 2 967 164 records in one response
   (as a stream, without accumulating the whole list in server memory) and introduce no truncation or continuation token; the client SHALL
   distinguish a complete response from a cut-off one (a storage error after the stream has started is not reported by status): a trailing integrity
   marker (an end marker or a record count in the summary); a response without it is considered incomplete (a requirement for the API). Storage: reading the number history for the duration of the results is a consistent snapshot (records and version as of the start), which does not block
   writes to the number (ADR 0029: blocking isolation was rejected); the snapshot retention period and its cost are a decision of the storage area. Volume estimate:
   at ~100 bytes per record, about 300 MB of response (arithmetic, not a measurement). This is the worst case; in practice a number history
   consists of a few records. Alternatives (rejected by the owner, OD-5, D-16): (a) introduce a cap on the number of records per number on write (for example, 10 000;
   on exceeding it, the add is refused) and thereby limit the results; (b) allow truncation with an "incomplete" marker — contradicts
   ADR 0007 [USER] "all records". Accepted: no cap, as ADR 0007 requires.
7. **Changed by OD-6 (2026-10-28).** Q-128: the number version in the results by number is a field of the result, once per number; records in the results by number do not carry it (item 4 "duplicated in each record" is cancelled); in results by name the version stays with each record. The gain is ≈ 190 MB of JSON on the maximum number (64 × 2 967 164). Q-128 is closed.

## Alternatives considered
- Comparison with the entered string without canonicalization: rejected, ADR 0004 item 3 requires that numbers written differently
  be one number, and CUJ4 must find such a number.
- A search-specific normalization, softer than on save (for example, accepting a number without "+"): rejected, ADR 0007
  item 3: the rules are the same as on save.
- Pagination of long number histories: rejected by ADR 0007 [USER].
- A cap on the number of records per number (item 6, alternative (a)): not introduced, it is a new write restriction that the owner did not set.

## Consequences
- The canonical form, the list of separators and the whitespace character set are defined by the `records` capability (Q-75, Q-84);
  search must use the same function, not a copy (implementation requirement: one canonicalization path).
- Not decided here: the form of the number in the results (Q-26), the row composition and the form of empty results (Q-29), the representation of the number version
  in the response (Q-32, Q-34; the fact that the version is present in the results is item 4), the latency target for 10^8 records (Q-47), showing past and future owners (Q-2).

## Sources
- ADR 0004, 0007, 0024, 0027; `facts.md` F-7; `docs/source/brief.md`, section 3 "Search and sorting"; CUJ4, CUJ6, CUJ8 (user-journeys).
- The derivation of items 1, 4, 5 is a logical consequence of ADR 0004 item 3, ADR 0006, ADR 0007 item 4, ADR 0029 item 4 and X-5; no external facts are used.
