# Final review: the "server" group (http-api, api-access, server-runtime)

Checked: all three capabilities in full, overlaps with records, write-operations, storage-durability, operator-lifecycle,
web-ui-shell, user-journeys; ADR 0060, 0065, 0066, 0072; facts.md F-101...F-108. The reference vectors for the cursor
and fingerprints (http-api, "Query fingerprint and reference vectors") were recomputed and match; the cursor cap of
1126 characters matches.

Total: 9 items (BLOCKER: 6, contradiction: 3).

## 1. BLOCKER: JSON body for refusals that hyper generates itself
- **Where:** http-api, "Error body", "Client-failure codes and statuses" (scenario "Invalid request"), "Header size
  cap"; server-runtime, "Message boundary substitution is ruled out", "Request size cap".
- **Essence:** the specification requires a JSON `{code, message}` with `invalid_request` (unparseable HTTP/1.1, TE+CL)
  and `headers_too_large` (431) for every refusal. But these errors are caught by the hyper parser
  (`max_buf_size`/`max_headers`, parse errors) before the service is called. After that hyper writes the response
  itself, consisting only of a status, with no body. In addition, hyper has a default cap of 100 headers (F-101): then
  431 would also be returned for a request under 16 KiB, although the specification says there are no other caps. This
  is not verified in facts.md (F-108 "not verified"), and hyper is set by the brief.
- **Fix:** for refusals at the HTTP parsing level (400, 431, and also closing on a timeout) require only a status. A
  code in the body is then optional, and the client interprets such a refusal by status class. Or explicitly require our
  own pre-parse before hyper and verify it in advance with a fact. State the header-count cap explicitly or lift it via
  `max_headers`.

## 2. BLOCKER (underspecification against OD-3, Q-27): network token on the loopback listener with the network listener enabled
- **Where:** api-access, "UI and file tokens are accepted only on the loopback listener", "Network listener over HTTPS
  is off by default" (scenario "Network off by default").
- **Essence:** api-access forbids a network token on the loopback listener (HTTP without TLS) only in the scenario with
  the network off. There is no requirement for the network on. So an implementation may accept a network token in
  cleartext on loopback, which contradicts OD-3 Q-27 ("a network token only over HTTPS"). ADR 0066 item 1 says "not
  recognized, always", but that was not carried into the specification.
- **Fix:** add to the requirement the rule: the loopback listener accepts only UI and file tokens, and a network token
  on it gets "unauthorized" under any network listener setting. Add a scenario for the network on.

