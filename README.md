# "Names and Phones": specification of a database to be implemented in Rust

A single-user database of names and phone numbers, to be written from scratch in Rust. It has its own storage engine
(B+-tree, WAL, snapshots), a web interface, an HTTP API and an operator CLI. The specification is written in
[OpenSpec](https://github.com/Fission-AI/OpenSpec) (the `spec-driven` schema), in English; the product's UI and CLI messages are in Russian.

**Status: the v1 specification is complete, implementation has not started, there is no code in the repository.**

| | |
|---|---|
| Capabilities | 17 (`openspec/specs/`) |
| Requirements / WHEN/THEN scenarios | 387 / 1,418 |
| ADRs | 0001–0101 (`docs/adr/`) |
| Owner decisions | OD-1…OD-8 (`docs/owner-decisions.md`) |
| Open questions to the owner | 0 (Q-100 is deferred separately, until the order is checked against Unicode 18) |
| Technical questions for the implementation | 62 (`docs/open-questions.md`, category (T)) |

## The original task

The only original source of requirements is the [brief](docs/source/brief.md), section 1:

- It must be a good database
- It must have a visual user interface and a programmatic API
- It must store names and phone numbers
- It must be possible to search names by phone number
- It must be possible to search phone numbers by name
- It must store many records reliably
- It must be written from scratch in Rust

The user journeys (CUJ1–CUJ16) and the owner decisions are in the same brief (sections 2–3) and in
[docs/owner-decisions.md](docs/owner-decisions.md). Later owner decisions take precedence over the brief.

## What we are building, in brief

- **Data model.** A record is (number, date) → name. A number is in the international format `+<country code>`. Over
  time one number is assigned to different owners; the date is the day from which the assignment holds. Adding
  without a date is allowed only for a new number (date 1876-03-07). A number has a version, which protects against
  concurrent changes
  ([ADR 0002](docs/adr/0002-record-model.md), [0006](docs/adr/0006-number-version.md)).
- **Search.** By number: exact match only, history by date DESC. By name: by word prefix (UAX #29) and exact
  whole-name; order: ICU4X ru collator, then date DESC, then number
  ([ADR 0007](docs/adr/0007-search-by-number.md), [0008](docs/adr/0008-search-by-name.md),
  [0009](docs/adr/0009-result-ordering.md), [0010](docs/adr/0010-unicode-stack.md)).
- **Storage.** Its own B+-tree, 16 KiB page, WAL with CRC-32C, checkpoints, doublewrite buffer, one writer and many
  readers on snapshots. Scale: 10^7 records, file ≤ 8 GB
  ([OD-6](docs/owner-decisions.md), capability `storage-*`).
- **Server and API.** HTTP/1.1 on hyper, `/v1/...`, error body `{code, field, reason, hint, message}`. Three kinds of
  tokens: UI (a link with a token), local programs (a token file, loopback), network services (HTTPS)
  ([ADR 0011](docs/adr/0011-interfaces.md), [0012](docs/adr/0012-access-and-tokens.md)).
- **Operator.** The `npdb` program: `init`, `serve`, `token-issue|list|revoke`, `backup`, `stop`; backup without
  stopping the server; a separate load-generation tool `npdb-fill` for checking scale.
- **UI.** Resources are embedded in the program, a strict CSP, sign-in by a link with a token.

## Reading order

1. [docs/source/brief.md](docs/source/brief.md): what we are building and why (the original task, CUJs, owner decisions).
2. [docs/owner-decisions.md](docs/owner-decisions.md): owner decisions after the brief (OD-1…OD-8).
3. [openspec/specs/user-journeys/spec.md](openspec/specs/user-journeys/spec.md): CUJ1–CUJ16 and the cross-cutting
   requirements X-1…X-5 as verifiable scenarios.
4. The remaining capabilities in `openspec/specs/`, by area (table below).
5. [docs/adr/README.md](docs/adr/README.md): the index of decisions with rationale and alternatives.
6. [docs/open-questions.md](docs/open-questions.md): what is left for the implementation to decide.
7. [docs/reviews/rust-expert.md](docs/reviews/rust-expert.md): the expert assessment of feasibility in Rust:
   risks and an effort estimate per component.

## Capabilities

| Area | Capability | Requirements / scenarios |
|---|---|---|
| User journeys | `user-journeys` | 21 / 213 |
| Data and operations | `records`, `write-operations`, `number-search`, `name-search`, `result-ordering` | 76 / 321 |
| Storage | `storage-format`, `storage-indexes`, `storage-durability`, `storage-snapshots` | 92 / 258 |
| Server and API | `http-api`, `api-access`, `server-runtime` | 87 / 271 |
| Administration | `operator-lifecycle`, `operator-control` | 62 / 189 |
| UI | `web-ui-shell`, `web-ui-flows` | 49 / 166 |

## How the principles are applied

| Principle | Where in the repository |
|---|---|
| Critical User Journey | Every requirement traces to a CUJ/X-n or to an item of the original wording (`openspec/config.yaml`) |
| Elephant + Goldfish | The orchestrator holds the context; subtasks and reviews are fresh agents given only the sources, the draft and the criteria (`docs/process.md`) |
| An ADR for every decision | `docs/adr/`, labels [USER] (owner), [OURS] (reasoned decision), [DEFAULT] (default value, approved by the owner in OD-5/OD-6) |
| Fact Grounding | `facts.md`: verified facts with a source and a date; `research/`: verification programs (Unicode 18, ICU4X, UAX #29, sort order, size calculations) |
| LLM-expert review | Final reviews by capability group (`docs/reviews/review-*.md`) and the Rust expert assessment (`docs/reviews/rust-expert.md`) |

## How the specification was created

The process is described in [docs/process.md](docs/process.md). In brief:

1. CUJ breakdown and questions to the owner → the `user-journeys` capability and the base ADRs 0001–0019.
2. Five areas in order: data, storage, server and API, administration, UI. In each: parallel subtasks (Sonnet), one
   review (Opus, only BLOCKERs and contradictions), one fix pass, change archive.
3. Default values collected and approved by the owner (OD-5, OD-6).
4. Final reviews of the whole spec (4 groups, Opus) and one fix pass.
5. The Rust feasibility assessment (Opus) and the owner decisions on it (OD-7, OD-8).

The history of changes to the specification is in `openspec/changes/archive/` (each change with `proposal.md`,
`design.md`, `tasks.md` and deltas).

## Repository map

```
README.md                 this file
facts.md                  verified facts (Fact Grounding)
docs/source/              the brief (not edited)
docs/owner-decisions.md   owner decisions OD-1…OD-8
docs/process.md           the process of building the specification
docs/open-questions.md    open questions (T) and the history of closed ones
docs/adr/                 ADRs 0001–0101 and the index
openspec/config.yaml      OpenSpec context and rules
openspec/specs/           the current specification (17 capabilities)
openspec/changes/archive/ change history
research/                 programs and results of fact checks
docs/reviews/             final reviews and the Rust expert assessment
```

## Validation

```
npx --yes @fission-ai/openspec validate --specs
```

## Where to start the implementation

The expert recommends starting with a test layer for files with failure injection (without it CUJ14/CUJ16 cannot be
checked), then the page format and the B+-tree, the WAL and recovery, and the multi-version page cache. Storage is
about half of the total work (`docs/reviews/rust-expert.md`, "Effort estimate"). Before the v1 release, Q-100 must be
closed: check the ICU4X sort order against Unicode 18 characters.
