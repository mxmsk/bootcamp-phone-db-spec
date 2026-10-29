# 0087. UI page security policy
- Status: Accepted
- Label: [OURS]
- Date: 2026-10-27
- CUJ: CUJ13; pitfalls P-1, P-2, P-5

## Context
The UI shows data entered by people and programs (names) on an origin that holds the token (ADR 0067 item 5). XSS through a name would expose the token and allow changing the database. ADR 0067 item 5 assigned to area 5 a strict CSP and "text, not HTML".

## Decision
1. [OURS] Every shell response carries `Content-Security-Policy: default-src 'none'; script-src 'self'; style-src 'self'; connect-src 'self'; base-uri 'none'; form-action 'none'; frame-ancestors 'none'`. `'none'` allows nothing not named (F-118); there are no images: the trash can icon and other symbols are in the markup or styles of its own origin; a request for `/favicon.ico` gets 404. The page has no inline `<script>`, event handlers or `style` attributes.
2. [OURS] Additionally: `Referrer-Policy: no-referrer`, `X-Content-Type-Options: nosniff`, `Cache-Control: no-cache` on the shell.
3. [OURS] The server sets no cookies; the page neither reads nor writes cookies or `localStorage`.
4. [OURS] Values from responses (names, numbers, dates, `current_name`, texts) are output only as text (`textContent`/text nodes); not `innerHTML`, not handler attributes, not addresses, not `style`. UI message texts are taken from its own table by code, the server's `message` is not displayed.
5. [OURS] There are no external resources: no fonts, no CDN, no analytics.

## Alternatives considered
- Allow `'unsafe-inline'` for styles: rejected, a strict CSP without exceptions is easier to verify; styles are in a file.
- CSP via `<meta>`: rejected, `frame-ancestors` has no effect in `<meta>` (not verified here) and the header is more reliable.
- HTML sanitization (a sanitizer): rejected, not needed if no HTML is created from data.

## Consequences
- Specification `web-ui-shell`: requirements on CSP, no cookies and no external resources, "text only".
- Area 3 for API responses with JSON: `X-Content-Type-Options: nosniff` is recommended; there is no requirement (JSON is served as `application/json`).
- New UI functionality requiring `eval` or inline code is not allowed without revision.

## Sources
- ADR 0065, 0067; F-118 (CSP3); `pitfalls` P-1, P-2, P-5.
