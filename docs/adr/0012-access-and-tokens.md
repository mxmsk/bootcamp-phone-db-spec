# 0012. API client access and tokens
- Status: Accepted
- Label: [USER]
- Date: 2026-10-27
- CUJ: CUJ7, CUJ8, CUJ9, CUJ10, CUJ13, CUJ15

## Context
The API is used by local programs on the same machine and by network services on other machines. The brief sets the
kinds of clients, permissions, token management and the request limit, but leaves open the certificate source and the
operator control channel.

## Decision
1. API clients: local programs (a token from a file, changed at every server start) and network services on other
   machines (API only, over HTTPS; TLS in the server itself) (brief §3/Interfaces).
2. The file token and the UI token are accepted only from loopback; the network token is accepted from anywhere, but
   only over HTTPS (OD-3, Q-27).
3. The network listener is off by default and is enabled by an operator setting; if it is enabled and there is no
   certificate, the server does not start (OD-3, Q-8).
4. Network services: a permanent token per client; permissions "read" or "read and write"; the operator issues, views
   and revokes tokens with a command on the server machine (brief §3/Interfaces). The token is shown once; the list has
   no secrets; the server is not stopped; revocation takes effect immediately; a server restart does not change network
   tokens (brief §2, CUJ10).
5. The client name is unique among active tokens; after revocation the name can be issued again (OD-3, Q-9; OD-4,
   item 4, docs/owner-decisions.md).
6. Revoking a nonexistent or already revoked token is an error message, with no changes; changing permissions is only by
   issuing a new token; a request already running at the moment of revocation completes (OD-3, Q-9).
7. The per-token request limit (brief §3/Interfaces) applies only to network tokens; local programs and the UI have no
   limit (OD-3, Q-24). Other clients do not receive refusals because of someone else's limit; latencies stay within the
   overall targets (OD-3, Q-18).
8. Tokens are stored separately from the data and are not part of a backup; a restore does not change tokens (a revoked
   one does not come back to life) (OD-3, Q-10). Network tokens are stored next to the server settings (like the
   certificate); when moving to another machine the operator transfers them along with the settings, or else reissues
   them (OD-4, item 9).
9. Refusals named in the brief: "unauthorized" (CUJ7, an old token after a restart), "no permission" (a "read" token
   tries to add, rename or delete; the data does not change), «слишком много запросов, повторите позже» ("too many requests, try again later") (the limit is
   exceeded) (brief §2, CUJ7, CUJ8). A consequence of CUJ7: "unauthorized" is received by any token unknown to the
   server (wrong, arbitrary, stale), not only the old token after a restart.

## Alternatives considered
- Changing permissions of an issued token: rejected, changing permissions only by issuing a new token (OD-3, Q-9).
- Name uniqueness among all tokens, including revoked ones (package 1 recommendation, Q-9: "the client name is
  unique"): clarified by OD-4, item 4, to among active ones.
- Applying the limit to local programs and the UI: rejected (OD-3, Q-24).
- No other alternatives were considered in the sources.

## Consequences
- Open questions: the value and model of the limit (Q-46); the source of the TLS certificate and its replacement
  (Q-69); TLS on loopback (Q-70); the set of network listeners and the form of refusal for non-HTTPS (Q-45); the
  control channel from the operator to a running server and who may run commands (Q-60); the design of the token
  command (Q-40); the local-program and UI token (Q-38); the text or code of refusals (Q-50); operator messages (Q-66);
  see `docs/open-questions.md`.
- Server limits (connections, body size, timeouts) are in ADR 0019.

## Sources
- `docs/source/brief.md`, section 3 "Interfaces"; section 2 (CUJ7, CUJ8, CUJ10).
- Owner decision of 2026-10-27 (OD-3): Q-8, Q-9, Q-10, Q-18, Q-24, Q-27. Place of record: docs/owner-decisions.md, OD-3.
- Owner decision of 2026-10-27 (OD-4), items 4 and 9. Place of record: docs/owner-decisions.md, OD-4.
