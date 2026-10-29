# 0056. Read snapshots: retention period, number stream, cost
- Status: Accepted (numeric values of the caps — ADR 0059, Accepted (OD-6); the place where earlier versions are kept, item 1 (b), — Accepted (OD-6))
- Label: [OURS]; [DEFAULT] (item 1 (b) — D-35)
- Date: 2026-10-27
- CUJ: CUJ4, CUJ5, CUJ6, CUJ8, CUJ9, CUJ15, X-1

## Context
ADR 0035 item 6 (OD-5, D-16): the number history results — up to 2,967,164 records (~300 MB by estimate) in one response, as a stream, a consistent
snapshot at the start, without blocking writes of the number; the retention period and cost are a storage decision. A snapshot retains earlier
page versions, and the retained volume grows with the writes during the snapshot's life.

## Decision
1. A snapshot is the state after the first N operations (ADR 0055 item 3). It retains earlier
   page versions changed by operations after N; what is retained returns to circulation when the last snapshot that needs
   it closes. Retention is done in two ways (agreed with the format): (a) pages removed from the trees by an operation stay in the free
   page list with `freed_at` and are not handed out while a snapshot that sees them is alive (ADR 0044 item 2); (b) earlier published versions of pages
   changed after N. The writer creates a new page version in memory for each operation (ADR 0051 item 3), published versions are immutable and form a chain; only the checkpoint
   changes the `data` file, and it holds the version of the last checkpoint (not "the last durable"), so a snapshot needs versions from the chain, not from the file. [DEFAULT, D-35, closes Q-125]
   Versions needed by snapshots are kept in process memory (not evicted from the cache), and beyond the memory budget for retention (default 256 MiB) they are evicted to a temporary `retain` file in the database
   directory (not part of the database, not included in the backup, deleted at start, ADR 0040 item 1). The retention cap (item 5, D-31) counts the sum of memory and file; a write error to `retain` revokes
   snapshots and does not move the database to "read-only" (the database files are not affected). Alternatives: memory only (the D-31 cap is up to 16 GiB of RAM at 64 GB of data, depends on the machine); disk only (every
   page change while a snapshot is open is a write on the acknowledgment path).
   Visibility and retention are consistent: a reader sees only published (durable) versions (ADR 0051 item 3, ADR 0055 item 4),
   the writer's unpublished versions are not part of retention.
2. Snapshot classes and lifetime ceilings (defaults — ADR 0059): a short read (one page, search by number with a
   short history, a point lookup) — lifetime ceiling 30 s, not extended by client activity; on expiry the snapshot is revoked, the read is interrupted with a "snapshot revoked" error.
   The number stream has no general lifetime ceiling: the accepted semantics (`number-search`, "Upper bound on the number of records per number"; ADR 0035 item 6) require the full history without truncation
   and allow a break only as a storage error, and a client slower than ~20 Mbit/s with a general ceiling of 120 s would deterministically never get the worst-case history. So the stream lives as long as the client accepts
   data; it is revoked (a) on idle — the client does not accept data for 120 s (default) — and (b) at the retention cap (item 5), last of the classes. Revoking a stream is a read refusal by storage
   (without the server moving to "storage error"), the stream is cut without the final completeness marker (ADR 0035 item 6): the client sees a cut response and retries. For the API: "snapshot
   revoked" is a separate cause of an internal error, its kind — area 3.
3. The class is determined by the operation: a stream starts as a short read and becomes a stream (without a lifetime ceiling, with a 120 s idle) as soon as the number of records of the number
   exceeds a portion (the portion is an implementation matter); there is no separate parameter for the client.
4. Release: end of the request, an error, cancellation, connection drop, a handler panic — via a scope guard
   (drop guard); a snapshot leak is a defect, the number of snapshots and the retained volume are observable in tests and in the log.
