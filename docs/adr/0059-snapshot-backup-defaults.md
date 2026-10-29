# 0059. Defaults: snapshot lifetimes, retention cap, backup speed, blocking rebuild
- Status: Accepted (OD-6)
- Label: [DEFAULT] (items 1, 2, 3, 5 — D-30, D-31, D-32, D-33; item 2 — also D-35; item 4 — D-26)
- Date: 2026-10-27
- CUJ: CUJ4, CUJ8, CUJ11, CUJ15, X-1

## Context
ADR 0055–0058 set the mechanism; the numbers and two features visible to clients and the operator are owner-level questions: revoking a number stream
on client idle and at the retention cap (visible in the API), a rebuild on format change lengthens start
(visible to the operator). Calculations — `research/snapshots/retention.py`, `out-retention.txt`; the assumptions (16 KiB page, 8 pages per
write) are not settled by other subtasks and are recomputed by substitution.

## Decision (defaults)
1. [D-30] The lifetime ceiling of a short read is 30 s. The number stream has no general ceiling (the accepted semantics require the full history without truncation, ADR 0035 item 6): it is revoked
   on client idle — the client does not accept data for 120 s — and at the retention cap, last of the classes (item 2). Rationale: a typical history is instant; the worst (300 MB) at W=100 writes/s
   is received by a client from ≈ 2 Mbit/s (ADR 0056 item 9). Alternatives: a general stream ceiling of 120 s (a client from 20 Mbit/s; contradicts "without truncation", rejected), 600 s (from 4 Mbit/s); an idle of 60 s or 600 s.
2. [D-31] The retained volume cap is the larger of 1 GiB and 25% of the data size (10 GiB at 40 GB), at most the free space minus
   the write reserve of 1 GiB. When exceeded, snapshots are revoked by class — backup, short reads, number streams; within a class the oldest first; writes do not suffer. The cap counts the sum of memory and the
   temporary `retain` file (the place where earlier page versions are kept — [D-35]: in memory up to a 256 MiB budget, beyond it in a temporary file; alternatives: memory only — up to 16 GiB of RAM, disk only).
   Alternatives: smaller (the backup
   is revoked more often under writes), no cap (a risk of filling the disk, rejected).
3. [D-32] The source read speed during backup is 64 MiB/s (40 GB in ~10 min). Alternatives: 256 MiB/s (2.5 min, more impact
   on search latency and disk load), 32 MiB/s (20 min, longer retention). The value is confirmed by a load test on 10^8
   (not measured in this iteration: there is no engine).
4. A rebuild of indexes on format change blocks start (ADR 0058); the time goal and the question to the owner — ADR 0049 item 5 (D-26):
   a full rebuild for 10^8 records ≈ 10–30 min, goal ≤ 60 min; frequency — only on a format change in our code. The alternative —
   a background rebuild without downtime (two code formats, duplicating writes during the build) — is more complex, deferred.
5. [D-33] The backup target's space margin is 5% above the snapshot size.

## Alternatives considered
See the decision items (each one with alternatives and cost).

## Consequences
- Goes to the owner as a package at the end of area 2. If the values change, only the numbers in `storage-snapshots` and ADR 0056–0058 change.

## Sources
- ADR 0035 item 6, 0056, 0057, 0058; `research/snapshots/out-retention.txt`; OD-1 (order: [DEFAULT]s are put to the owner).

- **Changed by OD-6 (2026-10-28).** D-30…D-33 accepted by the owner (OD-6). Recomputed for 10^7 (8 GB of data): item 2 — retention cap max(1 GiB; 25%) ≈ 1.9 GiB; item 3 — an 8 GB backup at 64 MiB/s ≈ 2 min (256 MiB/s — ≈ 30 s, 32 MiB/s — ≈ 4 min), the load test is on 10^7; item 4 — a full rebuild ≈ 1–3 min, goal ≤ 6 min.

- **Changed by OD-7 (2026-10-28).** Items 2, 3: the "backup" class is excluded from the revocation order (the retention cap is for short reads and streams); the backup speed of 64 MiB/s is kept, an 8 GB backup ≈ 2 min keeps the checkpoints suspended (ADR 0096, 0097).
