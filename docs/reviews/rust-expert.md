# Expert assessment of feasibility in Rust

**Verdict:** feasible with caveats. One requirement (`server-runtime`, "Message boundary substitution is ruled out") cannot be met as written on stock hyper. The rest is feasible on stable Rust (MSRV of the `icu_*` graph is 1.88, `File::try_lock` is 1.89) on macOS and Linux. The main cost is our own layers that the chosen crates lack: a multi-version page cache, a connection layer on top of hyper, and a test file layer with failure injection.

How this was verified:
- **Crate sources** downloaded: hyper 1.12.0, tokio 1.54.1, mio 1.2.4, rustls 0.23.46, icu_collator / icu_segmenter 2.4.0.
- **std 1.96 sources** (`sys/fs/unix.rs`).
- **Probes on rustc 1.96, macOS**:
  - `sync_data` = `F_FULLFSYNC`, about 9 ms;
  - `try_lock` = `flock`;
  - `set_len` creates a sparse file;
  - the ICU4X sort key: length, absence of prefix-freeness, "ё" and "е";
  - `eprintln!` with a broken stderr.
- **Not verified**: binding a TCP port on macOS (the sandbox forbade bind), the load bench, and everything marked below.

## Items (in descending order of importance)

### 1. INFEASIBLE — `server-runtime`, "Message boundary substitution is ruled out"; ADR 0072 item 8; F-108 (a)
- **Essence.** A request that has both `Transfer-Encoding: chunked` and `Content-Length` is not rejected by hyper 1.12. It removes `Content-Length`, reads the body as chunked, sets `keep_alive = false` and passes the request to the service (`proto/h1/role.rs`, roughly lines 284–360). The service cannot learn that there were two length headers. The requirement "400, operation not executed" is feasible only with our own pre-parse of the header bytes under hyper, that is, a second HTTP parser.
- **Recommendation.** Accept the RFC 9112 §6.1 behavior: the body is read by `Transfer-Encoding`, and the connection is closed after the response. There is then no boundary substitution, and the second scenario ("a second request inside the body") still holds. Rewrite the first scenario.

### 2. RISK — `storage-durability`, "Checkpoints bound the WAL"; ADR 0051 items 1, 3, 4; ADR 0052
- **Essence.** The `data` file changes only by a checkpoint, so all dirty pages together with their version chains live in memory until then. The spec has no memory budget for the cache.
- **Volume estimate** (from the numbers in ADR 0044, 0051, 0052):
  - a 5 min threshold at W = 100 gives on the order of 2–3 GiB of dirty pages;
  - a WAL threshold of 256 MiB can make a noticeable share of an 8 GB database dirty.
- **Where else it arises.** The same during restore and in `npdb-fill`.
- **Recommendation.** Introduce a dirty-page cap, for example 512 MiB, as a third trigger for a checkpoint. Lower the WAL threshold for 10^7 to ~64 MiB: this also shortens restore and start.

### 3. RISK — ADR 0055, 0056 item 1; `storage-snapshots`, "One writer, many readers"
- **Essence.** The multi-version page cache is the core of concurrency and the riskiest part (XL).
- **A race in the spec is not closed.** A reader of snapshot S does not find a version in the chain and reads the page from `data`, while a checkpoint has meanwhile written a newer version there.
- **The wording is too strict.** The literal "a reader does not hold a lock that the writer waits for" forbids even a short mutex on the page table and pushes toward lock-free structures (epoch-GC, arc-swap).
- **Recommendation.**
  - Add a rule: after reading from the file, compare `page_lsn` with S, and if the version is newer, repeat the lookup in the chain.
  - Add an invariant: the earlier version enters retention before the new one is published.
  - Weaken the wording to "no lock is held for the duration of I/O or a request".

### 4. SIMPLIFY — ADR 0056, 0057, 0059; `storage-snapshots`, "Cap on retained volume"; CUJ15
- **Essence.** An 8 GB backup at 64 MiB/s holds a snapshot for about 2 min. At W = 100 retention (~1.5 GiB) approaches the 1.9 GiB cap, so a backup under load is on the edge of revocation. The `retain` file and the revocation classes are needed mostly for the backup.
- **Recommendation.** Copy without retaining versions:
  - fix LSN X and pause checkpoints;
  - copy `data` as is and the WAL from `redo_start_lsn` to X;
  - run the same restore code up to X in the backup directory.
