## Why

Area 3 per `docs/process.md` ("Server and API") turns the accepted data semantics (area 1) and storage (area 2) into a programming interface and a server process: without it
neither a client (UI, local programs, network services) can be written, nor can access, limits and behavior under failures be verified (CUJ7–CUJ16, X-1, X-2, X-5). Its decisions are recorded in ADR 0060–0074;
the values of brief §4 (limits, error body) were revised and put to the owner as [DEFAULT].

## What Changes

- The programming API v1 (HTTP + JSON): versioned paths, six operations, the representation of number, date, version, cursor and page, the by-number results stream with a completeness marker, the error body, a single table
  of codes and statuses for all refusals (including access, limits, time and server state), a single request check order, size caps (body 256 KiB, headers 16 KiB), compatibility
  within v1 (ADR 0060–0064).
- Access: the UI token in the link fragment, the local-program token in a file, network tokens (SHA-256, constant time, revocation from the next request), permissions, loopback without TLS and network HTTPS,
  the certificate, loopback protection (Host, occupied port, brute-force), the request limit per network token (ADR 0065–0069).
- The server process: a single database owner, HTTP/1.1 on hyper, the request execution model on top of storage, connection limits and timeouts, slot protection from anonymous clients, display of
  storage states without advance display, startup (including index rebuild) and clean shutdown (ADR 0070–0074).
- Fix pass after the Opus review (R-1…R-10): a single body cap of 256 KiB, one measure and one refusal for headers (431), a single check order ("server starting" for the whole startup, before the token), a consistent
  moment of writing the token file, a single code table, merging the duplicates ADR 0063 and 0073, the narrowing of Q-27 recorded as [DEFAULT] (D-46), slot protection backed by a mechanism and honestly limited by a threshold
  (D-53). Details are in `design.md`, "Fix pass after the Opus review".
- New verified facts F-86…F-108 in `facts.md`; 18 questions closed, 3 narrowed, new Q-126…Q-141 in `docs/open-questions.md`; 17 default values (D-37…D-53) await the owner as a package at the end of the area. Client-visible novelties awaiting the owner's decision: an error body wider than the brief (D-40), caps of 256 KiB and 16 KiB instead of 32 KiB (D-43), the file token is not accepted on
  the network listener (D-46), a read processing deadline of 20 s and "write outcome unknown" (D-52), a conditional slot-protection guarantee (D-53).
- No BREAKING: v1 is not released; the requirements of areas 1 and 2 do not change.

## Capabilities

### New Capabilities

- `http-api`: the version in the path, resources and methods, value representations, the cursor and page, the by-number stream, the error body, codes and statuses of all refusals, failure headers, the check order,
  size caps, compatibility within v1.
- `api-access`: tokens (UI, local programs, network), permissions, access denials, listeners and TLS, loopback protection, token brute-force, the request limit per network token.
- `server-runtime`: the process and the HTTP base, the read and write execution model, limits and timeouts, slot protection, storage states in responses, startup and shutdown.

### Modified Capabilities

None: the requirements of areas 1 and 2 and `user-journeys` do not change. The numeric placeholders of CUJ in area 2 and the values of brief §4 are replaced after the owner answers the [DEFAULT] package (a separate change at archiving).

## Impact

- ADR 0060–0074 (new, 15; ADR 0073 merged with 0063: Superseded); they rely on ADR 0011–0013, 0025, 0026, 0035, 0049, 0053–0059; the index is `docs/adr/README.md`.
- Requirements for areas 4 and 5 and for the build are collected in `design.md`, "Requirements for other areas".
- Documents: `facts.md` (F-86…F-108), `docs/open-questions.md` (18 closed, 3 narrowed, 16 new, [DEFAULT] 17), `docs/adr/README.md`.
- No code: the change defines requirements, not implementation.
