# 0065. Access tokens: UI, local programs, network services
- Status: Accepted ([OURS]: items 1, 2, 4–7); Accepted (OD-6) ([DEFAULT]: item 3 — one token or two, Q-38, D-44; item 8, D-45)
- Label: [OURS] (items 1, 2, 4–7); [DEFAULT] (items 3, 8 — Q-38, D-44, D-45)
- Date: 2026-10-27
- CUJ: CUJ7, CUJ8, CUJ9, CUJ10, CUJ13; item 1.2 of the original wording

## Context
The owner's decisions are set by ADR 0011 and 0012: the UI token in the link and the local-program token in a file change at every start
and are accepted only from loopback; the network token is permanent, with a client name and permissions, kept next to the settings, not included in the
backup. Not set: whether the UI and the programs share the same token, the location and permissions of the file, storing the network token on
disk, the comparison method, the kinds of refusals (Q-38, Q-50; pitfalls P-7b, P-7c, P-7d).

## Decision
1. [OURS] The UI token: 32 random bytes (source — `getrandom`, F-78), written in hexadecimal (64 characters), new
   at every start, held in memory, not written to disk. Delivery — ADR 0067 item 5.
2. [OURS] The local-program token: 32 random bytes, hexadecimal, new at every start. Kept in
   the `local.token` file of the settings directory (the name is an implementation decision, area 4 takes it from the settings), one line. The file
   is created with permissions 0600 at once (`open` with a mode, not `chmod` afterwards), written to a temporary file nearby and replaced by
   `rename` (atomic for readers, P-17 — nothing is promised on power loss: the token changes at start anyway). Start
   order (ADR 0074 item 1): take the listeners (ADR 0067 item 2) → checks, recovery, rebuild → create the UI token and write the file →
   working mode → print the link. Before the working mode every request gets "server starting" without a token check, so
   there is no window in which tokens are checked while the file still holds the old one; a client that got "unauthorized" already in the working mode re-reads the
   ready file. On clean shutdown the file is
   not deleted (the old token is invalid after the next start; deletion would create a "no file" window on a fast
   restart).
3. [DEFAULT] The UI and local-program tokens are two different tokens with the same permissions (full), both from loopback, both without
   a limit (Q-24). The value: two. Rationale: the UI token is printed to the terminal and lands in the browser history; the file token
   is protected by OS permissions; the separation does not let a token leaked from the link serve as a key to the file and vice versa, and allows
   distinguishing the kind in the log. Alternative: one token (the file holds the token from the link) — simpler, but a link leak exposes the
   programs' token. A divergence from the owner is impossible: the brief does not require one token (Q-38). The value (D-44) is put to the owner as a package.
4. [OURS] The network token: 32 random bytes, hexadecimal; the secret is shown to the operator once at
   issuance. The stored record: client name, permissions (`read` | `read_write`), SHA-256 of the token (32 bytes), issuance time, a
   "revoked" marker and revocation time (the record of a revoked token remains so that the name can be issued again and a revoked one does not
   come back to life). Storage — the `tokens` file in the settings directory, permissions 0600, not included in the backup or in the database directory (ADR 0012 item 8,
   OD-4, item 9; P-7d). Changes — by atomic file replacement. A hash, not a weak stored form: the token is a random string
   of high entropy, a slow password hash (Argon2 and the like) is not needed for it — brute force is impossible by entropy
   (2^256), and every request would pay extra time (a calculation, not a fact about the outside world). A file leak does not give tokens.
5. [OURS] Comparison (P-7b): the presented token is hashed with SHA-256 and compared with the hashes by a constant-time function
   (`subtle::ConstantTimeEq`) over all current records without early exit; UI and file tokens — the same way. For a
   presented value of wrong shape (not 64 hexadecimal characters) the hash is computed over the same bytes; the response is the same.
6. [OURS] Access refusals (kinds; codes, statuses and headers — ADR 0062 item 8, spec `http-api`): "unauthorized" (a token not presented or not recognized:
   wrong, unknown, stale, revoked, a token of another kind, a file or UI token not on the loopback listener (ADR 0066 item 1, D-46),
   a network token not on the network listener); "no permissions" (the token is recognized, a write operation with the "read" permission); «слишком много запросов, повторите позже» ("too many requests, retry later") (ADR 0068); "host not allowed" (ADR 0067). One kind for all "not recognized" — the response does not reveal
   whether the token existed (a consequence of CUJ7 and Q-50 for an unknown token). The check order is single, ADR 0063 item 7 (Host → route and method →
   token → limit → permissions → body → database state → area 1 checks). The header
   `Authorization: Bearer <token>` is the only way; a query parameter and cookie are not read (RFC 6750 §2.3, F-90; P-1, P-5).
7. [OURS] The token is checked on every request against the current set of tokens, not on opening a connection: revocation takes effect from
   the next request, including over an open connection; a request already executing completes (ADR 0012 item 6). Tokens
   (of any kind) are not written to the log and messages; a network token appears in the log by client name.
8. [DEFAULT] (D-45) Issuance and revocation are delivered to the running server by an operator command (the channel — Q-60, area 4); until the channel is
   settled, the specification requires only the observable: after the command's "done" response the next request already sees the change.

## Alternatives considered
- The UI token in a cookie: rejected, cookies are not isolated by port (RFC 6265 §8.5, F-91; P-1).
- The token in the query string: rejected, it lands in logs and Referer (RFC 6750 §2.3; P-5).
- Store network tokens in the clear: rejected, a file leak would yield working tokens (P-7d).
- Argon2/bcrypt for tokens: rejected, excessive for random 256 bits.
- Blocking by the number of failures: see ADR 0067 item 3.

## Consequences
- Area 4: the token commands work with the `tokens` file; the composition of the list and the identifier for revocation — Q-40; "last
  use" is not stored (a disk write on every request is unacceptable) — only in memory, if needed.
- Crates: ADR 0069.
- Closes in the access specification part: Q-38 (tokens, file, permissions; keeping the UI sign-in — area 5), Q-50 (kinds of refusals).

## Sources
- ADR 0011, 0012; `docs/owner-decisions.md`, OD-3 (Q-8, Q-9, Q-10, Q-24, Q-27), OD-4 (items 4, 9).
- Facts: F-90, F-91, F-92, F-94; F-78 (`getrandom`).
- `pitfalls.md` P-1, P-5, P-7b, P-7c, P-7d.
