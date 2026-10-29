# 0046. Record index: search by number, number version, record read
- Status: Accepted
- Label: [OURS]
- Date: 2026-10-27
- CUJ: CUJ1, CUJ2, CUJ4, CUJ6, CUJ8, CUJ9, X-5

## Context
Search by number (ADR 0007, 0035): exact match only, all records of the number by date DESC without pagination or truncation (up to
2 967 164 records), the number version in the result and in each record, an empty list with the initial version for a number not found.
Storage must deliver this as a stream from a consistent snapshot (ADR 0035 item 6) and provide a "point lookup of the version by
number" for a name results page (ADR 0029).

## Decision
1. The index R (ADR 0045 item 1): the key `NumKey ‖ DateKey`, the value is the name; the "header" `NumKey ‖ 000000` is the number version.
   `DateKey` grows as the date decreases, so a direct walk over the leaves from the header gives dates DESC without sorting.
2. Search by number: canonical number → `NumKey` → the search "first entry ≥ `NumKey ‖ 000000`" → read forward while the
   first 8 bytes of the key equal `NumKey`. The header comes first (the snapshot version), then the records. No header means the number is not in the database:
   an empty list and the initial version of the generation (it is not stored, ADR 0027). A number that is the beginning of another is not
   mixed in: `NumKey` has a fixed length and is one-to-one (ADR 0045 item 4).
3. A reader works with one snapshot for the whole duration of the results (ADR 0045 item 9 (c)); writes to the number are not
   held up during the results, and the reader sees the data as of the start (ADR 0035 item 6). Pages needed by the snapshot are not freed until the
   stream is finished; there is no overall lifetime ceiling for a stream: it is revoked only when the client is idle (120 s) and on the retention cap, last (ADR 0056 item 2, D-30);
   the area 3 request timeout may be shorter — a requirement for area 3, and on the outcome of any deadline the snapshot is released. A cut-off stream is distinguishable by the client by the absence of the completion marker (area 3).
4. The version for a name results page: each results record is taken from R by a point lookup of `NumKey ‖ DateKey`, the value is
   the name; the header of the same number is in the same leaf or an adjacent one (the keys are adjacent), and the version is read from the same snapshot. A copy of the version
   is not stored in O and W (ADR 0029 "Consequences").
5. Idempotency and version: add/rename/delete change R (the record, the header) and the derived indexes in one atomic
   operation (ADR 0045 item 2); the version and the record do not diverge (ADR 0029).
6. Cost (research/indexes/output-2026-10-27.txt, 10^8 records, 4 levels, inner ones in the cache): a number with ≤ 10 records —
   1–2 leaf reads (cold: 4); a number with all 2 967 164 dates — ≈ 11 600 pages sequentially (≈ 190 MiB with
   names of 34 bytes; with names up to 800 bytes, up to ≈ 2.4 GB of response), and server memory does not depend on the size of the results.
7. **Changed by OD-6 (2026-10-28).** At 10^7 records the number of tree levels is expected to be 3, and the number's reads are no more than before (T, to be confirmed with a prototype). Q-128 (OD-6): the number version in the result of a search by number is once per number, not in each record (item 4 "and in each record" is cancelled); in name results pages the version stays with each record.

## Alternatives considered
- The version in every R record: rejected (ADR 0029): writing one date would change all records of the number.
- A separate tree of number headers: an extra read on results; having the header next to the records in R gives the version without a separate descent.
- Copying the number's results into memory before sending: rejected (ADR 0035 item 6, up to hundreds of MB); a stream from a snapshot.
- Locking the number's records for the duration of the results: rejected by ADR 0029.
- An index by date DESC as a separate key: not needed, the date is unique within a number (ADR 0035 item 2).

## Consequences
- file-format: the reader snapshot and its retention period (the assumption of ADR 0045 item 9 (c)); the cost is that pages are not reused while the
  reader is alive; the longest reader is bounded by the snapshot ceiling (ADR 0056).
- Area 3: the stream timeout and the completion marker.

## Sources
- ADR 0006, 0007, 0024, 0027, 0029, 0035, 0045; `openspec/specs/number-search`; `research/indexes/calc.py`.
