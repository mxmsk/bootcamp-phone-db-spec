<!-- Primary source of requirements; do not edit. -->
# Brief: what we are building

This is the only basis for the new specification. It contains only requirements, user journeys and decisions of the product
owner [USER]. There are deliberately no technical decisions here: the new specification makes them, with justification.

## 1. Original task statement (the only primary source of requirements)
- Must be a good database
- Must have a Visual User Interface and a programmatic API
- Must store Names and Phone numbers
- Names can be searched by Phone number
- Phone numbers can be searched by Name
- Must reliably store many records
- Written from scratch in Rust

## 2. The owner's Critical User Journeys [USER]
- CUJ1. The user adds a new phone number. They enter it in international format with +<country-code>. The number
  is saved and is found by name, and the name is found by number.
- CUJ2. The user adds a known phone number. On saving, we show that such a number already exists and ask the user to
  enter the date from which the number should be assigned to the user. The number is saved, and from that moment a number query
  returns the list of all its owners by date.
- CUJ3. The user adds a known phone number with a date that already exists. We show an overwrite prompt.
  If yes, we change the data, completely erasing the old. If no, we wait for new input.
- CUJ4. The user can search by phone number. For a number we return only an exact match. We show the list
  of everyone the number was added for, sorted by date DESC.
- CUJ5. The user can search by name. The search is by substring. We show the results as a paginated list,
  sorted alphabetically, then by date DESC.
- CUJ6. The user can delete a record of a number. To do so they search by number or name. In the found list,
  a delete button with a trash can icon is shown on the left. Before deleting we ask for confirmation. After deleting,
  we re-query and show the updated list.

CUJ7–CUJ16 were formulated by the previous orchestrator from the owner's answers to questions (2026-10-27) and confirmed
by the owner; they are not the owner's original text.

Programmatic API:
- CUJ7. A local program works with the database. A program on the same machine reads the token from a file and performs the same
  operations as the UI, plus exact whole-name search. After a server restart its request gets a "unauthorized"
  refusal; the program re-reads the token file and successfully repeats the request.
- CUJ8. A network service only searches. A service on another machine connects over HTTPS with its own permanent token with the
  read permission, and searches by number, by words of the name and by whole name. An attempt to add, change or delete a record gets a
  "no permissions" refusal, and the data does not change. If the service sends requests faster than its limit, the excess ones get the refusal "слишком
  много запросов, повторите позже" («слишком много запросов, повторите позже» ("too many requests, try again later")); other clients do not notice this.
- CUJ9. A network service changes data. A service with the write permission adds a number; if the number already has a different
  name for that date, it gets a conflict with the current name and the number version. It sends rename and delete with the number version; if the version
  is stale, it gets a conflict with the current version, re-reads the number history and retries.
- CUJ10. The operator manages access of network clients. On the server machine the operator issues a token to a new service with a command,
  with a client name and permissions (read, or read and write); the token is shown once. The same command lists
  the issued tokens (without secrets) and revokes them. The server does not stop; revocation takes effect immediately — the next request with that
  token is rejected. A server restart does not change network tokens.
- CUJ11. The server was upgraded, clients keep working. After a server upgrade within v1, programs and services
  work unchanged; new fields and error codes may appear, but existing ones do not change and do not disappear.

Outside the API:
- CUJ12. The record was changed concurrently. The user sees the number history and clicks "delete" or confirms an
  overwrite (CUJ3), while the record of this number has meanwhile been changed in another tab or via the API. The action is not
  performed; the UI reports «данные устарели, список обновлён» ("data is stale, the list has been refreshed"), re-reads the number history and offers to retry
  the action on current data. The name entered in the form is kept.
- CUJ13. First start and sign-in to the UI. The operator creates an empty database with a separate command; without it the server does not
  start on a directory without a database and reports that. On start the server prints an address with an embedded token; a user on the same machine
  opens it and starts working right away. After a server restart the old link stops working — a new one from the server
  output is needed.
- CUJ14. The data survived a failure. The user saved a number and saw the confirmation; then the process crashed or power was lost.
  After the server restarts the number is in place. A save that did not get a confirmation is, after a restart,
  either fully present or fully absent. If storage is corrupted, the server does not start and reports to the operator;
  data is not silently lost; the operator restores the database from a backup (CUJ15).
- CUJ15. Backup and restore. The operator on the server machine makes a copy of the database with a command while the server is running;
  users and programs keep working without interruption. The copy corresponds to one consistent moment.
  Restore is starting the server on the copy.
