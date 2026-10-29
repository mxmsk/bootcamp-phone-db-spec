# 0028. Versions after a crash restart and a restore: requirements for storage
- Status: Accepted
- Label: [USER] (restatement: items 1, 4, the first sentence of item 2); [OURS] (items 3, 5, 6, 8, 10); [USER] (OD-5:
  the second part of item 2, item 7: D-13, item 9: D-17)
- Date: 2026-10-27
- CUJ: CUJ14, CUJ15, X-5

## Context
The owner decided (OD-3, Q-77; OD-4, item 5): an action by a version read before a restore or a crash restart never
succeeds; after a planned restart the versions are valid. For the backup itself (the data directory), a literal "never"
across generations is unattainable: the archive is fixed, and starting it again leaves no trace in it of the earlier
start. It is attainable if there is state outside the backup: the server has it, namely the settings, next to which the
network tokens are stored; they are not part of the backup and are moved by the operator when relocating (OD-4, item 9).
Item 9 goes through the options with their costs; the choice was confirmed by the owner (OD-5, D-17). How to ensure this
and the cost is Q-78 (storage). Recorded here are the observable requirements and what they impose on storage and
administration; the mechanics are not here.

## Decision
1. [USER, OD-4 item 5] After a planned restart all issued versions are valid.
2. [USER, OD-3 Q-77; OD-4 item 5] After a crash restart and after a start from backup, any rename or delete by a version
   issued before that event ends in "version conflict" for any number, including a number with no records (both before
   and after the event): the initial version of the earlier generation does not equal the initial version of the new one
   (ADR 0027 item 2). This applies also to a version stored in the backup itself: the client could have read it before
   the restore. [USER, OD-5, D-17] The word "never" is clarified by item 9: the guarantee is limited to what is described
   there.
3. Versions are not repeated through lost changes: a version issued to an operation that did not survive a failure or is
   absent from the backup is not issued to another change of the same number after the event, within the bounds of the
   guarantee of item 9 (the bound is per item 9, owner decision OD-5, D-17); within a generation a repeat is excluded
   deterministically.
4. [USER, OD-3 Q-77] Mass conflicts after the event are acceptable.
5. A backup is a backup created by the backup command (CUJ15, ADR 0016); it carries a "backup" marker that stays in it
   until it is started. "Start from backup" is starting the server on such a backup. A version generation is the period
   between the events "crash restart" and "start from backup"; a planned restart does not change the generation.
6. Requirements for storage: (a) the number's version is stored with the records and changes in the same indivisible
   write to the medium; (b) storage distinguishes a clean shutdown from a crash shutdown and from a start from backup (by
   state at start; the definition is item 10); (c) after a crash restart and a start from backup, the versions of numbers
   with records and the initial version take values that do not coincide with those issued earlier (including those
   issued to a lost branch of changes and to another start from the same backup archive), within the bounds of item 9 (the
   "backup" marker is cleared only in the database started on it, not in the archive); within a generation a repeat is
   excluded deterministically; (d) re-evaluating 10^8 numbers at start is not acceptable as the only way (start time,
   Q-42): the method is up to storage; (e) the cost (version size, an extra write at start, write latency) is assessed by
   storage (Q-47, Q-58); (f) the ordinal part of the version is taken from a counter common to all numbers of the
   generation, not from a per-number counter: a per-number counter, after all records are deleted and one is added again,
   would issue an earlier version within the generation; at start the counter is restored to no lower than the maximum of
   the stored versions.
7. [USER, OD-5, D-13] Files copied other than by the backup command (for example, the directory of a stopped database)
   are not considered a backup; restoring from them is not supported, and starting them behaves according to the state at
   start: after a clean shutdown, as a planned restart, otherwise as a crash restart. Reason: at start the server sees only
   files, and the directory of a stopped database after a clean shutdown is byte-for-byte indistinguishable from the
   original. Therefore on such files the guarantee "the version is not repeated" (X-5) does not hold: if the original
   database continued to work after the copy, the versions of the two branches may coincide, and a stale change by a
   version from the other branch may succeed. The exception is recorded in X-5 and in the requirement "The version is not
   repeated". Storage is recommended (not required) to tie the clean-shutdown marker to the identity of the medium and
   directory, so that part of such copies goes as a crash start; this does not guarantee completeness (not verified). A
   requirement for area 4: document that restore is only from a backup created by the command.
