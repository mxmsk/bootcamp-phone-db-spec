# 0016. Backup and restore
- Status: Accepted
- Label: [USER]
- Date: 2026-10-27
- CUJ: CUJ10, CUJ14, CUJ15

## Context
CUJ15: the operator makes a backup while the server is running; the backup corresponds to one consistent moment;
restore is starting the server on the backup. Brief §5 leaves open the channel for operator commands to a running
server.

## Decision
1. A backup is made by an operator command on the server machine, without stopping the server; users and programs keep
   working without interruption (brief §3/Scale and reliability; brief §2, CUJ15).
2. The backup is a consistent snapshot, corresponding to one consistent moment (brief §3/Scale and reliability; brief
   §2, CUJ15).
3. "Without interruption" means: no refusals; a rise in latencies is acceptable within limits that will be set along
   with the latency targets (OD-3, Q-11).
4. Restore is starting the server on the backup (brief §3/Scale and reliability).
5. A backup contains only data; network tokens are not part of it, and a restore does not change tokens (OD-3, Q-10;
   OD-4, item 9; ADR 0012).
6. A backup made before an update is started by the new program (OD-4, item 8; ADR 0013). Number versions after a
   restore: ADR 0006.

## Alternatives considered
- Stopping the server for the duration of the copy: excluded, "without stopping the server" (brief §3/Scale and
  reliability).
- No other alternatives were considered in the sources.

## Consequences
- Open questions: the parameters of the backup command (Q-41); the acceptable rise in latencies during copying (Q-58,
  Q-59); which moment the backup fixes (Q-83); the operator command channel and who performs the backup (Q-60); behavior
  when the source is "read-only"/"storage error" during copying (Q-71); see `docs/open-questions.md`.

## Sources
- `docs/source/brief.md`, section 3 "Scale and reliability"; section 2 (CUJ15).
- Owner decision of 2026-10-27 (OD-3): Q-10, Q-11. Place of record: docs/owner-decisions.md, OD-3.
- Owner decision of 2026-10-27 (OD-4), items 8, 9. Place of record: docs/owner-decisions.md, OD-4.
