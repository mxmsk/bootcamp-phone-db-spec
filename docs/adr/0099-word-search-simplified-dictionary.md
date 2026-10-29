# 0099. Word search: dictionary D without aggregates, strategies G2 and G3 first
- Status: Accepted (OD-7)
- Label: [OURS] (owner decision OD-7 item 4 — [USER])
- Date: 2026-10-28
- CUJ: CUJ5, CUJ8, CUJ12, X-1

## Context
ADR 0045 item 3 kept aggregates in the internal nodes of D (a record count, a sum of counters), ADR 0048 — three candidate sources and a planner. Aggregates must survive split/merge, bottom-up loading and snapshots — the most expensive part of the search (`docs/reviews/rust-expert.md`, item 9). At 10^7 records D ≈ 20 MB.

## Decision
1. D is an ordinary B+-tree `esc(word key)` → posting count, with no aggregates in internal nodes.
2. `k(p)` (the number of words) and `C(p)` (the number of postings) of a prefix are computed by a bounded scan of D from the first entry of the prefix, stopping at a threshold: `k > K1` (1,024) or `C > M1` (131,072). Beyond the threshold the value is treated as "above the threshold"; the exact value is needed only below the threshold. The cost per prefix is a descent (≈ depth) plus ≤ 4 leaves of D.
3. Sources: first G2 (traversal of O) and G3 (intersection, leapfrog); G1 (bounded merge of postings) is added by the load profile. The choice is by estimates with thresholds M1, K1, S_max as before.
4. The result does not depend on the strategy (the requirement is preserved); the strategies are cross-checked by a differential test. CUJ5, CUJ7, CUJ8 do not change.

## Alternatives considered
- Aggregates in D (ADR 0045 item 3): rejected as too expensive for v1.
- G2 only: heavy rare prefixes read all of O (ADR 0048, alternatives); G3 covers this.

## Consequences
- "Changed by OD-7" in ADR 0045 item 3 and 0048 items 1–3, 10; two `storage-indexes` requirements are changed.

## Sources
- OD-7 item 4; `docs/reviews/rust-expert.md` item 9; ADR 0045, 0048.
