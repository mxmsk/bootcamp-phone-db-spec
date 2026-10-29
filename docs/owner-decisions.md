# Owner decision log after the brief

Decisions of the product owner [USER] made after `docs/source/brief.md` was written. The brief is the primary source; this file holds everything else the owner decided. OD-x is referenced by
the ADRs (`docs/adr/`), `docs/open-questions.md` and the specification `openspec/specs/user-journeys/spec.md`. Identifiers Q-x are
question numbers; unresolved ones are described in `docs/open-questions.md`, resolved ones (OD-3, OD-4) are given below together with the decision.
The mark "Rec.:" in the text below is the wording of a proposal that the owner accepted in full (the text is carried over verbatim).

## OD-1. When items of sections 4-5 of the brief are resolved, 2026-10-27
Decision of the owner [USER]:

The technical items of sections 4-5 of the brief (for example: storage page, file size and latency targets, API error body,
server limits, isolation level, write latency, operator control channel, test load generation) are not resolved at the CUJ
layout stage: without a data model, storage and API they cannot be proposed on a sound basis. They are recorded as
"Proposed: resolved in iteration X" with the label [DEFAULT] and brought to the owner with concrete numbers and options in the
corresponding iteration (data model / storage / API / UI). This does not lift the brief's requirement to "propose and
ask": each such item still goes to the owner, later. At the CUJ stage the owner is asked only product
questions whose answer does not depend on a data model, storage or API that has not been designed yet.

## OD-2. ADR format, 2026-10-27
Decision of the owner [USER]: `docs/adr/NNNN-slug.md`, MADR-like, in Russian; fields: context, decision, alternatives considered
with the reason for rejection, consequences, trace to CUJ, sources, label, status.

## OD-3. Answers to product ambiguities (P), 2026-10-27
The owner [USER] accepted all 27 recommendations below without changes. Where OD-4
diverges from OD-3, OD-4 prevails (apostrophe and UAX #29: Q-4; Unicode 18 is fixed for all of v1: Q-21; no
format conversion command: Q-13; rollback: OD-4, item 8; corruption detection and a "lying" disk: Q-20, OD-4, item 3).
The marks [s.4] and [s.5] on a question mean that it follows from section 4 or 5 of the brief.

Accepted decisions (verbatim; the order and the split into groups A-F are kept):

### A. Name
- Q-3. Name normalization on save. Rec.: trim whitespace at the edges, bring to NFC; "the same name" is an exact
  match after that; an empty name or a name of only whitespace is an error.
- Q-81. A name longer than the cap. Rec.: reject with an error, without truncation (the cap itself is set in the data model iteration, Q-55).
- Q-80. Renaming to the same name. Rec.: silent success, the number version does not change (like a retry of an add).

### B. Name search
- Q-4. What a "word" is. Rec.: word boundaries per Unicode UAX #29 (a hyphen and an apostrophe separate words); the order of
  query words does not matter; one word of the name may cover several query words.
- Q-5. Empty query. Rec.: a query with no words after trimming (including one of only spaces) is an error.
- Q-15. Exact search "whole name". Rec.: the same rules as for the name and word search: trimming, NFC, case-insensitive,
  "ё" = "е"; an empty query is an error.
- Q-25. Order for equal name and date. Rec.: then by number ascending (stable and predictable).
- Q-21. Whether a server upgrade within v1 may change search matches/order. Rec.: yes, only on a change of the Unicode/CLDR
  version, and this is stated in the release description.

### C. Number
- Q-23 [s.4]. Number input policy. Rec.: confirm: "+" is required, then 1-15 digits (E.164); spaces, hyphens,
  parentheses and dots are removed; a number without "+" is rejected with a hint; numbers written differently are one
  number after cleaning.
- Q-6. Search by number. Rec.: number not found is an empty list, not an error; a number of invalid shape is a format error (the same
  rules as on save).

### D. Operations and versions
- Q-19. Deleting a missing record. Rec.: "not found" (symmetric with rename).
- Q-77. Versions after a restore from a backup and after a crash. Rec.: an action by a version read before the restore or
  before the crash never goes through (always a conflict, the client re-reads); mass conflicts right after a restore are
  acceptable.
- Q-16. The guarantee "acknowledged survives a crash" covers changes and deletes. Rec.: yes (a deleted record does not
  "resurrect").

### E. Reliability and operation
- Q-20. Which failures we protect against. Rec.: process crash, OS failure, power loss, provided the disk honestly
  performs a flush to stable storage. A "lying" disk and medium corruption are outside the guarantee but are detected (the database does not open,
  CUJ14); a disk write error is a "storage error" (CUJ16).
