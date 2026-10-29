# 0032. Semantics of the "next" cursor
- Status: Accepted
- Label: [OURS]
- Date: 2026-10-27
- CUJ: CUJ5, CUJ7, CUJ8, CUJ11, CUJ12, CUJ14

## Context
The brief: "next" pagination by cursor, without a total result count (ADR 0008 item 7). Not defined: what the client sees
if data changes between pages, and when a cursor stops being accepted (CUJ5: "the cursor is not accepted: the data
changed or it is corrupted", Q-30). The cursor format is the API area.

## Decision
1. A cursor is a position in the total result order (ADR 0009, Q-73): the next page is the records strictly after the last
   one returned, computed from the data at the time of the request for this page. There is no data snapshot.
2. Consequences: a record that matches the query and did not change during the traversal appears exactly once; a deleted
   one does not appear; one added after the cursor appears, one added before the cursor does not; a record with a changed
   name is shown by the data at the time of the page request: if it matches the query and stands after the cursor it is
   shown (and the earlier copy on an already returned page stays with the client), otherwise not. The order within a
   traversal is never violated.
3. "The same query" is the same kind of search and the same query fingerprint: for word search the sets of word keys are
   equal (ADR 0030); the order of words, repeats, case, "ё" and the number of spaces do not matter ("ann smith" = "SMITH
   Ann" = "ann ann smith"); for exact search the query keys after trimming are equal (whitespace inside is significant).
   A cursor is valid for the same query regardless of data changes, time, restart, an update within v1 and a restore from
   backup: there is no server state for a cursor. There is no lifetime. The fingerprint is a fixed deterministic function
   (kind of search, the set of word keys or the query key) with a result that is unchanged within v1 on any platform,
   build and version of the compiler and libraries; a random or version-dependent hash (for example, Rust's
   `DefaultHasher`) does not fit: the cursor would not survive an update, and the requirement of item 3 would be false. The
   algorithm and encoding are chosen by the API area; the first release has no "earlier release", so the API area fixes,
   before the first release, normative test vectors (query → set of word keys → fingerprint → cursor, including repeats,
   the order of words, exact search, "ё"+U+0308) and a golden test on them; within v1 the vectors do not change, and later
   releases pass the same test. The input of the fingerprint function is unambiguous: the kind of search and the keys (for
   words, without repeats, in ascending code point order; each key with its length), with no separators that could occur
   in a key.
4. A cursor is an opaque value: the position (name, date, number of the last record returned, ADR 0036), the kind of
   search, the query fingerprint, a checksum. The error "cursor invalid" (ADR 0034) arises if the value does not decode
   (empty, truncated, with a tail), the checksum does not match, or the kind of search or the query fingerprint does not
   match. [OURS] There is no signature with a secret: a cursor correct by format, composed by the client, is accepted and
   gives the results right after the encoded position. Rationale: a cursor reveals nothing beyond the right to read the
   results (the position is a point in the result order; records are still selected by the query); a signature would
   require a long-lived secret that survives a restart, an update, a restore from backup and relocation, with no benefit.
   A cursor is "correct by format" if it decodes unambiguously (one content, one string, with no nonzero unused bits and no
   tail), its length is no greater than the encoding of the longest valid position, the checksum matches, and the position
   satisfies the value rules of `records`: the name is valid, normalized (NFC, trimmed), not longer than 200; the number is
   in canonical form; the date exists and is in range. Otherwise it is "cursor invalid". Reason: the position enters
   comparisons during selection; arbitrary values (a name with U+0000, a megabyte name, a date out of range) must not reach
   storage. A promise "a cursor we did not issue is rejected" would be an unverifiable requirement. The checksum catches
   corruption in transit, not forgery; what is verifiable are the deterministic cases (empty, truncated, with a tail) and
   the scenario of accepting a composed cursor.
5. A cursor is in the response only if at the time of the response there are more records after the page.
6. A cursor does not depend on the page size: it encodes a position, not a page number (if the interface allows choosing a
   size).
7. **Changed by OD-6 (2026-10-28).** The scale is 10^7 records (ADR 0017); the numbers and estimates for 10^8 in this ADR
   are calculation history, the current values for 10^7 are in ADR 0044, 0049, 0052, 0058, 0059; non-linear estimates are
   recalculated on a prototype (T, `docs/open-questions.md`).

## Alternatives considered
- A data snapshot for the duration of a traversal: rejected, a server context per traversal is needed (lifetime, limits,
  leaks), while the brief needs a simple "next"; CUJ5 does not require repeatable reads.
- An offset: rejected, with inserts and deletes records are skipped or repeated.
- A cursor invalid after a data change (as CUJ5 hints): rejected, the client would lose its place at every write in a
  database of 10^8 records.
- A cursor lifetime: rejected, the position does not require it.
- Signing the cursor with a secret (HMAC): rejected, a long-lived secret in storage that survives a restore and relocation
  is needed (a requirement for areas 2 and 3), with zero benefit (see item 4). If the owner demands "only issued ones",
  this is a change here and a new requirement for storage.

## Consequences
- Storage: a search "after a position" must be computable without previous pages and without server state (a requirement
  for storage). Affects the isolation level Q-56: consistency within one page is enough.
- The CUJ5 scenario "The cursor is not accepted (data changed…)" is clarified: data does not spoil a cursor.
- The position does not depend on the record at the cursor no longer existing.
- The API area sets the cursor format with a checksum and a query fingerprint (an unambiguous encoding, a cursor length
  cap, a fingerprint of at least 128 bits); the fingerprint algorithm (a hash of the word keys, ordered and without
  repeats) is chosen by it, but its result is observable (a cursor after an update), so unchangedness within v1 is a
  requirement (item 3).

## Sources
- `docs/source/brief.md`, section 3 "Search and sorting"; ADR 0008, 0009; Q-30, Q-73, Q-56.
