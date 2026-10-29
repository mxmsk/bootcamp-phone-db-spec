## Scope of the area

Area 1 defines only observable behavior: what a record is, which values are valid, what the write operations and
searches do, and in what order results are returned. Out of the area, with no requirements here: the storage format and indexes (area 2),
the protocol, codes and bodies of API errors, tokens (area 3), operator commands (area 4), the look of the UI and message texts
(area 5). Everything that area 1 requires of them is collected in the section "Requirements for other areas".

## Map of capabilities, ADRs and CUJs

| Capability | ADR | CUJs and requirements |
|---|---|---|
| `records` (16 requirements, 73 scenarios) | 0002, 0003, 0004, 0020–0024 | CUJ1, CUJ2, CUJ3, CUJ4, CUJ5, CUJ9 |
| `write-operations` (18, 91) | 0005, 0006, 0014, 0025–0029 | CUJ1, CUJ2, CUJ3, CUJ6, CUJ9, CUJ12, CUJ14, CUJ15; X-5 |
| `name-search` (23, 80) | 0008, 0030–0034 | CUJ5, CUJ7, CUJ8, CUJ11, CUJ12, CUJ14 |
| `number-search` (8, 29) | 0007, 0035 | CUJ1, CUJ2, CUJ4, CUJ6, CUJ8, CUJ9 |
| `result-ordering` (10, 29) | 0009, 0010, 0036–0039 | CUJ5, CUJ7, CUJ11 |
| `user-journeys` (12 requirements changed: CUJ1, CUJ2, CUJ3, CUJ5, CUJ6, CUJ7, CUJ8, CUJ9, CUJ12, CUJ14, CUJ15, X-5; 147 scenarios in the delta) | 0006, 0020–0025, 0027, 0028, 0031–0034, 0036 | placeholders for closed questions replaced with references to decisions; X-5, CUJ1, CUJ7, CUJ14, CUJ15 narrowed by owner decisions (OD-5, D-13, D-17); CUJ1, CUJ3, CUJ6, CUJ7, CUJ9 — the date is required in rename and delete (OD-5, Q-90) |

Boundaries between capabilities: `number-search` applies the canonical form from `records`; `name-search` uses the order from
`result-ordering` (ADR 0009); `write-operations` relies on values from `records`; `name-search` and `number-search`
refer to the requirement "Number version in search results" (`write-operations`), and take the length and normalization of the query from the
requirements "Name and query normalization" and "Text length and length cap" (`records`). ADR labels: [USER] (owner decisions, OD-5; the earlier [DEFAULT] D-1…D-19 were confirmed or replaced by an alternative; the list is in
`docs/open-questions.md`, "Closed (OD-5)") — 0004 item 6, 0022 (items 1–4), 0025, 0026, 0033, 0038, items 0021 item 5, 0023 item 5, 0024 item 5, 0028 items 7 and 9,
0029 item 5, 0030 item 3 (the "word" predicate, D-19 — the owner's alternative) and items 4–6, 8, 0031 items 2 and 4, 0035 item 6, 0039 item 6 (c); the rest are [OURS]. No change to ordering or search is released in v1 (ADR 0039 items 2,
8); fixing dependencies is allowed only without changing results (the requirement "Fixing dependencies in v1 does not
change order and search" of `result-ordering`; `name-search` refers to it).

## Requirements for other areas

Collected from the requirements of the subtasks; the source in parentheses is the capability and ADR.

### Storage (area 2)

- The record key is (canonical number, date); the number representation is one-to-one with "+" and 1–15 digits (`records`, ADR 0020,
  0023).
- A date is representable for any day 1876-03-07…9999-12-31 (2 967 164 days inclusive; F-7) (`records`, ADR 0024).
- The name is stored in NFC, trimmed, without further folding; "the same name" is an exact comparison (`records`, ADR 0003, 0021).
- A name takes at most 800 bytes of UTF-8 (200 scalar values × 4 bytes); storage fits without truncation a comparison key
  of up to 1 000 bytes and a sort key of up to 8 513 bytes (uniform names of 200 characters — by exhaustive search, F-39, F-20; mixed ones —
  by sampling: maximum 8 513, mixes at most 7 014, F-48; the bound of 8 513 for mixed names is not proved [imprecise/not proved, see Q-103], the implementation re-verifies
  it, ADR 0022 item 5). The suitability of a 16 KiB page (ADR 0019 item 2, not decided) for the order index
  (sort key up to 8 513 bytes, date, number) is a question of area 2 (Q-48; option — the key outside the page or overflow);
  if they do not fit, the question returns to the owner (`records`, ADR 0022).
- The estimate of the word index (2–4×10^8 occurrences, not verified) does not account for option B of D-14: in scripts without spaces up to
  200 words per name and up to 200 words in a query; the worst case is added to Q-47, Q-59 (`name-search`, ADR 0022 item 4).
- ICU4X data is Unicode 18 for all of v1; the ICU4X code and data are not updated in v1; no change to ordering or search is
  released in v1, fixing dependencies is allowed only without changing results (ADR 0010, 0030, 0039 items 6, 8).
- The number version is stored together with the records and changes in the same indivisible write to stable storage; "wrote the record, did not write the
  version" is impossible; a copy of the version in every record of the name index is not required (write amplification) (`write-operations`,
  ADR 0027, 0029; CUJ14).