- **What this gives.** The backup point is the same, and CUJ15 and the CRC check are preserved. Retention remains only for short reads and the by-number stream. Not prototyped.

### 5. RISK — `server-runtime`, "Request read timeouts", "Ordinary response write deadline of 10 s"; ADR 0071, 0072 item 3; F-101, F-108 (f)
- **Essence.** hyper has a single `header_read_timeout` timer, and it starts right after the previous response (`conn.rs` ≈ 216–260), that is, it counts idle time and headers together. The spec requires separately:
  - idle 60 s;
  - headers 10 s from the first byte;
  - first request 10 s together with TLS;
  - body 10 s;
  - write without progress 10 s and 120 s for a stream.

  All of this is our own I/O wrapper under TLS with a watchdog task per connection (L).
- **Recommendation.** Build one wrapper as a single mechanism. Reduce the deadlines to two numbers: "until the end of headers 10 s on a new connection, 60 s on keep-alive" and "write without progress N s".

### 6. SIMPLIFY — `server-runtime`, "Slot protection from anonymous clients", "Anonymous connections from one network address"; ADR 0072 items 1–2, 6
- **Essence.** Classes U/I/A, a protection period of 1 s, caps of 16 and 32 per address, no more than 16 simultaneous refusals — all of this is our own accept loop and a connection registry with a class change from the service (L, hard to test). A 503 `too_many_connections` response over HTTPS can be sent only after a full TLS handshake, that is, at the cost of asymmetric cryptography for each rejected connection.
- **Recommendation.**
  - On overflow over HTTPS, close the connection without a response.
  - Remove the "≤ 16 simultaneous refusals" cap.
  - Keep only the eviction of an anonymous connection older than 1 s and then of an idle one: it is what preserves CUJ7/CUJ13 under a flood.

### 7. RISK — `storage-durability`; ADR 0051 items 5, 6, 9; ADR 0053, 0095 items 1–2
- **Essence.** Each part of the WAL and restore is standard on its own:
  - rolling back an overlay on group commit;
  - a tail with `flushed_lsn` and a 32 MiB look-ahead;
  - epochs and an "end of recovery" record;
  - reusable segments;
  - zero pages;
  - `redo_start_lsn`.

  But bugs in their combination are caught only by failure simulation.
- **`errno` classification.** On APFS/btrfs `ENOSPC` probably comes from `fsync`, not from `write`. Then it is a "storage error" instead of "read-only" (not verified).
- **Recommendation.** Before the engine code, build a test file layer: it discards unflushed data, tears and reorders writes, and aborts the work at every system call. Add a "checkpoint equals recovery" check on random workloads. Record explicitly how `ENOSPC` from `fsync` is classified.

### 8. RISK — crates without an ADR are needed: ADR 0056 item 5, 0057 items 6, 11; 0066/0067 (busy port); 0080 item 4
- **Essence.** std 1.96 lacks several needed things:
  - `statvfs` (free space for the retention cap and the backup target);
  - I/O or thread priority ("low backup priority");
  - `posix_fadvise` / `F_NOCACHE`.

  `std::random` exists only in nightly, so the epoch requires `getrandom` (there is an ADR for it).
- **Port.** `tokio::net::TcpListener::bind` always sets `SO_REUSEADDR` (`mio/net/tcp/listener.rs:82`). On macOS/BSD a bind to 127.0.0.1:P can succeed while another process listens on *:P, and then code 7 is not issued (not verified).
- **`umask`.** `umask 0177` "before creating threads" is incompatible with an already running multi-threaded tokio.
- **Recommendation.** One decision on the `rustix` (or `libc`) crate plus `socket2` for bind. Remove "low priority" and "without evicting the OS cache": rate limiting and our own cache suffice. Create the socket in a directory with 0700 permissions and `chmod` after bind.

### 9. SIMPLIFY — `name-search`; ADR 0048 (G1/G2/G3, cost estimate), ADR 0045 item 3 (aggregates in D's inner nodes)
- **Essence.** Three candidate sources, a cost-based planner and a B+-tree with aggregates that survive split/merge, bottom-up load and snapshots are the most expensive part of search (XL). At 10^7 records the dictionary D weighs about 20 MB.
- **Recommendation.**
  - Keep D a plain tree, compute k/C by a bounded scan that stops at a threshold.
  - Start with G2 (walk of O) and G3 (leapfrog), add G1 based on profiling.
  - The result does not depend on the strategy, so strategies are cross-checked by a differential test. CUJ5/7/8 do not change.

