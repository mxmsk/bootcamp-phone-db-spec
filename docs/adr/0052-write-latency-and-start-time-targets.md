# 0052. Targets: write latency and start time at 10^8 records
- Status: Accepted (OD-6)
- Label: [DEFAULT] (item 1 — D-27; item 2 — D-28); [OURS] (item 3)
- Date: 2026-10-27
- CUJ: CUJ1, CUJ3, CUJ6, CUJ8, CUJ9, CUJ14, CUJ15, X-1

## Context
Brief section 5 leaves the target write latency to the owner (Q-58), and sections 3/4 give the scale 10^8; the start time with the check is not set
(Q-42). The mechanism is ADR 0050, 0051; write latency is determined by the WAL flush, and the start time by the WAL volume since the last
checkpoint. The figures below are a calculation from measurements on the development machine (macOS 26.6, arm64, built-in SSD; the scripts and output
are in `research/durability/`) and from assumptions marked "not verified". A Linux reference test bench was not measured.

## Decision (defaults, put to the owner)
1. [DEFAULT, D-27] Write latency (Q-58): from the server receiving the request to the acknowledgement, any write operation, a database of 10^8
   records, cold cache, local SSD, load up to 100 operations per second on the server: **p50 ≤ 20 ms, p99 ≤ 100 ms**; provided
   that the p99 of the flush to stable storage is no more than 40 ms (verified on the test bench). During a backup (ADR 0016; Q-11, Q-58):
   **p99 ≤ 250 ms**, no refusals. The load assumption: the user enters numbers by hand (brief section 3/Out of scope), and the network
   services of CUJ9 add units to tens of operations per second; 100 per second is headroom, not a measured profile.
2. [DEFAULT, D-28] Start time (Q-42), a database of 10^8 records, including the start check (ADR 0053): after a clean shutdown
   **≤ 5 s**; after a crash shutdown or from a backup **≤ 30 s**; in both cases without walking the records and without rebuilding indexes
   (a start with a rebuild on a change of the key format label — Q-82, ADR 0058 — and with a database format upgrade (ADR 0040 item 4) is not included in these numbers; the rebuild time is D-26, up to 60 min).
3. [OURS] The parameters from which the numbers of item 2 are derived: a checkpoint when the WAL from `redo_start_lsn` is ≥ 256 MiB, or after
   5 minutes with dirty pages present, or on clean shutdown (ADR 0051 item 4); recovery reads the pages needed by the
   WAL in order and in parallel (read-ahead; an implementation requirement).

## Calculation
Measurements (F-79, F-80), macOS, a file on the data volume: `F_FULLFSYNC` after appending 4 KiB — p50 4.78 ms, p99 8.30 ms, max 16.0 ms
(64 KiB: 4.12 / 10.69 / 17.2 ms; 1000 repetitions); a random cold read of 16 KiB at queue depth 1 — p50 0.108 ms, p99 0.165 ms, max 1.71 ms
(5000 reads, 6 GiB, no OS cache); sequential read 4 GB/s.

Write latency: T = T_wait + T_exec + T_flush. An operation may arrive right after the flush of the previous batch begins: it waits for its end
and then for its own, T_flush ≤ 2 × F_p99 = 2 × 10.7 = 21.4 ms. T_exec: ≤ 6 cold pages × 0.165 ms ≈ 1 ms (at the worst read of 1.71 ms — 10 ms) plus
computation ≲ 1 ms (an estimate, not measured). T_wait: there is one executor; at 100 operations/s and 2 ms per operation the load is 20 %, adding
on the order of T_exec. The result on the development machine: p99 ≈ 21 + 2 + 2 ≈ 25 ms. The 100 ms target is a ×4 margin for Linux and other disks (the condition of item 1: flush p99 ≤ 40 ms:
2 × 40 + 12 + margin = 100). Throughput: group commit shares one flush across a batch; at 2 ms of execution ≈ 500 operations/s
per executor (an estimate) — well within the 100/s assumption.

