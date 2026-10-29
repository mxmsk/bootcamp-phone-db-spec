# 0034. Errors and empty results of name searches
- Status: Accepted
- Label: [OURS]
- Date: 2026-10-27
- CUJ: CUJ5, CUJ7, CUJ8

## Context
We need abstract error kinds for name searches (without codes and JSON) and the distinction between an error and empty results.

## Decision
1. "Empty query" = (query, "empty") — for word search, a query with no words; for exact search, a query that is empty after trimming (ADR 0008 item 6, ADR 0031).
2. "Query too long" = (query, "too long") — the query length exceeds the name cap from ADR 0022 (200 Unicode scalar values, product owner decision
   OD-5, D-3); the query is converted to NFC and trimmed at the edges, like a name (ADR 0021), and its length is counted in the same units after
   that; it is not truncated (like a name, ADR 0003 item 4). This protects against expensive queries and keeps symmetry: a longer name cannot be saved.
3. "Cursor invalid" — ADR 0032 item 4 (it does not decode, the checksum does not match, the position violates the value rules,
   or it belongs to a different kind of search or query).
4. Check order (when there are several violations, the first is reported; the checks are deterministic and operate on the values themselves): "too
   long" → "empty" → "cursor invalid"; the cursor is checked only for a valid query. Rationale: the length is counted
   on the text before segmentation, so expensive segmentation is not run for a multi-megabyte query (the protection from item 2); "empty" for word
   search requires segmentation, and without a length cap it would be unbounded; the fingerprint for the cursor is computed from the
   words of the query, so a cursor makes sense only for a valid query. A query of 300 hyphens is "too long", not "empty";
   for exact search an empty query is never long, so the order is not visible there.
5. Control characters in a query [OURS]: the name validity check (ADR 0021 item 5, D-2) does not apply to a query, and there is no
   "control character" error for a query. In word search a control character (category Cc) does not form a word (ADR 0030 item 3, OD-5, D-19) and under
   UAX #29 is separated by boundaries from neighboring letters, that is, it acts as a separator (the query "Ann"+TAB+"Smith" searches for two words; a lone U+0000 is "empty"); in exact search it is compared as is, and such a name
   does not exist (empty results, not an error). Whitespace control characters at the edges (TAB, LF, VT, FF, CR, U+0085) are trimmed as
   for any query. Rationale: a query is not stored, so there is nothing to protect; a query pasted from the clipboard with a tab or line
   feed is an ordinary scenario, and a refusal instead of a search only gets in the way; a new reason in the `records` dictionary is not needed, the dictionary
   does not change.
6. No matches is a successful response with an empty list, without a cursor, not an error (like search by number, ADR 0007).
7. Searches do not change data; in the "read-only" state and while storage is being read, they work (CUJ16).

## Alternatives considered
- An error (query, "control character") modeled on the name: rejected — it requires a new reason in the `records` dictionary, and there is no benefit
  (a query is not stored); if the owner wants symmetry with the name, that is a fix to the `records` dictionary and to the check order (between
  "too long" and "empty").
- Order "empty" → "too long": rejected — segmentation of a query with no cap (see item 4).
- Not capping the query length: rejected — a query of hundreds of thousands of words becomes a load; a name has no such case.
- "Not found" as an error: rejected — symmetry with search by number (Q-6, ADR 0007).

## Consequences
- Three error kinds that the client can distinguish; the check order is item 4; codes belong to the API area. The error kinds for name values (the dictionary) are set by the `records` capability;
  "query too long" and "empty query" are value errors of the search query, the cap is per ADR 0022.

## Sources
- ADR 0003, 0007, 0008, 0021, 0022, 0032; Q-29.