- Q-12. Corruption found on a running server. Rec.: the server goes to the "storage error" state: changes are
  rejected (CUJ16), search works as long as it can; a message to the operator.
- Q-14. "Search works as long as it can". Rec.: in "read-only" mode all searches work; under a storage error, those
  that have enough readable data, without guarantees.
- Q-17. "Read-only" mode by operator command. Rec.: not needed (a new feature outside the brief); the state comes only from
  external causes.
- Q-11. "Without interruption" during a backup. Rec.: no refusals; latency growth is acceptable within limits to be set
  with the latency targets (storage iteration).
- Q-22 [s.4]. No program version rollback. Rec.: confirm; rollback only by restoring from a backup made before the
  upgrade.
- Q-13. New server version and an existing database. Rec.: within v1 it opens the database without operator action; if a format
  conversion is needed, it is a separate operator command, without it the server does not start and says so (like CUJ13).
- Q-7. Re-creating a database and a second process. Rec.: the create command on a directory where a database already exists is a refusal, never
  overwrites; a second server on the same database is a start refusal «база уже открыта другим процессом» ("database already opened by another process").

### F. Access and tokens
- Q-8. Network listener. Rec.: off by default, enabled by an operator setting; if enabled and there is no certificate, the
  server does not start.
- Q-27. Which tokens from where. Rec.: the file token and the UI token only from loopback; a network token is accepted from
  anywhere, but only over HTTPS.
- Q-9. Token management. Rec.: the client name is unique; revoking a nonexistent or already revoked one is an error message,
  with no changes; a permission change only by issuing a new token; a request already executing at the moment of revocation
  completes.
- Q-10. Tokens and backup. Rec.: tokens are stored separately from data and are not part of a backup; a restore does not
  change tokens (a revoked one does not come back to life).
- Q-24 [s.5]. Whom the request limit applies to. Rec.: only to network tokens; local programs and the UI have no limit.
- Q-18. "Other clients do not notice". Rec.: they do not get refusals because of another client's limit; latency stays within the general
  targets.

## OD-4. Clarifications of OD-3 decisions, 2026-10-27
The owner [USER] accepted all 9 recommendations below. OD-4 replaces OD-3 where they diverge: item 1
replaces the Q-4 wording about the apostrophe; item 2 cancels Q-21 ("yes, on a change of the Unicode/CLDR version"); item 3 clarifies Q-20 (in
OD-3 "detected (the database does not open)"); item 8 cancels in Q-13 the format conversion command and clarifies Q-22; item 9
clarifies Q-10; items 4, 5, 6, 7 clarify Q-9, Q-77, Q-25, Q-80 respectively. On a contradiction OD-4 takes precedence over OD-3.

Accepted decisions (verbatim; the item number is as in the question package):

1. Q-4, apostrophe. Rec.: word boundaries strictly per UAX #29, without exceptions of our own: a hyphen separates words, an apostrophe between
   letters does not ("O'Brien" is one word). A query with no words at all (only punctuation or spaces) is
   empty, an error. (Corrects package 1: there the apostrophe was named a separator.)
2. Q-21, Unicode version. Rec.: Unicode 18 is fixed for all of v1; a server upgrade within v1 does not change what
   search finds and in what order. (Cancels the recommendation of package 1.)
3. Q-20, corruption detection. Rec.: at start it is checked that the database opens consistent (structure and latest
   changes); corruption of other data is detected when it is read (checksums) and leads to the "storage error" state
   (decision Q-12). Detection of a "lying" disk is not promised.
4. Q-9, client name. Rec.: the name is unique among active tokens; after revocation the name can be issued again.
5. Q-77, coverage. Rec.: after a restore from a backup and after a crash restart conflicts are acceptable for any numbers;
   after a planned restart (clean shutdown) earlier versions stay valid.
6. Q-25, order by number. Rec.: by the number's string ("+19..." before "+2..."), so numbers of one country are adjacent.
7. Q-80, same name with a stale version. Rec.: a conflict with the current version (the version check is first), silent success
   only with the current version.
8. Q-22/Q-13, rollback and format. Rec.: a backup made before an upgrade is started by the new program (a new version within
   v1 opens databases of earlier v1 versions itself, without a command); there is no separate format conversion command; the earlier program
   is not obliged to open a database after the new one, and that is exactly "no rollback". (Cancels the conversion command of package 1.)
9. Q-10, tokens and backup. Rec.: a backup contains only data; network tokens are stored next to the server
   settings (like the certificate), and when moving to another machine the operator carries them together with the settings, or else
   reissues them.

