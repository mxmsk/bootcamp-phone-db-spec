# 0096. Backup without version retention
- Status: Accepted (OD-7)
- Label: [OURS] (owner decision OD-7 item 2 — [USER]; details — [OURS])
- Date: 2026-10-28
- CUJ: CUJ15, CUJ14, X-2, X-5

## Context
ADR 0057 made the backup from a snapshot with retention of earlier page versions. An 8 GB backup at 64 MiB/s takes ≈ 2 min; retention ≈ W·T·128 KiB = 1.5 GiB at W = 100 against a cap of ≈ 1.9 GiB (ADR 0056 item 10): a backup under load is on the verge of revocation. The Rust expert (`docs/reviews/rust-expert.md`, item 4) proposed a backup without retention; the owner accepted it (OD-7 item 2).

## Decision
1. Copying does not open a snapshot. The command suspends checkpoints, waits for the one in progress, fixes the number X — the LSN of the last acknowledged operation — and reads `redo_start_lsn` from `control`. While checkpoints are suspended, `data` does not change, WAL segments are not reused, and the WAL after `redo_start_lsn` grows; the thresholds of 256 MiB and 5 min do not apply during this time.
2. What is copied: `data` as is (pages are verified by CRC, ADR 0057 item 4) and the WAL from `redo_start_lsn` to X. The backup's `control` is the source's `control` at the moment of the pause with `run_state = backup` and `backup_end_lsn = X`; `dwb` without occupied slots, `LOCK`. After copying, checkpoints resume.
3. A start from backup is the same recovery code (ADR 0051), it applies the WAL up to `backup_end_lsn`, then "end of recovery", the generation change (ADR 0057 item 9). The backup point is the same: all operations acknowledged before the command was received are included, those after X are not (ADR 0057 item 2). CUJ15, the CRC check and `backup.desc` are preserved.
4. The modified-page limit (ADR 0097) during copying interrupts the copy (code 32, «копия не создана: слишком высокая нагрузка записи, повторите позже» ("backup not created: write load too high, try again later")), checkpoints resume; the writer is not rejected as a result. Estimate: 1 GiB is reached in ≈ 128 s at W ≈ 60 (the arithmetic of ADR 0056 item 7, an upper estimate ignoring repeated modification of pages; in practice higher).
5. Retention remains only for short reads and the number stream: the "backup" class is removed from the revocation order (short reads, then streams). The `retain` file and the cap of ADR 0059 item 2 are kept: a number stream lives as long as the client accepts data, and at W = 100 over 120 s it retains ≈ 1.5 GiB (ADR 0056 item 7); removing `retain` is the question Q-168 (T).
6. Not prototyped.

## Alternatives considered
- The previous backup from a snapshot with retention (ADR 0057): kept in history; rejected by the owner (OD-7): on the verge of revocation under load, it keeps `retain` and a revocation class for a single consumer.
- A fuzzy backup without pausing checkpoints: the data may contain pages newer than X, rollback is impossible; rejected.
- Not suspending checkpoints, but copying the whole WAL together with `dwb`: it requires handling torn pages on the backup and an undefined moment; rejected.

## Consequences
- "Changed by OD-7" in ADR 0056 (items 5, 7), 0057 (items 1–3, 11–13), 0059 (items 2, 3), 0083 (item 7).
- The backup size is `data` plus the WAL up to X (space check, ADR 0057 item 6).
- The start time from backup includes applying the WAL up to X (≤ the WAL threshold plus growth during copying).

## Sources
- OD-7 item 2; `docs/reviews/rust-expert.md` item 4; ADR 0016, 0051, 0056, 0057.
