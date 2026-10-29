## Scope of the area

Area 3 defines the programming interface and the server process: API v1 (HTTP + JSON) for the UI, local programs and network services, access (tokens, permissions, TLS, loopback protection), request and
connection limits, a single table of codes and statuses for all refusals, a single request check order, startup and clean shutdown, display of storage states. The semantics of data and operations are accepted in
area 1 (`records`, `write-operations`, `name-search`, `number-search`, `result-ordering`), storage in area 2 (`storage-*`); they do not change. Outside the area, with no requirements here: operator
commands, message channels and exit codes (area 4), the look and texts of the UI (area 5); everything that area 3 requires from them is collected in the section "Requirements for other areas". HTTP and JSON are the crates
hyper and serde_json (owner decision, brief §3); other crates only with an ADR and a cargo deny check (ADR 0017 item 3, ADR 0069).

The API requirements accumulated by areas 1 and 2 are met (`openspec/changes/archive/2026-10-27-add-data-model/design.md`, "Server and API"; `.../2026-10-27-add-storage-engine/design.md`,
"Requirements for other areas"): the input size cap in the right units and the kind of refusal (ADR 0064, Q-101, Q-102); the date format, the expression of (field, reason) pairs, the version as a string, conflicts (ADR 0060,
0062); the by-number stream with a completeness marker and the cursor with a fingerprint and vectors (ADR 0061); separate refusals "read-only", "storage error", "snapshot revoked", "server starting" (ADR 0063);
the by-number stream is exempt from the 10 s timeout (ADR 0071); the database state is checked before the operation checks (ADR 0063 item 7).

## Map of capabilities, ADRs and CUJs

| Capability | ADR | CUJs and requirements |
|---|---|---|
| `http-api` (30 requirements, 102 scenarios) | 0060–0064, 0062 item 8, 0063 | CUJ1–CUJ9, CUJ11, CUJ12, CUJ14, CUJ16; X-5 |
| `api-access` (22, 58) | 0065–0069 | CUJ7–CUJ10, CUJ13 |
| `server-runtime` (33, 81) | 0070–0074, 0063 | CUJ1, CUJ4, CUJ5, CUJ7–CUJ9, CUJ12–CUJ16; X-1, X-2, X-5 |

In total 85 requirements and 241 scenarios in three delta specs; ADR 0060–0074 are 15 decisions, of which 0073 is merged with 0063 (Superseded by 0063). Accepted [OURS] — 0067, 0069, 0070, 0074, 0063 (items 1–7),
0060 (except item 7), 0061 (except item 5), 0062 (items 1–4, 6), 0064 (except item 4), 0065 (except items 3, 8), 0066 (item 1 regarding the listener composition, item 3 except the validity period, item 7), 0072 (except the values
in item 2); Proposed [DEFAULT] — 0060 item 7, 0061 item 5, 0062 items 5, 7, 8, 0063 item 8, 0064 item 4, 0065 items 3, 8, 0066 items 1 (narrowing of Q-27), 2–6, 0068 items 1–3, 0071, 0072 item 2 (values). This area changes none of the owner decisions
([USER]), except the refinement of Q-27 (OD-3) with the default D-46; hyper and serde_json are an owner decision. The requirements of areas 1 and 2 are not changed
(no MODIFIED delta).

Boundaries between capabilities. `http-api` is everything the client sees in a request and response: paths, values, the error body, codes and statuses, failure headers, the check order, size caps;
`api-access` is who may do what: tokens, permissions, listeners, TLS, Host, the per-token request limit; `server-runtime` is how the process serves requests: threads, connections, timeouts, slot protection,
startup and shutdown. The single source of codes and statuses is `http-api`, "Client-failure codes and statuses", "Server-failure codes and statuses"; the single check order is `http-api`, "Request check order" and "…after access" (mechanism and rationale — ADR 0063
item 7); `api-access` and `server-runtime` name the refusals in Russian and refer to them. Shared objects: the token file and the UI token are `api-access`, the moment of creation is `server-runtime` ("Startup order");
size caps are `http-api`, their enforcement and delivery of the refusal are `server-runtime`.

Dependence on owner decisions and the [DEFAULT] package: the default values are collected in `docs/open-questions.md`, "Area 3: defaults" (D-37…D-53).

## Single decisions at the seams

