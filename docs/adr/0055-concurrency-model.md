# 0055. Concurrency model: one writer, many readers on snapshots
- Status: Accepted
- Label: [OURS]
- Date: 2026-10-27
- CUJ: CUJ1, CUJ6, CUJ9, CUJ12, CUJ14, X-2, X-5

## Context
ADR 0029 (items 1–5, the owner's decision OD-5, D-8) sets the isolation: a write is indivisible; operations on one number behave as if in turn;
a read after an acknowledged write sees it; consistency between result pages is not promised. The method is up to storage.
ADR 0029 rejects blocking isolation of readers ("without interruption" during backup, ADR 0016). One server process owns the
database (X-2). The ordinal part of the version is taken from the common generation counter (ADR 0028 item 6 (f)), that is, all writes already have a
common ordering point.

## Decision
1. All write operations are executed by one writer in one order (the commit order); each acknowledged operation
   is assigned a commit number N: strictly increasing, without gaps. Batch commit (several operations with one
   flush to stable storage) is allowed, the operations of the batch stay ordered and are published together.
2. There are many readers; a reader takes no locks that the writer waits on, and vice versa.
3. A reader works on a snapshot: the state after the first N operations. Opening a snapshot is remembering the published N and the LSN of the last published WAL entry
   (without copying data, independent of the database size; pages removed from the trees are compared by LSN, ADR 0044 item 2). Earlier page versions needed by snapshots are retained until the closing of
   the last snapshot that needs them (the cost — ADR 0056).
4. Visibility = durability: an operation is published (N becomes visible to new snapshots) only after it is
   durable (the method and moment of the flush are decided by durability); the acknowledgment to the client comes after publication. A reader does not see
   a change that could vanish on failure; "read after write" (ADR 0029 item 4) holds: the client received success
   after publication.
5. Checking the expected version and the change are one writer step (ADR 0029 item 2): there is no other write between them.
6. One request — one snapshot (including the versions of the numbers of a page by name, ADR 0029 item 4); the snapshot is released at the end
   of the request. Between requests (pages) the snapshot is not kept, the cursor carries no state on the server (ADR 0032): this is exactly
   "not promised between pages" from ADR 0029 item 5.
7. Implementation of isolation: snapshot isolation with a global order of writes; it is stricter than required (ADR 0029 item 3: operations on
   different numbers are independent) and satisfies items 1–4 of ADR 0029. A single order is an acceptable implementation if the latencies are within
   the targets (ADR 0029, "Alternatives", last item).
8. Snapshots live only in process memory; after a restart there are none (ADR 0056).

## Alternatives considered
- A readers–writer lock (RW-lock on the database or on pages): rejected by ADR 0029 (blocking isolation) and by "without
  interruption" during backup: a reading backup of 40 GB would hold the writer for minutes (ADR 0056, calculation).
- Several writers by numbers (shards): rejected. The name, word and order indexes are common to all numbers, the common version
  generation counter is an ordering point anyway, shards would give page conflicts and a complex check. The gain is measurable only
  with writes above the throughput of one writer, and the write target (Q-58) does not require this: the ceiling is the flush to stable storage,
  which under batch commit is shared between operations (an estimate, not a measurement; verified by the implementation's load test,
  if insufficient the question returns).
- Optimistic read with a version check without a snapshot: rejected, the reader reissues the read at every write, on long
  results (up to 300 MB) it may never finish; a snapshot gives the guarantee without retries.
- A snapshot per paged result (ADR 0029 item 5 (c)): not chosen by the owner (OD-5, D-8).
- Visibility before durability (the reader sees, acknowledged later): rejected, a reader or backup would see a state
  that will not exist after a failure; the cost of the chosen one — the reader sees a write later by the flush time (fractions of a batch).
- Prior art: the SQLite WAL mode — one writer, readers do not block the writer and vice versa, a reader remembers
  the "end of the WAL" for the duration of a transaction, a long reader stops the checkpoint and WAL growth is unbounded (`facts.md`,
  F-82); here it is the same, but with an explicit retention cap (ADR 0056).

## Consequences
- Write throughput is limited by one writer; a requirement for the implementation: measure against the Q-58 targets.
- The flush to stable storage and the commit moment (Q-57) are determined by the durability subtask; here only the order "durable → publication →
  acknowledgment".
- Requirements for the API (area 3): the acknowledgment is sent after publication.
- Q-56 is closed for storage: writes by number are sequential, a read after a write sees the result; for pages — a snapshot per request.

## Sources
- ADR 0006, 0014, 0016, 0028 item 6, 0029, 0032; X-2, X-5; `docs/open-questions.md`, Q-56.
- SQLite WAL (https://www.sqlite.org/wal.html), quotes in `facts.md`, F-82.