- CUJ16. The server cannot perform a change. If the database is read-only or there is a storage error, the user
  learns about it when their action fails («Сохранение не выполнено: база доступна только для чтения» ("Save failed: the database is read-only") or "ошибка
  хранилища, обратитесь к оператору"); search keeps working while it can. The UI does not show the server state in advance.

The clarifications below (section 3) take precedence over the CUJ text where they differ (for example, CUJ5: not a substring, but the start of a
word).

## 3. Owner decisions in force [USER]

### Data
- Record = (number, date) → name. For one date a number has exactly one owner.
- The same name may own the same number on another date — that is a separate record.
- Date is a calendar day; any date not earlier than 1876-03-07 is valid, future dates are allowed.
- The date in the CUJs is UI behavior. If the request to the server has no date, the server substitutes the zero date 1876-03-07 (grant of
  Bell patent US 174,465; see facts.md F-1).
- The number is entered in international format with "+" and a country code.
- Name repeat: "the same name" for a repeated add means the exact spelling; a different spelling (for example, "Ann" versus the
  saved "ANN") is a conflict that the user resolves explicitly. The clarification "after whitespace trimming and NFC" was in the
  description of the option the owner chose.
- Name searches are case-insensitive (see "Search and sorting").

### Operations
- Add only creates a record. If (number, date) already has a different name — a conflict error, the response carries the current
  name and the number version; there is no silent overwrite. Repeating the add of an identical record is a silent success with no changes.
- Renaming the name of an existing record is a separate explicit operation; the UI calls it after the confirmation in CUJ3.
  Renaming a missing record gives "not found". Fixing a name (a typo) in the UI is only through CUJ3; there is no separate "edit"
  button in the UI.
- Delete removes one record (number, date).
- Versioning is per number (strictly per number, not per date or name); any change to a number's records changes the version.
  The expected version is required for rename and delete; a mismatch is a conflict with the current version in the response.

### Search and sorting
- Search by number is exact match only; the result is all records of the number, date DESC; the full list without pagination.
- Search by name is by the start of any word of the name; all words of the query must match (AND); case-insensitive;
  "ё" = "е"; an empty query is an error; "next" pagination by cursor, without a total number of results.
- Separately, exact search "all records for the whole name" is kept (case-insensitive).
- Result order for name search is alphabetical under ru locale rules, then date DESC.

### Interfaces
- The UI is a web page in a browser on the same machine; the UI is available only via loopback; sign-in is by a link with an embedded
  token that the server prints (the token changes on every start).
- The programmatic API is HTTP + JSON; the same operations as in the UI, plus exact whole-name search; there is no administration
  through the API.
- API clients: local programs (token from a file, changes on every server start) and network services on other
  machines (API only, over HTTPS; TLS in the server itself).
- Network services: a permanent token per client, permissions "read" or "read and write"; the operator issues, views and
  revokes tokens with a command on the server machine; a per-token request limit.
- Compatibility: the API version is in the path; within v1 fields and error codes are only added.
- One server process owns the database; the UI and API are its clients.
- The UI shows server problems (read-only, storage error) only when an action fails.

### Scale and reliability
- Scale: 10^8 records.
- On data corruption the database does not open (refusal, no silent loss).
- An empty database is created by an explicit operator command; without it the server does not start.
- Backup is by an operator command, without stopping the server, a consistent snapshot; restore is starting the server on the
  copy.

### Out of scope
- Batch load and import (the typical story is a person typing in a number by hand); a "who changed what" log; administration through the API; a v1 → v2 transition policy.

### Implementation
- "From scratch" = the DB itself is written by hand: storage engine, indexes, durability, operation and server logic.
- Proven libraries are allowed for the rest (Unicode/collation, HTTP, JSON, random numbers, CRC, base64, etc.).
  Ready-made storages and search engines are forbidden. Each crate comes with a justification (Decision Record) and a
  cargo deny check. Fact Grounding is mandatory.
- The whole Unicode stack is ICU4X of one version, Unicode 18 (see facts.md F-2).
- HTTP and JSON are the hyper and serde_json crates.

## 4. Owner decisions made under old constraints — revisit before carrying over
The owner made these when crates were forbidden and the model was a "name–phone pair". The new specification must propose them
again or replace them, with a question to the owner:
- There is no rollback to an old program version (and format) (owner decision from the storage iteration, see archive
  iter2-storage-v13.md, section S5/S8).
- The goals "file ≤ 40 GB for 10^8 records" and "search p99 ≤ 10 ms": computed for the "name–phone pair" model; the owner
  confirmed keeping them when choosing word-prefix search (in the description of the chosen option). Recompute for the new
  model.
- A 16 KiB storage page with a proven bound on key length.
- Strict number format: "+" and 1–15 digits; separators are removed; a number without "+" is rejected (this is how the option
  the owner chose was worded; the specific list of separators and other processing details are not owner decisions).
- The API error body is `{code, field}`, HTTP status by code.
- Server limits: 256 connections, request body 32 KiB, request timeout 10 s, connection idle 60 s — accepted under the
  "server only on loopback" model, before network clients appeared.
- For checking the 10^8 scale — a test database load-generation tool, not a product operation (the option the owner
  chose when declining batch load).
- Retrying a rename after a connection drop gives "not found" (accepted before the number version appeared).
  Note from the previous orchestrator: probably now closed by versioning — verify.

## 5. Not decided by the owner (defaults that nobody confirmed)
The new specification must propose and ask: name length; isolation level of reads and writes; when a write is considered
acknowledged; target write latency; multi-word search latency; the per-token request limit value and whether it applies to local programs and the UI;
where the TLS certificate comes from (who issues it, how it is replaced) and whether TLS is used for the UI and local programs on
loopback; the channel through which operator commands (tokens, backup) reach the running server (an API for
administration is excluded); interface behavior (what to show after saving, the date format in the UI, undoing a delete) —
the UI iteration.

## 6. Principles of building the specification
See `process.md`: Critical User Journey, Elephant + Goldfish, (architectural) Decision Record, Fact Grounding;
LLM-expert review — when the whole spec is ready.
