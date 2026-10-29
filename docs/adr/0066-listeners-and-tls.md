# 0066. Listeners and TLS: loopback, network HTTPS, certificate
- Status: Accepted (OD-6) ([DEFAULT]: item 1 (file and UI token only on the loopback listener, D-46), items 2 (shape and composition, D-47), 3 (certificate validity period, D-48), 4 (D-49), 5, 6 (D-50)); Accepted ([OURS]: item 1 (composition of listeners), item 3 except the validity period, item 7)
- Label: [OURS] (item 1 (composition of listeners), item 3, item 7); [DEFAULT] (item 1 (narrowing of Q-27), items 2, 3 (validity period), 4, 5, 6 — Q-27, Q-45, Q-69, Q-70)
- Date: 2026-10-27
- CUJ: CUJ7, CUJ8, CUJ10, CUJ13

## Context
The owner: the UI and local programs — via loopback; network services — over HTTPS, TLS in the server itself; the network listener
is off by default, and if enabled without a certificate the server does not start (ADR 0011 item 1, ADR 0012 items 1–3, Q-8, Q-27).
Not settled: the composition of listeners and the form of refusing non-HTTPS (Q-45), the source and replacement of the certificate (Q-69), TLS on loopback (Q-70).

## Decision
1. [OURS] Two kinds of listeners. The loopback listener: 127.0.0.1 and ::1 (::1 — if there is an IPv6 loopback; no address
   in the system — skipped with a message; busy — start refusal, ADR 0067 item 2), a port from the settings, HTTP without TLS; on it the UI and
   API, UI and file tokens. The network listener: HTTPS, API only, network tokens only; created only if the network listener
   is enabled. A network token on the loopback listener gets "not recognized" (ADR 0065 item 6; the brief: network tokens only over HTTPS).
   [DEFAULT] (D-46) A UI/file-kind token on the network listener also gets "not recognized", even if the client connected from a loopback address.
   This narrows the owner's decision "the file token is accepted from loopback" (OD-3, Q-27): a local program that connected to the network
   port via 127.0.0.1 does not get the file token; it has a loopback listener, no capability is taken away. Rationale: the UI and file tokens
   must not travel over the TLS listener visible from the network (one rule "where which token is accepted", checked by listener, not by the client
   address, which is unreliable behind NAT and proxies). Alternative: accept the file token and the UI token on the network listener if the client address is
   loopback (a literal reading of Q-27): more complex and gives nothing, the loopback listener already exists. Put to the owner as a package.
2. [DEFAULT] Defaults: loopback — port 7878; the network listener — one HTTPS port 7843, the default address
   `0.0.0.0` (all IPv4 interfaces; a list of `address:port` is set in the settings, several are possible). The network listener
   is off. Rationale: ports outside the reserved ones, not 8080/8443 (others' services are often there), values only by
   default; the number is the operator's question. Alternatives: a random loopback port with a record in the token file (protects against
   P-7, but breaks bookmarks and scripts and requires a file format with an address); one common listener. Q-45: the form of refusing non-HTTPS —
   on the network port only TLS; plain HTTP is not served and is not redirected anywhere (a redirect invites sending the
   token in plain text, RFC 6750 §5.3, F-90); whether network tokens are accepted on loopback when the listener is off — no
   (there is no HTTPS entry on loopback when the listener is off).
3. [OURS] If the network listener is enabled and the certificate or key is not set, cannot be read, cannot be parsed as PEM or does not form
   a pair (the error return of `ServerConfig::with_single_cert` on a pair mismatch was not verified in the documentation, F-93: check at implementation), the server does not start at all (loopback too), and reports
   the cause. Start order: check the settings and certificate → open the listeners. With the listener off the certificate is not
   checked. Validity period: an expired certificate — start with a warning (clients will reject it themselves; a start refusal
   would leave without a UI because of someone else's error); the value is [DEFAULT], the alternative is a start refusal.
4. [DEFAULT] (Q-70) There is no TLS on loopback. Rationale: the link must open at once (CUJ13, "without entering credentials");
   a self-signed certificate would trigger a browser warning, and one trusted for 127.0.0.1 is not issued; `http://127.0.0.1`
   is considered potentially trustworthy by the browser (Secure Contexts §3.1, F-95); the traffic does not leave the host. Cost: passive
   eavesdropping on loopback by a privileged user (root) is not protected against — but that user reads the process memory and the token file anyway
   (not verified, reasoning). Alternative: HTTPS on loopback with a locally issued certificate — not accepted.
   Consequence for CUJ7/CUJ13: the loopback check is HTTP.
5. [DEFAULT] (Q-69) The certificate is PEM files provided by the operator: a certificate chain and a private key (formats PKCS#8,
   PKCS#1, SEC1 — read by `rustls-pki-types`, F-94), paths in the settings. The source (a corporate CA, a public CA,
   self-signed) is the operator's choice; the server does not issue or renew the certificate. Replacement: swapping the files and a planned
   restart (tokens are preserved, ADR 0012 item 4). A client that does not trust the certificate drops the connection itself before sending the
   token; the server sees a handshake error and counts such drops for the operator. Permission to read the key — owner only:
   if the key file has group/other permissions the server warns but starts. Alternatives: reloading the certificate without
   a restart (more complex, depends on the channel Q-60); a self-signed one issued by the server (clients would have to trust each server);
   ACME (an external dependency and outgoing network).
6. [DEFAULT] TLS 1.2 and 1.3 (`rustls` does not implement older versions, F-93); the cipher set — the library default; there are no client
   certificates (authentication is by token).
7. [OURS] HTTP/1.1 on both listeners (HTTP/2 is out of scope; not chosen here — a protocol subtask).

## Alternatives considered
See items 2, 4, 5. Additionally: a network token on loopback over HTTP — rejected (the brief: network tokens only over HTTPS).

## Consequences
- Q-45, Q-69, Q-70 and the narrowing of Q-27 (item 1) get [DEFAULT] values (D-46…D-50) — put to the owner as an area package.
- Area 4: the settings (`network.enabled`, address/port, certificate and key paths, loopback port); operator messages about
  the causes of start refusal (Q-66).
- The Host check, taking the port — ADR 0067.

## Sources
- ADR 0011, 0012; `docs/owner-decisions.md`, OD-3 (Q-8, Q-27).
- Facts: F-90, F-93, F-94, F-95.
