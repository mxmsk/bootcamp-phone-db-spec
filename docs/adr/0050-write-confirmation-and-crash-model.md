# 0050. Write acknowledgement and the crash model: what "flush to stable storage" means
- Status: Accepted
- Label: [OURS] (derived from the owner decision ADR 0014 items 1–3, Q-20; Q-57 is a consequence, not a new choice)
- Date: 2026-10-27
- CUJ: CUJ1, CUJ3, CUJ6, CUJ9, CUJ14, X-1; item 1.6 of the original wording ("store reliably")

## Context
Brief section 5 leaves open when a write is considered acknowledged (Q-57). The owner has already decided (ADR 0014 [USER], items 1–3):
an acknowledged operation survives a process crash, an OS failure and a power loss, provided the disk honestly performs a flush to stable
storage; rename and delete likewise (Q-16). From this the moment of acknowledgement is derived unambiguously: it cannot be
earlier than the moment the information about the operation reached the medium. Here we fix what exactly counts as "reached the medium" on
macOS and Linux (the development and target platforms), and what the server does when a flush fails.

## Decision
1. [OURS] The success of a write operation (add, rename, delete) is returned to the client only after the WAL
   entry of this operation (ADR 0051) has been successfully flushed to stable storage. A response earlier than this moment is forbidden, including
   for an identical retry of an add and a rename to the same name ("silent success"): it confirms that the record exists, so
   (like the version and name-occupancy checks) the executor checks the operation against its latest state, including the unflushed: otherwise a retry would insert the record a second time, and two
   operations on one version would both pass. A response whose outcome depends on an unflushed operation ("unchanged", "version conflict", "name conflict", "not found") is returned
   after the flush of the batch containing this operation, and if it fails, with the same refusal (ADR 0051 items 3, 9). For readers, item 6 applies.
2. [OURS] "Successfully flushed" means: Linux — a return from `fdatasync` (or `fsync`) on the WAL file without an error; the WAL
   lies in pre-created zero-filled segments of constant size, so `fdatasync` does not require flushing the
   size metadata (F-66); macOS — a return from `fcntl(fd, F_FULLFSYNC)` without an error. A plain `fsync` on macOS does not count as
   an acknowledgement: it only hands the data to the drive, and the drive is free to write it later and out of order (F-68, F-69);
   a measurement on the development machine — `fsync` 0.02 ms versus 4.8 ms for `F_FULLFSYNC` (median, 4 KiB; F-79), that is,
   `fsync` on macOS flushed nothing. Rust `File::sync_data`/`sync_all` on Apple already call `F_FULLFSYNC` (F-70),
   on other platforms `sync_all` is `fsync` (F-70); the server uses these methods and does not call `libc::fsync` directly.
3. [OURS] Creating a file (a WAL segment, the data file, the control file) and renaming are durable only after
   flushing the file itself and flushing the directory it lies in (Linux: "calling `fsync()` does not ensure that the entry in the
   directory containing the file has also reached disk; for that an explicit `fsync()` on a file descriptor for the directory is also needed", F-65). A WAL segment is created, filled with zeros and
   flushed together with the directory before the first operation is written into it.
4. [OURS] Group commit is allowed: several concurrent operations are written to the WAL with one `write` and
   acknowledged by one flush; each gets its response after the success of that flush. If the flush fails, all operations of the batch get the response "storage
   error" (item 5; ADR 0053).
5. [OURS] A failed flush (any `fsync`/`fdatasync`/`F_FULLFSYNC` error) is not retried on the same file: after
   an error the kernel may mark the pages clean, and a repeated `fsync` will report success although the data was not written (F-71, F-72).
   The server goes into the "storage error" state (ADR 0053), and the operation is not acknowledged; what is on disk is decided
   only by recovery on the next start, by checksums.
6. [OURS] Data that has not yet reached the medium is not shown to readers: a change is published for search after the WAL
   flush (ADR 0029 item 4 is thereby satisfied: a search begun after the response to the client sees the operation; no search
   shows what may disappear on a failure).
7. [OURS] The boundary of the guarantee is ADR 0014 item 3 unchanged: a disk that reports a successful flush without having done it, and media
   corruption, are outside the guarantee. Also outside the guarantee: file systems without flush-to-stable-storage support and network
   file systems (not verified).

## Alternatives considered
- Acknowledging before the flush, flushing the WAL periodically (deferred durability): faster by an order of magnitude at p50 (no
  4–10 ms wait), but an acknowledged operation can vanish — a direct violation of ADR 0014 item 1 (CUJ14). Rejected.
- Acknowledging after flushing the data pages (not the WAL): each operation changes up to dozens of pages (ADR 0051, estimate),
  flushing random pages and corruption on a torn page write — rejected in favor of the WAL.
- On macOS `fsync` or `F_BARRIERFSYNC` instead of `F_FULLFSYNC`: `fsync` does not guarantee (F-68); `F_BARRIERFSYNC` is not
  described in the manual page that was checked (not verified, F-69) — not used.
- Retrying `fsync` after an error (as a "temporary failure"): rejected, F-71, F-72.
- Opening the WAL with `O_DSYNC`/`O_SYNC` instead of an explicit flush: on macOS the equivalence with `F_FULLFSYNC` is not verified; an explicit call
  makes the acknowledgement point visible in code and tests (substituted in a test by flush-failure injection). Not chosen.

## Consequences
- Q-57 is closed: acknowledgement is after a successful WAL flush (items 1–2); the version and the record are in the same WAL entry
  (ADR 0051).
- Cost: each batch of acknowledgements waits for one flush; on the development machine `F_FULLFSYNC` p50 is 4.8 ms, p99 8.3–10.7 ms
  (F-79). The latency target is ADR 0052. On Linux the flush was not measured (not verified): measurement with the script
  `research/durability/fsync-bench.py` on a reference test bench is a task before implementation.
- Test: an injectable substitution "flush failed / flush not performed" (failure injection) and a kill test: the process is killed
  between `write` and the WAL flush (nothing is acknowledged, the state is "all or nothing").

## Sources
- ADR 0014 items 1–3, ADR 0029 item 4; `docs/owner-decisions.md`, OD-3 (Q-16, Q-20).
- Verified facts F-64…F-72, F-79 (`facts.md`); the measurement `research/durability/fsync-bench.py`
  (output `fsync-bench-macos.out`).