- The ordinal part of the version is taken from a counter shared by all numbers of the generation (not a per-number counter: otherwise after
  deleting all records and adding again the version would repeat); at start the counter is not below the maximum of the stored
  versions (`write-operations`, ADR 0028 item 6 (f), ADR 0027).
- The initial version is one for all numbers without records within a version generation and does not change on a planned restart;
  for a number without records no version data is stored, nothing is stored about deleted numbers (`write-operations`, ADR 0027).
- Storage distinguishes clean shutdown, crash recovery start and start from backup. The clean-shutdown marker is written
  durably as the last step of a clean shutdown (after answering all accepted operations and flushing data), and is removed durably
  before the first operation is accepted after start; at start the state on stable storage decides (SIGKILL while idle is a
  crash shutdown; ADR 0028 item 10). The "backup" marker is carried by the backup itself and is removed only in a database started on it.
- After a crash recovery start and a start from backup, the versions of all numbers with records and the initial version get values
  that do not coincide with previously issued ones: within a generation deterministically; between generations — under option (c) of
  owner decision OD-5, D-17 (ADR 0028 item 9) deterministically on an installation with saved settings, otherwise with a probability of at most
  2^-64 per event; including versions from a backup, from a lost branch, and a repeated start from the same archive. Re-evaluating all numbers
  at start as the only method is not allowed (Q-42). A possible (not mandatory) method under (c): version = (generation
  counter outside the backup, epoch, ordinal number) with a random epoch, initial version = f(generation counter, epoch).
- Under (c) the generation counter is stored outside the backup, next to the server settings (where the network tokens are, OD-4 item 9), and
  is increased durably on every crash recovery start and start from backup before operations are accepted; it is not part of the backup,
  and is carried together with the settings (`write-operations`, ADR 0028 items 8–9).
- The price: the size of a version per number, a write at start (including into the settings directory), write latency — Q-47, Q-58, Q-78.
- Operations on one number are ordered; the version check and the write are one action; the number history and the version are read from
  one consistent state. For a name-search results page (up to the maximum page size: 50 by default,
  maximum — area 3) a point lookup of the version by number is allowed if the records and the version are visible from one state
  (ADR 0029, "Consequences"); the price at 10^8 numbers — Q-47, Q-58 (`write-operations`, `number-search`, ADR 0029, 0035).
- Reading the number history for the duration of the results is a consistent snapshot (records and version as of the start) without locking
  writes to the number; a stream of up to 2 967 164 records (~300 MB by the estimate of ADR 0035); a completeness marker at the end; the
  snapshot retention period and its price are a decision of area 2 (`number-search`, ADR 0035 item 6).
- If the method includes a random epoch: at least 128 bits from an OS source, written durably before operations are accepted.
  Unavailability of the counter's location or of the randomness source is the third cause of a database state refusal: operations are not executed
  (CUJ16); the kind of refusal is area 3, a new visible state would require a question to the owner. The epoch and counter are a mechanism,
  not observable behavior. Optionally — binding the clean-shutdown marker to the storage medium (Q-99) (`write-operations`,
  ADR 0028 items 7–10).
