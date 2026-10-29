# 0058. Rebuilding derived indexes on format change: impact on start
- Status: Accepted
- Label: [OURS]
- Date: 2026-10-27
- CUJ: CUJ5, CUJ11, CUJ13, CUJ15

## Context
ADR 0039 item 3: the indexes derived from the name (order, words, exact search) carry a format label (the version of our code and the version of
`icu_collator`); when the label does not match, the index is rebuilt before requests are accepted. Q-82: the impact on start and the operator's
actions. OD-4 item 8: there is no separate format migration command; the new version opens the database itself. In v1 the ICU4X code and data do not
change (ADR 0039 item 4), so a rebuild happens on a change of the key record format in our code, on a backup of an earlier v1
format, and on an index error, not on an ordinary update.

## Decision
1. A rebuild runs at start if the label of at least one derived index is not equal to the server's label (including
   for a backup of an earlier v1 version, ADR 0057 item 10). If they match — there is no rebuild.
2. It blocks start: until the rebuild ends, client requests are not accepted (connections are not accepted or the refusal "server
   starting"; the kind — area 3), an incomplete index is not given to clients. The client waits and retries, as after a restart (CUJ7: the token
   is re-read). No operator actions.
3. Process: names are read from one snapshot; the new index is built next to the old one and swapped in one atomic step;
   a failure — the earlier index is intact, the rebuild starts over (no resume from the middle). Checks after the build: the number of cells O equals the number of records R, the number of word postings W equals the sum of the number of
   distinct words of the records, the number of whole-name postings equals the number of records, the sum of counters D equals the number of word postings; the order of adjacent keys by key bytes (ADR 0039 item 9 (d), F-45). A failure — the old index
   stays, start ends in a refusal with a message. Order and matches do not change (the reference tests of ADR 0039 item 5 pass).
4. To the operator during the rebuild: the reason ("the order index format has changed"), the share done, a time estimate; on
   completion — the duration. The channel — Q-66 (area 3/4).
5. Cost (measurement and calculation, `research/snapshots`): the sort key — 1.96 M names/s per thread, 8.4 M/s on 4 threads (measurement, Apple M3 Pro;
   average key 24.2 bytes): for 10^8 names 51 s on one thread, 12 s on four. I/O (read ~6 GB, index write ~4.7 GB,
   external sort; assumptions in the script): 10 s at 2 GB/s, 101 s at 200 MB/s. In total for the order index from ~20 s to ~2.5 min; the word and exact search indexes were not measured (case folding and word boundaries — up to
   4 words per name): add during implementation. The estimate for a full rebuild of O, W, D — ≈ 10–30 min for 10^8 (ADR 0045 item 8, ADR 0049 item 5:
   a model of ≈ 2 min plus a margin of 5–15 times for real efficiency), the goal — no longer than 60 minutes (D-26); the ADR 0058 estimate before this
   fix pass concerned only the order index. Parallel building across threads is allowed.
6. A requirement for releases: a change of the format label in a v1 release is stated in the release notes together with a time estimate on a 10^8 database;
   frequent changes are undesirable (each one is a start downtime).
7. Q-82: the nature of the impact — "start is longer by the rebuild time, clients wait, no operator actions"; updating ICU4X in v1 is not
   standard (ADR 0039 item 6) and the rebuild does not change the order.
8. An index error (ADR 0039 item 3 lists it among the causes) is a boundary question with the integrity subtask: a rebuild from names
   is allowed only if the data (names) is intact; otherwise — "storage error" (ADR 0014). See Q-123.
9. **Changed by OD-6 (2026-10-28).** The estimates for 10^8 (51 s on one thread; read ~6 GB, write ~4.7 GB; a full rebuild of O, W, D ≈ 10–30 min; goal ≤ 60 min) are recomputed proportionally for 10^7: 5 s on one thread, 1.2 s on four; I/O ~0.6 and ~0.47 GB; a full rebuild ≈ 1–3 min, goal ≤ 6 min (D-26 accepted by the owner, OD-6).

## Alternatives considered
- A background rebuild while the server runs (the old index serves requests, the new one is built in parallel, then swapped):
  not a start downtime, but needs the code to read the old format kept (the code of the earlier key version and an ICU4X-compatible comparison)
  and writes made during the build duplicated into the new index; in v1 with ICU4X pinned a rebuild is rare. Not chosen;
  put to the owner in ADR 0049 item 5 ([DEFAULT], D-26) as a possible future.
- A separate operator command "migrate the format": excluded by OD-4 item 8.
- Rebuild lazily on first access: rejected, the result order for 10^8 cannot be computed on the fly within the latency targets.
- Store keys together with the version for two formats: rejected, doubles the file size (the target is 40 GB).

## Consequences
- The start time (Q-42) on a rebuild is a separate figure for the owner: ADR 0049 item 5 (D-26).
- Format and index subtasks: the format label is in the header of every derived index; the rebuild is from names from one snapshot.

## Sources
- ADR 0010, 0013, 0039 items 3, 4, 6, 9; F-2, F-18, F-45; `research/snapshots/keybench`, `retention.py`.