| No. | Subject | Single value | Where defined, who refers |
|---|---|---|---|
| 1 | Codes and statuses of refusals | One table: 400 `invalid_request`, `invalid_input`, `invalid_cursor`; 401 `unauthorized`; 403 `forbidden`, `host_not_allowed`; 404; 405; 409; 413 `payload_too_large`; 415; 429 `rate_limited`; 431 `headers_too_large`; 500 `storage_error`, `internal_error`; 503 `read_only`, `server_starting`, `server_stopping`, `snapshot_revoked`, `request_timeout`, `too_many_connections`; 504 `write_outcome_unknown` | `http-api`, ADR 0062 items 4, 8; referred to by `api-access`, `server-runtime`, ADR 0063–0074 |
| 2 | Check order | Connection limit → header parsing (400, 431) → startup and shutdown → Host → route and method → token → limit → permissions → body (415, 413, 400) → database state (write) → values → operation | `http-api`, ADR 0063 item 7; referred to by ADR 0065 item 6, 0068 item 4, 0070 item 5, 0074 item 3 |
| 3 | `server_starting` | For the whole period from opening the listeners to working mode, before the route and token check; the tokens of this run are not created yet | `http-api`, `server-runtime`, ADR 0063 item 6, 0074 items 2–3 |
| 4 | Token file | Written before working mode, after the listeners are taken; requests in the window get `server_starting` without a token check | `server-runtime` ("Startup order"), `api-access`, ADR 0065 item 2, 0074 items 1, 6 |
| 5 | Body cap | 262,144 bytes (256 KiB), refusal 413; brief §4 (32 KiB) revised (D-43) | `http-api` ("Body size cap"), ADR 0064 items 1, 4, 0071 |
| 6 | Target and header cap | One measure — the request line with headers, 16,384 bytes; one refusal 431; there is no separate code for the target | `http-api`, ADR 0064 item 1, 0071, 0062 item 8 |
| 7 | UI and file tokens | Accepted only on the loopback listener; on the network one — "unauthorized" even from a loopback address (D-46) | `api-access`, ADR 0065 item 6, 0066 item 1 |
| 8 | Slot protection | Protection period for an anonymous connection 1 s, at most 16 anonymous ones per network address; the guarantee holds at a flow below 256 per second per pool (D-53) | `server-runtime`, `api-access` ("Token brute-force protection"), ADR 0072 item 2 |
| 9 | ADR on storage states | ADR 0063 is the only one; 0073 is Superseded by 0063 | ADR 0063, 0073 |

## Fix pass after the Opus review

The Opus review of area 3 (4 BLOCKER, 6 contradictions) was resolved in one fix pass; there is no repeat review. The accepted semantics of areas 1 and 2 and the owner decisions did not change, there is no MODIFIED delta.

- R-1: a single body cap — 256 KiB (D-43) in `http-api`, `server-runtime`, ADR 0064 items 1, 2, 4 and 0071; the revision of the brief §4 value is [DEFAULT], put to the owner; the `records` scenario with edges of 20,000
  characters fits (251,600 bytes when writing `　` < 262,144), `records` is unchanged.
- R-2: one measure and one refusal for the target and headers — the request line with headers no more than 16 KiB, 431 `headers_too_large`; 414 and `uri_too_long` removed; the worst valid search request is about 12,900
  bytes (Q-141: a request edge that does not fit into the cap).
- R-3: a single check order, access before the body: `http-api` ("Request check order"), ADR 0063 item 7; `server-runtime` and ADR 0070 item 5 refer to it.
- R-4: `server_starting` comes first after the connection limit and header parsing, before the route and token, for the whole startup period; the earlier "tokens are checked under `server_starting`" is rejected (there are no tokens
  of this run yet) — `http-api`, ADR 0063 items 6–7, 0074 item 3.
- R-5: the token file and the UI token are created before working mode, not "before accepting requests": `api-access`, ADR 0065 item 2 are aligned with the order of ADR 0074 item 1 (listeners → checks and rebuild → tokens and
  file → working mode → UI address).
- R-6: the order "token → limit → permissions" in all three capabilities and ADRs (0063 item 7, 0065 item 6, 0068 item 4, 0070 item 5); the "forbidden" refusal consumes the limit.
- R-7: the code table is extended with the kinds of access, limit, time, shutdown and connection refusals; references to a nonexistent protocol capability are replaced with `http-api` (ADR 0062 item 8, D-41); a new requirement "Failure
  headers".