- Write acknowledgment (Q-57): success to the client only after the operation and version survive a failure; versions of unacknowledged
  operations are not repeated (`write-operations`, ADR 0028; CUJ14).
- A backup contains versions, but starting on it invalidates them (`write-operations`, ADR 0028; CUJ15).
- The name word index (word key → record) at 10^8 records (estimate 2–4×10^8 occurrences, not verified) and the index for exact
  search by the key of the whole name; keys are built per Unicode 18; word boundaries are the UAX #29 rules over the `icu_properties` tables
  (Word_Break, Extended_Pictographic), `WordSegmenter` does not participate; the tables are pinned together with `icu_*` for all of v1; a change of
  rules or tables is an index migration (Q-74, Q-82); the control set `research/name-search` (WordBreakTest-18, table cross-check
  and the reference `out-uax29-golden.txt`, F-33, F-34, F-44) must pass at every release (`name-search`, ADR 0030,
  0031, 0039).
- The result order for name search has three keys (ADR 0036): the name sort key (the spelling of the name is not part of the key), date DESC,
  number. Results are produced without fully materializing the matches and continue from any triple (name, date, number) without
  server-side cursor state and without scanning previous pages (`name-search`, `result-ordering`, ADR 0032, 0036).
- Latency of the first page of a multi-word search: the p99 target is set as a goal (Q-59, Q-47), not as a promise "does not grow";
  the worst cases must be described — two frequent words with a distant intersection, a short prefix ("a" matches a very large
  share of records) (`name-search`, ADR 0033).
- Isolation (Q-56): consistency within a single page is sufficient; a snapshot across pages is not required (`name-search`,
  ADR 0032).
- The name sort key is derived data: the name is stored, the key is computable by code with a pinned ICU4X; the order index
  may be rebuilt from names only with the same comparison function (a change of the key record format in our code,
  restore, an index error, fixing dependencies per ADR 0039 item 6 (c)); the key carries a format label (the version of our
  code and the version of `icu_collator`); if the label does not match the running server, the index is rebuilt from the stored names
  before requests are accepted; the result order before and after is the same; a rebuild is not a way to adopt a different ICU4X version
  (`result-ordering`, ADR 0036, 0039 items 3, 6; Q-74, Q-82).
- Search by number is by the canonical string without walking all records; all records of a number are read as one list together with
  the number version (a result field, with empty results — the initial version; a copy in every record); up to 2 967 164 records per
  number without a cap or truncation (ADR 0035 item 6, owner decision OD-5, D-16) (`number-search`).
- The references of ADR 0039 item 5 (the reference sets, the order corpus and the word-boundary reference) are checked at every v1 release
  (`result-ordering`, `name-search`).

### Server and API (area 3)

- Only correct Unicode strings reach the records: invalid UTF-8 and lone surrogates are rejected by the protocol layer
  (`records`, ADR 0021, 0022). The maximum input size X (in bytes) is set by area 3: a name valid under `records`
  (200 characters, up to 4 input characters per character, F-20, plus trimmed edges) is accepted for input up to X, that is, X is not less than
  4 × 200 = 800 bytes for the name's characters plus a margin for the edges [imprecise: 4 is a number of scalar values, not bytes; not proved, see Q-101,
  Q-102]; exceeding X is rejected by the protocol layer before `records`, the kind of refusal
  is an area 3 decision. The `records` scenarios "Many spaces at the edges" and "A name of 'ё' in decomposed form" must pass
  at X (`records`, "Input size before normalization and request body cap", ADR 0022).
- Date format is YYYY-MM-DD; values with a time or zone and a five-digit year are Q-67; the date is required in
  rename and delete, its absence is a value error (date, "missing"); substituting 1876-03-07 is only on
  add (owner decision OD-5, Q-90: narrowing of ADR 0002 item 4; `records`, `write-operations`, ADR 0024, 0025 item 6).
