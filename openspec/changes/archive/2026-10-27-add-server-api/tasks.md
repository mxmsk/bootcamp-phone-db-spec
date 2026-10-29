## 1. Preparation for review

- [x] 1.1 Make sure `openspec validate add-server-api --strict` passes without errors or warnings; check: the command exits with code 0 and prints "is valid".
- [x] 1.2 Make sure ADR 0060–0074, the delta-specs, `design.md`, `facts.md` and `docs/open-questions.md` contain no temporary subtask identifiers or references to drafts; check:
  `grep -rnE "\bA-[0-9]|F-S[R]|Н-S[R]|Н-AP[I]|О-[0-9]|Ф[А]-|замет[к]|draft[s]/|api-protoco[l]" docs/adr openspec/changes/add-server-api facts.md docs/open-questions.md` finds nothing.
- [x] 1.3 Reconcile the seams between subtasks: a single code table in `http-api`, a single check order, one body cap and one header measure, ADR 0063 and 0073 merged; check: the table
  "Subtask reconciliation" in `design.md` exists, and there are no references to a nonexistent protocol capability.
- [x] 1.4 Reconcile numbering: facts F-86…F-108 in `facts.md`, questions Q-126…Q-141 and [DEFAULT] D-37…D-53 in `docs/open-questions.md`, rows 0060–0074 in `docs/adr/README.md`; check: every reference
  to F-, Q-, D- from the new ADRs and specs points to an existing entry.

## 2. Review

- [x] 2.1 Opus review (done: R-1…R-10, 4 BLOCKERs and 6 contradictions): consistency, conformance with `user-journeys`, the ADRs and `docs/owner-decisions.md`; check: a list of findings with classes
  BLOCKER/MAJOR/MINOR has been received.
- [ ] 2.2 Sonnet review (per the scheme in `docs/process.md`, step 5 — once the whole spec is ready, not in this area; not done): feasibility in Rust (hyper without ready-made timeouts and caps, connection classes
  and eviction, reader pool, shutdown, TLS); check: a list of findings with classes has been received.
- [x] 2.3 Work through the BLOCKER findings and contradictions (done in one fix pass per `docs/process.md`, second edition: R-1…R-10 fixed, see `design.md`, "Fix pass after the Opus review"; no repeat
  review); check: no BLOCKERs or contradictions remain, the remainder is recorded in `docs/open-questions.md`.
- [x] 2.4 Move the review remainder to `docs/open-questions.md` and recount the counters; check: the section "Area 3: new questions" and the total numbers are present.

## 3. Owner decisions

- [ ] 3.1 Hand the owner the area 3 [DEFAULT] package as one list: D-37 (unknown fields), D-38 (page maximum 100), D-39 (situation code and status do not change), D-40 (error body wider than the brief), D-41
  (access and server failure codes), D-42 (`Retry-After`), D-43 (256 KiB body instead of 32 KiB, 16 KiB headers), D-44 (two tokens), D-45 (token issuance and revocation are observable), D-46 (file and UI token
  not on the network listener — clarification of Q-27), D-47 (ports, address, non-HTTPS refusal shape), D-48 (expired certificate), D-49 (no TLS on loopback), D-50 (PEM files, restart, TLS 1.2 and 1.3),
  D-51 (limit of 100 per second), D-52 (timeouts and connection pools), D-53 (slot protection and its threshold), as well as questions (O) Q-128 and Q-135; check: the list has been handed over, the owner's answers are entered in the ADRs (status
  Accepted, label [USER]) and in `docs/open-questions.md`.
- [ ] 3.2 After the owner's answer, replace the numbers in the area 2 and 3 specs and in `user-journeys` (CUJ7, CUJ8, CUJ13: timeouts, limits, sizes) with the owner's values in a separate change; check: there are no values
  without the "default" mark.

## 4. Tests and checks arising from the area (for implementation)

- [ ] 4.1 Record in the test plan: a golden test of cursor vectors (ADR 0061 item 3); cap boundaries (Hangul in NFD, 800 scalars outside the BMP, the edges of 20,000 characters in a record of `　`, a 300 KiB body — 413, a head of 16,385 bytes
  — 431); `+` in the query string; the check order — one test for each pair of adjacent steps (including `server_starting` before the token and 404, a "read" token with a large body, the limit before permissions); a repeated
  member name and unpaired surrogates; framing (`Transfer-Encoding` and `Content-Length`, raw socket), `Expect`, HEAD, `Allow`; delivery of 401, 413, 429, 431 with an unread body; eviction and
  slot protection (1 s timeout, 16 anonymous clients from one address, a stream below and above 256 per second, separate pools); timeouts (slow headers, body, response write, 120 s stream idle); token revocation over an open
  connection; a stream of 10^6 records to a slow client (memory does not grow); shutdown under load no later than 35 s; check: the items are present in the test plan of area 3 and of the build.
- [ ] 4.2 Run the hyper behavior checks (Q-138, F-108) and measurements: word-search time at 10^8 (Q-137), thread count (Q-140), `SO_REUSEPORT` on macOS and Linux (Q-130), a mismatched key and
  certificate pair (Q-131); check: the results are recorded in `research/`, the "not verified" assumptions are replaced with measurements before implementation.
- [ ] 4.3 Resolve the build questions: access crates (`rustls`, `tokio-rustls`, `rustls-pki-types`, `sha2`, `subtle`; provider, cargo deny, `subtle` license, ADR 0069), no compression crate is needed (Q-129);
  check: an ADR for each crate and the cargo deny result.

## Workflow follow-up

- Status: the Opus review is done, the fix pass is done, the archive is open. Archive the change (`openspec archive add-server-api`) after the owner answers the [DEFAULT] package; the delta-specs `http-api`,
  `api-access`, `server-runtime` are moved to `openspec/specs/`.
- Check the archived result: `openspec validate --specs` without errors.