## OD-5. Area 1 package [USER], 2026-10-27
The owner instructed the orchestrator to choose the options. Accepted:
- all defaults D-1...D-18 and the questions Q-85, Q-87, Q-88, Q-93, Q-97, Q-98, Q-100 as the package defaults;
- **Q-90, the alternative:** on rename and delete the date is mandatory; a rename or delete request without a date is a value
  error (date missing); the 1876-03-07 substitution remains only for add. This narrows ADR 0002 item 4 for
  rename and delete.
- **D-19, the alternative:** only a query of whitespace characters and punctuation counts as empty (as in OD-4 item 1); a UAX #29
  segment with symbols or emoji ("😀", "+", "$", "_": to be checked by categories) is a word and takes part in search.
The choice of Q-90 and D-19 was made by the orchestrator on the owner's instruction; the rationale is in ADR 0025 and ADR 0030.
Clarification of D-19 (the orchestrator on the same instruction, after review): "spaces" include whitespace control characters (TAB, LF),
and invisible formatting characters (Cf, for example U+200B) also do not form words; "_" is punctuation (Pc). Result: a word is a
segment with a character outside categories Z*, P*, Cc, Cf.

## OD-6. Final [DEFAULT] package of areas 2-5 [USER], 2026-10-28
The owner answered a selection of important items (D-21, D-23, D-27, D-43, D-47, D-49, D-55, Q-128, Q-145, Q-160); the remaining
[DEFAULT]s D-20...D-94 and the (O) questions are taken as the defaults. Changes:
1. **Scale is 10^7 records** instead of 10^8 (changes brief section 4 "Scale: 10^8 records" and X-1). The architecture is the same; all
   numbers derived from 10^8 are recomputed for 10^7. The file size target (D-21) is **<= 8 GB** at 10^7 (the calculation without
   compression is about 6.1 GB). The latency targets D-23/D-27 are the same numbers, but at 10^7.
2. **Request body cap is 32 KiB** (as in brief section 4; cancels the 256 KiB of D-43). The `records` scenario "20,000 whitespace
   characters at the edges" changes: a body over 32 KiB is a "request too large" refusal (413) before parsing; the data model (a cap of 200
   characters after trimming the edges) does not change.
3. **Q-128: one version per number**, and in by-number results it is passed once per number, not in each record (as
   agreed from the start: the version is a property of the number). A number usually has few records; extreme cases are neglected.
4. **Q-160: an add without a date is allowed only if the number has no records;** otherwise it is a conflict (like a name
   conflict, with the current number version). The CUJ1/CUJ2 race is closed by a server rule, without a new API parameter.
5. Q-145: the load-test rig is outside v1 (default); D-47/D-49, D-55 are defaults.

## OD-7. Results of the expert Rust feasibility assessment [USER], 2026-10-28
Source: docs/reviews/rust-expert.md (Opus expert, verdict "feasible with reservations"). The owner accepted all the
orchestrator's proposals:
1. item 1: a request with `Transfer-Encoding` and `Content-Length` is handled per RFC 9112 section 6.1 (as hyper does: CL
   is ignored, the body is chunked, the connection is closed after the response), without a mandatory 400.
2. item 4: a backup without version retention: the LSN is fixed, `data` and the WAL up to that LSN are copied, the backup
   is brought up to date by the same recovery code; snapshot retention remains only for short reads and the by-number stream.
3. item 6: on slot overflow (especially over HTTPS) the connection is closed without a response; the cap on simultaneous refusals is
   removed; what remains is the eviction of an anonymous connection older than 1 s, then an idle one.
4. item 9: word search: dictionary D is an ordinary tree without aggregates in inner nodes; strategies G2 and G3, G1 by
   profile; the result does not depend on the strategy.
5. item 15: on shutdown idle connections are closed without a response; 503 `server_stopping` only for requests whose headers
   were already accepted.
6. item 2: a memory cap is set for modified, not yet written pages (the default value is at the discretion of the fix pass).
7. item 12: the UI guarantees responsiveness of the number history up to about 10^4 records; beyond that it is not guaranteed (a number usually has few
   records, extreme cases are neglected: the owner, discussion of Q-128); the by-number stream has a "bounded buffer".
8. item 8: the crates `rustix` (or `libc`) and `socket2` are recorded in an ADR.

## OD-8. Retry of an add without a date (Q-180) [USER], 2026-10-28
The expert's proposal was accepted: an add without a date, when the number has a record (number, 1876-03-07) with the same name, has
the outcome "unchanged" (success), not "number known". In all other cases when the number has records it is
"number known" (OD-6 item 4 otherwise unchanged). Goal: a retry after a lost response is safe.