8. A possible method (not mandatory): version = the triple "generation counter, epoch, ordinal number"; the generation
   counter is stored outside the backup, next to the server settings (item 9, option (c)); the epoch is chosen randomly at
   a crash start and a start from backup; both values are durably written before the first operations are accepted; the
   ordinal number comes from the common counter of the generation (item 6(f)). The epoch is from the OS randomness source,
   at least 128 bits: the probability that a new value coincides with one of n earlier ones is at most n/2¹²⁸, that is, at
   most 2^-64 for n up to 2^64 (an elementary estimate). The requirement for storage under this method: the counter and the
   epoch are durably written before the first operations are accepted; unavailability of the counter's location or of the
   randomness source is a database state failure (operations are not performed, CUJ16; the kind of refusal is area 3). This
   is a mechanism, not observable behavior: it is not part of capability `write-operations`, which has only the bounds of
   the guarantee (item 9). The randomness source and the counter storage must be substitutable in tests (item 9, tests).
9. [USER, OD-5, D-17] The guarantee between generations. The owner's decision (OD-3, Q-77): "never succeeds". Within a
   generation, deterministically (item 6(f)). Between generations (a repeated start from one archive, lost changes, a
   crash restart) the options were considered; the owner chose (c) (OD-5, D-17).
   (a) A probabilistic bound: an epoch of at least 128 bits from the OS at every abnormal start; the chance that a
   version coincides with one earlier issued to this number is at most 2^-64 per event. Covers all cases equally,
   including a backup on another machine. Cost: a theoretical repeat always remains, even where deterministic protection
   is possible; an OS randomness source is needed; behavior between generations is not observed by a test (the mechanism
   is checked).
   (b) Deterministically: a generation counter in a place outside the backup, next to the server settings (like the
   network tokens, OD-4, item 9); it is increased and durably written at every crash start and start from backup before
   operations are accepted; the version contains its value. Covers: a repeated start from one archive on the same
   installation, lost changes, a crash restart, deterministically. Does not cover: a backup moved to another machine
   without the settings (the counter starts again and may repeat a value used by the original installation), settings
   restored from an old copy of the settings (the counter rolled back), a backup together with copied settings brought up
   in parallel with the original installation (both increase from one value); when the counter is lost, a repeat is not
   improbable but possible. Cost: new durable external state; a write to the settings directory at every abnormal start;
   the settings must be writable; a write failure is a database state failure (CUJ16); a requirement for area 4 (document
   that the counter is not part of the backup and moves with the settings).
   (c) [chosen, OD-5] A combination: the version contains both the generation counter from (b) and the random epoch from
   (a). Deterministic between generations on an installation where the settings are preserved (a repeated start from the
   archive, lost changes, a restart); probabilistic (at most 2^-64 per event) where (b) does not work: a backup on another
   machine without the settings, settings rolled back, settings cloned together with the backup. Cost: the sum of the
   costs of (a) and (b) (a randomness source and an external counter, writing both before operations are accepted), a
   longer version (storage assesses the size, Q-47).
   (d) Keeping the set of issued epochs in the backup: does not work for a repeated start from one archive (the archive is
   fixed, the trace of the second start does not appear in it) and for lost changes (they did not survive in the backup).
   (e) An external counter service, one for all installations: deterministic always, but a new external component and a
   network dependency not provided for by the brief; unavailability of the service stops the start.
   (f) Forbid a repeated start from one archive: deterministic, but breaks CUJ15 (a backup is brought up again, for
   example after a failed restore check).
   Reason for choosing (c): closest to the owner's decision ("never"), the narrowing remains only outside the documented
   relocation procedure (OD-4, item 9: the settings are moved together with the tokens) and on a settings rollback; (b)
   alone is worse than (c) in these cases (it gives not a small probability but a possible repeat), (a) alone is less
   strict where strictness is cheaply attainable; (d)–(f) do not work or break CUJ15 and the brief. Wording for X-5 and
   ADR 0006 (a note): between generations on an installation with preserved settings the version is not repeated,
   deterministically; otherwise, with a coincidence probability of at most 2^-64 per event. Tests: the randomness source
   and the counter storage are injected; checked (with a deterministic source) are the change of the counter and epoch at
   a crash start and a start from backup, no change at a planned restart, refusal on a source or storage error, and that a
   counter rollback does not lead to coinciding versions with different epochs; the 2^-64 bound is not observed by a test,
   the capacity of the epoch (at least 128 bits) is checked.
