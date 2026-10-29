## 1. Preparation for review

- [x] 1.1 Make sure `openspec validate add-operator-cli --strict` passes with no errors or warnings; check: the command exits with code 0 and prints "is valid".
- [x] 1.2 Make sure ADR 0075–0084, the delta specs, `design.md`, `facts.md` and `docs/open-questions.md` contain no temporary subtask identifiers or references to subtask drafts; check: a search for patterns of temporary subtask labels (local numbers of decisions, questions and facts, references to subtask drafts) in these files finds nothing.
- [x] 1.3 Reconcile the seams between subtasks: a single startup order with the control channel, a common code table (0–13, 20–36, 40–49), a common list of settings-directory files, the same keys that a command reads; check: the "Subtask reconciliation" table in `design.md`.
- [x] 1.4 Reconcile the numbering: facts F-109…F-115 in `facts.md`, questions Q-142…Q-157 and [DEFAULT] D-54…D-75 in `docs/open-questions.md`, rows 0075–0084 in `docs/adr/README.md`; check: every reference to F-, Q-, D- from the new ADRs and specs points to an existing entry.

## 2. Review

- [x] 2.1 Opus review (done: R-1…R-10, 5 BLOCKER and 5 contradictions): consistency, conformance with `user-journeys`, ADRs, accepted areas and `docs/owner-decisions.md`; check: a list of findings with BLOCKER/MAJOR/MINOR classes is obtained.
- [ ] 2.2 Sonnet review (per the `docs/process.md` scheme, step 5 — when the whole spec is ready, not in this area; not done): feasibility in Rust (`flock` and the socket, `peer_cred`, crash start of `npdb-fill`, the shutdown timer); check: a list of findings with classes is obtained.
- [x] 2.3 Resolve the BLOCKER findings and contradictions (done in one fix pass per `docs/process.md`, second edition: R-1…R-10 fixed, see `design.md`, "Fix pass after the Opus review"; no repeat review); check: there are no BLOCKERs or contradictions, the remainder is recorded in `docs/open-questions.md`.
- [x] 2.4 Move the review remainder to `docs/open-questions.md` and recount the counters; check: the sections "Area 4: new questions", "Closed in area 4" and the final numbers are present.

## 3. Owner decisions

- [ ] 3.1 Hand the owner the area 4 [DEFAULT] package as a single list: D-54 (one program), D-55 (mandatory paths), D-56 (no `settings.toml`), D-57 (SIGHUP), D-58 (shutdown deadline 60 s), D-59 (message channel), D-60 (failure counters), D-61 (`npdb-fill`), D-62 (circle of channel users),
  D-63 (channel location), D-64 (offline commands), D-65 (client name), D-66 (token list), D-67 (issuance with the network disabled), D-68 (command wait timeouts), D-69 (backup tied to the command), D-70 (valid target), D-71 (settings directory not writable),
  D-72 (start-refusal codes caused by the settings directory), D-73 (file owner for an offline write), D-74 (keys that a command reads), D-75 (`npdb-fill`: `--config` and crash start), as well as questions (O) Q-145 and Q-150 and Q-147 (platforms); check: the list is handed over, the owner's answers are entered into the ADRs (status Accepted, label [USER]) and into `docs/open-questions.md`.
- [ ] 3.2 After the owner's answer, replace the values in the specs of areas 3 and 4 (shutdown deadline, limits) with a separate change, if the owner chose otherwise; check: there are no values without the "default" note.

## 4. Tests and checks arising from the area (for implementation)

- [ ] 4.1 Record in the test plan: `init` on a directory with data and no `control` (code 11, file checksums unchanged), with the `init.partial` marker (remnants replaced), with a foreign file; the order of start refusals (6, 3, 4, 5, 12, 3 or 10 per `tokens`, 7, 8, 9); `control.socket` and a long path; a second server on a different database and the same settings directory;
  a read-only settings directory on planned and crash starts; a foreign user (`EACCES` and an accepted connection with `control_denied`); `root` offline (owner of `tokens` and `control.lock`); `stop` at an early stage (code 0); signals and the shutdown deadline; backup progress and cancellation; `npdb-fill` after SIGKILL with `--from`; check: the items are present in the test plan of area 4 and of the build.
- [ ] 4.2 Run checks of external behaviors (Q-142, Q-148, Q-154, Q-155): ignoring SIGHUP and SIGPIPE in Rust, socket file permissions and `peer_cred` on Linux and macOS, `fchown` on volumes without ownership change, the order of clearing the `init.partial` marker; check: results are recorded in `research/`, the "not verified" assumptions are replaced by measurements before implementation.
- [ ] 4.3 Resolve the build questions: the `toml` crate (cargo deny, ADR 0076), a separate build target for `npdb-fill`, exporting the engine write path as a library; check: an ADR for the crate and the cargo deny result.

## Workflow follow-up

- Status: the Opus review is done, the fix pass is done, the archive is open. Archive the change (`openspec archive add-operator-cli`) after the owner answers the [DEFAULT] package; the delta specs `operator-lifecycle`, `operator-control` are moved to `openspec/specs/`, the MODIFIED requirement "Startup order" replaces the previous one in `server-runtime`.
- Check the archived result: `openspec validate --specs` with no errors.
