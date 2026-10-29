# 0017. Scale and implementation constraints
- Status: Accepted
- Label: [USER]
- Date: 2026-10-27
- CUJ: X-1, X-4; items 1.1, 1.6, 1.7 of the original wording ("Written from scratch in Rust")

## Context
The original wording requires storing many records reliably and writing from scratch in Rust. Brief §4 notes that the
owner made some decisions when crates were forbidden; now verified libraries are allowed.

## Decision
1. Scale: 10^8 records (brief §3/Scale and reliability).
2. "From scratch" means the DB itself is written by hand: the storage engine, indexes, durability, the logic of
   operations and of the server (brief §3/Implementation); the language is Rust (brief §1, item 7).
3. Verified libraries are allowed for the rest (Unicode/collation, HTTP, JSON, random numbers, CRC, base64, etc.); each
   crate comes with a rationale (Decision Record) and a cargo deny check; Fact Grounding is mandatory (brief
   §3/Implementation).
4. Ready-made storages and search engines are forbidden (brief §3/Implementation).
5. HTTP and JSON are the hyper and serde_json crates (brief §3/Implementation).
6. The whole Unicode stack is ICU4X of one version, Unicode 18 (brief §3/Implementation; ADR 0010).
7. **Changed by OD-6 (2026-10-28).** The scale is 10^7 records instead of 10^8 (changes brief §3 "Scale and
   reliability", X-1; the architecture is unchanged); the numbers derived from 10^8 are recalculated in ADR 0044, 0049,
   0052, 0056–0059, 0071, 0079. The text of item 1 ([USER], brief) was not rewritten.

## Alternatives considered
- A full ban on third-party crates (in force under the earlier decisions, brief §4: "when crates were forbidden"):
  replaced by permission for verified libraries (brief §3/Implementation).
- A ready-made storage or search engine: forbidden (brief §3/Implementation).

## Consequences
- The scale criterion can be verified only after the targets are decided: file size and search latency (Q-47), write
  latency (Q-58), multi-word search latency (Q-59), start time (Q-42); until then the requirement has no numeric
  threshold (requirement X-1 of spec `user-journeys`). The way to populate the database to check scale is Q-53 (ADR
  0019).
- Each crate requires a separate ADR and a cargo deny check.

## Sources
- `docs/source/brief.md`, section 1 (items 6, 7); section 3 "Scale and reliability", "Implementation"; section 4.