### 10. RISK — ADR 0010 item 2, ADR 0039 items 3–6; OD-4 item 2
- **What is feasible.** Immutability of matches and order for all of v1 is technically feasible: `=2.4.0` on the whole `icu_*` graph, `Cargo.lock`, `--locked`, `check-pin`.
- **Where the risk is.** ICU4X explicitly does not support a frozen sort-key algorithm (`icu_collator-2.4.0/src/comparison.rs` ≈ 1735: "you should not expect ICU4X to support your using an older, frozen copy…"). So any security fix in `icu_*` or the transitive `zerovec`/`yoke` arrives only as a backport into a vendored copy.
- **Format label.** There is no public version constant, so the label has to be hardcoded.
- **Recommendation.**
  - Set up `vendor/` and `[patch]` right away, move the references (WordBreakTest, the order corpus, golden) into the product tests and run them in CI with product functions.
  - Make the H64 hash and the cursor fingerprint an algorithm with frozen test vectors.

### 11. RISK — ADR 0052 (D-27), Q-117; F-79
- **Essence.** `F_FULLFSYNC` on macOS flushes the entire drive cache. Checkpoint flushes (two for every batch of 128 pages) compete with the WAL flush on the acknowledgment path. The write p99 targets have not been measured.
- **Recommendation.** Create the next WAL segment in advance in the background, limit the checkpoint rate. Measure p99 on a bench (macOS and Linux) before fixing the numbers and before writing the core.

### 12. RISK — `web-ui-flows`, "Search by number (CUJ4)", "What to show after saving"; ADR 0035 item 6; Q-163; X-1
- **Essence.** One number can have up to ≈ 2.97 million records (by day from 1876 to 9999), and the UI re-reads the full history after every save, delete and conflict. In the worst case this is hundreds of MB of JSON: the V8 string limit and a DOM with millions of rows hang the tab (not verified).
- **Server.** The by-number stream in hyper is bounded by a buffer (`io.rs:586`), not by "socket acceptance" as the spec says.
- **Recommendation.**
  - Close Q-163: the UI guarantees responsiveness up to ~10^4 records of a number, beyond that — progressive rendering (a virtual list, without a new API operation).
  - In the server, word it as "the buffer is bounded" and set a small `max_buf_size`.

### 13. RISK — ADR 0045 items 4, 6; ADR 0041 item 5
- **Essence.** A name sort key of 200 characters reaches 8 513 bytes (verified: ﷺ × 200), twice as much after escaping.
- **Consequences.**
  - Overflow pages and comparison with reading the rest of the tail are a mandatory branch of the base B+-tree, not a rarity.
  - `OrderKey` in every posting of W for a 200-word name gives 200 copies.
- **Recommendation.** Build in order labels instead of `OrderKey` in postings from the start (Q-110), or at least prefix compression. Add tests on pathological names.

### 14. RISK — ADR 0044, 0058; Q-115
- **Essence.**
  - There is no page-merge rule for the B+-tree: with inserts mixed with deletes the size is not bounded by calculation, and the file does not shrink.
  - An index rebuild "next to the old one" at 10^7 needs another ~3.9 GB.
  - Writing it through the WAL in records of up to 4 MiB is expensive, and a direct write to `data` violates the rule "`data` is changed only by a checkpoint".
- **Recommendation.** Set merging at fill below ~25 %. Describe the rebuild write path explicitly: pages are written bottom-up directly into `data` with a flush, and only the root swap is logged.

### 15. SIMPLIFY — `server-runtime`, "Clean shutdown: HTTP side"; F-102–F-104
- **Essence.** `graceful_shutdown` in hyper (and `GracefulShutdown` in hyper-util) immediately closes idle keep-alive connections (`dispatch.rs:91–99`). So "a new request on an open connection gets 503 `server_stopping`" requires our own flag and our own connection teardown, and the race "the client writes into a closing connection" is unavoidable in any case.
- **Recommendation.** On shutdown, close idle connections without a response, and return 503 only to requests whose headers have already been received.

