# 0006. Number version and concurrent changes
- Status: Accepted
- Label: [USER] (items 1–5); [OURS] (item 6)
- Date: 2026-10-27
- CUJ: CUJ1, CUJ2, CUJ3, CUJ6, CUJ7, CUJ9, CUJ12, CUJ14, CUJ15, X-5

## Context
CUJ12 requires that an action based on stale data is not performed, and CUJ9 requires that a network service gets a
conflict with the current version. The version must be defined so that this holds, including after a failure and a
restore from backup.

## Decision
1. Versioning belongs to the number (strictly the number, not the date and not the name); any change to a number's
   records changes the version (brief §3/Operations).
2. The expected version is mandatory for rename and delete; a mismatch is a conflict with the current version in the
   response (brief §3/Operations).
3. An action based on a version read before a restore from backup or before a failure never succeeds (always a
   conflict, the client re-reads); mass conflicts right after a restore are acceptable (OD-3, Q-77). After a restore
   from backup and after a crash restart, conflicts on any numbers are acceptable (OD-4, item 5).
4. After a planned restart (clean shutdown), earlier versions remain valid (OD-4, item 5).
5. CUJ12: if a number's record was changed concurrently (another tab or the API), the action is not performed; the UI
   reports "data is stale, the list has been updated", re-reads the number history and offers to retry the action on
   current data; the name entered in the form is kept (brief §2, CUJ12). Changing another date of the same number also
   makes the version stale (a consequence of item 1: the version belongs strictly to the number).
6. [OURS] Constraints derived from items 1–2 and CUJ12 (requirement X-5 of spec `user-journeys`), not new decisions:
   (a) any change to a number's records yields a version different from all those the client has previously received for
   this number (including when the number's last record is deleted and the number appears again; after a restore from
   backup and after a failure: item 3, Q-77); (b) the change to the record and the change of the number version are one
   indivisible change.

## Clarification notes (do not change the decision text above)
- Items 3 and 6(a): clarified by ADR 0027 item 4 (for a number with records; returning to "empty" gives the initial
  version; Q-91 is closed); the narrowing is an owner decision (OD-5, D-13, D-17), ADR 0028 (items 7, 9; Accepted).
- Item 4: "clean shutdown" is defined by ADR 0028 item 10.

## Alternatives considered
- The version on the date or on the name: rejected, versioning is "strictly the number" (brief §3/Operations).
- The answer "not found" on a repeated rename after a connection drop (brief §4; accepted before the number version
  appeared): not revisited, see ADR 0019 and Q-54.
- No other alternatives were considered in the sources.

## Consequences
- The mechanics of versions across restore and failure, and the cost of the decision (storage size, write latency), are
  open question Q-78 (data model iteration); the initial value and the version mechanics are Q-72 (T).
- Open API questions: whether the version is returned in responses to successful operations and in what form (Q-32);
  where the UI and the client get the version (Q-34); the check order of "not found" and conflict (Q-33); a retry after
  a connection drop (Q-54). Open UI questions: what is re-read on a conflict from name search results (Q-36); the
  reaction when a record disappears (Q-68); the isolation level (Q-56); see `docs/open-questions.md`.

## Sources
- `docs/source/brief.md`, section 3 "Operations"; section 2 (CUJ9, CUJ12); section 4 (retrying a rename after a
  connection drop).
- Owner decision of 2026-10-27 (OD-3), Q-77: "an action based on a version read before the restore or before the
  failure never succeeds (always a conflict, the client re-reads); mass conflicts right after the restore are
  acceptable". Place of record: docs/owner-decisions.md, OD-3.
- Owner decision of 2026-10-27 (OD-4), item 5: "after a restore from backup and after a crash restart, conflicts on any
  numbers are acceptable; after a planned restart (clean shutdown), earlier versions remain valid". Place of record:
  docs/owner-decisions.md, OD-4.
- Item 6: requirement X-5 of spec `user-journeys` (a project inference from the brief and Q-77).
