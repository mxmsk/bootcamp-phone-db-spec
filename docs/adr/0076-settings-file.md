# 0076. Settings file: format, location, composition, errors
- Status: Accepted (OD-6)
- Label: [OURS] (items 1, 2, 4–8); [DEFAULT] (item 3 — D-56)
- Date: 2026-10-27
- CUJ: CUJ7, CUJ8, CUJ10, CUJ13, CUJ14, X-1; item 1.3 of the original wording

## Context
Areas 2 and 3 moved into settings: the loopback port, the network listener, addresses, certificate and key (ADR 0066), the per-token request limit (ADR 0068), the server limits and deadlines
(ADR 0071; the body cap not below 32 KiB), the thread counts (ADR 0070 item 4), snapshot and backup parameters (ADR 0059), the settings directory as the place of `generation`, `tokens`,
`local.token` (ADR 0054 item 5, ADR 0065 items 2, 4). The format, location and behavior on error are not defined (Q-45, Q-66).

## Decision
1. [OURS] The format is TOML (the TOML specification, version 1.0 and above), the file `settings.toml` in the settings directory (`--config`, ADR 0075 item 2). Parsing — the `toml` crate (serde-compatible, MIT OR Apache-2.0,
   F-112; the cargo deny check and version choice — at build, ADR 0017 item 3). Reason: the operator edits the file by hand — comments and readable tables are needed (JSON does not allow comments); the crate is
   serde, and serde_json is already in the project. Alternative: JSON (nothing new in dependencies, but no comments); YAML, INI (less well-defined; not verified).
2. [OURS] Keys and defaults (values — from ADR 0066, 0068, 0071, 0059, 0070, 0074):

   | Table.key | Default | Source |
   |---|---|---|
   | `loopback.port` | 7878 | ADR 0066 item 2 |
   | `network.enabled` | `false` | ADR 0012 item 3, 0066 item 2 |
   | `network.listen` | `["0.0.0.0:7843"]` (a list of `address:port`) | ADR 0066 item 2 |
   | `network.certificate`, `network.private_key` | none (mandatory when `enabled = true`) | ADR 0066 item 5 |
   | `control.socket` | `control.sock` (the path of the control channel file; relative — from the settings directory) | ADR 0080 item 3 |
   | `limits.connections_per_listener` | 256 (loopback and network pools are separate) | ADR 0071 |
   | `limits.connections_per_address` | 32 | ADR 0071 |
   | `limits.anonymous_per_address` | 16 | ADR 0071, 0072 |
   | `limits.anonymous_protect_seconds` | 1 | ADR 0071, 0072 |
   | `limits.header_bytes` | 16384 | ADR 0071 |
   | `limits.body_bytes` | 32768 (the lower bound is 32768; OD-6) | ADR 0071 item 1 |
   | `limits.first_request_seconds` / `idle_seconds` / `body_seconds` | 10 / 60 / 10 | ADR 0071 |
   | `limits.read_seconds` / `write_seconds` | 20 / 10 | ADR 0071 |
   | `limits.response_stall_seconds` / `stream_stall_seconds` | 10 / 120 | ADR 0071 |
   | `rate_limit.capacity`, `rate_limit.per_second` | 100, 100 | ADR 0068 item 1 |
   | `runtime.io_threads`, `runtime.reader_threads` | number of cores; max(4, number of cores) | ADR 0070 item 4 |
   | `snapshots.short_read_seconds` | 30 | ADR 0059 item 1 |
   | `snapshots.retention_limit_mib` | `auto` (the larger of 1 GiB and 25% of data) | ADR 0059 item 2 |
   | `backup.read_rate_mib_per_second` | 64 | ADR 0059 item 3 |
   | `backup.space_reserve_percent` | 5 | ADR 0059 item 5 |
   | `shutdown.timeout_seconds` | 60 (the lower bound is 60) | ADR 0074 item 9, ADR 0077 item 4 |
   | `log.requests` | `true` | ADR 0070 item 9 |
   | `log.counters_seconds` | 60 (the period of the refusal counter report) | ADR 0078 item 6 |

   Relative paths in the file are counted from the settings directory.
   The list is complete for v1; a new key is a non-breaking change (ADR 0013): an absent key takes the default.
