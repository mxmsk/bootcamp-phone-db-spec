# 0013. Compatibility and upgrades within v1
- Status: Accepted
- Label: [USER]
- Date: 2026-10-27
- CUJ: CUJ7, CUJ11, CUJ14, CUJ15

## Context
CUJ11: after a server update within v1, clients work unchanged. Brief §4 counted the absence of rollback among the
decisions made under the old constraints; the owner confirmed it when answering ambiguity Q-22 and clarified it in OD-4.

## Decision
1. Compatibility: the API version is in the path; within v1, fields and error codes are only added (brief
   §3/Interfaces). After an update, existing fields and codes do not change and do not disappear; new ones may appear
   (brief §2, CUJ11).
2. Within v1 the new server version opens a database of earlier v1 versions by itself, with no operator action (OD-3,
   Q-13, docs/owner-decisions.md; OD-4, item 8).
3. There is no separate format conversion command (OD-4, item 8).
4. There is no rollback of the program version; rollback is only by restoring from a backup made before the update
   (OD-3, Q-22). A backup made before the update is started by the new program; the earlier program is not required to
   open the database after the new one, and that is what "no rollback" means (OD-4, item 8).
5. The policy for the v1 → v2 transition is out of scope (brief §3/Out of scope).
6. A server update within v1 does not change search matches or order (OD-4, item 2; ADR 0010).

## Alternatives considered
- A separate operator command for format conversion, without which the server does not start and says so (package 1
  recommendation, Q-13): cancelled by OD-4, item 8; the new version within v1 opens databases of earlier v1 versions by
  itself.
- Rollback to an old version of the program (and format): absent (brief §4, confirmed by OD-3, Q-22).
- No other alternatives were considered in the sources.

## Consequences
- Open questions: the API error body and the HTTP status per code (Q-49, ADR 0019); the wording of refusals (Q-50);
  calling the API without a version in the path (Q-51); whether an existing code may be replaced by a new one in the
  same situation (Q-79); the effect of an ICU4X update on start and operator actions (Q-82); see
  `docs/open-questions.md`.

## Sources
- The storage archive of the earlier run, referenced by brief §4 (the file name is given there), is not part of the
  project; the decision is confirmed anew by OD-3 Q-22 and OD-4, item 8.
- `docs/source/brief.md`, section 3 "Interfaces", "Out of scope"; section 4 (rollback); section 2 (CUJ11).
- Owner decision of 2026-10-27 (OD-3): Q-13, Q-22. Place of record: docs/owner-decisions.md, OD-3.
- Owner decision of 2026-10-27 (OD-4), item 8: "a backup made before the update is started by the new program (the new
  version within v1 opens databases of earlier v1 versions by itself, with no command); there is no separate format
  conversion command; the earlier program is not required to open the database after the new one, and that is what 'no
  rollback' means". Place of record: docs/owner-decisions.md, OD-4.
