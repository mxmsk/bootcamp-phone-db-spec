# 0086. Sign-in by link: token from the fragment, storage, transmission
- Status: Accepted ([OURS]: items 1, 3, 4); Accepted (OD-6) ([DEFAULT]: item 2 — D-77)
- Label: [OURS] (items 1, 3, 4); [DEFAULT] (item 2 — D-77)
- Date: 2026-10-27
- CUJ: CUJ13
- Closes Q-38 for retaining the UI sign-in

## Context
The link `http://127.0.0.1:<port>/#<token>` (ADR 0067 item 5) leaves token retention to area 5: ADR 0067 allows tab `sessionStorage` or memory (risks P-1 cookie, P-2 XSS and the shared origin storage, P-5 token in the address).

## Decision
1. [OURS] The page reads the fragment on load and on every fragment change (`hashchange`): after `replaceState` the tab's address is `/`, and pasting `/#<new token>` into it changes only the fragment and does not reload the page, and this is the main CUJ13 case after a server restart. If the fragment is 64 hexadecimal characters, it is a token and replaces the previous one; the fragment (any) is removed from the address bar by calling `history.replaceState` without adding a history entry (the address stays on the same origin, F-116). The handler does not depend on whether the event fires for `replaceState` itself: it ignores an empty fragment. The fragment does not go to the server or into the Referer (F-97, F-99). The page does not insert the token into links, request addresses or text.
2. [DEFAULT] (D-77) The token is stored in the tab's `sessionStorage`, in memory if access fails. Rationale: reloading the tab (F5) should not require a new link from the terminal; `sessionStorage` is isolated by origin (including the port, P-2) and by tab (F-117); neither `sessionStorage` nor memory protects against XSS — protection against XSS is CSP and text-instead-of-HTML (ADR 0087). What is ensured: the token does not reach cookies, `localStorage`, request addresses, texts or the console; it lives until the server restarts (a new one at every start, ADR 0065 item 1; ADR 0011 item 2); it is accepted only on the loopback listener. What is not ensured: "the token is not on disk" and "the token disappears when the tab is closed". Residual risks, accepted explicitly: the browser may save `sessionStorage` to disk and return it on "reopen closed tab" and on session restore after a browser restart (implementation behavior, not verified, F-117); a duplicated tab gets a copy of `sessionStorage`; the link `/#<token>` is already written to the browser history before `replaceState` and stays in the terminal (whether the fragment reaches the history is not verified, F-116, Q-158). Mitigation: the token is new at every start, so any saved token is invalid after the server restarts. The wording of ADR 0067 ("Consequences": "the token disappears when the tab is closed") is clarified by this item: the owner's decision (ADR 0011 item 2) is fully met. Alternative: memory only — the token is lost on reload, the user needs a new link; the "to disk" risk is smaller, but there is no gain against XSS and the browser history does not go away.
3. [OURS] The token is sent only in `Authorization: Bearer` on the page's requests to `/v1/` of its own origin (ADR 0065 item 6). Cookies, `localStorage`, a query parameter are not used (P-1, P-5).
4. [OURS] If on load there is neither a fragment nor a token — the display «Нет ссылки для входа…» ("No sign-in link…"), no requests; on 401 the token is deleted, while the form and the results remain (ADR 0088 item 2). A link with a new token, including one pasted in the same tab, replaces the stored one without a reload and without losing what was entered.

## Alternatives considered
- `localStorage`: rejected, survives closing and restart, shared between tabs.
- Cookie: rejected by ADR 0065/0067 (P-1).
- Entering the token manually: rejected, CUJ13 requires "works immediately" by the link.
- Exchanging the token for a session key: rejected, an extra mechanism with no gain given a token that lives until restart.
- Reading the fragment only on load: rejected, pasting a new link into an open tab does not reload the page.

## Consequences
- Specification `web-ui-shell`: the requirements "Sign-in by link: token from the fragment", "Token storage: where it is kept and what is not guaranteed", "Token is sent only in the Authorization header".
- Q-38 is closed for the UI: the sign-in is retained until the server restarts or the tab's `sessionStorage` is lost.

## Sources
- ADR 0011, 0065, 0067; F-97, F-99, F-116, F-117; P-1, P-2, P-5.