3. [DEFAULT, D-56] The `settings.toml` file is absent in an existing settings directory — the server starts on defaults with a warning (`settings_missing`). Rationale: the defaults are safe (network off, loopback); `init`
   creates the file, its loss is the operator's case, and a refusal would stop the server because of a file without secrets. Alternative: start refusal (stricter, but turns a missing file into downtime). The settings directory is absent — refusal
   (the `generation` counter and tokens are there, ADR 0054 item 7).
4. [OURS] The file is read once, at step 2 of the start order (ADR 0074 item 1). A change on a running server is not picked up: neither by SIGHUP nor by a command (ADR 0077 item 5); changing settings is a restart. Replacing the certificate is
   swapping the files and a planned restart (ADR 0066 item 5).
5. [OURS] A settings error is a start refusal before the listeners are opened, code 3: the file cannot be read or parsed as TOML; an unknown table or key (a typo must not silently leave the default: `loopback.prot`); a value
   of another type; a value out of the valid range (port 0 or above 65535, an empty `network.listen`, `limits.body_bytes` < 32768, `shutdown.timeout_seconds` < 60, a zero deadline, `connections_per_address` > `connections_per_listener`, a `control.socket` path longer than the OS limit for a socket path, ADR 0080 item 3).
   The message names the key, the value and the reason; all keys are checked, up to 20 errors in one message. The first error does not hide the others.
6. [OURS] A separate certificate and key check — code 4 (ADR 0066 item 3: missing, unreadable, not PEM, not a pair). Performed only when `network.enabled = true`. An expired certificate is a warning, start proceeds (ADR 0066 item 3);
   a key readable by group or others is a warning (ADR 0066 item 5).
7. [OURS] The location of files in the settings directory is fixed: `settings.toml`, `local.token`, `tokens`, `generation`, `control.lock`, `control.sock` (the path of the last is the `control.socket` key); the names are part of the operator's convention (ADR 0065 item 2 left the `local.token` name to the implementation; here it is fixed).
   Permissions: the directory 0700, `local.token`, `tokens`, `generation`, `control.lock`, `control.sock`, the key — 0600; the permissions of `settings.toml` itself are not checked (it holds no secrets — paths to the key, not the key).
8. [OURS] The settings contain no secrets and are not printed in full; messages name only the key and the reason.
9. **Changed by OD-6 (2026-10-28).** D-56 accepted by the owner (OD-6); `limits.body_bytes` — default 32768, lower bound 32768 (ADR 0071, OD-6); the `snapshots.*`, `backup.*` values did not change.

## Alternatives considered
- Command-line parameters instead of a file: rejected, six dozen parameters, not stored; inconvenient for services. The parameters `--db`, `--config` stay on the command line (ADR 0075).
- Environment variables: rejected, not visible in the file, leak into child processes and process lists.
- Re-reading on SIGHUP: rejected (Q-136, ADR 0077 item 5): the certificate and listeners cannot be replaced without restarting the listeners; two ways of changing settings complicate checking "settings at start".
- Ignoring unknown keys: rejected, a typo in `limits.body_bytes` would silently leave the default.

## Consequences
- D-56 goes to the owner as an area 4 package. New crates: `toml` (ADR 0017 item 3, cargo deny).
- The requirements of ADR 0080 and 0081 on this ADR are met: the `control.socket` key (item 2), the `control.lock` and `control.sock` files (item 7). The management commands read from `settings.toml` only `control.socket` and `network.enabled` (ADR 0080 item 3); the rest is checked by the server.
- The default values are the same [DEFAULT]s D-43, D-46…D-53, D-30…D-33; when they change, only the numbers of this ADR change.
- Test: each table row is a separate parsing scenario; an unknown key — refusal.

## Sources
- ADR 0012 item 3, 0017, 0054 item 5, 0059, 0065, 0066, 0068, 0070 items 4, 9, 0071, 0074 item 9; Q-45, Q-66; F-112.
