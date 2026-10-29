## 1. Review preparation

- [x] 1.1 Make sure `openspec validate add-storage-engine --strict` passes with no errors or warnings; check: the command
  exits with code 0 and prints "is valid".
- [x] 1.2 Make sure ADR 0040–0059, the delta specs, `design.md`, `facts.md` and `docs/open-questions.md` contain no temporary
  subtask identifiers and no references to drafts; check: `grep -rE "F-[D]|Q-[D]|D-[D]|Q-[F]F|F-[F]F|\bS-[1-4]\b|note[s]|draft[s]/" docs/adr openspec/changes/add-storage-engine facts.md
  docs/open-questions.md` (matches in ADR 0039, Q-101 and Q-104 are area 1 reviewer labels, not facts) finds nothing.
- [x] 1.3 Reconcile the seams between subtasks: the "Subtask reconciliation" table in `design.md` describes 15 discrepancies; check: the
  2 KiB spill threshold in all ADRs and specs, the format version only in META, `S` as a varint in the header, `freed_at` as an LSN,
  the 4 MiB cap on a WAL entry.
- [x] 1.4 Reconcile the numbering: facts F-52…F-85 in `facts.md`, questions Q-110…Q-125 and [DEFAULT] D-20…D-33 in
  `docs/open-questions.md`, rows 0040–0059 in `docs/adr/README.md`; check: every F-, Q-, D- reference from the new ADRs and
  specs points to an existing entry.

## 2. Review

- [x] 2.1 Opus review (done: R-1…R-10, 4 BLOCKERs and 6 contradictions): consistency, conformance with `user-journeys`, the ADRs and
  `docs/owner-decisions.md`, check of the seams from "Subtask reconciliation" (by sources and criteria, without history; it does not
  run programs and re-checks facts selectively — those on which the new decision rests); check: a list of findings with
  BLOCKER/MAJOR/MINOR classes is obtained.
- [ ] 2.2 Sonnet review (per the `docs/process.md` scheme, step 5 — when the whole spec is ready, not in this area; not done): feasibility in Rust (WAL and recovery, page and tree, snapshots and retention, backup, locking;
  programs may be run only for new claims about the stack that are not confirmed in `facts.md` and `research/`); check: a list
  of findings with classes is obtained.
- [x] 2.3 Process the BLOCKER/MAJOR findings (done in one fix pass per `docs/process.md`, second edition: R-1…R-10 fixed, see `design.md`, "Fix pass after the Opus review"; no repeat review); check: no BLOCKER or MAJOR
  remains, or the remainder is recorded in `docs/open-questions.md`; after the 5th iteration a BLOCKER is turned into a question to the owner.
- [x] 2.4 Move the review remainder to `docs/open-questions.md` and recount the counters; check: the section and the totals are present.

## 3. Owner decisions

- [ ] 3.1 Hand the owner the [DEFAULT] package of area 2 as one list: D-20 (page), D-21 (size target: 40 / 50 / 64 / 80 GB / no
  number), D-22 (disk reserve), D-23 (search latencies by class), D-24 (word search with no deadline and no new error), D-25 (latency growth during backup), D-26
  (blocking rebuild, ≤ 60 min), D-27 (write latency), D-28 (start time), D-29 (leaving the states), D-30 (snapshot lifetime: 30 s ceiling for a short read, no common ceiling for a stream), D-31 (retention cap), D-32 (backup speed), D-33 (backup space reserve), D-34 (`ENOSPC` and others — "read-only", clarification of Q-20), D-35 (space for earlier page versions, `retain`), D-36 (refusal when a key is above the cap), as well as question (O) Q-124; check: the list
  is handed over, the owner's answers are entered in the ADRs (status Accepted, label [USER]) and in `docs/open-questions.md`.
- [ ] 3.2 After the owner's answer, replace the numeric placeholders in `user-journeys` (CUJ1, CUJ5, CUJ15, X-1: Q-47, Q-58, Q-59, Q-104) and
  the numbers in `storage-*` with the owner's values in a separate change; check: no "not decided" placeholders remain for Q-42, Q-47, Q-58, Q-59, Q-104.

## 4. Tests and checks arising from the area (for implementation)

- [ ] 4.1 Record in the test plan: CRC-32C reference vectors; page capacity and keys from 0 to at least 16,000 bytes in length, names
  "character × 200", alternating pairs and random mixes; "all search methods give one result"; the equality "index = function of
  records"; a break between `write` and flush, a corrupted WAL byte and page byte, order inside a damaged batch, a ghost of an old epoch,
  injected file operation failures; the exit test for `E` and `G` (epoch capacity); rebuild on a label change and a failure in the middle of it;
  check: the items are present in the test plan of area 2 and of the build.
- [ ] 4.2 Prepare the 10^8 bench (the load-generation test tool, Q-53; profile P0, inserts in random order) and measurements:
  `data` size (D-21), query classes (D-23), write latency (D-27), start time (D-28), backup (D-25, D-32), rebuild
  time (D-26); check: results are recorded with a path in `research/`, the "not verified" assumptions are replaced by measurements before implementation.
- [ ] 4.3 Resolve the build questions: `rust-version` and file locking (Q-118), randomness and locking crates (Q-116), the H64 hash (Q-112);
  check: an ADR for each crate and the cargo deny result.
- [ ] 4.4 Run measurements on the reference Linux (Q-119, Q-117): `fdatasync`, `fsync`, the effect of checkpoint writes on WAL flush;
  check: the `research/durability/` scripts have been run on the target volume, the results are recorded.

## Workflow follow-up

- Status: Opus review done, fix pass done, archive open. Archive the change (`openspec archive add-storage-engine`) after the owner answers the [DEFAULT] package; the delta specs
  `storage-format`, `storage-indexes`, `storage-durability`, `storage-snapshots` are moved to `openspec/specs/`.
- Check the archived result: `openspec validate --specs` with no errors.
