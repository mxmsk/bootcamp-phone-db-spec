# 0082. Token commands: issue, list, revoke; client name; writing the token set
- Status: Accepted ([OURS]: items 1, 2, 5–8, item 9 except the warning); Accepted (OD-6) ([DEFAULT]: item 3 — name rules, D-65; item 4 — list composition and volume, D-66; item 9 — issuance with the network off, D-67)
- Label: [OURS] (items 1, 2, 5–9); [DEFAULT] (item 3 — D-65; item 4 — D-66; item 9 — D-67)
- Date: 2026-10-27
- CUJ: CUJ10, CUJ7, CUJ8; item 1.2 of the original wording

## Context
ADR 0012 items 4–6, 8 [USER] and OD-3 (Q-9), OD-4 (items 4, 9): a token is issued with a client name and permissions and shown once; the list has no
secrets; revocation is by token, takes effect immediately, the server is not stopped; the name is unique among active tokens, free after revocation; revoking
a nonexistent or already revoked one is an error with no changes; changing permissions is only by a new token; tokens live next to the settings and are not part of the
backup. ADR 0065 items 4–8 (area 3) define the stored record, the comparison and [DEFAULT] D-45. Q-40: how a token is identified on revocation and the composition of the
list.

## Decision
1. [OURS] Issuance: `npdb token-issue --config <settings directory> --name <name> --access <read|read_write>` (the subcommand is the first argument, parameters are `--name value`: ADR 0075 item 1; the permission names are the same
   as in ADR 0065 item 4). The secret is 32 random bytes, a hexadecimal representation of 64 characters (ADR 0065 item 4), the source is `getrandom` (F-78). The server
   (online) or the command (offline) computes SHA-256, writes the record (item 5) and only then prints the secret. The secret goes to standard
   output as a single line with nothing extra (so that a script takes it whole), explanations go to the standard error stream. The secret is written to no
   file, no log and no message (ADR 0065 item 7); it cannot be obtained again anywhere. If lost — revoke and issue a new one.
2. [OURS] Revocation: `npdb token-revoke --config <settings directory> --name <name>` — revokes the active token with this name. The token identifier for the operator is the client name: it is
   unique among active tokens (item 3), hence unambiguous; the secret is not needed for revocation (an operator who lost the secret can still revoke).
   If there is no active token with that name (never issued, or already revoked, or spelled differently) — the error «действующего токена с именем X нет» ("there is no active token with the name X"), the set does not change, code 26 (ADR 0012 item 6; the message does not distinguish "never existed" from "already revoked": both are "no active one"). The record stays in
   the file with the revocation time (ADR 0065 item 4). Closes Q-40 (identifier).
3. [DEFAULT] (D-65) Client name: 1–64 characters from `A–Z a–z 0–9 . _ -`; case is significant (`Billing` and `billing` are different names); spaces and other characters —
   a refusal «недопустимое имя клиента» ("invalid client name") (code 2, before any changes). Rationale: the name appears in the log and in the list (ADR 0065 item 7); control characters, leading/trailing spaces and invisible
   differences would make the list and the log misleading; ASCII avoids Unicode normalization (that is the rule for record names, not clients).
   Alternatives: any Unicode ≥ 1 character — requires normalization and protection against control characters; case-insensitive uniqueness — one
   additional comparison rule, the cost against "Billing/billing" is small, but the brief does not specify uniqueness by name. An owner-level question.
4. [DEFAULT] (D-66) List: `npdb token-list --config <settings directory> [--show active|all]`. By default (`active`) — only active tokens; `--show all` adds revoked ones. Row fields: client name, permissions
   (`read` / `read_write`), issue time (UTC, RFC 3339 to seconds), state («действует» ("active") or «отозван» ("revoked") with the revocation time). Order — by issue time
   ascending, then by name. There are no secrets or their hashes in the list. "Last use" is not shown (ADR 0065, "Consequences": nothing is written to disk
   on every request). An empty list is the line «действующих токенов нет» ("no active tokens"), code 0. Rationale: CUJ10 — "a list of issued tokens
   (without secrets)"; revoked records exist on disk (ADR 0065 item 4) and the operator needs them for investigation ("who and when"). Alternatives: always all records —
   the list grows and is noisy; no `--show` at all — the history is lost; JSON format — not needed (the brief has no programmatic consumer).