Start time after a crash: WAL ≤ 256 MiB; at 2 KiB per operation ≤ 128 thousand operations × ≤ 6 pages = 768 thousand page accesses.
By the measurement: sequentially (depth 1) 768 thousand × 0.108 ms ≈ 83 s — hence the read-ahead is parallel; the lower bound by throughput is 768 thousand × 16 KiB = 12 GiB
/ 4 GB/s ≈ 3 s; at queue depth 8–16 — on the order of 5–10 s (an estimate, not measured). The 5-minute timer at ≤ 100 operations/s limits
the WAL to 30 thousand operations (~60 MiB), 180 thousand accesses: sequentially ≈ 19 s, in parallel ≲ 3 s. The rest: reading a 256 MiB WAL ≈ 0.07 s; `dwb`
2 MiB; three durable writes at the end (the generation counter, `control`, the "end of recovery" entry) ≈ 15 ms. The total is ≲ 10 s; the 30 s target is a ×3 margin.
After a clean shutdown: open, check the index roots and the empty WAL, write `control` — ≲ 1 s; the target is 5 s.

The cost of a checkpoint (background): in 5 minutes at 100 operations/s up to 180 thousand pages (2.9 GiB) to `dwb` and the same to `data`; ~5.8 GiB, ~20 MB/s on average;
batches of 128 pages ≈ 1400, two flushes per batch ≈ 14 s of waiting. A checkpoint shares the disk with commits: an `F_FULLFSYNC` flush empties the entire drive cache, so the checkpoint
write can lengthen the WAL flush (not measured: Q-117, a measurement on the test bench; the remedy is rate-limiting the checkpoint write).

Contribution to the file size (Q-47): versions ≤ 0.5 GB (varint ≤ 5 bytes × 10^8 numbers, ADR 0042 item 5; the upper estimate at 8 bytes is 0.8 GB), the WAL up to 256 MiB plus a segment reserve, `dwb` 2 MiB,
`control` 8 KiB: ≈ 1.2–2 GB against a 40 GB target (the indexes are the formats subtask; the full size of Q-47 is not here).

## Alternatives considered
- A p99 target of 20 ms: achievable only on drives with power-loss protection (flush ≲ 5 ms); rejected as a condition not verified on Linux.
- A p99 target of 500 ms: easily met, but an acknowledgement that "hangs" is noticeable to a person in the UI (CUJ1); rejected, the calculation gives a ×4 margin even at 100 ms.
- A WAL threshold of 64 MiB (start 4 times shorter) or 1 GiB (4 times longer): 64 MiB makes checkpoints more frequent (background writing, the "Cost" paragraph); 1 GiB gives a calculated start
  of up to 40 s, above the target; 256 MiB was chosen.
- Checking the whole database on start (reading and checking the sums of all pages): 40 GB ≈ 10 s on the development machine, ≈ 80 s on a SATA SSD (500 MB/s, assumption), ≈ 270 s on an HDD (150 MB/s, assumption),
  and grows with the database; contradicts the 30 s target and the owner decision (ADR 0014 item 5: a check of the structure and the latest changes). Rejected.

## Consequences
- Closes Q-58 and Q-42 as defaults; goes to the owner in a package at the end of the area. The scenarios CUJ1, CUJ14, CUJ15, X-1 get numbers after the owner's answer.
- Q-104 (scenarios with placeholders) is closed by substituting these numbers (the `storage-durability` capability, the requirements "Target write latency", "Start time").
- Acceptance test: a Linux and macOS test bench with a database of 10^8 records (the way to fill it is Q-53), `research/durability/fsync-bench.py` on the target volume; the "not verified" assumptions are replaced by measurements before implementation.

## Sources
- `docs/source/brief.md`, section 5 (acknowledgement, target write latency); ADR 0014, 0016, 0017 (Q-42, Q-47, Q-58 in `docs/open-questions.md`).
- `research/durability/fsync-bench.py`, `fsync-bench-macos.out`; `read-bench.py`, `read-bench-macos.out` (F-79, F-80).

- **Changed by OD-6 (2026-10-28).** D-27 (write latency) and D-28 (start) — the same numbers, at 10^7 (OD-6); D-28 does not depend on the number of records (start without a walk). Contribution to the file size: versions ≤ 0.05 GB (5 bytes × 10^7), the rest (the WAL up to 256 MiB with a reserve, `dwb` 2 MiB, `control` 8 KiB) does not depend on the number of records; a full check of the whole database on start: 8 GB — ≈ 2 s on the development machine. A rebuild on a label change — up to 6 min (D-26). Accepted by the owner (OD-6).

- **Changed by OD-7 (2026-10-28).** Item 3: checkpoints are also triggered by the cap on modified pages (ADR 0097); the writer waits for the checkpoint to finish before the next operation, there is no refusal.
