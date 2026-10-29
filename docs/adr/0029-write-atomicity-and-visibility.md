# 0029. Atomicity and visibility of a write operation
- Status: Accepted
- Label: [OURS] (items 1–4); [USER] (item 5: OD-5, D-8)
- Date: 2026-10-27
- CUJ: CUJ1, CUJ6, CUJ9, CUJ12, CUJ14, X-5

## Context
CUJ1 and CUJ6 require that what is saved can be found, and that after a delete the re-read list is up to date. CUJ12 and
CUJ9 require that of concurrent actions by one version at most one succeeds. Brief §5 leaves the isolation level of reads
and writes to the owner (Q-56).

## Decision
1. [OURS] Each write operation is one indivisible action: the record and the version change together; neither readers nor
   recovery after a failure see a partial result (ADR 0006, item 6 (b); CUJ14).
2. [OURS] The check of the expected version and the change are one action: of concurrent operations on one number with
   one version, at most one successfully changes data (CUJ12, X-5).
3. [OURS] Operations on one number are ordered (as if executed one after another); operations on different numbers do not
   depend on each other.
4. [OURS] A search started after the client received the success of an operation sees its result entirely; a search
   concurrent with an operation sees the state before or after, entirely. Every record in the result of any search (by
   number and by name) contains the version of its number, consistent with the records of this number in the result:
   search by number returns all records of the number and the version (for a number with no records, the initial one); in
   one page, records of one number have one version, taken from the same state. This gives the client a valid value of
   the expected version for rename and delete from CUJ6, CUJ9, CUJ12 (re-read, repeat) without a trial wrong request.
   Consistency between result pages is not promised; a stale version is safe: it gives "version conflict".
5. [USER, OD-5, D-8] The isolation level of reads and writes is an item of brief §5, decided by the owner (OD-1: "propose
   and ask"). Decision: items 1–4 (derived from CUJ1, CUJ6, CUJ12; the owner confirmed) and, for results over several
   pages of one read (a cursor), consistency between requests is not promised: each page reads the state at the time of
   its request, and a stale version is safe (item 4). Alternatives and cost: (a) eventual consistency (a reader may not
   see what was acknowledged): cheaper implementation and replication, but violates CUJ1 ("is found") and CUJ6 (an updated
   list after a delete); not chosen; (b) strict serializability of all operations (one global queue): easier to reason
   about, but write latencies grow with load, and the semantics does not require this (operations on different numbers
   are independent); not chosen; (c) a snapshot for the whole result over pages: the pages are consistent with each other,
   the cost is holding state for the duration of the traversal (storage, Q-56), the snapshot lifetime and its release; (d) a
   consistent slice for one read across several numbers: there is no such operation in the system (search by number is one
   number, no operations are to be introduced), so it is not needed. The choice is items 1–4 and "between pages is not
   promised": the minimum cost to storage; the client sees the difference only as "version conflict" after re-reading.
6. **Changed by OD-6 (2026-10-28).** The scale is 10^7 records (ADR 0017); the numbers and estimates for 10^8 in this ADR
   are calculation history, the current values for 10^7 are in ADR 0044, 0049, 0052, 0058, 0059; non-linear estimates are
   recalculated on a prototype (T, `docs/open-questions.md`).

## Alternatives considered
- Eventual consistency (a reader may not see what was acknowledged): rejected, contradicts CUJ1 ("is found") and CUJ6 (an
  updated list after a delete); the option is recorded in item 5 (a), rejected by the owner (OD-5, D-8).
- For item 5: the other alternatives with cost are in item 5 itself ((b)–(d)); the choice was confirmed by the owner
  (OD-5, D-8).
- Blocking isolation of readers for the duration of a write: rejected as a latency requirement contradicting "without
  interruption" during backup (ADR 0016); the way to implement is up to storage.
- One global queue of all operations: not required by the semantics; acceptable to an implementation if latencies are
  within the targets.

## Consequences
- For storage: reading a number's records together with the version comes from one consistent state; the write is
  indivisible; the number's version is available when a name-search results page is produced (up to the maximum page size:
  50 by default, the maximum is area 3; records of different numbers). It is acceptable to read the version by a separate
  point lookup by number if the records and the version are visible to the reader from one state; storing a copy of the
  version in every record of the name index is not required: write amplification (up to 2,967,164 records of a number by
  the number of valid dates, F-7) is stipulated by no one. The requirement is observable ("the version is consistent with
  the records"), the method is up to storage (the "Requirements for other areas" section of `design.md`; the cost at 10^8
  numbers is Q-47, Q-58).
- For search (name-search, number-search): a reference to the requirement "Number version in search results" of capability
  `write-operations`; Q-34 is narrowed to the form in which the version is passed (API).
- Q-56: the isolation level as a whole is decided by the owner (OD-5, D-8); for results over several pages it is open and
  handed to the search and storage areas.

## Sources
- `docs/source/brief.md`, section 2 (CUJ1, CUJ6, CUJ9, CUJ12, CUJ14), section 5 (isolation level).
- ADR 0006, 0014; `docs/open-questions.md`, Q-56.
