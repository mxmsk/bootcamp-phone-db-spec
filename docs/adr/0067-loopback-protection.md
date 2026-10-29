# 0067. Loopback protection: Host, busy port, brute force, UI token delivery
- Status: Accepted
- Label: [OURS]
- Date: 2026-10-27
- CUJ: CUJ7, CUJ8, CUJ13

## Context
Loopback is reachable by any process on the machine and by a browser page of any site (through a name that is overridden to
127.0.0.1). Pitfalls: DNS rebinding (P-3, P-4, P-7e), cookies without port isolation (P-1), a token in the URL (P-5), the port being taken
by another user (P-7), token brute force (P-7c).

## Decision
1. [OURS] The Host check (P-3): the loopback listener serves a request only if the Host header equals `127.0.0.1:<port>`,
   `[::1]:<port>` or `localhost:<port>` (the name is case-insensitive, the port is this listener's port). Otherwise "host not
   allowed" (`host_not_allowed`, 403) before the token. Source: "Web servers can reject HTTP requests with an unrecognized Host header" (F-96). 127.0.0.1 and
   localhost are different origins (P-4); all three Hosts are acceptable, the UI opens from a link with 127.0.0.1. A missing Host — 400 `invalid_request` (RFC 9110 §7.2, F-97).
   The check accepts only these values, not "any local address", so that a foreign name bound to 127.0.0.1 is rejected.
2. [OURS] A busy port (P-7): if the port on any loopback address is busy, the server does not start and reports; another port is not
   chosen. The link with the token and the token file are created after the port is successfully taken (ADR 0065 item 2): a process that took the
   port before the server gets no valid tokens (the token does not exist yet). A process that took the port after the server stopped
   gets from clients only a stale token (invalid after the next start). What remains: the user may open from an old bookmark a page taken by another process and enter data there
   (the old token is invalid). This is a residual risk; it is reduced by the fact that the UI page without a token gives no data and only the server gives the link. The behavior
   under `SO_REUSEADDR`/`SO_REUSEPORT` on macOS and Linux was not verified (Q-130): the implementation does not enable `SO_REUSEPORT`
   and checks in a test that a second `bind` on the same port is refused.
3. [OURS] Token brute force (P-7c): tokens are 256 random bits; no blocking by the number of failures (blocking by address gives
   an attacker a way to disable a legitimate client behind a shared NAT or local address, while guessing 2^256 is impossible). Failures
   are counted by a counter visible to the operator (Q-66). The pressure of a stream of refusals on slots and CPU is handled only by connection limits and server timeouts
   (ADR 0071, 0072): slot protection limits the harm of a stream of anonymous connections below the threshold of 256 per second per pool; at a faster stream
   new connections may get "too many connections" (ADR 0072 item 2, D-53); there is no unconditional guarantee. A refusal does not spend the limit (ADR 0068 item 4).
4. [OURS] The network listener (P-7e): Host is not checked against a list (the client goes by any name/address of the server; authenticity
   is given by the certificate); the token is not in a cookie, so DNS rebinding has no "automatically sent" credentials. CORS:
   the server sends `Access-Control-Allow-*` on no listener; a request with `Authorization` from a browser page
   of another origin requires a preflight and without the server's consent does not go out (a consequence of browser behavior, not verified here, F-98).
5. [OURS] UI token delivery (P-1, P-2, P-5): the link `http://127.0.0.1:<port>/#<token>`. The fragment is not part of the request
   (RFC 9110 §4.2.5, F-97) and is cut from Referer (Referrer Policy, F-99), so it does not land in server and
   proxy logs or Referer. The UI page reads the fragment, removes it from the address bar (`history.replaceState`) and keeps the token in the tab's
   `sessionStorage` (the origin includes the port, P-2) or in memory, passing it in the Authorization header. Cookies are not used
   (P-1). The XSS risk (P-2) is a requirement for the UI: text from the database only as text (not HTML), a strict Content-Security-Policy
   (area 5). The UI shell (HTML/JS/CSS) is served without a token and without database data.
6. [OURS] Tokens are not written to logs and messages (ADR 0065 item 7).

## Alternatives considered
- A token in the link's query string (`?token=`): rejected (P-5, F-90).
- A cookie with HttpOnly: rejected, not isolated by port (P-1, F-91); SameSite does not take the port into account either.
- Any Host: rejected (P-3). Allow any Host but check Origin: rejected, not all requests carry Origin.
- Blocking an IP after N failures: rejected (item 3).
- A random loopback port: see ADR 0066 item 2.

## Consequences
- The kind of the "host not allowed" refusal is `host_not_allowed`, 403 (ADR 0062 item 8).
- Area 5: the UI does not use cookies, does not put the token in the URL after loading, CSP, no HTML from data; keeping sign-in — when the tab is closed
  the token disappears (Q-38/Q-39 — area 5).

## Sources
- Facts F-90, F-91, F-96, F-97, F-98, F-99; pitfalls P-1…P-5, P-7, P-7c, P-7e.

- **Changed by OD-7 (2026-10-28).** "too many connections" is now closing the connection without a response (ADR 0098).
