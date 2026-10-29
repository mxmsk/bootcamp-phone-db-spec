# 0027. Number version: when it changes, the initial version, non-repetition
- Status: Accepted
- Label: [OURS]
- Date: 2026-10-27
- CUJ: CUJ1, CUJ3, CUJ6, CUJ12, X-5

## Context
ADR 0006 fixes: the version belongs to the number, any change to records changes the version, the expected version is
mandatory, the version is never repeated (including after all records are deleted and one is added again), and the
record and the version change are indivisible. Not defined: the initial value, the version of a number with no records
and the exact rule for "when it changes" (Q-72, level (T)).

## Decision
1. Within one version generation, the version changes if and only if the number's records change: creating a record,
   replacing the name with a different one, deleting a record. An identical repeat, rename to the same name, errors,
   reads, a planned restart and operations on other numbers do not change the version. A version generation begins at the
   first start of the database and changes at a crash restart and a start from backup (ADR 0028); on a generation change
   the versions of all numbers change without any change to records; this is the only exception to "if and only if".
2. A number with no records has a single initial version of the current generation, common to all numbers and not
   dependent on planned restarts; on a generation change it is replaced by a new one, different from any issued before
   (within the bounds of ADR 0028 item 9: an owner decision, OD-5, D-17; otherwise an action on the initial version of an
   empty number, read before the event, would bypass "always a conflict", ADR 0028). A number with records never has the
   initial version. A number whose records were all deleted is indistinguishable from an unknown one.
3. The version of a number with records within a generation is not equal to a version issued to this number earlier,
   including after all records are deleted and one is added again (deterministically). Between generations the boundary
   of the guarantee is per ADR 0028 item 9 (an owner decision, OD-5, D-17: deterministic when the server settings are
   preserved, otherwise with a collision probability of at most 2^-64 per event); on files copied other than by the
   command, a repeat is possible (ADR 0028 item 7, D-13). Consequence: an action by a version obtained before any change
   to the number's records does not change data, with these caveats.
4. A clarification of ADR 0006 item 6(a) and requirement X-5 (not a cancellation): within a generation the initial
   version may "repeat" for a number that went through "empty, present, empty" (the last record deleted). Literally, X-5
   ("any change to records yields a version different from all those previously obtained by the client") does not hold
   here; the same X-5 already contains the scenario "A number with no records (it has the initial version)". The
   clarification: the X-5 guarantee is for a number with records, and returning to "empty" gives the initial version.
   Reason: otherwise data about all numbers that ever existed is needed (unbounded growth, alternatives below). It is
   harmless: the target record does not exist, an action by the initial version ends with "not found" (ADR 0025), and data
   does not change; an action by the version of a number with records obtained before the delete does not succeed (item
   3).
5. The version is opaque: clients compare it only for equality; no "newer/older" order is promised.
6. The expected version is mandatory on rename and delete, and not required on add (OD-3; brief §3/Operations).

## Alternatives considered
- An initial version that is not common but a separate counter per number that ever existed (including after all records
  are deleted): satisfies X-5 literally, but requires storing information about deleted numbers forever (unbounded
  growth); the harmlessness of the single repeat (item 4) makes the cost unjustified.
- The version as a hash of the number's record content: rejected. A return to earlier content (deleted and added the
  same) would repeat the version, and a rename "to the same" would be indistinguishable from no change for someone else's
  client.
- The version on the date or the name: rejected by the owner's decision (ADR 0006).

## Consequences
- Q-72 is closed: the initial version is one per generation; the properties are defined. How to achieve item 3 without
  storing data about deleted numbers is a storage task (ADR 0028, the "Requirements for other areas" section of
  `design.md`); the ordinal part of the version is taken from a monotonic counter common to all numbers of the generation
  (a requirement for storage, ADR 0028 item 6(f)); a per-number counter does not work: after all records are deleted and
  one is added again it would repeat a version within the generation.
- The clarification of item 4 is made: the wording of X-5 was changed in change `add-data-model` (delta-spec
  `user-journeys`), ADR 0006 item 6(a) has a clarification note; Q-91 is closed.

## Sources
- `docs/source/brief.md`, section 3 "Operations"; section 2 (CUJ9, CUJ12).
- ADR 0006 (items 1, 2, 6); requirement X-5 of spec `user-journeys` (`openspec/specs/user-journeys/spec.md`);
  `docs/open-questions.md`, Q-72, Q-91.
