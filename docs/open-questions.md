# Open questions

The list was compiled from the analysis of CUJ1-CUJ16 and the requirements X-1...X-5 (spec `user-journeys`), dated 2026-10-27.
Questions closed in area 1 are collected in the section "Closed in area 1", those closed by the owner's answer to the area 1 package in the section "Closed (OD-5)". Resolved questions are recorded in ADRs (`adr/README.md`) and in the owner decision log (`owner-decisions.md`, OD-3, OD-4, OD-5). Only
what is unresolved is kept here. The Q-x identifiers are continuous; numbers that are absent here are closed by owner decisions and recorded in
ADRs. Primary source: `source/brief.md` ("the brief").

Categories (ADR 0001):
- (O) - an owner decision is needed, but with options and numbers from an iteration (data model, storage, API, UI). Status for all:
  Proposed - resolved in the named iteration, label [DEFAULT]. No answer is proposed at this stage.
- (T) - technical, decided by the specification with a rationale [OURS]; the owner is not needed unless it changes visible behavior.
- [DEFAULT] - a value accepted without owner confirmation; brought to the owner at the end of the work. There are no open [DEFAULT]s: D-1...D-19
  are confirmed or replaced by an owner alternative (OD-5), D-20...D-94 are accepted (OD-6, section "OD-6: final package").

If an item's iteration is given as a compound ("UI/API" and the like), it is placed in the group of the first iteration named.
Items marked [s.4] or [s.5] follow from sections 4 and 5 of the brief (`source/brief.md`; decisions made under the old constraints, and
what is unresolved).

Total after area 1 and OD-5 (history; the current count is the line "Total after OD-6" below): (O) 50; (T) 12; [DEFAULT] 0. Closed in area 1 (section at the end): 13; narrowed by area 1 subtasks: 9 (Q-29,
Q-32, Q-34, Q-54, Q-56, Q-68, Q-74, Q-76, Q-78; a consequence of ADR 0022 was added to Q-48); new questions: 25 (Q-85...Q-109; of
them Q-86, Q-89, Q-91, Q-94 and Q-96 were closed in the same area, section at the end; Q-100 was split into a technical part (T) and an
(O) part, and both are counted in (O) and (T)); new [DEFAULT]s: 18 (D-2...D-19; together with D-1 all closed in OD-5). The remainder of the area 1 review is a separate section (Q-101...Q-109, (O) Q-104...Q-106). Count after area 1: was (O) 51, (T) 6,
[DEFAULT] 1. The owner's answer OD-5 closed 8 (O) items (Q-31, Q-85, Q-87, Q-88, Q-90, Q-93, Q-97, Q-98: was 58); for Q-100 (O) a decision order was set and 19 [DEFAULT]s
(D-1...D-19); Q-100 (T) remains.