10. [OURS] Clean and crash shutdown (the definition for ADR 0006 item 4 and item 6(b)). A clean shutdown is a shutdown by
   operator command or by the SIGTERM/SIGINT signal, in which the server: stops accepting new operations; acknowledges or
   rejects every accepted write operation (none is left in flight); durably flushes data to the medium; durably writes the
   clean-shutdown marker; exits the process with code 0. Everything else is a crash shutdown, even if no writes were lost
   and the data is intact: SIGKILL (including at idle), power loss, a panic, running out of memory (OOM), termination at
   the shutdown deadline without writing the marker, a nonzero exit code. The clean-shutdown marker is durably cleared
   before the first operation after start is accepted; at start the marker on the medium decides. If during a clean
   shutdown a write in flight did not complete by the shutdown deadline (area 4), the shutdown becomes a crash one: the
   process does not claim a clean exit. It is acceptable that a crash shutdown without losses also gives conflicts (OD-4,
   item 5: conflicts are acceptable).
11. **Changed by OD-6 (2026-10-28).** The scale is 10^7 records (ADR 0017); the numbers and estimates for 10^8 in this ADR
    are calculation history, the current values for 10^7 are in ADR 0044, 0049, 0052, 0058, 0059; non-linear estimates are
    recalculated on a prototype (T, `docs/open-questions.md`).

## Alternatives considered
- Not updating versions at a crash event and relying on the WAL: rejected. The versions of acknowledged changes may
  survive the event, but the versions of read yet lost changes do not; a client with the version of a lost change would
  pass the check on a different state.
- Storing versions in a separate log of "issued versions": rejected as redundant and contradicting the absence of a
  change log (ADR 0018).
- A counter from the backup without updating the epoch: rejected, versions from the lost branch would repeat.
- For item 9 (D-17, owner decision OD-5): (a)–(f) with their costs are recorded in item 9 itself; (c) is chosen.
- Treating as a backup any directory whose start does not continue the earlier history: rejected, it is undetectable
  (item 7).
- For item 7 (the cost of each alternative; the choice confirmed by the owner, OD-5, D-13):
  (1) [chosen] an exception to X-5: start by state at start; the cost is that a version repeat is possible on a cold copy
  when the original database continued to work, a stale change may succeed; a planned restart keeps versions.
  (2) Any start on files with no tie to the medium is a new generation: rejected as infeasible: at a clean shutdown the
  directory and the copy are indistinguishable, there is no reliable tie; tying to the medium is possible as a partial
  improvement (item 7).
  (3) Every start, including a planned one, is a new generation: versions never repeat, but planned restarts invalidate
  all versions (contradicts OD-4, item 5): rejected.
  (4) Forbidding a start on files with neither the "backup" marker nor a continuation marker: the server cannot tell the
  original from a copy, so an ordinary planned restart would be forbidden too, or a manual operator command would be
  required at every start; the operating cost is higher than the cost of a rare version repeat.
- For item 10: treating any shutdown with no lost writes as clean: rejected, "kill -9 at idle" is indistinguishable from
  a clean shutdown without a marker, and the server cannot check at start that "there were no losses".

## Consequences
- Q-77 is clarified in the "never" part by item 9 (a note in ADR 0006 by reference; the text of item 3 of ADR 0006 was not
  changed).
- Q-78 stays open for the mechanics and cost (storage); the observable policy is fixed. The requirements for storage and
  administration are in the "Requirements for other areas" section of `design.md`.
- Storage must distinguish a backup (the marker) from a shutdown and a failure; administration must issue a backup with
  the marker; under option (b)/(c) the generation counter is stored next to the settings (area 4).
- Start time and the scope of the check at start: Q-42, Q-43 (storage).

## Sources
- OD-3, Q-77; OD-4, items 5, 9 (`docs/owner-decisions.md`).
- `docs/source/brief.md`, section 2 (CUJ14, CUJ15); ADR 0006 items 3–4, 0014, 0016.
- `docs/open-questions.md`, Q-78, Q-42, Q-47, Q-58, Q-99.