### Minor but verified remarks (outside the 15)
- **ALPN `["http/1.1"]`.** A client offering only `h2` gets a fatal `no_application_protocol` (`rustls server/hs.rs:114`). Then either do not set ALPN on the server, or fix the "HTTP/1.1 only" scenario.
- **Check order.** "The first invalid pair number → date → name → version" together with type errors cannot be done with `serde` derive. Manual parsing of `serde_json::Value` and table tests are needed.
- **`eprintln!` crashes the process** with a broken stderr (verified by running). Write all operator output with `writeln!`, ignoring the error.
- **SIGHUP.** Register the handler first thing in `main`.
- **Spec texts that will lead an implementer down a cancelled branch** (the spec itself takes precedence):
  - ADR 0094 item 6 ("retry is safe") versus ADR 0095 item 12;
  - ADR 0090 items 1–2 and 0091 item 1 describe the cancelled preliminary `by-number`;
  - CUJ6/CUJ12 in `user-journeys` say "version from the result row", but after Q-128 the version is only in the root of the results.
- **Q-160 (OD-6).** Repeating an add without a date after a lost response gives `number_known`. This is an owner decision, but it confuses the UI after "outcome unknown". If the owner agrees: answer `unchanged` to an identical record on 1876-03-07.

## Effort estimate

Relative complexity: S < M < L < XL. Share is a rough share of the total work, without UI design.

| Component | Complexity | Share | Why |
|---|---|---|---|
| Test file layer: failure injection, power loss, reordering | L | 7 % | Without it CUJ14/CUJ16 cannot be verified (item 7) |
| Page format, CRC-32C, key encoding, overflow | M | 4 % | Standard; long keys are mandatory (item 13) |
| B+-tree: insert, delete, split/merge, cursors, keys up to 64 KiB | L | 7 % | Shared by R, O, W, D, H |
| Multi-version page cache: version chains, retention, `retain`, eviction, revocation | XL | 11 % | The core of concurrency (items 2–4) |
| Writer with group commit, overlay rollback, number version and generation (X-5) | L | 6 % | Tied to the WAL and recovery |
| WAL: segments, tail, epochs; checkpoint with `dwb`; recovery | L | 8 % | Correctness only through simulation (item 7) |
| `control` / `generation` / `LOCK`, "read-only" and "storage error" states, free pages | M | 4 % | Many states, simple code |
| Backup (engine) | M–L | 3 % | M per item 4, L with current retention |
| Index rebuild: bottom-up build, root swap | L | 3 % | Item 14 |
| Unicode: normalization, folding key, own UAX #29, the "word" predicate, sort keys | M | 4 % | Prototypes in `research/` are ready |
| Pinning ICU: vendor, references in CI | M | 2 % | Process, not code (item 10) |
| Word-prefix search: planner, leapfrog, cursor; exact search; search by number | L (XL in full) | 7 % | Item 9 |
| Connection layer: accept, registry, classes U/I/A, deadline wrapper, staged closing, TLS | L | 8 % | Absent in hyper (items 5, 6, 15) |
| HTTP service: check order, errors, by-number stream, bridge to storage (pools, deadlines, cancellation) | M | 6 % | Many rules, all inside the service |
| Access: three token kinds, SHA-256 + `ct_eq`, per-token limit, atomic revocation | S | 1 % | Crates are ready |
| Operator: `settings.toml`, messages, a Unix-socket channel with peer check, CLI commands, codes, offline mode | M | 5 % | Volume, not difficulty |
| Start and stop: signals, deadlines, exit codes | M | 2 % | — |
| UI: shell, embedded resources, CSP, request layer, flows on ES modules without a build (~2k lines) | L | 6 % | State machine for CUJ2/3/6/12 |
| E2E harness: temporary database, failure proxy on hyper, Playwright, browser matrix | L | 4 % | Safari needs a macOS runner |
| Bench measurements (flush, p99, start at 10^7) and `npdb-fill` | M | 2 % | Mandatory before fixing the numbers |

Total by area: storage ≈ 53 %, server, access and operator ≈ 22 %, search and Unicode ≈ 13 %, UI and e2e ≈ 10 %, bench ≈ 2 %. The risk is concentrated in the multi-version cache, the WAL with recovery, and the connection layer.
