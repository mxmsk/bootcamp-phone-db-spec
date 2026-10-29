# 0014. Storage reliability and behavior on failures
- Status: Accepted
- Label: [USER]
- Date: 2026-10-27
- CUJ: CUJ13, CUJ14, CUJ16, X-3; items 1.1, 1.6 of the original wording ("a good database", "store many records reliably")

## Context
CUJ14: what is saved and acknowledged survives a failure, and corruption does not lead to silent loss. CUJ16: the
server cannot perform a change. Brief §5 leaves unresolved the moment of acknowledgment, the isolation level and a
number of targets.

## Decision
1. An acknowledged save survives a process crash or a power loss; a save that did not manage to get acknowledged is,
   after a restart, either fully present or fully absent (brief §2, CUJ14).
2. The guarantee covers renames and deletes: what was deleted does not "come back to life" (OD-3, Q-16).
3. Protection against failures: a process crash, an OS failure, a power loss, provided the disk honestly performs the
   flush to stable storage. A "lying" disk and media corruption are outside the guarantee; a disk write error is a
   "storage error" (OD-3, Q-20).
4. On data corruption the database does not open (refusal, no silent loss) (brief §3/Scale and reliability); the server
   does not start and tells the operator (brief §2, CUJ14).
5. At start it is checked that the database opens consistent (structure and latest changes); corruption of the rest of
   the data is detected when it is read (checksums) and leads to the "storage error" state. Detection of a "lying" disk
   is not promised (OD-4, item 3; corrects OD-3, Q-20).
6. Corruption found on a running server: the "storage error" state; changes are rejected (CUJ16), search works as long
   as it can; the operator gets a message (OD-3, Q-12).
7. "Search works as long as it can": in "read-only" mode all searches work; on a storage error, those that have enough
   readable data, with no guarantees (OD-3, Q-14).
8. A "read-only" mode on operator command is not needed; the state arises only from external causes (OD-3, Q-17).
9. The operator restores a corrupted database from a backup (brief §2, CUJ14; ADR 0016).
10. **Changed by OD-6 (2026-10-28).** The scale is 10^7 records (ADR 0017); the numbers and estimates for 10^8 in this
    ADR are calculation history, the current values for 10^7 are in ADR 0044, 0049, 0052, 0058, 0059; non-linear
    estimates are recalculated on a prototype (T, `docs/open-questions.md`).

## Alternatives considered
- Detecting a "lying" disk (the wording of OD-3, Q-20: "but they are detected (the database does not open)"): not
  promised (OD-4, item 3). Media corruption is still outside the durability guarantee, but is detected by the check at
  start or on read (item 5).
- A "read-only" mode on operator command: rejected as a new function outside the brief (OD-3, Q-17).
- No other alternatives were considered in the sources.

## Consequences
- The boundary of the guarantee: a "lying" disk (loss of an acknowledged write without corruption) is outside the
  guarantee, detection is not promised (OD-4, item 3). The CUJ14 wording "data is not silently lost" holds within the
  accepted check (a consequence of item 5; the "Limits of the guarantee" scenario in CUJ14).
- Number versions after a failure: ADR 0006.
- Open questions: the scope of the check at start (Q-43) and the start time at 10^8 records (Q-42); when a write counts
  as acknowledged (Q-57); the isolation level (Q-56); the sources of and exit from the "read-only"/"storage error"
  states (Q-71); API codes (Q-44); operator messages (Q-66); UI texts (Q-65); see `docs/open-questions.md`.

## Sources
- `docs/source/brief.md`, section 3 "Scale and reliability"; section 2 (CUJ14, CUJ16); section 5 (acknowledgment,
  isolation).
- Owner decision of 2026-10-27 (OD-3): Q-12, Q-14, Q-16, Q-17, Q-20. Place of record: docs/owner-decisions.md, OD-3.
- Owner decision of 2026-10-27 (OD-4), item 3: "at start it is checked that the database opens consistent (structure and
  latest changes); corruption of the rest of the data is detected when it is read (checksums) and leads to the 'storage
  error' state; detection of a 'lying' disk is not promised". Place of record: docs/owner-decisions.md, OD-4.