- Expressing errors in the contract (Q-49): a value error is a pair (field, reason) from the `records` dictionary for the fields number, date,
  name, query (number — "invalid shape" with the "missing plus" marker; date — "missing" (rename and delete), "nonexistent day", "out of range"; name —
  "empty", "control character", "too long"; search query — "too long", "empty"; control characters are not rejected in the query,
  ADR 0034 item 5); for writes — invalid input (field, including the separate field "expected version" that is not in the
  `records` dictionary), name conflict, version conflict, not found; for search — empty query, query too long, invalid
  cursor (check order for a query: "too long", "empty", "invalid cursor"); a number value error is distinguishable
  from an empty list (`records`, `write-operations`, `name-search`, `number-search`, ADR 0020–0025, 0034, 0035).
- The year in input is a 32-bit signed integer; a value outside 32 bits is rejected by the protocol layer before `records` (Q-67); the "missing
  plus" marker is expressed in the protocol (Q-49) (`records`, ADR 0023, 0024).
- A name conflict carries the current name and version; a version conflict carries the current version. The version type is defined so that
  membership in the type is checked by format, without consulting the database; a missing or wrongly shaped expected version in
  rename and delete is "invalid input" (the field "expected version"), not a "conflict"; a value of the version type that does not equal the
  current version (including one not issued by the server) is a "version conflict"; for add the passed version is ignored
  (of any kind, not checked) (`write-operations`, ADR 0025 item 7).
- Order of refusals: access and database state (the order between them is defined by area 3, CUJ16) → checks of area 1
  (`write-operations`, ADR 0025 items 2–3). Unavailability of the random source or of the generation counter location is a refusal of
  the database state; the kind of refusal is set by area 3 (CUJ16), a new visible state would require a question to the owner
  (`write-operations`, ADR 0028 items 8–9).
- The version is opaque and compared for equality; the representation is Q-32; for search by number the version is a result field (with
  empty results — the initial one) and a copy in every record, for name search — a record field (Q-34); a successful write response contains the outcome ("changed" /
  "unchanged") and the version; a retry after a connection drop is Q-54 (`write-operations`, `number-search`, `name-search`,
  ADR 0026, 0029).
- Search by number is available to the "read" token, has no cursor or truncation; the response is a stream of up to 2 967 164 records. The client
  must distinguish a complete response from a cut-off one: a trailing marker or a record count in the totals (the completeness marker of by-number results);
  a response without it is incomplete; an error after the stream has started does not change the response status but is distinguishable by the absence of the
  marker (`number-search`, ADR 0007, 0035 item 6).
- The cursor is an opaque value with a position, a search kind, a query fingerprint (over the sets of word keys or the query key) and a
  checksum; no secret is needed for the cursor; the encoding is unambiguous (one content — one string, no tail),
  the length cap is the encoding of the longest valid position; the position is checked by the `records` value rules;
  the query fingerprint is at least 128 bits, deterministic and unchanged within v1 (not `DefaultHasher`); the input of the
  fingerprint is the search kind and the keys without repeats by code points, each with its length (ADR 0032 item 3); normative cursor test
  vectors (query → keys → fingerprint → cursor) are fixed before the first release, a golden test of cursors of the previous
  release — from the second; strict checking of unused encoding bits is a test; the cursor does not depend on the page size; the default page size is 50 (chosen by the client,
  valid values and the maximum are decided by the API); there is no total count; identical semantics for all tokens with the read permission (CUJ8)
  (`name-search`, ADR 0032, 0033).
- The sort locale is always ru, not from client headers (`result-ordering`, ADR 0037 item 5).
- The number in responses — canonical or entered form — Q-26 (`records`, ADR 0023).

### UI (area 5)

- The UI does not normalize differently from the server rules: the source of truth is the server. Needed: a hint for a number without "+" (Q-65), showing the
  name cap (200 characters), the date format (Q-62), showing the date 1876-03-07 (Q-1); the record date is passed explicitly in rename
  (overwrite in CUJ3) and delete (CUJ6) — there is no substitution in them (OD-5, Q-90) (`records`, ADR 0021–0025).
