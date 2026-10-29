# 0097. Memory limit for modified, not yet written pages
- Status: Accepted (OD-7)
- Label: [OURS] (the need for a limit — OD-7 item 6 [USER]; value and behavior — [OURS])
- Date: 2026-10-28
- CUJ: CUJ1, CUJ14, CUJ15, X-1

## Context
The `data` file is changed only by a checkpoint (ADR 0051 item 3), so modified pages and their version chains live in memory until then. There was no budget: a 5 min threshold at W = 100 gives ≈ 2–3 GiB (`docs/reviews/rust-expert.md`, item 2). The owner: a limit is set, the value is at the discretion of the fix pass (OD-7 item 6).

## Decision
1. The limit on modified pages (with version chains) not yet written to `data` is **1 GiB** by default. Basis: 12.5% of 8 GB of data; substantially below the 2–3 GiB estimate without a limit; gives the backup (ADR 0096) headroom up to W ≈ 60 (1 GiB / (128 KiB × 128 s)). The expert's example — 512 MiB — would halve the headroom (W ≈ 30). The value is confirmed on a prototype (Q-169, T).
2. Reaching the limit is a third trigger for a checkpoint (in addition to the WAL threshold and 5 min). The writer, before the next operation (before writing to the WAL), waits for the checkpoint to end; after it the pages are clean, and the operation is executed. The operation is neither rejected nor lost; readers do not wait. The limit is "soft" by one batch.
3. The wait is before the operation starts, so a write deadline that expires during it gives "request timeout" without any data change (`server-runtime`, "Write deadline does not present unknown as not executed"). A checkpoint failure leads to the "read-only" / "storage error" states as before (ADR 0053).
4. During copying checkpoints are suspended (ADR 0096), there is nothing to wait for: reaching the limit interrupts the copy, checkpoints resume, then the writer waits per item 2.
5. The WAL threshold of ADR 0051 item 4 (256 MiB) was not changed; lowering it to ≈ 64 MiB is the expert's suggestion, Q-170 (T).

## Alternatives considered
- Reject a write at the limit: rejected, a write does not fail because of internal memory (X-1).
- Spill modified pages to a temporary file: rejected, it violates "only a checkpoint changes `data`".
- 512 MiB: see item 1. No limit: memory risk (see "Context").

## Consequences
- A new requirement of `storage-durability`; "Checkpoints bound the WAL" is changed. "Changed by OD-7" in ADR 0051, 0052.
- Application during recovery and in `npdb-fill` — Q-171 (T).

## Sources
- OD-7 item 6; `docs/reviews/rust-expert.md` item 2; ADR 0044, 0051, 0052, 0056 item 7.