5. The retained volume cap (value — ADR 0059): when exceeded, and also when free space falls to the write reserve,
   snapshots are revoked by class: backup, then short reads, then number streams (last: they are owed the full history); within a class — the oldest first. A write is neither rejected nor slowed because of snapshots: reading yields to writing,
   because CUJ1/CUJ6/CUJ9 are the main paths, and a revoked read is a repeatable error without data loss.
6. Memory per number result is bounded by a portion and does not depend on the number of records (no accumulation of a list, ADR 0035 item 6).
7. Cost (calculation `research/snapshots/retention.py`, assumptions: 16 KiB page, 8 pages per write (an average write touches ≈ 7 leaves,
   ADR 0045 item 9 (d): an estimate with margin), 40 GB (the file size does not affect retention); an upper estimate, arithmetic, not a measurement): retention ≈ W·T·128 KiB, where W is acknowledged writes per second, T is the snapshot lifetime. Short read,
   30 s: 4 MiB at W=1, 38 MiB at W=10, 375 MiB at W=100. Stream 120 s: 15 MiB / 150 MiB / 1.5 GiB. Backup (ADR 0057),
   64 MiB/s, ~10 min: 75 MiB / 745 MiB / 7.3 GiB. At W=1000 the backup hits the cap. The page size and the number of pages per write
   are determined by the format and index subtasks; the calculation is recomputed by substitution.
8. The cost of opening and closing a snapshot is O(1), independent of the database size.
9. Stream client: as long as it accepts data, the stream is not revoked by time. Retention grows as W·T·128 KiB (item 7): at W=100 the 16 GiB cap (64 GB of data) is reached in 21 min,
   so such a stream gets the full worst-case history (300 MB) at a client speed of at least ≈ 2 Mbit/s (arithmetic); below that, under high write load, the stream will be revoked, last of the classes.
   A typical history of a few records always fits.
10. **Changed by OD-6 (2026-10-28).** With 8 GB of data (OD-6) the retention cap D-31 is max(1 GiB; 25% of data) ≈ 1.9 GiB; an 8 GB backup at 64 MiB/s takes ≈ 2 min, retention ≈ W·T·128 KiB = 1.5 GiB at W = 100 records/s; the cap is reached in ≈ 2.5 min at W = 100. The 256 MiB memory budget (D-35) did not change — T, revisit on the prototype. "Memory only: up to 16 GiB at 64 GB of data" is now ≈ 1.9 GiB.

## Alternatives considered
- No revocations at all, a snapshot lives as long as the client: rejected, a hung client would hold space without a cap
  (in the SQLite WAL mode this effect is documented, F-82).
- A ceiling extended on client activity: rejected, a slow client still holds a snapshot unboundedly.
- Keep the number history in a buffer and release the snapshot at once: rejected, 300 MB per request in memory (ADR 0035 item 6).
- Do not hold a snapshot, but check the number's version at the end: rejected, during the results a write of the number would interrupt the results
  of a large history, and a stream cannot be interrupted after it starts (ADR 0035 item 6).
- Reject writes at the retention cap: rejected, snapshots must not hold back CUJ1 (ADR 0029).

## Consequences
- A number stream is cut only on client idle (120 s) or at the retention cap, last; at W=100 and a client slower than ≈ 2 Mbit/s for the maximum history; values — ADR 0059 (for the owner).
- Area 3 requirements: the "snapshot revoked" error, the stream completeness marker, agreement with the request timeout (brief §4: 10 s,
  under revision).
- For durability and format: `retain` may grow up to the cap (D-35); the WAL segment is not occupied by retention.

## Sources
- ADR 0029, 0035 item 6, 0055; `research/snapshots/retention.py`, `out-retention.txt`.

- **Changed by OD-7 (2026-10-28).** Items 5, 7, 10: the "backup" class is removed from the revocation order (the backup is without retention, ADR 0096); the backup cost in item 7 is no longer relevant; `retain` and the cap of item 5 remain for short reads and number streams.