- After a name conflict (CUJ3) the version is taken from the conflict response or from the search by number (Q-34, Q-37); on a "version
  conflict" caused by a concurrent delete the UI re-reads the number (Q-36, Q-68); the "unchanged" outcome is not shown as an
  error (`write-operations`, ADR 0025, 0026).
- Empty results and loading "next"; a record may appear twice if the name changes between pages (Q-29); the query "李"
  finds many names (Q-97: no protection, OD-5); input with a tab and a line feed is allowed (`name-search`, ADR 0032, 0034).
- The composition and look of a row — Q-26, Q-29; the row order is as in the server response, the UI does not re-sort (`result-ordering`,
  ADR 0036).

### Administration (area 4)

- Clean shutdown (ADR 0028 item 10): an operator command and the signals SIGTERM/SIGINT; the shutdown deadline (after it the forced
  termination is a crash one, the clean-shutdown marker is not written); exit code 0 on a clean shutdown
  (`write-operations`, "Clean and crash shutdown").
- At start the server reports to the operator the kind of the previous shutdown: "clean", "crash" or "start from backup" (channel —
  Q-66).
- A backup is made only by the backup command, with the "backup" marker; document that restoring from
  files copied in any other way is not supported (such a file is not considered a backup, ADR 0028 item 7, owner decision OD-5, D-13); a start
  on a backup follows ADR 0028 (`write-operations`).
- Under option (c) of D-17 the generation counter is stored next to the server settings (where the tokens are, OD-4 item 9), is not part of the backup
  and is carried together with the settings; the settings directory is writable at start; document that without the settings
  the guarantee between generations is probabilistic (at most 2^-64 per event) (ADR 0028 item 9).

### Build and tests