Area 2 "Storage" (change `add-storage-engine`, 2026-10-27): 22 questions closed (15 by [OURS] decisions, section "Closed in
area 2"; 7 by defaults that await the owner: Q-42, Q-47, Q-58, Q-59, Q-71 (leaving the states), Q-106, Q-125); 4 narrowed
(Q-52, Q-53, Q-104, Q-108); 16 new questions (Q-110...Q-125: 15 (T), 1 (O); Q-121 and Q-125 closed by the area 2 review fix pass); 17 new [DEFAULT]s
(D-20...D-36, section "Area 2: defaults"; D-24, D-29, D-30, D-31 corrected by the review fix pass). Count after area 2: (O) 36; (T) 20; [DEFAULT] 17 (await the owner).

Area 3 "Server and API" (change `add-server-api`, 2026-10-27): 18 questions closed (9 by [OURS] decisions, section "Closed in area 3"; 9 by defaults that
await the owner: Q-38, Q-45, Q-46, Q-49, Q-52, Q-69, Q-70, Q-79, Q-102); 3 narrowed (Q-40, Q-60, Q-66); 16 new questions (Q-126...Q-141: 14 (T), 2 (O)); 17 new [DEFAULT]s
(D-37...D-53, section "Area 3: defaults"; D-46 refines the owner's decision Q-27 (OD-3)). The Opus review fix pass closed R-1...R-10 without changing areas 1 and 2.
Count after area 3: (O) 22; (T) 32; [DEFAULT] 34 (17 of area 2 and 17 of area 3, await the owner).

Area 4 "Administration" (change `add-operator-cli`, 2026-10-27): 8 questions closed (2 by [OURS] decisions: Q-66, Q-124, section "Closed in area 4"; 6 by defaults that await the owner: Q-40, Q-53, Q-60, Q-108, Q-133, Q-136);
1 narrowed (Q-45: settings file and values); 16 new questions (Q-142...Q-157: 14 (T), 2 (O): Q-145, Q-150); 22 new [DEFAULT]s (D-54...D-75, section "Area 4: defaults"). The Opus review fix pass closed R-1...R-10 without changing areas 1-3,
except the MODIFIED requirement of `server-runtime` "Startup order" (the step "`control.lock`, `tokens`, control channel"). Count after area 4: (O) 19; (T) 43; [DEFAULT] 56 (17 of area 2, 17 of area 3, 22 of area 4, await the owner).

Area 5 "UI" (change `add-web-ui`, 2026-10-27): 13 questions closed (3 by [OURS] decisions: Q-28, Q-37, Q-39, section "Closed in area 5"; 10 by defaults that await the owner: Q-1, Q-2, Q-29, Q-35, Q-36, Q-61, Q-62, Q-63, Q-65, Q-68);
1 narrowed (Q-38: keeping the UI sign-in); 7 new questions (Q-158...Q-164: 6 (T), 1 (O): Q-160); 19 new [DEFAULT]s (D-76...D-94, section "Area 5: defaults"). The Opus review fix pass closed R-1...R-9 without changing areas 1-4, except the
MODIFIED requirement of `http-api` "API version in the path" (paths `/` and `/ui/...`: the shell on the loopback listener, 404 on the network one). Count after area 5: (O) 7; (T) 49; [DEFAULT] 75 (17 of area 2, 17 of area 3, 22 of area 4, 19 of area 5, awaited the owner until OD-6).

The OD-6 final package (change `apply-owner-decisions-od6`, 2026-10-28, `owner-decisions.md`): scale 10^7, body 32 KiB, Q-128, Q-160; the remaining [DEFAULT]s D-20...D-94 and (O) are accepted as defaults
(section "OD-6: final package"). **Total after OD-6: (O) 0 open (7 closed: Q-128 and Q-160 by a new value; Q-104, Q-124, Q-135, Q-145, Q-150 accepted as defaults; Q-100 (O) remains separately, deferred until the order test on Unicode 18, OD-5);
(T) 51 (49 + Q-165, Q-166); [DEFAULT] 0 open (75 accepted: D-20...D-94; a new value or a recount to 10^7: D-21...D-24, D-26...D-28, D-32, D-35, D-43, D-52, D-61, D-82; the other 62 unchanged).**

Final fix pass (change `final-review-fixes`, 2026-10-28, ADR 0095): 28 findings of the final review closed, Q-166 closed, Q-167 (T) added, no new (O). **Total after the final fix pass: (O) 0 open (Q-100 separately, OD-5); (T) 51 (Q-166 closed, Q-167 added: the count is the same); [DEFAULT] 0 open.**

OD-7 applied (change `apply-od7-rust-expert`, 2026-10-28, results of the Rust expert assessment, `docs/reviews/rust-expert.md`): 8 owner items, ADR 0096-0101, facts F-120...F-127. Q-163 closed; 13 (T) added: Q-168...Q-180. **Total after OD-7: (O) 0 open (Q-100 separately, OD-5); (T) 63 (51 - Q-163 + 13); [DEFAULT] 0 open.**

OD-8 applied (change `apply-od8-idempotent-add`, 2026-10-28, `owner-decisions.md`): Q-180 closed (a retry of an add without a date is "unchanged"). **Total after OD-8: (O) 0 open (Q-100 separately, OD-5); (T) 62 (63 - Q-180); [DEFAULT] 0 open.**

---

# (O) For the owner in iteration X

## Iteration: storage

**Q-53. Test tool for populating the database for 10^7 [s.4] (was 10^8, OD-6).** X-1. Not a product operation (the option the owner chose,
having rejected bulk loading); confirm with a concrete composition. ADR: 0019, 0017, 0018. Area 2: the tool is needed to verify D-21, D-23, D-27, D-28 (inserts in random order, profile P0, ADR 0044, 0049, 0052); the composition is not defined. Area 4: the tool's composition is D-61 (ADR 0079 items 4, 5; `--config` and the crash start are D-75). OD-6: accepted (D-61, D-75), scale 10^7.

## Iteration: API

**Q-40. Design of the token command.** CUJ10. Iteration: API/CLI. What identifies a token on revocation; the contents of the list (name,
permissions, issue date, last use). ADR: 0012. Area 3: "last use" is not stored (a disk write on every request is unacceptable; ADR 0065, "Consequences"); the list contents and the revocation identifier are area 4. Area 4: the revocation identifier is the client name [OURS] (ADR 0082 item 2), the list contents are D-66.

**Q-60. Operator control channel to a running server [s.5].** CUJ10, CUJ15. There is no administration through the API; the server does not
stop when a token is issued or revoked or when copying. How a command reaches the server process that owns the database, and who
performs the backup, is not decided. It is also not stated who on the server machine may run operator commands (any local
OS user or only a specific one). ADR: 0012, 0016. Area 3: only the observable is required: after the command's "done" response the next request already sees the issue or revocation of the token (D-45, ADR 0065 item 8); the channel itself remains an area 4 question. Area 4: the channel is a Unix socket [OURS] (ADR 0080), the set of persons is D-62, the location is D-63, commands with the server stopped are D-64.

**Q-66. The channel through which the server reports to the operator.** CUJ8, CUJ13, CUJ14. Iteration: API/CLI. A start refusal (no database, no
certificate, corruption) must be reported to the operator; the channel is not named (process output, exit code, log). The question is
mostly technical, but the visible part (what and where the operator sees) is for the owner. ADR: 0014, 0015. Area 3: the content of messages at startup and on refusals is defined (ADR 0074 item 4, 0053, 0058 item 4: the kind of the previous shutdown, discarded WAL bytes, the cause and share of a rebuild, the refusal cause, listener addresses, "ready"; the UI address only on standard output); the channel, format and exit codes are area 4. Area 4: stderr, a `key=value` format, the list of events and exit codes [OURS]; the channel is D-59 (ADR 0077 item 6, ADR 0078, ADR 0084).

## Iteration: UI

**Q-1. Showing the date 1876-03-07 in results.** CUJ1, CUJ2. An explicitly entered date and one substituted by the server are indistinguishable (the same
record). How results show a record with the date 1876-03-07 is not stated. ADR: 0002. Area 5: closed by default D-86 (ADR 0092 item 3).

**Q-2. Showing owners in by-number results.** CUJ2, CUJ4. The by-number search result is defined: all records of the number, date DESC,
no pagination; an operation "who owned on date D" does not exist and is not introduced. It is not stated how results distinguish past, current and
future owners (future dates are allowed) and whether that is needed. ADR: 0002, 0007. Area 5: closed by default D-87 (ADR 0092 item 4).

**Q-28. How the UI determines that a number is already known.** CUJ1, CUJ2, CUJ3. Iteration: UI/API. On save the UI must show that the
number already exists; an add only creates a record, and without a date 1876-03-07 is set. If the UI sends an add without checking and there is no record on
1876-03-07, the record is created silently and the date prompt does not appear; the mechanism is not named. Whether the CUJ1 form has
an optional date field is Q-62. ADR: 0005, 0002. Area 5: closed [OURS]: a query by number before an add without a date, the response is taken as complete only with `complete: true` (ADR 0090 items 1-2); the race is closed by the server (OD-6, Q-160; ADR 0090 item 1 replaced).

**Q-29. Composition of a results row and the look of empty results.** CUJ4, CUJ5, CUJ6, CUJ13. Narrowed by the name-search subtask: empty results are not an
error (ADR 0034). Known: a "list", a "delete" button on the left; a record may appear twice when the name changes between pages
(ADR 0032). A results record contains number, date and name (the date is needed for rename and delete, OD-5, Q-90; `name-search`). What remains: the human-readable look of a row and the look of empty results. ADR: 0007, 0008, 0011, 0034. Area 5: closed by defaults D-84, D-85 (ADR 0092 items 1-2).

**Q-35. What is re-queried after a delete.** CUJ6. "After a delete we re-query and show the updated list":
the original query, the cursor position, an empty page are not stated. ADR: 0005. Area 5: closed by default D-90 (ADR 0093 item 4).

**Q-36. What is re-read on a conflict from name-search results.** CUJ12. The number history is re-read; the state of the original
name-search results (page, row) is not stated. ADR: 0006. Area 5: closed by default D-91 (ADR 0094 item 2).

**Q-37. CUJ2/CUJ3 dialogs.** CUJ2, CUJ3, CUJ12. What the overwrite prompt shows (the current name from the conflict response?), what
stays in the form, whether one can decline to enter a date in CUJ2, how an invalid-date error is shown. ADR: 0005. Area 5: closed [OURS] (ADR 0091 items 1-4).

**Q-39. An open UI tab after a server restart.** CUJ13. The old link stops working; what is shown in an already
open tab is not stated. ADR: 0011. Area 5: closed [OURS] (ADR 0088 item 2, ADR 0086 item 1).

**Q-61. UI: what to show after saving [s.5].** CUJ1, CUJ2, CUJ3. The value is not defined. ADR: 0011, 0005. Area 5: closed by default D-83 (ADR 0091 item 5).

**Q-62. UI: date format [s.5].** CUJ1, CUJ2. The value is not defined; also whether the CUJ1 form has an optional date field.
ADR: 0002, 0011. Area 5: closed by defaults D-78 (format, ADR 0089 item 1) and D-82 (there is no date field in the CUJ1 form, ADR 0090 item 3).

**Q-63. Cancelling a delete [s.5].** CUJ6, CUJ12. Two readings are possible: (1) declining in the confirmation dialog: what the UI shows
after the decline; (2) cancelling an already executed delete (undo): a new feature that restores the record, changes the number version,
and interacts with concurrent changes (availability window, what is restored, which version the restored
record has). Which reading the owner meant and whether the second is needed is not stated. ADR: 0005, 0011. Area 5: closed by default D-89 (ADR 0093 item 3); undo is not introduced.

**Q-65. Exact UI message texts.** CUJ1, CUJ12, CUJ16. Whether the brief's messages must be verbatim; what text to show
on a failed delete («Сохранение не выполнено» (Saving failed) fits it poorly); the text «данные устарели, список обновлён» (data is stale, list refreshed); the text of the
hint on rejecting a number without "+"; what the user sees if a search fails because of a storage error and whether
a search counts as an "action". ADR: 0011, 0014. Area 5: closed by defaults D-81 (CUJ16, value errors, ADR 0088 items 5-6) and D-93 (CUJ12, dialogs, ADR 0094 item 5).

**Q-68. UI reaction when a record vanishes or the state is already reached after a conflict.** CUJ3, CUJ6, CUJ12. Narrowed by the
write-operations subtask: the server outcome is defined: on a concurrent delete of the record "version conflict" (ADR 0025, owner decision OD-5,
D-6), with a stale version and a name equal to the one entered also a conflict (OD-4, item 7); the outcome "unchanged" is not an error (ADR 0026).
What remains: what the user sees in the UI. ADR: 0005, 0006, 0025. Area 5: closed by default D-92 (ADR 0094 items 1, 3, 4).

---

# (T) Technical: decided by the specification [OURS]

**Q-76. Choice of ICU4X functions and configuration.** CUJ5. Iteration: storage. Narrowed by the name-search subtask: case folding
was verified by running it (`fold_string` is full, C+F, F-11); word boundaries are UAX #29 rules over the `icu_properties` tables, not
`WordSegmenter` (F-33, F-34; the choice was confirmed by the owner: OD-5, D-14, ADR 0030 items 3-4). What remains: choose concrete ICU4X APIs for
normalization, folding and the collator; working with the pinned feature set was verified by running it (F-30, F-43). If the
choice changes visible behavior, it either diverges from the accepted word-boundary rule (a question for the owner) or from the owner's sorting decision (ADR 0037, 0038). ADR: 0008, 0010, 0030, 0039.

**Q-92. Checking name order when the ICU4X version changes.** CUJ5, CUJ11. Iteration: storage. The total result order is the name sort key, date DESC, number (ADR 0036; there is no code-point key for names that are equal under the collator). The values of the ADR 0038 tables and the `result-ordering` scenarios were obtained on ICU4X 2.4.0 and reconciled with ICU4C 78.3 on a corpus from the scenarios (F-19, F-25; `research/ordering/`). With another version of `icu_collator` (only per the ADR 0039 item 6 procedure) re-check with the `research/ordering/` program and extend the corpus (the reference sets and corpus of ADR 0039 item 5; F-41); the behavior of the collator data on Unicode 18 characters is Q-100.
ADR: 0036, 0038, 0039.

**Q-95. Decision on a vulnerability in `icu_*` that changes order or matches.** CUJ5, CUJ11. Iteration: storage/tests. Narrowed:
the order corpus is defined (ADR 0039 item 5 (b), `research/ordering/order-corpus`, a hash snapshot in the repository; F-41) and is checked
in every release. What remains: if a vulnerability fix in `icu_*` changes order or matches, the choice between security and
stability of order belongs to the owner (a revision of OD-4 item 2): a question to the owner at the moment it arises (ADR 0039 items 6 (d), 8).
The policy for `icu_*` fixes in v1 (backport, upgrade on a corpus match, prohibition) is the owner's decision OD-5, D-18 (ADR 0039 item 6 (c): backport only).
ADR: 0039.

**Q-100 (T). Order test on characters added in Unicode 18.** CUJ5, CUJ11. Iteration: storage/tests. The `icu_collator_data` 2.4.0
data claims Unicode 18.0.0 and CLDR 49.0.0 (the crate README, F-42), but its behavior on characters added
in Unicode 18 has not been verified; a comparison with ICU4C 78.3 (Unicode 17) on such characters is impossible. A collator order test is needed on
characters from the list `research/name-search/out-uax29-diffage.txt` (558 characters, F-35), mandatory before the v1 release. The choice on a
mismatch is the (O) part of the same question (the decision order was set by the owner, OD-5; not closed). ADR: 0010, 0039 (item 10).

---

# Remainder of the area 1 review (implementation questions)

Result of the area 1 review (two reports, iteration 5, no BLOCKERs): MAJOR and MINOR findings not closed by fix passes were moved
here (`process.md` item 3). Categories: (T) is decided by the implementation or by the spec of the next area without the owner; (O) is
touching an owner decision. Computed figures: the reviewers' corrections are given in the items; the area 1 decisions were not
recomputed, and in design.md and ADR 0022 the inexact figures are marked with a reference to Q-101 and Q-103. The Opus R-3 divergences were formalized as
D-19 and resolved by the owner (OD-5, "Closed (OD-5)").

**Q-104. Scenarios with "not decided" placeholders are unverifiable (Sonnet S-3). (O).** Area: API and storage (the values are
owner decisions, brief section 5). Where in the spec: `user-journeys` (MODIFIED), CUJ1 "Write acknowledgement latency" (Q-58), CUJ5
"Name-search latency" (Q-47, Q-59), CUJ15 "No refusals, latency growth"; ADR 0017 "Consequences" (scale 10^8 with no
numeric threshold, X-1). Essence: a scenario with a placeholder cannot be turned into an automated test; the scale X-1 stays without a measurable
acceptance condition. Acceptable for an intermediate state. The reviewer's proposal: mark such scenarios as non-acceptance until
Q-47, Q-58, Q-59 are decided (a separate subsection or a tag), add to tasks.md a task "fill in the numbers before implementing
area 2", propose concrete values to the owner (for example, p99 of a write and of the first search page) in the question package of
areas 2-3 (not part of the area 1 package: the numbers depend on the storage model). ADR: 0017, 0019. Area 2: the numbers were filled into `storage-format`, `storage-indexes`, `storage-durability` as [DEFAULT] (D-21, D-23...D-28; accepted by the owner, OD-6); the placeholders in `user-journeys` are replaced after the owner's answer. OD-6: the numbers are accepted (D-21: 8 GB at 10^7; D-23...D-28), (O) closed; replacing the "not decided" placeholders in `user-journeys` is Q-166 (T).

**Q-108. "Clean shutdown": a planned restart that exceeded the deadline (Opus R-8). (T).** Area: administration (area
4). ADR 0028 item 10 makes such a restart a crash one, and versions are invalidated; this is a refinement of OD-4 item 5, visible to the operator and
clients. Exit codes and the list of signals (the `write-operations` requirement "Clean and crash shutdown") are a matter of
area 4; in `write-operations` leave only the observable "clean/crash" distinction. The consequence for the owner
was shown to the owner in the package and accepted (OD-5, D-13, D-17). ADR: 0028. Storage did its part: the marker is written as the last step, the estimate of the steps is <= 30 s (ADR 0054 item 8); what remains is the shutdown deadline and exit codes (area 4). Area 4: a deadline of 60 s and code 13 are D-58; exit codes are ADR 0077 item 6.

**Other MINORs (without a Q number).** Text fixes or decided during implementation; (R-x) is an Opus report, (S-x) a Sonnet report.
- R-5: done (OD-5): the Q-54/Q-31 placeholders in `user-journeys` (CUJ9, CUJ12, CUJ5) were replaced by references to ADR 0026 (D-7), 0037, 0038; Q-54 (representation) remains.
- R-6: status of ADR 0029: items 1-4 are both Accepted [OURS] and part of D-8; separate them (D-8 is the owner's decision, OD-5: inter-page
  consistency and the general isolation level); the status of ADR 0029 is now Accepted as a whole.
- R-7: ADR 0034 item 2 (query cap, [OURS] Accepted): refer to D-3 as the source of the rule (confirmed by the owner, OD-5).
- R-9: clarify the scenario "ICU4X code is not upgraded": a backport per ADR 0039 item 6 (b) is allowed.
- R-10: status of part of item 10 of ADR 0039 (Q-100 (O)): the decision on a mismatch belongs to the owner (OD-5); the label [USER] is item 6 (c); ADR 0028 item 9 (c) is
  an observable property, the triple (counter, epoch, number) is an example of the mechanism of item 8.
- R-11: the phrase "SHALL NOT promise stability" in `result-ordering` is a statement about the spec text; move it to
  Purpose or ADR 0039 item 2.
- R-12: the scenario "Control characters in the query" (exact search) is correct only under D-2 (confirmed by the owner, OD-5); mark the dependency.
- R-13: Q-93: show the consequence ("Её" < "Еж" < "Ёж" < "Ежи") in the owner package (done in the package).
- R-14: the proposal "Impact" does not mention the fix to ADR 0019 item 6 (the note about Q-54).
- S-8: "on any input" (ADR 0021 item 2, `records`) is wider than the check; phrase it as "on Unicode 18 data; the build test
  repeats the check".
- S-9: in `number-search` state that a stream of 2,967,164 records is a computed worst case, not a target load (D-16,
  X-1).

---

# Area 2: new questions

Found by the area 2 subtasks (format, indexes, durability, snapshots and backup) and when reconciling the seams. Categories are as above; (T) is decided by
the implementation or build without the owner. Duplicates between subtasks were merged: the question of ordinal record labels (format, indexes) is Q-110;
the absence of a real corpus and the 10^8 prototype (format, indexes, snapshots) is Q-111; the size target (format, indexes) is not a question but a single
[DEFAULT] D-21.

**Q-110 (T). Ordinal record labels instead of `OrderKey` in postings.** X-1. Area: indexes. In O and W the order key is repeated in
every posting: about 217 of 404 B per record of profile P0 (54%); labels (a monotonic u64) would give about -35% for W and would lower the worst case per
write from about 180 to about 4 KiB, at the cost of relabeling neighbors and rewriting their postings in one transaction when a gap is exhausted. The size with
labels was not recomputed (`sizing.py` has no such variant). They are needed if the prototype does not fit D-21 (especially if 40 or
50 GB is chosen) or the worst case per write is unacceptable. ADR: 0045 (alternatives), 0044 item 9, 0049. OD-7 (expert, item 13): a name sort key of 200 characters reaches 8,513 bytes (verified: ﷺ × 200), twice that after escaping, so overflow pages are mandatory; build in ordinal labels at once or prefix compression; tests on pathological names. Source: `docs/reviews/rust-expert.md`.

**Q-111 (T). No real name corpus.** X-1. Area: storage. Prefix compression (25-60%), the number of distinct words (10^7), 1.25
records per number, the name length distribution (size grows almost linearly with it: 40 characters is 112.6 GB without compression) and leaf
fill are assumptions (F-60, F-61, F-62). A prototype on a synthetic 10^8 corpus with a real word distribution (the test tool,
Q-53) must confirm the sizes (D-21), the planner constants (M1, S_max, K1) and the time of a full rebuild of O, W, D (estimate 10-30 min,
D-26; only the computation of order index keys was measured, F-84). ADR: 0044, 0048, 0049, 0058.

**Q-112 (T). H64 hash for exact search: algorithm and crate.** CUJ7. Area: build. Not chosen, not verified; stability within
v1 is needed (the algorithm is part of the key format label, ADR 0045 item 7), an ADR for the crate and cargo deny (ADR 0017 item 3). ADR: 0045, 0047.

**Q-113 (T). The upper bound of the sort key of mixed names is not proven.** CUJ1, CUJ5. Area: storage/tests. Remainder of Q-103:
storage does not require a proof (a key of any length, a refusal of a single operation only above 65,535 bytes: not a "storage error", D-36, ADR 0041 item 8, ADR 0045
item 6), but the key length when "long" characters alternate is not monotonic (398 bytes more than the maximum of the components, F-60); what remains is a
proof or a test on mixtures in every release. ADR: 0041, 0045.

**Q-114 (T). Residual worst case of word search.** CUJ5, X-1. Area: indexes. Several very frequent mutually exclusive
prefixes with `k > 1,024`: a full traversal of O of about 14 s, there is no search deadline (D-24), the query completes with a result; an optimization is possible (a word signature in the posting,
about +0.5 GB). ADR: 0048 items 6, 8, 0049.

**Q-115 (T). Page merging and minimum fill under deletes.** X-1. Area: indexes. The format sets no merge rules or
minimum fill; the fill of 0.67 (F-61) was obtained for random inserts without deletes, and under long operation with inserts and
deletes the size is not bounded by the calculation. A merge or redistribution rule is needed (ADR 0045 item 2) and a test on a mixed load.
ADR: 0041, 0044. OD-7 (expert, item 14): merge when fill drops below about 25%; for the rebuild write path see Q-177. Source: `docs/reviews/rust-expert.md`.

**Q-116 (T). Randomness source and file locking: crates or OS calls.** X-2, X-5. Area: build. For the epoch `E`: the crate
`getrandom` 0.4.3 (F-78) or direct `getrandom(2)`/`getentropy`; for `LOCK`: `File::try_lock` (Q-118) or `libc::flock`; an ADR for
each and a cargo deny check (ADR 0017 item 3); both are injectable interfaces (ADR 0054 item 10). ADR: 0054.

**Q-117 (T). Effect of checkpoint writes on WAL flush latency.** CUJ1, X-1. Area: storage/rig. `F_FULLFSYNC`
empties the whole drive cache, and a checkpoint write (about 20 MB/s on average at 100 ops/s, ADR 0052) may lengthen the WAL flush;
not measured. A measurement on the rig and a limit on the checkpoint write rate are needed. ADR: 0051 item 4, 0052. OD-7 (expert, item 11): create the next WAL segment ahead of time in the background, limit the checkpoint rate; measure p99 on the rig (macOS and Linux) before fixing the numbers. Source: `docs/reviews/rust-expert.md`.

**Q-118 (T). `rust-version` 1.88 versus `File::try_lock` (Rust 1.89).** X-2. Area: build ("Build and tests", area 1). A question for the
build, not decided: raise `rust-version` to 1.89 (1.88 is an ICU4X requirement, not a ceiling: F-2, F-51) or take the lock via
`libc::flock` with a separate ADR (ADR 0017 item 3). F-75, F-76. ADR: 0054 item 3.

**Q-119 (T). Measurements on the reference Linux and unverified OS properties.** CUJ14, X-1. Area: rig. Not verified: `fdatasync` and
`fsync` on Linux (the D-27 condition: p99 flush <= 40 ms), the queue depth during recovery (D-28: 5-10 s is an estimate), `sync_data` = `fdatasync`
in Rust on Linux (F-70), `F_FULLFSYNC` on a macOS directory, `O_CLOEXEC` by default for `File::open`, `ENOSPC` on deferred allocation on
copy-on-write file systems, the OS page size on x86-64 (F-63). Scripts are in `research/durability/`; measurements before
implementation. ADR: 0050, 0052, 0053, 0054.

**Q-120 (T). Start on a non-writable database after a crash shutdown.** CUJ14, CUJ16. Area: storage. Recovery
requires writing, and the server does not start (ADR 0053 item 5). The option "apply the WAL only in memory" was rejected as a second WAL replay
mode; return to it if the owner wants to read such a database (for example, from a read-only medium). ADR: 0053.

**Q-122 (T). Throughput of the single writer.** CUJ1, X-1. Area: implementation/rig. A single writer with batch commit
(ADR 0055): the estimate of about 500 ops/s at 2 ms per operation against the assumption of 100 ops/s (D-27) is not measured, there is no engine; if insufficient, return to
sharding (rejected in ADR 0055). ADR: 0052, 0055.

**Q-123 (T). The boundary "index error": rebuild or "storage error".** CUJ14, CUJ16. Area: storage (integrity check).
ADR 0039 item 3 names an index error among the causes of a rebuild; a rebuild from names is allowed only if the R data is intact (ADR 0058
item 8). The check "index = a function of R" is possible (ADR 0045 item 8), but who runs it and when, and at what moment the state becomes "storage
error", is not decided. ADR: 0014, 0039, 0045, 0053, 0058.

**Q-124 (O). Does the operator need a separate backup check without starting it.** CUJ15. Area: administration (area 4). A backup
is checked page by page at creation (ADR 0057 item 4), and archive corruption after creation is detected on read and at start.
The operation "check a backup without starting" is a new operation outside the brief and was not introduced. ADR: 0057. Area 4: not introduced (ADR 0079 item 7). OD-6: accepted as the default.

---

# Area 2: defaults

The [DEFAULT]s of the area 2 subtasks, numbering continued after D-1...D-19. All accepted by the owner (OD-6, 2026-10-28; ADR: Accepted); D-21, D-22, D-23, D-24, D-26, D-27, D-28, D-32, D-35 have a new value or a recount to 10^7.
The old wording "defaults and alternatives to the owner as a package" was removed. In the ADRs each item carries its D number.

- **D-20. Page size is 16 KiB.** Value: 16,384 bytes, constant in v1, recorded in the file header. Alternatives: 8 KiB
  (spill threshold 1 KiB, twice as many pages, the same tree height, file size at most 6% larger); 32 KiB (a key of 8.5 KiB
  fits, but reads and writes are 32 KiB each). Rationale: the figure of brief section 4; the "proven key bound" was replaced by spill to overflow;
  without a measurement on the target medium the choice cannot be settled by calculation. ADR 0041 item 1 (Q-48). Accepted by the owner, OD-6.
- **D-21. File size target at 10^7 records is no more than 8 GB (OD-6; was: 10^8 records, 64 GB)** (`data`, without `wal/`, `dwb` and backups; reference profile P0: name of 22
  characters, 2.5 words, 1.25 records per number, random inserts). The owner's value (OD-6): the calculation without compression is about 6.1 GB at fill 0.67 (with 25-45% compression 5.3-4.6; recomputed proportionally from 61.2 / 52.8-46.2 GB at 10^8). Historical alternatives at 10^8 (disk for a rebuild and
  backup / backup time at 64 MiB/s): 40 GB (brief section 4) requires dense loading and compression >= 25% or ordinal labels (Q-110), 112 GB /
  10 min; 50 GB needs compression >= 33.6%, not measured, 140 GB / 12.4 min; 64 GB is 179 GB / 15.9 min; 80 GB leaves headroom for names of about 30
  characters, 224 GB / 19.9 min; no number leaves X-1 without a measurable threshold. The calculations of the two subtasks and their divergence are in ADR 0044 item 6. ADR
  0044 item 6, 0049 item 1 (Q-47, Q-105, Q-104). Accepted by the owner, OD-6.
- **D-22. Disk headroom for a rebuild and a backup is an estimate for the operator:** up to 1.8 x the file size plus the backup (at 8 GB at 10^7 up to about 22 GB; was at 64 GB at 10^8 up to about 179 GB);
  not an acceptance requirement. ADR 0044 item 10. Accepted by the owner, OD-6.
- **D-23. Search latency targets by class are the same numbers at 10^7 records (OD-6)** (p99, 10^7 records, NVMe, >= 8 cores, cache >= 256 MiB): a number of up to 100 records <= 10 ms;
  an exact-search page and a search with a word of <= 10^4 postings <= 20 ms; a single prefix of any frequency and C <= 10^7 <= 50 ms; heavy
  intersections <= 2 s. Alternatives: a general "p99 <= 10 ms" (unachievable for frequent words); targets only in page reads; for HDD no targets
  are set. ADR 0049 item 2 (Q-47, Q-59). Accepted by the owner, OD-6.
- **D-24. Word search: the number of words is not limited, there is no search deadline and no new error, a valid query always completes with a result.**
  The first edition introduced a 5 s deadline with an error; that is new visible behavior of area 1 (`name-search`, X-1 "no refusals caused by volume"), corrected by the review fix pass.
  The work is bounded by a traversal of O (up to about 1.4 s at 10^7; was about 14 s at 10^8, proportionally, OD-6). Alternatives: a deadline with the error "search deadline exhausted" (MODIFIED `name-search`, an owner decision); a cap of 32 words (changes ADR 0022
  [USER]). ADR 0049 item 3 (Q-106). Accepted by the owner, OD-6.
- **D-25. Search latency growth during a backup is up to 2 x the D-23 target, no refusals.** Write latency during a backup is D-27. ADR 0049 item 4
  (Q-59, Q-11). Accepted by the owner, OD-6.
- **D-26. An index rebuild on a format change blocks start; the target is no longer than 6 minutes at 10^7 records (OD-6; was 60 minutes at 10^8)** (estimate 1-3 min, proportionally from 10-30 min;
  frequency: only when the key format in our code changes). Alternatives: a background rebuild without downtime (two code formats,
  duplicate writes during the build); a target without a number. Question for the owner: is start downtime acceptable (accepted by the owner: up to 6 min). ADR 0049 item 5,
  0058, 0059 item 4 (Q-82, Q-42). Accepted by the owner, OD-6.
- **D-27. Write latency: p50 <= 20 ms, p99 <= 100 ms** (up to 100 ops/s, 10^7 records, the same numbers at 10^7, OD-6; cold cache, with p99 flush to stable storage <= 40
  ms); during a backup p99 <= 250 ms and no refusals. Alternatives: p99 20 ms (needs drives with power-loss protection), 500 ms.
  Calculation on the development machine: p99 about 25 ms. ADR 0052 item 1 (Q-58, Q-11, Q-104). Accepted by the owner, OD-6.
- **D-28. Start time at 10^7 records (the numbers are the same, independent of the number of records): <= 5 s after a clean shutdown, <= 30 s after a crash one and from a backup**, without scanning records;
  a checkpoint at a WAL of 256 MiB or after 5 min. Alternatives: a WAL of 64 MiB or 1 GiB (start up to 40 s); a full page check
  at start (10-270 s, grows with the database). ADR 0052 items 2-3 (Q-42). Accepted by the owner, OD-6.
- **D-29. Leaving the states:** from "read-only" by itself, with a probe write: the state is checked before the operation checks (the area 1 order is preserved), and every write accepted
  in "read-only" begins with a durable service WAL entry (the "probe"); from "storage error" only by restart. Alternative: restart in both cases. ADR 0053 item 8,
  0051 items 9-10 (Q-71). Accepted by the owner, OD-6.
- **D-30. The ceiling of a short read is 30 s (not extended by client activity); the by-number stream has no general ceiling:** it is revoked when the client is idle (120 s without taking data) and at
  the retention cap last among the classes (after copying and short reads); revocation is a read refusal by the storage without the server going into "storage error". The accepted semantics (the full
  history without truncation) rules out a general ceiling. Alternatives: a general stream ceiling of 120 s (contradicts the accepted semantics), idle of 60 or 600 s. Consequence for area 3: a request timeout of
  10 s (Q-52). ADR 0059 item 1, 0056 item 2. Accepted by the owner, OD-6.
- **D-31. The cap on the volume retained by snapshots is max(1 GiB; 25% of data), the on-disk part no more than free space minus a write reserve of 1 GiB;**
  the volume is counted as the sum of memory and the retention temporary file (D-35); revocation by classes: copying, short reads, by-number streams, within a class the oldest first; writes do not suffer.
  Alternatives: smaller (a backup is revoked more often); no cap (risk of filling the disk, rejected). ADR 0059 item 2, 0056 item 5. Accepted by the owner, OD-6.
- **D-32. Source read speed during a backup is 64 MiB/s** (8 GB in about 2 min at 10^7, OD-6; was 40 GB in about 10 min, 64 GB in about 16 min). Alternatives: 256 MiB/s
  (greater effect on latency), 32 MiB/s (longer retention). Confirmed by a load test at 10^7. ADR 0059 item 3. Accepted by the owner, OD-6.
- **D-33. Space reserve at the backup target is 5% over the snapshot size.** ADR 0059 item 5. Accepted by the owner, OD-6.
- **D-34. The errors `EROFS`, `EACCES`, `EPERM`, `ENOSPC`, `EDQUOT` on write are "read-only", not "storage error"** (a refinement of the owner's decision Q-20 "a disk write error is a 'storage
  error'", CUJ16): nothing is lost with them and the fix depends on the operator; `EIO`, a failed flush and any other error remain a "storage error". Alternative: all write errors are
  "storage error" (a restart is needed after freeing space). ADR 0053 item 5 (Q-20, Q-71). Accepted by the owner, OD-6.
- **D-35. Earlier page versions for open snapshots are in process memory, beyond a budget of 256 MiB in a temporary file `retain` in the database directory** (not part of the database, not included in a backup, deleted at
  start); the D-31 cap is counted over the sum; a write error to `retain` revokes snapshots but does not change the database state. Closes Q-125. Alternatives: memory only (up to about 1.9 GiB of RAM at 8 GB of data; was up to 16 GiB at 64 GB); disk
  only (a write on the acknowledgement path). ADR 0056 item 1 (b). Accepted by the owner, OD-6.
- **D-36. A tree key longer than 65,535 bytes is a refusal of a single operation with no data change and no "storage error"** (a single cap in `storage-format`; unreachable for valid names: about 20,400 bytes);
  the kind of refusal for the client is area 3 (internal error). Alternative: "storage error" (a state of the whole server caused by user input). ADR 0041 item 5, 0045 item 6 (Q-103). Accepted by the owner, OD-6.

---

# Area 3: new questions

Found by the area 3 subtasks (API protocol, access, server process) and during the review fix pass. Categories are as above; (T) is decided by the implementation or build without the owner.

**Q-126 (T). The D-23 latency targets at `page_size` 100.** CUJ5. Area: storage. A page maximum of 100 (D-38) means up to about 100 point reads of R per exact-search page; the D-23 targets are set for
a page of 50. ADR: 0061 item 5, 0047 item 4, 0049.

**Q-127 (T). Index rebuild duration for `Retry-After` and the `server_starting` body.** CUJ14. Area: implementation. `Retry-After: 5` (D-42) is not tied to the rebuild time (up to 6 min at 10^7, D-26, OD-6);
a duration estimate for the client is not set. ADR: 0063 item 8, 0058.

**Q-128 (O): resolved by OD-6. One version per number, passed once per number in by-number results, not in each record.** CUJ4, CUJ9. A version of 64 characters in each record gave about 190 MB of versions alone on the maximum number (64 x 2,967,164); the owner: the version is a property of the number, a number usually has few records, extreme cases are neglected. Changed: `write-operations`, `number-search`, `http-api`, `web-ui-flows` (change `apply-owner-decisions-od6`); in name-search results the version stays in each record. ADR: 0060 item 6, 0035 item 4.

**Q-129 (T). `Accept-Encoding: gzip` in responses.** CUJ4. A stream of 300 MB or more; response compression is not promised, a request `Content-Encoding` is 415. ADR: 0064 item 6.

**Q-130 (T). `SO_REUSEADDR` and `SO_REUSEPORT` when another process occupies the port.** CUJ13. Behavior on macOS and Linux is not verified (P-7); an implementation test: a second `bind` to the same port is refused.
ADR: 0067 item 2, 0070 item 3; F-108.

**Q-131 (T). `ServerConfig::with_single_cert` with a mismatched key and certificate pair.** CUJ8. Behavior per the documentation is not verified; checked during implementation. ADR: 0066 item 3; F-93.

**Q-132 (T). Permissions 0600 and atomic replacement of the token file are POSIX only.** CUJ7, CUJ10. Defined for macOS and Linux; other OSes are out of scope. ADR: 0065 items 2, 4.

**Q-133 (T). Authentication failure counter and metrics for the operator.** CUJ8. Area: administration (Q-66): which numbers are visible to the operator ("unauthorized" refusals, handshake drops). ADR: 0067 item 3. Area 4: the `auth_counters` entry is D-60 (ADR 0078 item 6).

**Q-134 (T). Token form and a marker for secret scanners.** CUJ7, CUJ10. 64 hexadecimal characters accepted; a prefix for secret scanners is not decided, an implementation choice. ADR: 0065 items 1, 2, 4.

**Q-135 (O). A "dripping" by-number stream client.** CUJ4, CUJ8. A client takes one byte every 119 s and holds a slot and a snapshot (revocation: idle of 120 s or the retention cap, last in
order; D-30). A "minimum speed" instead of idle would change the accepted area 2 ("slow but reading"). The risk is bounded by the number of connections and the retention cap. Decided when revisiting
D-30. ADR: 0071, 0056 item 2, 0072 item 4. OD-6: accepted as the default (D-30).

**Q-136 (T). SIGHUP and closing the terminal.** CUJ14. Area: administration. A process started from a terminal dies on SIGHUP, which is a crash shutdown (ADR 0028 item 10: clean ones are only SIGTERM,
SIGINT and the command). Options: run under a service manager or treat SIGHUP as a clean shutdown (a change to ADR 0028 item 10). ADR: 0074 item 7, 0028. Area 4: SIGHUP is ignored, D-57 (ADR 0077 item 5).

**Q-137 (T). Accuracy of deadline estimates.** CUJ5, X-1. A read deadline of 20 s against about 14 s for word search is an estimate without measurement (ADR 0049 item 3); a load test (Q-114) confirms or changes the 20 s.
ADR: 0071.

**Q-138 (T). Checks of hyper behavior.** CUJ7, CUJ8. A `Transfer-Encoding` and `Content-Length` conflict, `Expect: 100-continue`, a response without reading the body, a stream drop without a terminating chunk, cancellation
on a connection break, the `header_read_timeout` count on keep-alive, `SO_REUSEPORT`: by implementation tests (F-108). ADR: 0070, 0072.

**Q-139 (T). OS users on loopback and a connection stream above the slot-protection threshold.** CUJ7, CUJ8. OS users cannot be distinguished by address; there is no portable verified way by socket uid.
The protection (deadline 1 s, D-53) limits harm to 256 new anonymous connections per second per pool; above that, "too many connections" refusals and CPU load; OS- and network-level protection is outside the
specification. ADR: 0072 items 2, 6.

**Q-140 (T). Number of worker threads and the share of word search.** X-1. P = max(4, number of cores) and ceil(P/2) are defaults without measurement; a load test. ADR: 0070 items 4, 6.

**Q-141 (T). A search query with edges that do not fit in 16 KiB.** CUJ5. Area: API (after the archive). The query is passed in the query string (GET): an edge of thousands of whitespace characters in percent-encoding does not
fit the header cap: 431 `headers_too_large` (ADR 0064 item 2), although `records` does not require rejecting a valid query because of its size before normalization, and leaves the input cap to the API area.
If unconditional acceptance is needed, search via POST with a body (new paths within v1 are allowed, ADR 0064 item 3). ADR: 0064, 0060 item 1.

---

# Area 3: defaults

The [DEFAULT]s of the area 3 subtasks, numbering continued after D-36. All accepted by the owner (OD-6; ADR: Accepted); D-43 has a new value (32 KiB), D-52 a recount to 10^7.
In the ADRs each item carries its D number.

- **D-37. Unknown members of the request JSON and unknown parameters are ignored; a repeated parameter and a repeated member name are 400 `invalid_request`.** Rationale: request fields may be added in v1
  (CUJ11). Alternative: a strict refusal of unknown ones catches typos but makes any added field incompatible for old servers. ADR 0060 item 7. Accepted by the owner, OD-6.
- **D-38. Maximum `page_size` is 100** (default 50, the owner's decision D-10). Rationale: about 50 point reads per page, 100 gives <= 2 x the D-23 target. Alternatives: 200, 1000; no page size choice.
  ADR 0061 item 5 (Q-126). Accepted by the owner, OD-6.
- **D-39. The code and status of a situation in which a v1 release responded do not change within v1** (Q-79). Alternative: allow new codes with the same status; clients that enumerated the known codes break.
  ADR 0062 item 5. Accepted by the owner, OD-6.
- **D-40. Error body `{code, field, reason, hint, message}`**: an extension of the brief section 4 decision `{code, field}` (a "field, reason" pair from the `records` dictionary). Alternative: only `{code, field}`. ADR 0062 item 7. Accepted by the owner, OD-6.
- **D-41. Codes and statuses of access refusals, limits, time and server state:** `unauthorized` 401, `forbidden` and `host_not_allowed` 403, `rate_limited` 429, `headers_too_large` 431,
  `request_timeout`, `too_many_connections`, `server_stopping` 503, `write_outcome_unknown` 504 (distinguishable from "not executed"); there is no separate code for the request target (414). Alternatives: 408 for
  `request_timeout`; 500 or 503 for `write_outcome_unknown`; 421 for `host_not_allowed`. ADR 0062 item 8. Accepted by the owner, OD-6.
- **D-42. `Retry-After`: 5 s for `server_starting`; whole seconds, at least 1, for `rate_limited`; 0 for `snapshot_revoked`; not set for the others.** Alternative: do not send a hint. ADR 0063 item 8 (Q-127). Accepted by the owner, OD-6.
- **D-43. The request body cap is 32 KiB (OD-6; as in brief section 4; was 256 KiB) and the cap of the request line with headers is 16 KiB (refusal 431).** Owner decision: the `records` scenario "20,000 whitespace characters at the edges" was replaced: a body over 32 KiB gets 413 before parsing; the name cap of 200 characters does not change. Memory for bodies at 512 connections is 16 MiB (was up to 128 MiB). Historical alternatives: 256 KiB (accepted edges of 20,000 characters); 64 KiB; 1 MiB and more. ADR 0064 item 4, 0071 (Q-52, Q-102, Q-141). Accepted by the owner, OD-6.
- **D-44. The UI token and the local-program token are two different tokens with full permissions and no limit.** Alternative: one token (a leaked link would reveal the program token). ADR 0065 item 3 (Q-38). Accepted by the owner, OD-6.
- **D-45. Issuing and revoking network tokens is delivered to the running server by an operator command; the observable requirement: after the "done" response the next request sees the change.** The channel is Q-60. ADR 0065 item 8. Accepted by the owner, OD-6.
- **D-46. The file token and the UI token are accepted only on the loopback listener, not on the network one, even from a client with a loopback address** (a refinement of the owner's decision OD-3, Q-27 "the file token is accepted from
  loopback"). Alternative: accept them on the network listener from clients with a loopback address (a literal reading; more complex and gives nothing, since the loopback listener already exists). ADR 0066 item 1. Accepted by the owner, OD-6.
- **D-47. Default ports: loopback 7878, network HTTPS 7843, network address 0.0.0.0; the network listener is off; on the network port only TLS, plain HTTP is neither served nor redirected;
  network tokens are not accepted on loopback.** Alternatives: a random loopback port written to the token file; one shared listener. ADR 0066 item 2 (Q-45). Accepted by the owner, OD-6.
- **D-48. An expired certificate means a start with a warning** (clients will reject it themselves). Alternative: start refusal. ADR 0066 item 3. Accepted by the owner, OD-6.
- **D-49. No TLS on loopback.** Rationale: the link opens at once (CUJ13), and the browser treats `http://127.0.0.1` as potentially trustworthy (F-95). Alternative: HTTPS on loopback with a locally
  issued certificate. ADR 0066 item 4 (Q-70). Accepted by the owner, OD-6.
- **D-50. The certificate and key are operator PEM files at paths from settings, replacement is swapping the files and a restart; TLS 1.2 and 1.3, no client certificates.** Alternatives: reload without
  restart; a self-signed server certificate; ACME. ADR 0066 items 5-6 (Q-69). Accepted by the owner, OD-6.
- **D-51. The request limit per network token is a bucket of 100 per second, burst 100, shared by all network tokens, one allowance for reads and writes, `Retry-After`.** Alternatives: a fixed window; a limit
  at token issuance; 10 per second. ADR 0068 items 1-3 (Q-46). Accepted by the owner, OD-6.
- **D-52. Server limits (except sizes, D-43):** connection pools of 256 on loopback and 256 on the network, 32 per network address, first request 10 s (with TLS), idle 60 s, body 10 s, read processing 20 s, write 10 s with
  the refusal "write outcome unknown" for an operation accepted by the writer, writing an ordinary response without progress 10 s, a stream 120 s; a stream is exempt from the processing deadline after the first byte. The read deadline of 20 s was chosen for 14 s at 10^8; at 10^7 the search is about 1.4 s (T: Q-165). Alternatives and
  rationales are in the ADR 0071 table. HTTP/1.1 without HTTP/2 in v1 is [OURS] (ADR 0070 item 2). ADR 0071 (Q-52, Q-135, Q-137). Accepted by the owner, OD-6.
- **D-53. Slot protection: an anonymous connection (class U) younger than 1 s is not evicted, on the network listener no more than 16 anonymous ones from one address (beyond that the oldest of the same address is evicted);
  the guarantee "a legitimate client gets no refusals from a stream of anonymous connections" holds at a stream below 256 per second per pool, above that "too many connections".** A narrowing visible to the owner:
  the unconditional reading of CUJ8 "other clients do not notice" for a stream of anonymous connections is unachievable without reserving slots or telling sources apart. Alternatives: a protection deadline of 5 s (threshold
  51/s); reserving slots for active tokens (rejected: a permanent slot cost). ADR 0072 item 2 (Q-139). Accepted by the owner, OD-6.

---

# Area 4: new questions

Found by the area 4 subtasks (lifecycle, control) and during the review fix pass. Categories are as above; (T) is decided by the implementation or build without the owner.

**Q-142 (T). `tokio::signal` behavior when SIGHUP is ignored and SIGPIPE resilience.** CUJ14. Area: implementation. Verify by test: ignoring SIGHUP, writing to a closed stderr without the process terminating. ADR: 0077 item 5, 0078 item 8; F-110.

**Q-143 (T). Platform default directories (XDG, `/var/lib`, `~/Library`).** CUJ13. Area: implementation. Not introduced (D-55); whether they are needed in the operator manual as a recommendation is for the implementation to decide. ADR: 0075 item 2.

**Q-144 (T). Service manager shutdown deadline: numbers for the manual.** CUJ14. Area: documentation. The systemd `DefaultTimeoutStopSec` value and the launchd analog (`ExitTimeOut`) are not verified; the operator manual must give numbers. ADR: 0077 item 4, 0079 item 6; F-111.

**Q-145 (O). Load measurements on a 10^7 database with the `npdb-fill` tool are not performed.** X-1. Area: implementation. Search and write latency on a populated database is separate work; whether a separate load generator is needed is outside the brief. ADR: 0079, "Consequences". OD-6: the load rig is outside v1 (accepted as the default).

**Q-146 (T). Generating P0 profile numbers in `npdb-fill`.** X-1. Area: implementation. `--from` relies on a deterministic function (`seed`, i); the way to get 1.25 records per number (the needed share of repeats) is for implementation. ADR: 0079 item 5, 0044 item 7.

**Q-147 (T). Platforms: the channel and `flock` assume a Unix-like OS (Linux, macOS); Windows is not supported.** Area: build. The brief does not name a platform; check with the owner in the package. ADR: 0080, "Consequences", 0054 item 1.

**Q-148 (T). Channel tests: socket file permissions and `peer_cred`.** CUJ10. Area: implementation. Socket file permissions under `umask 0177` on Linux and macOS; refusing a foreign uid by peer check; `peer_cred` on macOS; `connect` to an orphaned socket (`ECONNREFUSED`). ADR: 0080 items 3, 4; F-113...F-115.

**Q-149 (T). Cap on simultaneous channel connections (8).** CUJ10. Area: implementation. The value is an implementation decision, not checked under load. ADR: 0080 item 6.

**Q-150 (O). Scheduled backup automation distinguishes codes.** CUJ15. Area: administration. Whether the ADR 0084 item 3 codes (including 3 and 5) are enough for an operator script; the brief does not require a schedule. ADR: 0084 item 3. OD-6: accepted as the default.

**Q-151 (T). Format of `tokens` version 1: unknown fields.** CUJ10. Area: implementation. "Ignored or refusal" is not defined; proposal: ignored under the same `format`. ADR: 0082 item 5.

**Q-152 (T). Token command syntax: `token-issue` or `token issue`.** CUJ10. Area: implementation. A two-level subcommand was rejected for the sake of the ADR 0075 item 1 grammar; if subcommand groups are allowed, a rename without a behavior change. ADR: 0075 item 1, 0082.

**Q-153 (T). The warning «сетевой приём выключен» ("network listener is off") is taken from the file, not from the running server's state.** CUJ10. Area: implementation. If `settings.toml` was edited after start, the online `token-issue` output is inaccurate; an option: the server returns the effective value, offline reads the file. ADR: 0080 item 3 (D-74), 0082 item 9.

**Q-154 (T). File owner on an offline write and the window between `LOCK` and `control.lock`.** CUJ10. Area: implementation. `fchown` on volumes without ownership changes and with ACLs: a test; an offline command that managed to take `control.lock` in the start window makes the server refuse with code 12 (retry the start): verify and describe in the manual. ADR: 0081 items 1, 7 (D-73).

**Q-155 (T). The `init.partial` marker: implementation check and manual.** CUJ13. Area: implementation. Flush order (the marker before the other files, removal after `control`), a drop between `control` and removing the marker, behavior on volumes without a directory flush; an operator manual for a directory with `data` without `control` (restore from a backup). ADR: 0075 items 4, 8.

**Q-156 (T). `npdb-fill` at 10^8: crash start and continuing with `--from`.** X-1. Area: implementation. WAL recovery time and crash-start time after SIGKILL on a multi-hour fill; the number of records after recovery against the last printed progress. ADR: 0079 items 4, 5 (D-75).

**Q-157 (T). Server without a control channel with a read-only settings directory.** CUJ14, CUJ16. Area: implementation. On a crash start in "storage error" there is no channel, and `stop`, `backup` answer «сервер не запущен» ("server not running") (code 20) although the process is alive; a separate message and code or a `control.lock` check is needed. ADR: 0077 item 9 (D-71).

# Area 4: defaults

The [DEFAULT]s of the area 4 subtasks and the review fix pass, numbering continued after D-53. All accepted by the owner (OD-6; ADR: Accepted); D-61 has a recount to 10^7 (`--count` defaults to 10^7).
In the ADRs each item carries its D number.

- **D-54. One `npdb` program with subcommands.** Rationale: one code and format version for the server and the operator commands. Alternative: separate `npdb-server` and `npdb-admin`. ADR 0075 item 1. Accepted by the owner, OD-6.
- **D-55. `--db` and `--config` are mandatory, there are no defaults and no environment variables.** Rationale: no hidden location for the database, tokens and `generation`. Alternative: platform defaults. ADR 0075 item 2 (Q-143). Accepted by the owner, OD-6.
- **D-56. No `settings.toml` in an existing settings directory means a start on defaults with a warning.** Alternative: start refusal. If there is no settings directory, refusal. ADR 0076 item 3. Accepted by the owner, OD-6.
- **D-57. SIGHUP is ignored (Q-136).** Rationale: does not change ADR 0028 item 10; closing the terminal does not make the shutdown a crash one. Alternatives: SIGHUP is a clean shutdown (a fix to ADR 0028 item 10 and `write-operations`); leave it as a crash one. ADR 0077 item 5. Accepted by the owner, OD-6.
- **D-58. Shutdown deadline 60 s, the process's own timer, code 13, lower bound 60.** Rationale: a budget of 35 s plus headroom; the deadline does not depend on the service manager. Alternative: no timer, the deadline only at the manager. ADR 0077 item 4 (Q-108). Accepted by the owner, OD-6.
- **D-59. Messages go to stderr, `key=value`, the text is Russian; `serve` stdout is only the UI address.** Alternatives: a log file; syslog/journald; JSON. ADR 0078 items 1, 2, 7 (Q-66). Accepted by the owner, OD-6.
- **D-60. Failure counters: an `auth_counters` entry once per 60 s on a nonzero increment.** Alternatives: an entry on every refusal; on a command request. ADR 0078 item 6 (Q-133). Accepted by the owner, OD-6.
- **D-61. The load-generation tool is a separate `npdb-fill` program on a stopped database, through the engine write path, profile P0, random order; defaults to 10^7 records (OD-6).** Alternatives: filling through the API; bulk loading in the product (forbidden by X-3); building the file "bottom up". ADR 0079 item 4 (Q-53). Accepted by the owner, OD-6.
- **D-62. Operator commands may be run by the server's OS user and `root`.** Rationale: `root` reads tokens and the database anyway; without it `sudo` does not work. Alternatives: any local user; only the server uid; a configurable group. ADR 0080 item 5 (Q-60). Accepted by the owner, OD-6.
- **D-63. The channel is `control.sock` in the settings directory, not the database directory; the path is changed by `control.socket`.** Rationale: the contents of the database directory are fixed by ADR 0040 and are copied. ADR 0080 item 3. Accepted by the owner, OD-6.
- **D-64. Token commands work offline, backup and stop do not («сервер не запущен» ("server not running"), code 20).** Alternative: tokens online only. ADR 0081 item 4. Accepted by the owner, OD-6.
- **D-65. Client name: 1-64 characters `A-Z a-z 0-9 . _ -`, case-sensitive.** Alternatives: any Unicode; case-insensitive uniqueness. ADR 0082 item 3. Accepted by the owner, OD-6.
- **D-66. Token list: active ones by default, `--show all` shows all; without "last use".** Alternatives: always all; no `--show`. ADR 0082 item 4 (Q-40). Accepted by the owner, OD-6.
- **D-67. Issuance with the network listener off is allowed, with a warning.** Alternative: refusal. ADR 0082 item 9. Accepted by the owner, OD-6.
- **D-68. Command wait timeouts: connect 5 s, token response 10 s, silence during a backup 60 s.** Alternatives: no deadline; a deadline for the whole backup. ADR 0084 item 5. Accepted by the owner, OD-6.
- **D-69. A backup is tied to the command; interruption is cancellation; there are no separate status and cancel commands.** Alternative: a background job with `backup status` and `backup cancel`. ADR 0083 item 8. Accepted by the owner, OD-6.
- **D-70. A valid backup target: a nonexistent path with an existing parent or an empty directory; inside the database or settings directory is a refusal.** ADR 0083 item 3. Accepted by the owner, OD-6.
- **D-71. Settings directory not writable: a planned start is code 10; a crash start and a start from a backup are "storage error" without a control channel and without the local-program token (per `storage-durability`).** Rationale: bring lifecycle in line with the accepted `storage-durability` without changing it. Alternatives: MODIFIED `storage-durability` (always refusal 10); a planned start without `local.token`. ADR 0077 item 9 (Q-157). Accepted by the owner, OD-6.
- **D-72. Start refusal codes due to the settings directory: `control.lock` busy is 12; `tokens` unparseable or of a newer format is 3 (`key=tokens`); `tokens` unreadable because of access is 10.** Alternatives: 10 or 6 for a busy settings directory. ADR 0077 item 6, ADR 0081 items 1, 5. Accepted by the owner, OD-6.
- **D-73. An offline write creates `tokens` and `control.lock` with the owner and group of the settings directory; no directory is code 10.** Rationale: `root` is allowed (D-62), and a `sudo` write would otherwise leave a `root` file that the server cannot read. Alternative: refusal when the command uid is not the directory owner. ADR 0081 item 7 (Q-154). Accepted by the owner, OD-6.
- **D-74. Control commands read two keys from `settings.toml`: `control.socket` and `network.enabled`; the network warning is the same online and offline.** Alternative: read one key, warn only online. ADR 0080 item 3 (Q-153). Accepted by the owner, OD-6.
- **D-75. `npdb-fill` requires `--config`, takes `control.lock`, performs a crash start with a `generation` write after a crash; a non-writable settings directory is code 10.** Alternative: refusal on `run_state` other than `clean` with a hint to start and stop `serve`. ADR 0079 items 4, 5 (Q-156). Accepted by the owner, OD-6.

# Area 5: new questions

Found by the area 5 subtasks (ui-shell, ui-flows) and during the review fix pass. Categories are as above.

**Q-158 (T). Whether the token stays in browser history and in a restored `sessionStorage`.** CUJ13. Area: implementation. Whether a `/#<token>` link with a fragment lands in the visited-address history after `replaceState`; whether supported browsers return the `sessionStorage` of a closed tab and after a browser restart. The measure taken: the token is new on every start and is accepted only on loopback; if the residual risk proves unacceptable, it is an owner decision. ADR: 0086 item 2; F-116, F-117.

**Q-159 (T). `X-Content-Type-Options: nosniff` on API responses.** CUJ13. Area: server. Area 3 does not require it; ADR 0087 recommends it (JSON is served as `application/json`); it changes no visible behavior. ADR: 0087, "Consequences".

**Q-160 (O): resolved by OD-6. An add without a date is allowed only if the number has no records; otherwise a "number known" conflict (HTTP 409 `number_known`, the body carries `version`).** CUJ1, CUJ2. The CUJ1/CUJ2 race is closed by a server rule, without a new API parameter; the UI sends an add without a date at once. Changed: `records`, `write-operations`, `http-api`, `web-ui-flows`, `web-ui-shell`, `user-journeys`. ADR: 0090, 0025, 0062.

**Q-161 (T). A record repeated between word-search result pages.** CUJ5. Area: UI. A record may come twice when the name changes between pages; the UI does not hide it (ADR 0032). Merging by (number, date) changes the visible result: decide if repeats are a problem. ADR: 0092 item 7, 0032.

**Q-162 (T). A name error for a known number is visible only after the date is entered.** CUJ2. Area: UI. The name is checked by the add; a preliminary name check in the UI would require name rules of its own (forbidden, ADR 0090). ADR: 0090, "Consequences".

**Q-163 (T): resolved by OD-7. The UI guarantees responsiveness of the number history up to about 10^4 records; beyond that it is not guaranteed (a number usually has few records; extreme cases are neglected).** CUJ4. Area: UI implementation. There is no truncation and no new API operation; gradual rendering is at the implementation's discretion. Changed: `web-ui-flows`, `server-runtime`. ADR: 0100, 0092.

**Q-164 (T). Preservation of entered input on page reload and in a new tab.** CUJ12, CUJ13. Area: UI. Input is held only on the screen: with a new link in the same tab (a fragment change) it is kept, on reload (F5) and in a new tab it is lost; a draft in storage is not required by the brief. ADR: 0086 item 1, 0088 item 2.

# Area 5: defaults

The [DEFAULT]s of the area 5 subtasks and the review fix pass, numbering continued after D-75. All accepted by the owner (OD-6; ADR: Accepted); D-82 was refined by Q-160 (the date field by the `number_known` response).
In the ADRs and specs each item carries its D number.

- **D-76. UI resources are embedded in the program.** Rationale: one program without a resource directory, no mixing of versions. Alternative: a resource directory in settings. ADR 0085 item 2. Accepted by the owner, OD-6.
- **D-77. The token is in the tab's `sessionStorage`, with memory as the fallback; the guarantee is honest: not in a cookie or `localStorage`, lives until the server restarts, loopback only; "not on disk" and "disappears when closed" are not promised.** Rationale: F5 does not require a new link. Alternative: memory only. ADR 0086 item 2 (Q-158). Accepted by the owner, OD-6.
- **D-78. The date format in the UI is ГГГГ-ММ-ДД (YYYY-MM-DD).** Alternatives: ДД.ММ.ГГГГ; a native date field. ADR 0089 item 1 (Q-62). Accepted by the owner, OD-6.
- **D-79. Browsers: the two latest stable versions of Chrome, Edge, Firefox, Safari.** ADR 0089 item 2. Accepted by the owner, OD-6.
- **D-80. Auto-retry on `server_starting` is up to 60 s, then a button.** Alternatives: unlimited; 30 s; no auto-retry. ADR 0088 item 3. Accepted by the owner, OD-6.
- **D-81. Refusal texts «<Действие> не выполнено: <причина брифа>» ("<Action> failed: <brief reason>") and a table of value error reasons.** Alternative: one brief text for all actions. ADR 0088 items 5, 6 (Q-65). Accepted by the owner, OD-6.
- **D-82. The CUJ1 form has no date field; it appears for a known number (server response `409 number_known`, OD-6, Q-160).** Alternative: always an optional field. ADR 0090 item 3 (Q-62). Accepted by the owner, OD-6.
- **D-83. After saving: «Сохранено: {имя}, {номер}, с {дата}», the form is cleared, the number history is shown; for `unchanged` «Запись уже есть, данные не изменились».** Alternatives: message only; keep the input; a timer. ADR 0091 item 5 (Q-61). Accepted by the owner, OD-6.
- **D-84. Results row: trash icon, name, number, date; the same in both results.** Alternatives: the number once in a heading; the date as the first column. ADR 0092 item 1 (Q-29). Accepted by the owner, OD-6.
- **D-85. Empty results are a neutral text, not an error.** Alternative: an empty list without text. ADR 0092 item 2 (Q-29). Accepted by the owner, OD-6.
- **D-86. The date 1876-03-07 is shown as an ordinary one, with an optional hint.** Alternative: "no date". ADR 0092 item 3 (Q-1). Accepted by the owner, OD-6.
- **D-87. Results do not distinguish past, current and future owners.** Alternative: a "in the future" mark by the browser clock. ADR 0092 item 4 (Q-2). Accepted by the owner, OD-6.
- **D-88. "Next" appends a page under the ones shown.** Alternative: replace the page with a "Back" button. ADR 0092 item 6. Accepted by the owner, OD-6.
- **D-89. "Cancel delete" is a decline in the dialog; undo of an executed delete is not introduced.** Alternative: undo for a limited time (a new API operation, a question for the owner). ADR 0093 item 3 (Q-63). Accepted by the owner, OD-6.
- **D-90. After a delete: the same query, as many pages as are shown.** Alternatives: only the first page; remove the row locally. ADR 0093 item 4 (Q-35). Accepted by the owner, OD-6.
- **D-91. Re-read from name-search results: the number history in the dialog, updating the rows of that number.** Alternative: repeat the whole results. ADR 0094 item 2 (Q-36). Accepted by the owner, OD-6.
- **D-92. `record_not_found` with a current version is treated as stale data; `unchanged` is success.** Alternative: "not found". ADR 0094 item 4 (Q-68). Accepted by the owner, OD-6.
- **D-93. The CUJ12 texts verbatim («данные устарели, список обновлён»); outcome texts: «Запись уже есть, данные не изменились», «Записи уже нет».** Alternative: free wording. ADR 0094 item 5 (Q-65). Accepted by the owner, OD-6.
- **D-94. Delete confirmation: a dialog «Удалить запись: {имя}, {номер}, {дата}?», buttons «Удалить» and «Отмена», focus on «Отмена», Esc = «Отмена».** Confirmation before a delete is the owner's decision (CUJ6); the look is the author's choice. Alternatives: focus on «Удалить»; confirmation by a second press. ADR 0093 item 2. Accepted by the owner, OD-6.

# OD-6: final package

The owner's answer to a selection of important [DEFAULT] items (`owner-decisions.md`, OD-6, 2026-10-28): the rest are defaults. Change `apply-owner-decisions-od6` applied.

**Accepted by the owner, OD-6:** D-20...D-94 (75 values: 17 of area 2, 17 of area 3, 22 of area 4, 19 of area 5), ADRs in status Proposed moved to Accepted; (O) Q-104, Q-124, Q-135, Q-145 (the load rig is outside v1), Q-150.

**New value (OD-6):**
- Scale of 10^7 records instead of 10^8 (X-1; brief section 4): D-21: file <= 8 GB (about 6.1 GB without compression); D-23/D-27/D-28: the same numbers at 10^7; recomputed proportionally: sizes (64 -> 8 GB; 46-53 -> 4.6-5.3 GB; indexes 39 -> 3.9 GB; disk up to about 179 -> about 22 GB), the order index traversal about 14 -> about 1.4 s, rebuild 10-30 min -> 1-3 min and the target 60 -> 6 min, backup 16 -> about 2 min, versions <= 0.5 -> 0.05 GB, `npdb-fill` defaults to 10^7.
- D-43: body cap 32 KiB (was 256 KiB); the scenario "20,000 whitespace characters at the edges" gets 413 before parsing.
- Q-128: the version once per number in by-number results.
- Q-160: an add without a date only for a number without records, otherwise 409 `number_known`.

**Q-165 (T). Recompute on a prototype the quantities that are not linear in the number of records (10^7).** X-1. Area: implementation. Recomputed proportionally and require confirmation by measurement: the number of tree levels (3 expected at 10^7, ADR 0041, 0046); the D-23 latency class boundaries (10^4 postings, `C <= 10^7`, the G1/G2 boundary `C ~ 1.3·10^5`, heavy intersections `C = 3·10^6` per word): the target numbers were not changed; the memory budget of 256 MiB for earlier page versions (D-35) and the retention cap max(1 GiB; 25%) with 8 GB of data; the read deadline of 20 s (D-52), chosen for 14 s at 10^8: at 10^7 the search is about 1.4 s, the question is whether to return the brief's 10 s; the index share of 3.9 GB and prefix compression (not measured); `Retry-After: 5` with a rebuild of <= 6 min (Q-127). ADR: 0041, 0044, 0045, 0046, 0049, 0052, 0056, 0058, 0059, 0071.

**Q-166 (T). Reconciliation leftovers after OD-6 that do not reduce to a delta.** Area: spec. (a) `user-journeys` still has placeholders "<not decided: Q-47, Q-58, Q-59, Q-42, Q-43>" despite accepted numbers (Q-104); (b) Purpose sections are not changed by deltas and stay as they were on archiving: `number-search` ("and in each record"), `storage-format`, `storage-indexes`, `operator-lifecycle` (10^8): fix manually; (c) the marks "[DEFAULT]"/"awaits the owner" in requirements not affected by OD-6 in substance (`storage-format` "Page size 16 KiB", `server-runtime`, `api-access`, `operator-control`, `http-api`, `storage-durability`) remained as an origin label. ADR: 0017, 0019. **Closed by the final fix pass (change `final-review-fixes`, 2026-10-28):** (a) the placeholders "<not decided: Q-x>" in `user-journeys` were replaced by the accepted values (D-x, ADR), the outdated "Open: Q-x" by "Resolved: ..."; (b) the Purpose of `storage-format`, `storage-indexes`, `operator-lifecycle` was fixed by direct edit, and the placeholder mechanism was removed from the Purpose of `user-journeys`; (c) the "awaits the owner" marks were removed (`api-access`, `server-runtime`, `storage-durability`), the "[DEFAULT]" labels in traces were kept as an origin label.

**Q-167 (T). Verify on a prototype the assumptions of the final fix pass.** Area: implementation. (a) hyper: parse refusals (an unparseable request, `Transfer-Encoding` with `Content-Length`, 431) go out as a status only, without a body with `code`; `max_headers` is lifted (default 100, F-101) and there is a single cap, 16 KiB; (b) `File::set_len` without block reservation: `ENOSPC` is detected at a checkpoint (Linux, macOS), the behavior of "read-only" and a repeated checkpoint; (c) `flock` on `control.lock` when opened for reading (a server without a control channel). ADR: 0095; facts F-101, F-108.

# OD-7: results of the Rust expert assessment

Owner decisions (`owner-decisions.md`, OD-7, 2026-10-28), change `apply-od7-rust-expert` applied; source: `docs/reviews/rust-expert.md` (verdict "feasible with reservations"). Closed by decisions: item 1 (TE+CL per RFC 9112 section 6.1) and items 3, 5: ADR 0098; item 2 (backup without retention): ADR 0096; item 6 (cap on modified pages 1 GiB): ADR 0097; item 4 (dictionary D, G2/G3): ADR 0099; item 7 (Q-163): ADR 0100; item 8 (crates): ADR 0101.

New (T): expert items without a decision in OD-7 and consequences of decisions; decided by the specification or a prototype [OURS]:

**Q-168 (T). Remove the `retain` file altogether.** X-1, CUJ4. Area: storage. After ADR 0096 retention is needed by short reads (30 s) and by-number streams; `retain` was kept for the sake of streams (at W = 100 over 120 s about 1.5 GiB, ADR 0056 item 7). Option: memory only with a cap of about 1 GiB and revoking streams earlier. ADR: 0096 item 5, 0056.

**Q-169 (T). Confirm the value of the cap on modified pages (1 GiB).** X-1, CUJ15. Area: rig. 1 GiB is arithmetic (ADR 0097 item 1); measure the share of repeatedly modified pages, the backup headroom (W about 60) and the writer wait latency. ADR: 0097.

**Q-170 (T). Lower the WAL threshold to about 64 MiB for 10^7.** X-1. Area: storage. The expert's proposal (item 2): shortens recovery and start; not accepted in OD-7. ADR: 0051 item 4, 0097 item 5.

**Q-171 (T). Cap on modified pages during recovery and in `npdb-fill`.** CUJ14. Area: implementation. The same pages and chains in memory: a checkpoint or a flush during the pass is needed. ADR: 0097.

**Q-172 (T). Connection deadlines: one I/O wrapper, two numbers.** CUJ7, CUJ8. Area: server. hyper has a single `header_read_timeout` that counts idle and headers together (F-121); the spec requires separate deadlines (L). The expert's proposal (item 5): reduce to "up to the end of headers 10 s on a new connection, 60 s on keep-alive" and "write without progress N s". ADR: 0071, 0072 item 3.

**Q-173 (T). ALPN `["http/1.1"]`.** CUJ7. Area: server. A client with only `h2` gets a fatal `no_application_protocol` (F-127): do not set ALPN or fix the scenario "HTTP/1.1 only". ADR: 0066.

**Q-174 (T). Multi-version page cache: a race and the lock wording.** X-1, CUJ1, CUJ6. Area: storage. (a) A reader does not find a version in the chain and reads `data`, where a checkpoint has written a newer one: a rule of comparing `page_lsn` with the snapshot and repeating the lookup, the invariant "the earlier version is in retention until the new one is published"; (b) weaken "a reader does not hold a lock the writer waits for" to "a lock is not held during I/O or a request" (`storage-snapshots`, "One writer, many readers"). Expert, item 3. ADR: 0055, 0056 item 1.

**Q-175 (T). File test layer and classification of `ENOSPC` from `fsync`.** CUJ14, CUJ16. Area: implementation. Before the engine code: a layer that drops unflushed data, with torn and reordered writes, and a drop at every system call; a check that "checkpoint equals recovery" on random loads; record how `ENOSPC` from `fsync` is classified (on APFS/btrfs it probably comes from `fsync`: "storage error" instead of "read-only"; not verified). ADR: 0051, 0053, D-34.

**Q-176 (T). Pinning ICU4X: a vendored copy and references.** CUJ5, OD-4 item 2. Area: process. ICU4X does not support a frozen key algorithm (`icu_collator-2.4.0/src/comparison.rs` about 1735): create `vendor/` and `[patch]`, move the references (WordBreakTest, the order corpus, golden) into product tests; hardcode the format label; H64 and the cursor fingerprint are algorithms with frozen test vectors. ADR: 0010 item 2, 0039 items 3-6.

**Q-177 (T). Write path of an index rebuild.** CUJ11, X-1. Area: storage. A rebuild "next to the old one" at 10^7 needs another about 3.9 GB; through the WAL with entries up to 4 MiB it is expensive, and writing directly to `data` violates "`data` is changed only by a checkpoint". The expert's proposal: pages bottom up, straight into `data` with a flush, with the root swap journaled. ADR: 0044, 0058.

**Q-178 (T). Minor implementation remarks (expert).** Area: implementation. (a) "The first invalid pair number -> date -> name -> version" with type errors cannot be had through `serde` derive: manual parsing of `serde_json::Value` and table tests; (b) `eprintln!` crashes the process on a broken stderr (verified): all operator output via `writeln!` ignoring the error; (c) register the SIGHUP handler first in `main`. ADR: 0060, 0078, 0077 item 5.

**Q-179 (T). Spec texts leading down a cancelled branch.** Area: spec. ADR 0094 item 6 ("a retry is safe") against ADR 0095 item 12; ADR 0090 items 1-2 and 0091 item 1 describe the cancelled preliminary `by-number`; CUJ6/CUJ12 in `user-journeys` say "the version from the results row", but after Q-128 the version is at the root of the results. The spec itself prevails. Related to Q-166.

**Q-180 (T): resolved by OD-8. An add without a date, when the number has a record (number, 1876-03-07) with the same name, is "unchanged" (HTTP 200 `unchanged`); otherwise, if there are records, it is "number known".** CUJ1, CUJ2, CUJ9. Name comparison is as for an identical retry of an add with a date. A retry after a lost response is safe; the UI shows «Запись уже есть, данные не изменились». Changed: `records`, `write-operations`, `http-api`, `web-ui-flows`, `user-journeys` (change `apply-od8-idempotent-add`). ADR: 0090, 0025, 0026.

# Closed in area 1
Closed by the area 1 subtasks (change `add-data-model`). The [DEFAULT] mark was removed when closing by a default: the package
D-1...D-19 was confirmed by the owner (OD-5, section "Closed (OD-5)"). Items Q-4, Q-5, Q-15 (closed earlier) were only
refined by the name-search subtask in ADR 0030, 0031, 0034; they were not reopened.

- **Q-30. Pagination and order of exact search.** Page size 50: ADR 0033 (owner decision, OD-5, D-10); exact search is paginated
  and sorted like word search: ADR 0031 (owner decision, OD-5, D-9); the cursor, its lifetime and behavior when data changes:
  ADR 0032. The choice of page size by the client remains with the API.
- **Q-33. Order of error checks.** ADR 0025 (owner decision, OD-5, D-6).
- **Q-55. Name length: maximum and unit of measure.** ADR 0022 (owner decision, OD-5, D-3); the same cap and counting method for the name-search
  query (including the question of the query length cap). Fitting into a page is Q-48.
- **Q-64. Technical upper date limit.** ADR 0024 item 5 (owner decision, OD-5, D-5).
- **Q-72. Number version mechanics.** ADR 0027 ([OURS]): the initial version is one per version generation, a number with records never repeats a version;
  the mechanics after a crash and restore are Q-78.
- **Q-73. Cursor mechanics and total record order.** ADR 0036 ([OURS]), the cursor position is ADR 0032.
- **Q-75. Internal number representation.** ADR 0023 ([OURS]).
- **Q-84. The set of "whitespace" characters.** ADR 0021 item 1 ([OURS]; F-3).
- **Q-89. Whether a date error distinguishes "nonexistent day" and "out of range".** They are distinguished: the value error kind dictionary
  (ADR 0020 item 7, ADR 0024 item 6, `records`). Its expression in the protocol is Q-49.
- **Q-91. Refinement of X-5 in `user-journeys`.** ADR 0027 item 4 ([OURS]): the guarantee "the version differs from all previously received ones" holds
  for a number with records; a number without records has the generation's initial version. The wording of X-5 was changed in change
  `add-data-model` (delta spec `user-journeys`), and in ADR 0006 item 6(a) there is a note about the refinement and the D-13 exception.
- **Q-86. Word boundaries without dictionaries for Chinese, Japanese, Thai, Lao.** By behavior: ADR 0030 items 3-4, 7 (default UAX #29
  rules for all scripts without dictionaries; F-33...F-36); the choice was confirmed by the owner: OD-5, D-14.
- **Q-94. Word boundaries for Complex_Context scripts.** Myanmar, Tibetan and Khmer were verified by running
  (`research/name-search`, F-36); the character class of all scripts by Word_Break is F-34, the behavior is set by the UAX #29 rule
  (ADR 0030 items 3-4, 7).
- **Q-96. Reconciliation on the pinned feature set.** `research/name-search` was built on the pinned build (`=2.4.0`,
  `default-features = false`, `compiled_data`, without `auto`/`lstm`/`unstable`): `uax29 wbt` 1944 of 1944, `uax29 data` 0
  mismatches, the other modes as saved outputs; the product reference is `out-uax29-golden.txt` (F-43, F-44; ADR 0030 item 3,
  ADR 0039 items 4 (c), 5). The identity of the segmentation of variant A (F-35) with and without `auto` was not checked: variant A is not part of the product.

---

# Closed (OD-5)

The owner's answer to the area 1 package (`owner-decisions.md`, OD-5, 2026-10-27). Defaults
D-1...D-18 and questions Q-31, Q-85, Q-87, Q-88, Q-93, Q-97, Q-98 are confirmed as the default (Q-100 (O): the decision order was accepted, the choice after the test); Q-90 and D-19 are
owner alternatives. ADRs moved to Accepted, label [USER] (OD-5). The detailed rationale and alternatives remain in the ADRs.

**Owner decisions, defaults confirmed (former [DEFAULT]):**
- **D-1.** "(E.164)": "+", 1-15 digits, first digit 1-9; there is no country code list. ADR 0004 item 6.
- **D-2.** Control characters Cc in a name are forbidden (allowed in a query). ADR 0021 item 5.
- **D-3.** The cap of a name and a query is 200 Unicode scalar values after normalization; no truncation. ADR 0022.
- **D-4.** Number separator hyphens: U+002D, U+2010, U+2011. ADR 0023 item 5.
- **D-5.** The upper date limit is 9999-12-31. ADR 0024 item 5.
- **D-6.** Write check order: input, version, existence. ADR 0025.
- **D-7.** A successful write returns the outcome and the new number version; a retry with the old version is a "version conflict". ADR 0026.
- **D-8.** Isolation: sequential per number, a read after a write sees the result; consistency between result pages is not
  promised. ADR 0029 item 5.
- **D-9.** Exact whole-name search is paginated and sorted like word search; a query of only punctuation is not an error.
  ADR 0031.
- **D-10.** The name-search page size is 50. ADR 0033.
- **D-11.** Case in sorting: distinguishes only alphabetically equal names, lowercase before uppercase (Tertiary). ADR 0038.
- **D-12.** Spaces and punctuation are significant and come before letters and digits (NonIgnorable). ADR 0038.
- **D-13.** Files copied by anything other than the backup command are not considered a backup; a start follows the state at start. ADR 0028 item 7.
- **D-14.** Word boundaries in Chinese, Japanese, Thai, Lao, Myanmar are UAX #29 rules without dictionaries (variant B).
  ADR 0030 item 4.
- **D-15.** Numbers in names are compared by characters ("Ann 10" before "Ann 2"). ADR 0038.
- **D-16.** The cap on the number of records of one number is 2,967,164 (the number of valid dates), by-number results are a full list. ADR 0035 item 6.
- **D-17.** Versions between generations: a generation counter outside the backup plus a random epoch of 128 bits or more (variant (c)). ADR 0028 item 9.
- **D-18.** ICU4X fixes in v1: only a backport into the pinned copy without changing data or comparison. ADR 0039 item 6 (c).
- **Q-31.** Name sorting: Tertiary without case-first, NonIgnorable, numeric ordering off (D-11, D-12, D-15 above). ADR 0037, 0038.
- **Q-85.** Apostrophes U+0027 and U+2019 are different characters. ADR 0030 item 5.
- **Q-87.** A word-prefix match is by code points after normalization ("q" finds "q̣", "e" does not find "é"). ADR 0030 item 6.
- **Q-88.** There is no "has visible content" check: a name of invisible non-whitespace characters (U+200B) is valid. ADR 0021.
- **Q-93.** "ё" is a variant of "е" by ru rules, not a separate letter: "Её" < "Еж" < "Ёж" < "Ежи"; custom sorting rules
  are not needed. ADR 0037 item 3, 0038.
- **Q-97.** There is no protection against a query of a single ideograph or a single Thai letter (a minimum word length, a UI warning). ADR 0030.
- **Q-98.** U+202F inside a name is part of a word (UAX #29), although it is trimmed at the edges of a name. ADR 0030 item 8, 0021.
- **Q-100 (O): deferred until the test result, not closed.** If the order check on Unicode 18 characters (Q-100 (T), above) shows a mismatch, the owner decides
  after the check result (options: change the collator version or implementation; weaken ADR 0010 item 1 to an order "by
  CLDR 49 data"; accept it as a known v1 limitation). The (T) part remains open: the check is before the v1 release. ADR 0010, 0039 item 10.

**Owner alternatives (the default was replaced):**
- **Q-90. The date is mandatory on rename and delete.** A request without a date is a value error (date, "missing"); the
  1876-03-07 substitution only on add. Narrows ADR 0002 item 4 for rename and delete (the [USER] text was not changed, a note in ADR 0002).
  Consequence: clients (the UI in CUJ3 and CUJ6, programs in CUJ7 and CUJ9) pass the record's date explicitly. ADR 0025 item 6, 0024 item 3, 0020 item 7;
  spec `records` ("Zero date when no date is given", the reason dictionary), `write-operations`.
- **D-19. A "word" is a UAX #29 segment that does not consist only of whitespace and punctuation characters.** Precisely: at least one character
  whose General_Category is not Z*, not P*, not Cc and not Cf; symbols S* and emoji are words; "_" (Pc) is punctuation, not a word; an empty
  query is a query without words. The earlier value (a word is a segment with L*/N*) was rejected. ADR 0030 item 3; spec `name-search`; the reference
  `research/name-search/out-uax29-golden.txt` was regenerated.

---

# Closed in area 2

Closed by the area 2 subtasks (change `add-storage-engine`). [OURS] decisions (ADR Accepted) and defaults
accepted by the owner (OD-6; [DEFAULT]; the list is in the section "Area 2: defaults") are separated.

**[OURS] decisions:**
- **Q-41. Backup parameters (storage).** ADR 0057 items 4-7: page checks during copying, completeness, target, space,
  write error. The channel and command are area 4 (Q-60).
- **Q-43. Scope of the check at start.** ADR 0053 item 1: `control`, `db_id`, `dwb`, the WAL from the checkpoint, WAL pages,
  index roots and placement metadata; O(number of indexes + WAL).
- **Q-48. 16 KiB page and key length.** ADR 0041 (items 5-8, spill to overflow, threshold 2 KiB), ADR 0045 item 6; the 16 KiB value is
  D-20.
- **Q-56. Isolation of reads and writes (storage).** ADR 0055: one writer, reads on snapshots, a read after a write, a snapshot per request.
- **Q-57. When a write is acknowledged.** ADR 0050: after a successful flush of the WAL entry (Linux `fdatasync`/`fsync`, macOS
  `F_FULLFSYNC`).
- **Q-74. Index resilience to an ICU4X upgrade (mechanics).** ADR 0040 item 3 (`key_format_label`), 0045 items 7-8, 0058; checking the
  tables on a version change is Q-92.
- **Q-78. Version mechanics across restore and crash.** ADR 0051 item 7, 0054 items 5-6: the version is (G, E, S), only S is stored with a number
  (varint, <= 5 bytes), a generation change without scanning numbers.
- **Q-82. ICU4X upgrade: effect on start and the operator.** ADR 0045 item 8, 0058: a rebuild blocks start, no operator actions;
  the acceptable downtime is D-26.
- **Q-83. Which moment a backup fixes.** ADR 0057 item 2: a snapshot at command receipt.
- **Q-99. Binding the shutdown marker to the medium.** Not introduced, a reliable volume identifier was not verified: ADR 0054 item 4.
- **Q-103. Bound of the sort key (storage).** ADR 0041 items 5, 8, ADR 0045 item 6: a key of any length, a refusal of a single operation only above 65,535 bytes
  (D-36); the remainder is the proof of the bound, Q-113.
- **Q-105. Size of the order index.** ADR 0044 item 8, 0049 item 1: "41-63 GB" is right for names of 200 characters, a typical name's sort
  key is 35 bytes; O together with W is about 54% of the size.
- **Q-107. Search when the generation counter location is unavailable.** ADR 0054 item 7: "storage error", search works, versions of the earlier
  generation are valid only for reading.
- **Q-121. Contents of the backup slice and the backup's `control`.** ADR 0057 item 13: `run_state = backup`, `next_S` and the LSN of the backup WAL start not less than the largest S and `page_lsn` of the snapshot, empty `dwb` and `wal/`;
  the backup description is checked only on `backup`.
- **Q-109. The sort key as a prefix-free key.** ADR 0045 items 4-5: `esc` and the terminator `00 01` do not require prefix-freeness.

**Defaults (accepted by the owner, OD-6):**
- **Q-42. Start time at 10^7 records:** D-28 (ADR 0052 items 2-3).
- **Q-47. Targets "file <= 40 GB" and "search p99 <= 10 ms":** D-21 (file, ADR 0044 item 6), D-23 (latency, ADR 0049 item 2).
- **Q-58. Target write latency:** D-27 (ADR 0052 item 1).
- **Q-59. Multi-word search latency and latency growth during a backup:** D-23, D-25 (ADR 0049 items 2, 4).
- **Q-71. Sources of and exit from the "read-only" and "storage error" states.** The sources are ADR 0053 items 5-6 ([OURS]; assigning `ENOSPC`, `EDQUOT`, `EACCES`, `EPERM`, `EROFS` to "read-only" is D-34); the exit is
  D-29 (ADR 0053 item 8).
- **Q-106. Work cap of multi-word search:** D-24 (ADR 0049 item 3, ADR 0048 item 6): no deadline and no new error.
- **Q-125. Where earlier page versions for open snapshots live:** D-35 (ADR 0056 item 1 (b)): memory, beyond the budget a temporary file `retain`.

---

# Closed in area 3

Closed by the area 3 subtasks (change `add-server-api`). [OURS] decisions (ADR Accepted) and defaults accepted by the owner (OD-6; [DEFAULT]; the list is in the section
"Area 3: defaults") are separated.

**[OURS] decisions:**
- **Q-26. Number form in API responses.** The canonical form ("+" and digits), any valid form on input. ADR 0060 item 4.
- **Q-32. Version representation in JSON.** An opaque string of 64 lowercase hexadecimal characters; its form is checked without the database. ADR 0060 item 6.
- **Q-34. Where the client gets the number version.** The version is in each results record, at the root of the by-number search, in a write success and in the `version_conflict` and `name_conflict` responses; the client takes it from the
  conflict response or re-reads the number. ADR 0060 item 6.
- **Q-44. API refusals under storage states.** "Read-only" is 503 `read_only`, "storage error" is 500 `storage_error`; search under "storage error" works as far as corrupted pages are needed; the body has no
  `errno` or files. ADR 0062, 0063 item 2.
- **Q-50. Refusal wording: code or text.** The contract is `code` and status, `message` is outside the contract; "unauthorized" is 401 `unauthorized`, "no permissions" is 403 `forbidden`, "too many requests" is 429
  `rate_limited`; one response for any unrecognized token. ADR 0062 items 2, 8; 0065 item 6.
- **Q-51. Calling the API without a version in the path.** 404 `route_not_found`, no redirect. ADR 0060 item 3.
- **Q-54. Retry after a connection drop (by API).** There are no idempotency keys; the client determines the outcome by re-reading the number. ADR 0064 item 5.
- **Q-67. Date format in the API.** Strictly `YYYY-MM-DD`; other syntax and a year not of four digits is `invalid_input`, `invalid_format`, before `records`. ADR 0060 item 5.
- **Q-101. Byte cap of the input in the right units.** 3,200 bytes of raw UTF-8 and up to 9,600 bytes with `\uXXXX` escaping for a valid name; the body cap is D-43. ADR 0064 item 1.

**Defaults (accepted by the owner, OD-6):**
- **Q-38. Local-program token and UI token:** D-44 (two tokens; the `local.token` file with permissions 0600 is [OURS], ADR 0065 items 1-2); keeping the UI sign-in and the tab after a restart are area 5 (Q-39).
- **Q-45. Composition of network listeners and the form of a non-HTTPS refusal:** D-47.
- **Q-46. Request limit per token:** D-51.
- **Q-49. API error body:** D-40, codes and statuses D-41; the (field, reason) pair, `hint` and `expected_version` are [OURS], ADR 0062 items 1, 3.
- **Q-52. Server limits:** D-43, D-52, D-53; the by-number stream is exempt from the 10 s timeout (ADR 0071 item 3).
- **Q-69. TLS certificate source:** D-50. **Q-70. TLS on loopback:** D-49.
- **Q-79. A new code for a former situation within v1:** D-39.
- **Q-102. Protocol cap versus "however many spaces at the edges":** D-43 (OD-6: X = 32 KiB; edges over the cap get 413 before parsing, `records` states the body cap); the remainder is Q-141.

The owner decisions Q-8, Q-9, Q-10, Q-24, Q-27 (OD-3, OD-4) are carried out; Q-27 is refined by the default D-46.

# Closed in area 4

Closed by the area 4 subtasks (change `add-operator-cli`). [OURS] decisions (ADR Accepted) and defaults accepted by the owner (OD-6; [DEFAULT]; the list is in the section "Area 4: defaults") are separated.

**[OURS] decisions:**
- **Q-66. The channel through which the server reports to the operator.** stderr and the `key=value` format ([DEFAULT] D-59), the list of events, secrets do not appear, exit codes. ADR 0077 item 6, 0078, 0084.
- **Q-124. A separate backup check without starting.** Not introduced: a backup is checked page by page at creation. ADR 0079 item 7.

**Defaults (accepted by the owner, OD-6):**
- **Q-40. Design of the token command:** the revocation identifier is the client name [OURS] (ADR 0082 item 2); the list contents are D-66.
- **Q-53. Test load-generation tool:** D-61 (ADR 0079 items 4, 5; D-75).
- **Q-60. Operator control channel:** the channel is [OURS] (ADR 0080); the set of persons is D-62; the location is D-63; server not running is D-64.
- **Q-108. Clean shutdown deadline and exit codes:** D-58 (ADR 0077 items 4, 6).
- **Q-133. Failure counters for the operator:** D-60 (ADR 0078 item 6).
- **Q-136. SIGHUP and closing the terminal:** D-57 (ADR 0077 item 5).

Q-45 is narrowed: the settings file and its values are ADR 0076 (the values D-46...D-50 did not change). Q-41 is closed in area 2; the backup command is ADR 0083.

# Closed in area 5

Closed by the area 5 subtasks (change `add-web-ui`). [OURS] decisions (ADR Accepted) and defaults accepted by the owner (OD-6; [DEFAULT]; the list is in the section "Area 5: defaults") are separated.

**[OURS] decisions:**
- **Q-28. How the UI determines a known number.** Was: a query by number before an add without a date. Now "known" is the server response `409 number_known` to an add without a date (OD-6, Q-160; the preliminary query by number was cancelled). ADR 0090.
- **Q-37. CUJ2/CUJ3 dialogs.** ADR 0091 items 1-4.
- **Q-39. An open tab after a server restart.** After 401 the token is removed, the form and results stay, sign-in by a new link without a reload. ADR 0088 item 2, ADR 0086 item 1.

**Defaults (accepted by the owner, OD-6):**
- **Q-1:** D-86. **Q-2:** D-87. **Q-29:** D-84, D-85. **Q-35:** D-90. **Q-36:** D-91. **Q-61:** D-83. **Q-62:** D-78, D-82. **Q-63:** D-89. **Q-65:** D-81, D-93. **Q-68:** D-92.

Q-38 is narrowed: keeping the UI sign-in is ADR 0086 (D-77, residual risks are Q-158). Q-71 was checked: the sources of and exit from the "read-only" and "storage error" states are outside the UI; the UI does not poll for them. Q-90 is carried out: the date is taken from the results row and from the request (ADR 0093 item 5, ADR 0091 item 2).