- R-8: ADR 0063 and 0073 are merged into 0063 (kinds of refusals, correspondence of conditions, check order, `Retry-After`); 0073 is Superseded by 0063.
- R-9: the narrowing of owner decision Q-27 (the file and UI token is not accepted on the network listener even from loopback) is turned into [DEFAULT] D-46 with an alternative; a note in `api-access` and ADR 0066 item 1.
- R-10: slot protection is backed by a mechanism — a protection period of 1 s for an anonymous connection and at most 16 anonymous ones per network address (ADR 0072 item 2, D-53); the guarantee "legitimate clients do not suffer"
  is honestly narrowed to a flow of anonymous connections below 256 per second per pool (`server-runtime`, `api-access`); the narrowing relative to the unconditional reading of CUJ8 is visible to the owner ([DEFAULT]).

## Requirements for other areas

### Administration (area 4)

- The files `local.token` (the name is an implementation decision), `tokens` and the certificate with the key are in the settings directory, permissions 0600; network clients' tokens are outside the backup (OD-4, item 9); the settings of the network listener
  (enable, addresses and ports, certificate and key paths), the loopback port, the request limit and server limits (ADR 0071; the body cap not below 256 KiB), the number of threads.
- The issue and revoke token commands deliver the change to the running server (Q-60): after the response "done" the next request sees it; the list composition and the identifier for revocation are Q-40; "last
  used" is not stored.
- The channel and format of server messages (Q-66): the content is ADR 0074 item 4; start refusal reasons (no database, no certificate, key does not match, port occupied, no ::1, corruption); warnings
  (expired certificate, permissions on the key file); counters of "unauthorized" refusals and handshake drops (Q-133). The UI address with an embedded token is only on standard output, not in the log; when started
  under a service manager, document that the address will appear in its log.
- Exit codes: 0 — clean shutdown; nonzero ones distinguish start refusals (database not found, busy, certificate, port bind, corruption).
- The forced-termination deadline is at least 60 s (clean shutdown ≈ 35 s, ADR 0074 item 9); the stop command equals SIGTERM; a clean shutdown cancels backup; SIGHUP (Q-136).
- The `read_only` and `storage_error` errors give the operator details (`errno`, file, offset) only in server messages, not in the API response.

### UI (area 5)

- The token is taken from the link fragment, removed from the address bar and passed in the `Authorization: Bearer` header; cookies are not used; text from the database is shown only as text (no HTML),
  a strict Content-Security-Policy; the UI shell is served without a token and data; the behavior of an open tab after a restart is Q-39, keeping the sign-in is the tab or memory.
- The CUJ16 texts are chosen by `code`: `read_only` («Сохранение не выполнено: база доступна только для чтения» — Saving failed: the database is read-only), `storage_error`; the CUJ12 conflict by `version_conflict` (the current version is in the body); the record
  date is taken from the results or from the add response (`record.date`). There is nothing to poll the storage state with.
- The `server_starting` and `server_stopping` responses are temporary: the UI shows that the server is unavailable and retries; this is not a storage state and does not violate "do not show in advance".
- `unauthorized` after a server restart — the old link is invalid (CUJ13); during startup `server_starting` arrives, not `unauthorized`.

### Storage (area 2)

- The reader checks the cancellation token between page reads, the snapshot is released by a guard (ADR 0070 item 7); the engine distinguishes the request class (short read or stream) and tells the API layer
  "completed fully" (`storage-snapshots`, "By-number results are streamed on one snapshot"); a writer panic is a crash termination of the process (ADR 0070 item 8).
- Short reads: `snapshot_revoked` before sending starts; the by-number stream is written as the snapshot is read. The latency targets D-23 are not set for `page_size` 100 (Q-126).

### Build and tests

- Crates (ADR 0069, cargo deny): `rustls` 0.23, `tokio-rustls` 0.26, `rustls-pki-types`, `sha2`, `subtle`; the choice of the `rustls` provider and the `subtle` license are at build time; hyper and serde_json.
- Tests per task 4.1 of `tasks.md` (golden test of cursors, cap boundaries, check order, framing on a raw socket, displacement, timeouts, shutdown); checks of hyper behavior (Q-138).

## Verified facts

`facts.md`, F-86…F-108: RFC 9110, 8259, 4648, 6750, 6265, 6585, 9112; rustls, tokio-rustls, rustls-pki-types, sha2, subtle; hyper, hyper-util, tokio, http-body-util; Secure Contexts, Referrer Policy. Not
verified (marked in `facts.md`): browser behavior with preflight (F-98), `with_single_cert` with a mismatched pair (F-93), hyper behaviors (F-108), the `subtle` license, cargo deny.

## Open questions

`docs/open-questions.md`: 18 questions closed (section "Closed in area 3"), 3 narrowed, new Q-126…Q-141 (14 (T), 2 (O): Q-128, Q-135), [DEFAULT] D-37…D-53 await the owner as a package.