5. [OURS] Writing the token set (consistent with ADR 0065 item 4): the file `tokens` in the settings directory, permissions 0600, the format is one JSON document
   (`serde_json`): `{"format": 1, "tokens": [{name, access, sha256, issued_at, revoked_at}]}`, `revoked_at = null` for an active token. A change:
   write a new file alongside with permissions 0600 (`open` with a mode), flush to stable storage, replace with `rename`, flush the directory. The command replies "done"
   only after this. A write error (out of space, directory not writable) is a refusal, the file stays as it was, the set in the server's memory stays as it was, code 10 (`file_access`, ADR 0077 item 6). A file created or replaced offline gets the owner and group of the settings directory (ADR 0081 item 7, D-73).
   A `format` version newer than the known one — a read refusal («формат файла токенов новее программы» ("the token file format is newer than the program")).
6. [OURS] The order of "done" (ADR 0065 item 8, D-45): (1) check the conditions (name, uniqueness) under a single lock on the set; (2) write the file (item 5);
   (3) swap the set in the server's memory (a single atomic reference switch); (4) reply. A request started after the reply sees the change:
   the token check reads the current set on every request (ADR 0065 item 7), including requests on connections opened before the revocation; a request in progress at the
   moment of revocation completes (ADR 0012 item 6). A process failure between (2) and (4): the file is changed, there is no reply; the operator sees the result in the list
   (on a lost reply the command reports "outcome unknown, check the list", code 24, ADR 0084 item 5).
7. [OURS] Name uniqueness is checked among records with `revoked_at = null`. Revoked ones do not participate: after revocation the name is issued again as a new record, while the
   revoked one keeps its own (OD-4 item 4, ADR 0012 item 5). Two simultaneous issuances of the same name are executed in turn (item 6, (1)), the second gets «имя занято» ("name taken") (code 25).
8. [OURS] Tokens and backup (ADR 0012 item 8, 0016 item 5, OD-4 item 9): the `tokens` file is not part of the backup and is not changed by backup or by starting on a backup;
   a server started on a backup with the same settings reads the same tokens, a revoked one stays revoked. A server restart does not change the set.
   Moving to another machine: the operator copies the `tokens` file along with the settings (check permissions 0600 after the transfer) or issues tokens anew.
9. [OURS] Changing permissions is only issuing a new token (ADR 0012 item 6): there is no "change permissions" command; issuing an existing active name is a refusal «имя занято» (code 25), with a hint in the message: «чтобы сменить права, отзовите токен и выдайте новый» ("to change permissions, revoke the token and issue a new one"). The network listener is off (`network.enabled = false` in `settings.toml`; the command reads this key both online and offline, ADR 0080 item 3, D-74): issuance proceeds,
   with a warning on the standard error stream «сетевой приём выключен: токен начнёт приниматься после его включения» ("network listener is off: the token will start being accepted after it is enabled") (D-67: [DEFAULT], the value is to allow with a
   warning; the alternative is a refusal: it would get in the way of preparing tokens before the network is enabled).

## Alternatives considered
- Revocation identifier as a fingerprint (hash prefix): rejected, the operator would have to type a long random identifier; the name is already unique among
  active tokens.
- Revocation identifier as the secret itself: rejected, the secret is lost or is held by the client; worst of all in shell history.
- A separate numeric identifier: rejected, one more field, and choosing by name is already unambiguous.
- Issuance with an operator-specified secret: rejected, weak tokens (ADR 0065 item 4).
- Storing the last use time: rejected in ADR 0065.
- The secret in a file instead of standard output: rejected, the secret would lie on disk in plain text.

## Consequences
- ADR 0065 does not change: item 4 is extended with the file format (item 5), item 8 (D-45) is fulfilled by items 5, 6.
- Closes Q-40 (revocation identifier, list composition) and, for the channel part, Q-60 (with ADR 0080, 0081).
- Area 3: at startup the server parses `tokens` per item 5; an unparsable file or a format newer than the program is a start refusal with code 3, one unreadable because of access — with code 10 (ADR 0081 item 5, D-72).

## Sources
- ADR 0011, 0012, 0016, 0065, 0080, 0081, 0084; `docs/owner-decisions.md` OD-3 (Q-9, Q-10), OD-4 (items 4, 9); F-78.
