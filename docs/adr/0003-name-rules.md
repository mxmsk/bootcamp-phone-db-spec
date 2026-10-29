# 0003. Name rules: "the same name", normalization, empty and too long
- Status: Accepted
- Label: [USER]
- Date: 2026-10-27
- CUJ: CUJ1, CUJ2, CUJ3, CUJ5, CUJ9; item 1.3 of the original wording

## Context
For repeated add, conflict and overwrite, we need to define when two names count as the same one, how a name is
normalized on save, and what to do with an empty or too long name.

## Decision
1. "The same name" for repeated add means exact spelling; a different spelling (for example, "Ann" against a stored
   "ANN") is a conflict that the user resolves explicitly (brief §3/Data).
2. On save the name is trimmed of whitespace at the edges and converted to NFC; "the same name" is an exact match after
   that (OD-3, Q-3; the clarification "after whitespace trimming and NFC" was in the description of the option the
   owner chose, brief §3/Data).
3. An empty name or a name consisting only of whitespace is an error (OD-3, Q-3).
4. A name longer than the cap is rejected with an error, without truncation (OD-3, Q-81). The cap itself is defined in
   the data model iteration (OD-3, Q-81; open question Q-55).

## Alternatives considered
- Truncating a name that is too long. Rejected: reject with an error, without truncation (OD-3, Q-81).
- Comparing names case-insensitively on add. Rejected: add distinguishes "Ann" and "ANN" (brief §3/Data); name search
  ignores case (ADR 0008); the rules are deliberately different.
- No other options are named in the sources.

## Consequences
- Whitespace inside a name is not changed, only the edges are trimmed; "Ann  Smith" (two spaces) and "Ann Smith" are
  different names on add and in exact search (a consequence of item 2; the set of "whitespace" characters is open
  question Q-84).
- The name length cap (value and unit) is not set: Q-55; it is linked to the storage page decision (Q-48, ADR 0019).

## Sources
- `docs/source/brief.md`, section 3 "Data" (name repetition, "Ann"/"ANN").
- Owner decision of 2026-10-27: all 27 recommendations of package 1 were accepted (OD-3), including
  Q-3 ("trim whitespace at the edges, convert to NFC; 'the same name' is an exact match after that; an empty name or a
  name only of whitespace is an error") and Q-81 ("reject with an error, without truncation"). Place of record:
  docs/owner-decisions.md, OD-3.
