# ADR index

The format and labels are in ADR 0001. Open questions are in `../open-questions.md`.

Sources referenced by the ADRs: "brief" — `../source/brief.md`;
"OD-x" — the owner decision log `../owner-decisions.md`; "F-x" — `../../facts.md`;
the requirements CUJ1–CUJ16 and X-1…X-5 — `../../openspec/specs/user-journeys/spec.md`.

Status "Accepted (OD-6)" — accepted by the owner in the OD-6 package (`../owner-decisions.md`, 2026-10-28); "supplemented by OD-7" — an item "Changed by OD-7" was added to the ADR (the outcomes of the Rust expert review, the original text is not rewritten); "supplemented by OD-6" — an item "Changed by OD-6" was added to the ADR (a new value or a recalculation for the 10^7 scale), the original text of the [USER] decisions is not rewritten.

| Number | Title | Status | Label |
|---|---|---|---|
| [0001](0001-decision-records-and-process.md) | ADR format, labels and the order for resolving technical items | Accepted | [USER] (items 1, 3, 4, 5); [OURS] (items 2, 6, 7) |
| [0002](0002-record-model.md) | Record model: (number, date) → name | Accepted; supplemented by OD-6, OD-8 | [USER]; item 4 for rename and delete narrowed by OD-5 (ADR 0025 item 6) |
| [0003](0003-name-rules.md) | Name rules: "the same name", normalization, empty and too long | Accepted | [USER] |
| [0004](0004-phone-number-format.md) | Phone number format on input | Accepted | [USER] (items 1–5; item 6 — OD-5, D-1) |
| [0005](0005-write-operations.md) | Write operations: add, rename, delete | Accepted; supplemented by OD-6, OD-8 | [USER]; items 3, 6: the date is mandatory, OD-5 (ADR 0025 item 6) |
| [0006](0006-number-version.md) | Number version and concurrent changes | Accepted | [USER] (items 1–5); [OURS] (item 6) |
| [0007](0007-search-by-number.md) | Search by number | Accepted; supplemented by OD-6 | [USER] |
| [0008](0008-search-by-name.md) | Search by name | Accepted; supplemented by OD-6 | [USER] |
| [0009](0009-result-ordering.md) | Result order of name search | Accepted | [USER] |
| [0010](0010-unicode-stack.md) | Unicode stack: one version of ICU4X, Unicode 18 for all of v1 | Accepted | [USER] |
| [0011](0011-interfaces.md) | Interfaces: UI, programmatic API, server process | Accepted | [USER] |
| [0012](0012-access-and-tokens.md) | API client access and tokens | Accepted | [USER] |
| [0013](0013-compatibility-and-upgrades.md) | Compatibility and upgrades within v1 | Accepted | [USER] |
| [0014](0014-durability-and-failures.md) | Storage reliability and behavior on failures | Accepted; supplemented by OD-6 | [USER] |
| [0015](0015-database-lifecycle.md) | Creating the database and the single server process | Accepted | [USER] |
| [0016](0016-backup-and-restore.md) | Backup and restore | Accepted | [USER] |
| [0017](0017-scale-and-implementation.md) | Scale and implementation constraints | Accepted; supplemented by OD-6 | [USER] |
| [0018](0018-out-of-scope.md) | Out of scope | Accepted; supplemented by OD-6 | [USER] |
| [0019](0019-pending-legacy-decisions.md) | Decisions of brief section 4 that have not yet been revised | Accepted (OD-6); supplemented by OD-6 | [DEFAULT] |
| [0020](0020-record-identity.md) | Record: the key (number, date), uniqueness and record identity | Accepted | [OURS] |
| [0021](0021-name-rules-details.md) | Name: normalization, whitespace character set, validity | Accepted | [OURS] (items 1–4, 6); [USER] (item 5 — OD-5, D-2; Q-88 — OD-5) |
| [0022](0022-name-length-limit.md) | Name and query length cap: 200 Unicode scalar values after normalization | Accepted | [USER] (items 1–4 — OD-5, D-3); item 5 — [OURS] |
| [0023](0023-number-canonical-form.md) | Number: cleaning, canonical form, comparison | Accepted | [OURS] (items 1–4, 6, 7); [USER] (item 5 — OD-5, D-4) |
| [0024](0024-date-model.md) | Date: calendar day, bounds, zero date, technical limit | Accepted | [OURS] (items 1–4, 6, 7); [USER] (item 5 — OD-5, D-5; item 3 for rename and delete — OD-5, Q-90) |
| [0025](0025-write-error-kinds-and-check-order.md) | Write error kinds and check order | Accepted; supplemented by OD-6, OD-8 | [USER] (OD-5: D-6; Q-90 — the owner's alternative) |
| [0026](0026-write-results-and-idempotency.md) | Results of successful write operations and idempotency | Accepted; supplemented by OD-6, OD-8 | [USER] (OD-5, D-7) |
| [0027](0027-number-version-semantics.md) | Number version: when it changes, the initial version, non-repetition | Accepted | [OURS] |
| [0028](0028-versions-after-crash-and-restore.md) | Versions after a crash restart and a restore: requirements for storage | Accepted; supplemented by OD-6 | [USER] (restatement: items 1, 4, the first sentence of item 2); [OURS] (items 3, 5, 6, 8, 10); [USER] (OD-5: the second part of item 2, item 7 — D-13, item 9 — D-17) |
| [0029](0029-write-atomicity-and-visibility.md) | Atomicity and visibility of a write operation | Accepted; supplemented by OD-6 | [OURS] (items 1–4); [USER] (item 5 — OD-5, D-8) |
| [0030](0030-name-matching-key-and-words.md) | Name comparison key and word boundaries | Accepted | [OURS] (items 1, 2, 7; item 3 except the "word" predicate); [USER] (OD-5: item 3, the "word" predicate — D-19 as clarified (OD-5, "Clarification of D-19": Z*, P*, Cc, Cf); item 4 — D-14; item 5 — Q-85; item 6 — Q-87; item 8 — Q-98; Q-97) |
| [0031](0031-exact-name-search.md) | Exact whole-name search: rules, order and pagination | Accepted | [OURS] (items 1, 3); [USER] (items 2, 4 — OD-5, D-9) |
| [0032](0032-cursor-semantics.md) | Semantics of the "next" cursor | Accepted; supplemented by OD-6 | [OURS] |
| [0033](0033-page-size.md) | Page size of name-search results | Accepted; supplemented by OD-6 | [USER] (OD-5, D-10) |
| [0034](0034-name-search-errors.md) | Errors and empty results of name searches | Accepted | [OURS] |
| [0035](0035-number-search-semantics.md) | Search by number: match on the canonical form | Accepted; supplemented by OD-6 | [OURS] (items 1–5); [USER] (item 6 — OD-5, D-16) |
| [0036](0036-full-result-order.md) | Total result order of name search | Accepted | [OURS] |
| [0037](0037-ru-collation-parameters.md) | Name comparison under ru locale rules: what follows from the rules | Accepted | [OURS]; [USER] (OD-5: Q-93 — "ё" as a variant of "е") |
| [0038](0038-collation-case-and-punctuation.md) | Role of case, punctuation and digits in name sorting | Accepted | [USER] (OD-5: D-11, D-12, D-15; Q-93) |
| [0039](0039-order-stability-in-v1.md) | Stability of the result order within v1 | Accepted (item 10, part (T) of Q-100, — a test before the v1 release); supplemented by OD-6 | [OURS]; [USER] (item 6 (c) — OD-5, D-18) |
| [0040](0040-database-files-and-header.md) | Data file: directory contents, header, format version and compatibility in v1 | Accepted; supplemented by OD-6 | [OURS] |
| [0041](0041-page-size-and-layout.md) | 16 KiB page, page layout and spilling of large keys (overflow) | Accepted (OD-6); supplemented by OD-6 | [DEFAULT] (item 1, D-20; item 5 — refusal for a key above the cap, D-36); [OURS] (items 2–8) |
| [0042](0042-record-storage.md) | Record storage: number, date, name and number version | Accepted; supplemented by OD-6 | [OURS] |
| [0043](0043-page-checksums-and-crc-crate.md) | Page checksums: CRC-32C and the `crc32c` crate | Accepted | [OURS] |
| [0044](0044-free-space-and-file-size.md) | Free space, file growth and size at 10^8 records | Accepted (OD-6); supplemented by OD-6 | [OURS] (items 1–5, 7–9); [DEFAULT] (item 6 — D-21; item 10 — D-22) |
| [0045](0045-index-structure-and-keys.md) | Indexes: the set, B+-trees, key encoding, format label and rebuild | Accepted ([OURS]); Accepted (OD-6) ([DEFAULT]: item 6, refusal for a key above the cap); supplemented by OD-6; supplemented by OD-7 | [OURS]; [DEFAULT] (item 6 — D-36) |
| [0046](0046-number-search-and-record-index.md) | Record index: search by number, number version, record read | Accepted; supplemented by OD-6 | [OURS] |
| [0047](0047-order-cursor-exact-search.md) | Name results in order, continuation from the cursor position, exact name search | Accepted; supplemented by OD-6 | [OURS] |
| [0048](0048-word-prefix-search.md) | Word-prefix search: postings, dictionary, planner, work cap | Accepted; supplemented by OD-6; supplemented by OD-7 | [OURS] |
| [0049](0049-index-size-and-latency-goals.md) | Targets for index size and search latency at 10^8 records | Accepted (OD-6); supplemented by OD-6 | [DEFAULT] (items 2–5: D-23, D-24, D-25, D-26; item 1 — the calculation for D-21) |
| [0050](0050-write-confirmation-and-crash-model.md) | Write acknowledgement and the crash model: what "flush to stable storage" means | Accepted | [OURS] (derived from the owner's decision of ADR 0014 items 1–3, Q-20; Q-57 is a consequence, not a new choice) |
| [0051](0051-write-ahead-log-and-recovery.md) | Write-ahead log, atomicity of the operation and version, crash recovery | Accepted; supplemented by OD-6; supplemented by OD-7 | [OURS] |
| [0052](0052-write-latency-and-start-time-targets.md) | Targets: write latency and start time at 10^8 records | Accepted (OD-6); supplemented by OD-6; supplemented by OD-7 | [DEFAULT] (item 1 — D-27; item 2 — D-28); [OURS] (item 3) |
| [0053](0053-integrity-checks-and-failure-states.md) | Integrity checks at start and at read; the "read-only" and "storage error" states | Accepted ([OURS]: items 1–4, 6, 7, 9); Accepted (OD-6) ([DEFAULT]: items 5 (errno classification), 8); supplemented by OD-6 | [OURS] (items 1–4, 5 except the errno classification, 6, 7, 9); [DEFAULT] (item 5 — D-34; item 8 — D-29) |
| [0054](0054-single-owner-clean-stop-and-generations.md) | Single database owner, clean-shutdown marker, version generations | Accepted; supplemented by OD-6 | [OURS] (a mechanism following the owner's decisions of ADR 0015, 0028 items 7, 9 [USER]) |
| [0055](0055-concurrency-model.md) | Concurrency model: one writer, many readers on snapshots | Accepted | [OURS] |
| [0056](0056-read-snapshots-lease-and-cost.md) | Read snapshots: retention period, number stream, cost | Accepted (the numeric values of the caps — ADR 0059, Accepted (OD-6); the place where earlier versions are stored, item 1 (b), — Accepted (OD-6)); supplemented by OD-6; supplemented by OD-7 | [OURS]; [DEFAULT] (item 1 (b) — D-35) |
| [0057](0057-backup.md) | Backup: cut, point, parameters, "backup" marker, start | Accepted; supplemented by OD-6; supplemented by OD-7 | [OURS] |
| [0058](0058-derived-index-rebuild.md) | Rebuilding derived indexes on format change: impact on start | Accepted; supplemented by OD-6 | [OURS] |
| [0059](0059-snapshot-backup-defaults.md) | Defaults: snapshot lifetimes, retention cap, backup speed, blocking rebuild | Accepted (OD-6); supplemented by OD-6; supplemented by OD-7 | [DEFAULT] (items 1, 2, 3, 5 — D-30, D-31, D-32, D-33; item 2 — also D-35; item 4 — D-26) |
| [0060](0060-api-v1-resources-and-values.md) | API v1: paths, methods, resources and value representation | Accepted ([OURS]: items 1–6, 8); Accepted (OD-6) ([DEFAULT]: item 7); supplemented by OD-6, OD-8 | [OURS]; [DEFAULT] (item 7 — D-37) |
| [0061](0061-cursor-page-number-stream.md) | Cursor, page and number results stream | Accepted ([OURS]: items 1–4, 6); Accepted (OD-6) ([DEFAULT]: item 5) | [OURS]; [DEFAULT] (item 5 — D-38) |
| [0062](0062-error-body-and-statuses.md) | Error body and HTTP statuses (a single table of codes for all refusals) | Accepted ([OURS]: items 1–4, 6); Accepted (OD-6) ([DEFAULT]: items 5, 7, 8); supplemented by OD-6, OD-8; supplemented by OD-7 | [OURS]; [DEFAULT] (item 5 — D-39; item 7 — D-40; item 8 — D-41) |
| [0063](0063-storage-states-in-responses.md) | Server refusals in API responses: storage states, start and stop, check order | Accepted ([OURS]: items 1–7); Accepted (OD-6) ([DEFAULT]: item 8) | [OURS]; [DEFAULT] (item 8 — D-42) |
| [0064](0064-input-limit-and-compatibility.md) | Input size cap (body 256 KiB, headers 16 KiB) and API compatibility within v1 | Accepted ([OURS]: items 1–3, 5, 6); Accepted (OD-6) ([DEFAULT]: item 4); supplemented by OD-6 | [OURS]; [DEFAULT] (item 4 — D-43) |
| [0065](0065-access-tokens.md) | Access tokens: UI, local programs, network services | Accepted ([OURS]: items 1, 2, 4–7); Accepted (OD-6) ([DEFAULT]: items 3, 8) | [OURS]; [DEFAULT] (item 3 — D-44; item 8 — D-45) |
| [0066](0066-listeners-and-tls.md) | Listeners and TLS: loopback, network HTTPS, certificate | Accepted (OD-6) ([DEFAULT]: item 1 (narrowing of Q-27), items 2, 4, 5, 6); Accepted ([OURS]: item 1 (composition), item 3 except the validity period, item 7) | [OURS]; [DEFAULT] (item 1 — D-46; item 2 — D-47; item 3, validity period — D-48; item 4 — D-49; items 5–6 — D-50) |
| [0067](0067-loopback-protection.md) | Loopback protection: Host, busy port, brute force, UI token delivery | Accepted; supplemented by OD-7 | [OURS] |
| [0068](0068-network-token-rate-limit.md) | Request limit per network token | Accepted (OD-6) | [DEFAULT] (items 1–3 — D-51); [OURS] (items 4–6) |
| [0069](0069-access-crates.md) | Access crates: TLS, hash and token comparison | Accepted | [OURS] |
| [0070](0070-server-process-and-http-base.md) | Server process: database ownership, HTTP base and request execution model | Accepted; supplemented by OD-6; supplemented by OD-7 | [OURS]; [USER] (hyper and serde_json — brief §3) |
| [0071](0071-server-limits.md) | Server limits: revising the brief §4 values for network clients and long requests | Accepted (OD-6); supplemented by OD-6; supplemented by OD-7 | [DEFAULT] (D-43 — sizes, D-52 — deadlines and pools) |
| [0072](0072-slot-protection.md) | Protecting connection slots from slow and anonymous clients | Accepted ([OURS]); Accepted (OD-6) ([DEFAULT]: item 2, values); supplemented by OD-7 | [OURS]; [DEFAULT] (item 2 — D-53) |
| [0073](0073-merged-into-0063.md) | Mapping storage states to server responses (merged into 0063) | Superseded by 0063 | [OURS] |
| [0074](0074-server-start-and-stop.md) | Server start and clean shutdown: what clients and the operator see | Accepted; supplemented by OD-6; supplemented by OD-7 | [OURS] (per ADR 0015 [USER], 0028 item 10) |
| [0075](0075-operator-program-and-database-creation.md) | Operator program, database and settings directories, creating an empty database | Accepted (OD-6) | [DEFAULT] (items 1, 2 — D-54, D-55); [OURS] (items 3–8) |
| [0076](0076-settings-file.md) | Settings file: format, location, composition, errors | Accepted (OD-6); supplemented by OD-6 | [OURS] (items 1, 2, 4–8); [DEFAULT] (item 3 — D-56) |
| [0077](0077-serve-signals-and-exit-codes.md) | The `serve` command, signals, shutdown deadline and exit codes | Accepted (OD-6) | [OURS] (items 1–3, 6 except codes 3 and 12, 7, 8); [DEFAULT] (item 4 — D-58; item 5, SIGHUP — D-57; item 6, codes 3, 10 and 12 for the settings directory — D-72; item 9 — D-71) |
| [0078](0078-operator-messages.md) | Operator messages: channel, format, list | Accepted (OD-6) | [DEFAULT] (items 1, 6 — D-59, D-60); [OURS] (items 2–5, 7–9) |
| [0079](0079-restore-upgrade-and-fill-tool.md) | Restore by starting on a backup, program upgrade, test load-generation tool | Accepted (OD-6); supplemented by OD-6 | [OURS] (items 1–3, 5 except `--config` and code 42, 6, 7); [DEFAULT] (item 4 — D-61; items 4, 5, `--config` and the tool's crash start — D-75) |
| [0080](0080-operator-control-channel.md) | Operator control channel to a running server: Unix socket, access, location | Accepted ([OURS]: items 1–4, 6–8); Accepted (OD-6) ([DEFAULT]: item 5 — the set of permitted users, D-62; item 3, location — D-63; item 3, keys read by the command — D-74); supplemented by OD-7 | [OURS] (items 1–4, 6–8); [DEFAULT] (item 5 — D-62; item 3 — D-63, D-74) |
| [0081](0081-offline-and-settings-lock.md) | Commands with the server stopped: settings directory lock and mode selection | Accepted ([OURS]: items 1 (except the refusal code), 2, 3, 5 (except the codes), 6); Accepted (OD-6) ([DEFAULT]: item 4 — which commands work offline, D-64; items 1 and 5, start refusal codes, — D-72; item 7 — file owner on an offline write, D-73) | [OURS] (items 1–3, 5, 6, except the refusal codes); [DEFAULT] (item 4 — D-64; items 1, 5, refusal codes — D-72; item 7 — D-73) |
| [0082](0082-token-commands.md) | Token commands: issue, list, revoke; client name; writing the token set | Accepted ([OURS]: items 1, 2, 5–8, item 9 except the warning); Accepted (OD-6) ([DEFAULT]: item 3 — name rules, D-65; item 4 — list composition and volume, D-66; item 9 — issuance with the network off, D-67) | [OURS] (items 1, 2, 5–9); [DEFAULT] (item 3 — D-65; item 4 — D-66; item 9 — D-67) |
| [0083](0083-backup-command.md) | Backup command: target, progress, cancellation, outcomes | Accepted ([OURS]: items 1, 2, 4–7, 9); Accepted (OD-6) ([DEFAULT]: item 3 — valid target, D-70; item 8 — cancellation, D-69); supplemented by OD-7 | [OURS] (items 1, 2, 4–7, 9); [DEFAULT] (item 3 — D-70; item 8 — D-69) |
| [0084](0084-command-protocol-and-messages.md) | Command protocol, output streams, exit codes, operator log entries about commands, stop by command | Accepted ([OURS]: items 1–4, 6–8); Accepted (OD-6) ([DEFAULT]: item 5 — wait timeouts, D-68); supplemented by OD-6 | [OURS] (items 1–4, 6–8); [DEFAULT] (item 5 — D-68) |
| [0085](0085-ui-page-delivery.md) | UI page delivery: shell routes, embedded resources, language | Accepted ([OURS]: items 1, 3); Accepted (OD-6) ([DEFAULT]: item 2 — D-76) | [OURS] (items 1, 3); [DEFAULT] (item 2 — D-76) |
| [0086](0086-ui-token-entry-and-storage.md) | Sign-in by link: token from the fragment, storage, transmission | Accepted ([OURS]: items 1, 3, 4); Accepted (OD-6) ([DEFAULT]: item 2 — D-77) | [OURS] (items 1, 3, 4); [DEFAULT] (item 2 — D-77) |
| [0087](0087-ui-page-security-policy.md) | UI page security policy | Accepted | [OURS] |
| [0088](0088-ui-failure-display.md) | Failure display in the UI: no token, stale token, startup and shutdown, storage states, value errors | Accepted ([OURS]: items 1–4 except the values, 6); Accepted (OD-6) ([DEFAULT]: item 3 — D-80; items 4–6 — texts, D-81); supplemented by OD-6 | [OURS]; [DEFAULT] (D-80, D-81) |
| [0089](0089-ui-date-format-and-browsers.md) | Date format in the UI and supported browsers | Accepted (OD-6) | [DEFAULT] (item 1 — D-78; item 2 — D-79) |
| [0090](0090-ui-add-form-and-known-number.md) | UI: add form, detecting a known number, the date field and format | Accepted ([OURS]: items 1–2); Accepted (OD-6) ([DEFAULT]: item 3); supplemented by OD-6, OD-8 | [OURS] (items 1–2); [DEFAULT] (item 3 — D-82) |
| [0091](0091-ui-known-number-dialogs-and-save-result.md) | UI: date prompt (CUJ2), overwrite prompt (CUJ3), what to show after saving | Accepted ([OURS]: items 1–4); Accepted (OD-6) ([DEFAULT]: item 5); supplemented by OD-6 | [OURS] (items 1–4); [DEFAULT] (item 5 — D-83) |
| [0092](0092-ui-search-results.md) | UI: results by number and by name: row, empty results, "Next" | Accepted ([OURS]: items 5, 7); Accepted (OD-6) ([DEFAULT]: items 1–4, 6); supplemented by OD-7 | [OURS] (items 5, 7); [DEFAULT] (items 1–4, 6 — D-84…D-88) |
| [0093](0093-ui-delete.md) | UI: delete, confirmation, cancel, re-query | Accepted ([USER]: item 1, item 2 — confirmation before deletion; [OURS]: items 5–6); Accepted (OD-6) ([DEFAULT]: item 2 — dialog, items 3–4) | [USER] (item 1; item 2 — confirmation); [OURS] (items 5–6); [DEFAULT] (item 2 — D-94; item 3 — D-89; item 4 — D-90) |
| [0094](0094-ui-stale-data-and-messages.md) | UI: stale data (CUJ12), re-read, vanished record, message texts | Accepted ([OURS]: items 1, 3, 6); Accepted (OD-6) ([DEFAULT]: items 2, 4, 5) | [OURS] (items 1, 3, 6); [DEFAULT] (item 2 — D-91; item 4 — D-92; item 5 — D-93) |
| [0095](0095-final-review-decisions.md) | Final fix pass: decisions on storage, startup, access and the UI | Accepted; supplemented by OD-7 | [OURS] |
| [0096](0096-backup-without-retention.md) | Backup without version retention | Accepted (OD-7) | [OURS] (owner decision OD-7 — [USER]) |
| [0097](0097-dirty-page-limit.md) | Memory limit for modified, not yet written pages | Accepted (OD-7) | [OURS] (owner decision OD-7 — [USER]) |
| [0098](0098-connection-layer-simplifications.md) | Connection layer: TE+CL per RFC 9112, closing without a response on overflow and shutdown | Accepted (OD-7) | [OURS] (owner decision OD-7 — [USER]) |
| [0099](0099-word-search-simplified-dictionary.md) | Word search: dictionary D without aggregates, strategies G2 and G3 first | Accepted (OD-7) | [OURS] (owner decision OD-7 — [USER]) |
| [0100](0100-ui-history-responsiveness-and-stream-buffer.md) | Number history: UI responsiveness up to ≈ 10^4 records, stream buffer is bounded | Accepted (OD-7) | [OURS] (owner decision OD-7 — [USER]) |
| [0101](0101-rustix-socket2-crates.md) | The rustix (or libc) and socket2 crates; what was removed because std lacks them | Accepted (OD-7) | [OURS] (owner decision OD-7 — [USER]) |
