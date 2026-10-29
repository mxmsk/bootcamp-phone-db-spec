# 0047. Name results in order, continuation from the cursor position, exact name search
- Status: Accepted
- Label: [OURS]
- Date: 2026-10-27
- CUJ: CUJ5, CUJ7, CUJ8, CUJ12

## Context
The result order is three keys (ADR 0036): the name sort key, date DESC, number. The cursor is the position (name, date, number)
of the last returned record (ADR 0032); continuation is strictly after it, without a snapshot and without reading earlier pages. Exact search
(ADR 0031) returns all records whose name comparison key equals the query key, in the same order, with the same cursor.

## Decision
1. All results go in the order of `OrderKey` (ADR 0045 item 4): its bytes order records exactly as (name ru, date DESC,
   number as a string). The cursor position is a triple; its `OrderKey` is rebuilt from the name of the position: the sort key is computed by
   the same code (the name in the cursor is valid and normalized, ADR 0032), and the date and number are encoded as `DateKey`, `NumKey`.
2. Continuation. The next page = the records with `OrderKey` strictly greater than the `OrderKey` of the position, in ascending order. The position
   need not exist (the record may have been deleted): the search "first entry > key" works for any key. The cost
   of continuation does not depend on the page number: one tree descent and a forward read; earlier pages are not read, and there is no cursor
   state on the server.
3. A page is full if there are more records after it: one record more than the page size is read (a cursor is issued only if
   a next record exists), and the cursor is the position of the last one returned. Consistency within a page is one snapshot
   (ADR 0045 item 9 (c)); there is no snapshot between pages (Q-56).
4. Exact search: the range `0x02 ‖ H64(query comparison key)` in W, with records already in `OrderKey` order. Each record
   of the range is checked by reading the record from R (needed for the results — name, version) and comparing the comparison key of its name with
   the query key (a hash collision is filtered out; the page is filled further along the range until it is full or the
   range ends). Reads per page: a W descent (1–2 with inner pages cached), 1–2 W leaves, ≈ 50 point reads of R;
   parallel fetching of R reads is allowed and recommended (the latency estimate of ADR 0049).
5. Records with an equal sort key (names equal alphabetically, ADR 0036 item 2) are distinguished by the date and number inside
   `OrderKey`, so the order is total without a separate key by spelling.
6. The name in the results is taken from R, not from the key: the sort key is not invertible to a name.
7. **Changed by OD-6 (2026-10-28).** The scale is 10^7 records (ADR 0017); the numbers and estimates for 10^8 in this ADR are the history of the calculation, and the current values for 10^7 are in ADR 0044, 0049, 0052, 0058, 0059; nonlinear estimates are recomputed on a prototype (T, `docs/open-questions.md`).

## Alternatives considered
- The index O and exact search through a range of O "equal by primary weights": relies on the structure of the ICU4X sort key
  (the first 0x01 separator) and on the assumption "equal comparison keys have equal primary weights", neither proven; rejected
  in favor of the independent space 0x02 (the cost is ≈ 4 GB at 10^8, ADR 0049).
- The cursor as an ordinal number of the record in the results: impossible without full materialization (ADR 0032).
- Server-side cursor state (an open iterator): rejected by ADR 0032 (the cursor survives a restart and a restore).
- Storing the name in W/O to return results without reading R: grows the file by ≈ 3.4 GB (O) and ≈ 8.5 GB (W), and the number version is still
  taken from R (ADR 0029); there is no gain in reads.

## Consequences
- Q-73 (total order) is closed by storage; continuation from any position without reading earlier pages is ensured.
- For exact search of a frequent name (10^6 records) a page costs the same as any other: the range is not materialized.

## Sources
- ADR 0009, 0031, 0032, 0036, 0045; `openspec/specs/{name-search,result-ordering}`.
