# 0064. Input size cap X and API compatibility within v1
- Status: Accepted ([OURS]: items 1–3, 5, 6); Accepted (OD-6) ([DEFAULT]: item 4 — cap values, D-43)
- Label: [OURS] (items 1–3, 5, 6); [DEFAULT] (item 4)
- Date: 2026-10-27
- CUJ: CUJ1, CUJ3, CUJ5, CUJ9, CUJ11

## Context
Q-101: "4 × 200 = 800 bytes" is a number of scalars, not bytes. Q-102: the unconditional "however many spaces at the edges" in `records` versus the protocol
cap. Brief §4: a 32 KiB body — accepted for "loopback only" and does not fit the `records` scenario with edges. ADR 0013: in v1 fields and codes are only
added; Q-54: retry after a connection drop.

## Decision
1. [OURS] Q-101: a strict estimate of a valid name (200 NFC characters, up to 4 input scalars per character — F-20, up to 4 UTF-8 bytes per scalar): 200 × 4 × 4 =
   3,200 bytes of raw UTF-8. In JSON the client may escape every scalar: `\uXXXX` is 6 bytes, outside the BMP as a pair 12 bytes, in total
   up to 200 × 4 × 12 = 9,600 bytes. The caps (one value each, measured on the wire, no decompression, `Content-Encoding` — 415): the body — 262,144 bytes
   (256 KiB), the request line together with headers — 16,384 bytes (16 KiB). Body: a valid name in the worst record with edges of up to 20,000 whitespace
   characters each — 9,600 + 2,000 (other members) + 40,000 × 6 (`　`) = 251,600 < 262,144. Head: the worst valid search request — 3,200 bytes,
   in percent-encoding 9,600 characters, plus a cursor 1,126, parameter names ≈ 100 and headers ≈ 2,000 (a 64-character token, Host and others) ≈ 12,900 < 16,384,
   margin ≥ 3,400. Exceeding — 413 `payload_too_large` (body) and 431 `headers_too_large` (head; there is no separate cap and code for the request target — one
   measure and one refusal, ADR 0062 item 8); the server does not read the rest and closes the connection after the response (P-9: send the response before closing). The cap is checked
   after access (ADR 0063 item 7), before `records`, values are not parsed.
2. [OURS] Q-102: X = 256 KiB fits the `records` scenario "Many spaces at the edges" (20,000 whitespace characters on each edge, up to 240,000 bytes in the
   `　` form), the `records` value does not change. For a search request (GET, query string) the head cap applies: a request with edges that does not fit in
   16 KiB — 431; `records` stipulates that the byte cap of input is set by the API area (Q-141).
3. [OURS] Compatibility (CUJ11): in v1 it is allowed to add response members (the number stream — ADR 0061 item 4), error codes and reasons, optional
   request members and parameters, new paths; it is forbidden to change or remove paths, members, their types and meaning, the code and status of a situation (ADR 0062 item 5), the shape of the version and cursor.
   The client must ignore unknown members, unknown `reason`, `hint`, `code` (by status class).
4. [DEFAULT] (D-43) A body cap of 256 KiB instead of the brief §4 32 KiB (a revision of the brief; the same cap — ADR 0071) and a head cap of 16 KiB (the brief
   has none). Rationale: 32 KiB does not fit the accepted `records` scenario (edges of 20,000 characters, 240,000 bytes in the `　` form); 256 KiB is the minimal
   power of two that fits it with margin. Alternatives: keep 32 KiB — then `records` loses the scenario or area 1 introduces a ceiling on edges
   (changes accepted semantics, rejected); 64 KiB — fits edges only up to ≈ 4,500 characters on each side; 1 MiB and more — 256 KiB × 512 connections is already 128 MiB
   in the worst case. The value is put to the owner as a package.
5. [OURS] Q-54: there are no idempotency keys (ADR 0026 rejected an operation log). Add and rename to the same name are idempotent when the number's records are unchanged;
   a retry of rename and delete with the earlier version after success — 409 `version_conflict`; the client finds out the outcome with `by-number`, comparing the history. One's own success cannot be told from
   another's identical operation by version (ADR 0026 item 6). `POST` over HTTP is not idempotent (RFC 9110 §9.2.2, P-12); the safety of a retry is by semantics.
6. [OURS] The server neither accepts nor interprets `Content-Encoding`, `Expect: 100-continue` is handled by the transport (P-11); these and other
   transport details (smuggling P-8) are on hyper, checked by boundary tests.
7. **Changed by OD-6 (2026-10-28).** D-43 decided by the owner (OD-6): the body cap is 32 KiB (32,768 bytes, as in brief §4), not 256 KiB; the 16 KiB head cap — D-43 accepted. The calculation of item 1 ("251,600 < 262,144") and of item 2 (Q-102: edges of 20,000 characters) no longer apply: a valid name without edges is ≈ 11,600 bytes of body, edges in the `　` form fit up to ≈ 1,700 characters on each side; a body over 32 KiB — 413 `payload_too_large` before parsing, `records` does not see it; the 200-character name cap does not change. The test from "Consequences" — "20,000 edge spaces pass" — is replaced: 413. The alternative "keep 32 KiB" (item 4) is chosen.

## Alternatives considered
- X = 32 KiB (brief §4): rejected, see item 4. Two head caps (414 for the target, 431 for headers): rejected — one measure and one refusal are easier to verify,
  and a search request fits the common measure with margin.
- X = 4 × 200 × 4 without accounting for escaping: rejected — `\uXXXX` is permitted by JSON.
- X equal to the name's byte count 800: rejected (Q-101, the Hangul counterexample).
- Truncating or ignoring edges above the cap: rejected — silent corruption of input.
- Strict refusal on unknown fields to catch typos: ADR 0060 item 7.

## Consequences
- Q-101, Q-102, Q-54 (for the API), Q-79 (together with ADR 0062) are closed.
- Product test: a name of 200 Hangul syllables in NFD with `\uXXXX` encoding passes; 200 × 4 outside the BMP as pairs with 20,000 edge spaces (`　`) passes; a 300 KiB body —
  413, a 16,385-byte head — 431.

## Sources
- `docs/source/brief.md` §4; ADR 0013, 0021, 0022, 0026; facts F-20, F-5; `docs/open-questions.md` Q-52, Q-54, Q-101, Q-102; RFC 8259 §7–8 (F-87); RFC 9110 §9.2.2; ADR 0071.