## 3. Contradiction: whether an "unauthorized" refusal closes the connection
- **Where:** api-access, "A stream of invalid tokens does not displace legitimate clients" ("every refusal closes the
  connection"); http-api, "Failure headers" ("Closing the connection after a refusal is determined by
  `server-runtime`"); server-runtime, "Refusal before body read reaches the client", "Slot protection from anonymous
  clients".
- **Essence:** api-access assumes that 401 always closes the connection. The slot protection (class U) relies on this,
  and so do ADR 0072 items 1 and 3. But server-runtime, which http-api refers to, does not require this: there only
  connections with an unread body are closed. Hence two implementations: in one 401 keeps keep-alive, and a stream of
  invalid tokens flows over a single connection; in the other it closes. Moreover, a response with `Connection: close`
  and one without are distinguishable by headers, while the requirement "one response for any unrecognized token"
  promises that responses are identical.
- **Fix:** in server-runtime state directly that 401 always closes the connection after the response (with
  `Connection: close`, graceful close), and list which other refusals close the connection.

## 4. Contradiction: 32 connections per address versus eviction of 16 anonymous ones from the same address
- **Where:** server-runtime, "Connection limits" (scenario "One network address": the 33rd gets "too many
  connections") and "Anonymous connections from one network address" (a new U beyond 16 evicts the oldest U of that
  address).
- **Essence:** at the moment of acceptance a new connection is always class U. Suppose an address already has 32
  connections, 16 of them class U. By the first requirement the 33rd is refused; by the second the oldest U is evicted
  and the new one is accepted. The specification does not say which rule takes precedence, and the outcomes are
  observably different.
- **Fix:** set an order. For example: with 16 U from the same address, eviction first (so the total does not grow), and
  refusal by the cap of 32 only if the address has no U connections. Add a scenario for this case.

## 5. BLOCKER: content of the local-program token file is not defined
- **Where:** api-access, "Local-program token in a file"; operator-lifecycle (the name `local.token`).
- **Essence:** CUJ7 is a contract for an external program: it reads the file and sends `Authorization: Bearer <token>`.
  But the file format is not defined in the specification: 64 hexadecimal characters or another alphabet, whether there
  is a trailing newline, the encoding. Only the UI token format (web-ui-shell) and the network token secret
  (operator-control) are defined. If a program sends the token together with the newline, one implementation accepts it
  and another rejects it.
- **Fix:** record the format in api-access: "exactly 64 lowercase hexadecimal characters and one LF, ASCII" (per
  ADR 0065 item 2). Add a rule for the client: strip whitespace characters at the edges.

## 6. BLOCKER: the Host check rejects a browser on port 80 (and 443 in the future)
- **Where:** api-access, "Loopback protection: Host check"; operator-lifecycle (port from settings: any 1...65535).
- **Essence:** only Host values of the form `127.0.0.1:<port>`, `[::1]:<port>`, `localhost:<port>` are allowed. On a
  standard port the browser sends Host without a port (`127.0.0.1`), and the request gets "host not allowed". Then the
  UI does not work from the printed link (CUJ13) under a valid setting.
- **Fix:** allow Host without a port when the listener's port equals 80. Or forbid port 80 in settings with a start
  refusal.

## 7. Contradiction: edges of a search query: the records promise versus the 16 KiB header cap
- **Where:** records, "Input size before normalization and request body cap" (the query length is judged on the
  normalized text, however many spaces are at the edges, "if the input fits the 32 KiB body cap"); http-api, "Header
  size cap"; server-runtime, "Request size cap".
- **Essence:** a search query is passed in the query string (GET), and the 16 KiB header cap applies to it, not the body
  cap. Example: a word-search query with 2,000 U+3000 characters at the edges. In percent-encoding that is about 18,000
  bytes, that is, less than 32 KiB but more than 16 KiB. records promises to process such a query, while http-api
  answers 431.
- **Fix:** in records, for a search query refer to the 16 KiB header cap (431 before parsing). In http-api add a
  scenario "query edges exceed the header cap -> 431".

## 8. BLOCKER: `"date": null` on add is not defined
- **Where:** http-api, "Adding a record", "Date representation".
- **Essence:** for rename it is stated that `null` is equivalent to an absent field (`missing`). For add it is not
  stated whether 1876-03-07 is substituted and whether the `number_known` rule applies (OD-6, Q-160), or it is
  `invalid_input` with reason `wrong_type`. Two reasonable implementations give different responses: 201 or 409 versus
  400 (CUJ1, CUJ2, CUJ9). `null` for `number`, `name` and `expected_version` is likewise undefined.
- **Fix:** one general rule for `null` in a known member. For example: "`null` is equivalent to an absent member"; for
  add, that means without a date, with substitution per OD-6.

## 9. BLOCKER: outcome of write requests begun at the moment of the stop signal
- **Where:** server-runtime, "Clean shutdown: HTTP side"; http-api, "Snapshot revocation, server start and shutdown".
- **Essence:** it is stated that "accepted writes" complete and a "new request" gets `server_stopping`. It is not stated
  what to do with two kinds of writes: a request whose headers had been read by the time of the signal but whose body is
  still being read, and an operation in the queue not yet taken by the writer. They can be executed or rejected with
  `server_stopping`. It is also unclear whether a write is considered "accepted" when the HTTP handler accepted it, or
  only when the writer took it.
- **Fix:** define: a write not yet taken by the writer (including one with an unread body) gets 503 `server_stopping`
  and is not executed; one taken by the writer is acknowledged or rejected.