- Pinning ICU4X (ADR 0039 item 4): the direct `icu_*` dependencies are exactly those listed in
  `research/ordering/icu-pin/expected-icu-tree.txt` (15 crates; the number is set by the file, F-28), each with the exact version
  `=2.4.0`, including `*_data`, `icu_provider`, `icu_collections`, `icu_locale_core`, `icu_locale_fallback` (`icu_collator`
  requires `icu_collator_data ~2.4.0`, F-29); `default-features = false`, `compiled_data`, without `auto`, `lstm`, `unstable`;
  `Cargo.lock` in the repository, all builds and tests with `--locked`; `icu_segmenter` and `icu_segmenter_data` are not part of the product
  (only `research/`, F-35). The test `check-pin.sh [directory]` (needs `jq`, `cargo`; F-40, F-46) checks the direct `=2.4.0`, the features of all `icu_*` and
  the equality of the tree with `expected-icu-tree.txt`; it is called on the root package of the product (the file `expected-icu-tree.txt` lies
  there too), and warns on `[patch]`/path; the script does not check the contents of a vendored copy — they are covered by the
  references and review of the diff (ADR 0039 items 4 (d), 5, 6 (b)). `compiled_data` contains ru data (F-19);
  the normalization data is Unicode 18; the collator data declares Unicode 18 and CLDR 49, behavior on Unicode 18 characters was not
  verified (F-42; a test (T) before the v1 release and the owner's choice on mismatch — Q-100) (`result-ordering`, `records`, `name-search`, ADR 0010, 0039).
- Minimum compiler version: `rust-version = "1.88"` (the MSRV of ICU4X 2.4.0, F-2). Tests do not require a network: the UCD 18.0.0 files
  (PropList, UnicodeData, WordBreakTest, CaseFolding, emoji-data and others needed by the `records` and `name-search` tests)
  are in the repository with SHA-256 checksums, the crates are in `cargo vendor` or the build cache (`--locked --offline`);
  only the first download of crates and the UCD update remain network-dependent (ADR 0039 item 9).
- References (ADR 0039 item 5), every v1 release: `research/ordering/expected-order.txt` (program
  `research/ordering/icu4x-order`); the word-boundary reference `research/name-search/out-uax29-golden.txt` (command in
  `research/name-search/README.md`, pinned build `research/name-search/Cargo.toml`, F-43, F-44) together with
  `out-uax29-wbt.txt` and `out-uax29-data.txt`; the order corpus `research/ordering/order-corpus` — `check-corpus.sh`
  compares the SHA-256 of the output (19 628 names) with the snapshot `order-corpus.sha256` (F-41) and the byte order of the sort key with
  `compare()` on every adjacent pair (F-45). A mismatch is a release-blocking defect. The outputs of `WordSegmenter`
  (`out-cases.txt` and others) are not references.
- The references are checked by the product code, not only by the `research/` programs (those are prototypes that produce the snapshots): the product
  library exposes the functions `normalize_name`, `name_key`, `words` and `order_key`; the build test of every release runs
  through them the names of `expected-order.txt`, the order corpus (hash), `out-uax29-golden.txt`, `out-uax29-wbt.txt` and
  `out-uax29-data.txt` and compares with the files; the collator parameters are set by one product construct ([OURS] ADR 0037,
  owner decision OD-5, ADR 0038). A change of the order of key steps or of the collator options is caught by this test (ADR 0030 item 3, ADR 0039 item 9 (d)).
  The mode `uax29 gc` (F-47) is part of the control set for updating `icu_properties`.
- Tests: boundary values from `records` (200/201 scalar values, 15/16 digits, 1876-03-06/07, 9999-12-31, (10000, 1, 1),
  (0, 2, 29)); a repeat of the `research/records` check (the order "NFC, trim", White_Space, key sizes of `sortkey-check`) on
  the build's data version; normative cursor vectors before the first release, then a golden test of cursors issued by the previous
  v1 release (ADR 0032); a check of order and matches against the references at every v1 release.
- The mechanism of versions between generations (ADR 0028 item 9): the randomness source and the generation counter storage are injected
  (trait); a test with a deterministic source checks the change of the counter and epoch on a crash recovery start and a start from backup,
  no change on a planned one, a refusal on a source or storage error, and that a counter rollback with different epochs does not
  lead to coinciding versions; the 2^-64 bound is not observed by a test, the epoch capacity (at least 128 bits) is checked.
- Before implementation, re-verify what was not verified by running: immutability of the Word_Break tables between minor versions 2.x
  (F-21, F-34; pinning the version removes the risk in v1); order on characters added in Unicode 18 (Q-100 (T), before the v1 release); the choice of specific ICU4X APIs
  (Q-76).

## Verified facts

External claims rely on `facts.md`: F-3…F-7, F-20, F-27, F-39, F-48 (`records`), F-8…F-12, F-21…F-24, F-31…F-38,
F-43, F-44, F-47 (`name-search`), F-13…F-19, F-25, F-26, F-28…F-30, F-40…F-42, F-45, F-46 (`result-ordering`), as well as
F-1, F-2 (earlier). The status
of each (verified against the primary source, per page summary, verified by running, not verified) is stated in the fact itself; F-27 is a
secondary source. Verified by running on Unicode 18 / ICU4X 2.4.0 with the programs from `research/` (`records`, `name-search`,
`ordering`): F-3, F-4, F-6, F-11, F-19–F-26, F-28, F-30, F-33–F-41, F-43–F-47; F-48 — by the reviewer's run, on a sample; F-49–F-51 — by the reviewer's run, the program is not saved in `research/`; F-25 — with an
independent cross-check on ICU4C 78.3. F-42 — from the crate README, behavior not verified; F-29 — from the crate's `Cargo.toml`.

## Open questions

What remains unresolved is moved to `docs/open-questions.md`: Q-85…Q-109 (new; Q-86, Q-89, Q-91, Q-94 and Q-96 closed within the same area;
Q-31, Q-85, Q-87, Q-88, Q-90, Q-93, Q-97 and Q-98 closed by owner decision OD-5; the (O) part of Q-100: the decision procedure was set
by the owner (OD-5) — the choice after the test result, on mismatch to the owner; Q-100 (T) — the test before the v1 release — remains
open), the narrowed Q-29, Q-32, Q-34, Q-54, Q-56, Q-68, Q-74, Q-76, Q-78, Q-95 and the remainder of the review of area 1
Q-101…Q-109. All [DEFAULT]s are closed by the owner (OD-5): D-1…D-18 confirmed, D-19 replaced by an alternative. They do not
change the structure of the change.
