# 0022. Name and query length cap: 200 Unicode scalar values after normalization
- Status: Accepted
- Label: [USER] (items 1–4: OD-5, D-3); item 5: [OURS]
- Date: 2026-10-27
- CUJ: CUJ1, CUJ2, CUJ3, CUJ5, CUJ7, CUJ9; item 1.3 of the original wording

## Context
The owner decided: a name longer than the cap is rejected with an error, without truncation (ADR 0003, item 4), but did
not set the cap itself or the unit of measurement (brief §5; Q-55). The cap changes visible behavior, so it is a
question for the owner; a default value is proposed. Storage (Q-48, ADR 0019) may require a different bound; that is a
requirement on the storage area.

## Decision
1. [USER, OD-5, D-3] The cap is 200 Unicode scalar values (code points other than surrogates) in the normalized text
   (after NFC and edge trimming, ADR 0021). A name of exactly 200 is valid, of 201 is an error (name, "too long"), the
   record is not created or changed. This is the only source of the cap: `write-operations` and `name-search` refer to
   it and set no values of their own.
2. The unit is the Unicode scalar value, not a byte and not a grapheme cluster. A stored name takes at most 4 × 200 = 800
   bytes of UTF-8 (UTF-8 encodes a character in 1–4 octets, F-5). This is a bound for the name itself only; the sizes of
   derived keys (comparison, sort) are set by measurement, not by this bound (item 5).
3. The length is counted on the normalized text: NFC may change the number of code points (for example, "e" + U+0301 →
   "é": 2 → 1), so the check is performed after NFC and trimming (F-4). The text is counted, not the keys derived from
   it: case folding, replacing "ё", splitting into words do not change the length (200 characters "ß" are a valid 200).
4. [USER, OD-5, D-3, part] A name-search query (both kinds of search, ADR 0008, 0031) has the same cap and is counted the
   same way: after the same normalization (ADR 0021, item 2), before the comparison key is built. Exceeding it is an
   error (query, "too long"); the query is not truncated (ADR 0034). The number of words in a query is not limited by
   this: when splitting into words by the rules of ADR 0030 (option B), in scripts without spaces (Han ideographs,
   hiragana, Thai, Lao, Myanmar) each character may be a separate word, so there can be up to 200 words (a name "李…" of
   200 characters is 200 words); no other cap on the number of words is introduced. The worst case (a 200-word query, a
   database of names made of such words) is a load on search and the word index, a question for the storage area.
5. [OURS] Upper bounds of the derived values of a valid name (200 scalar values), measured by a run on Unicode 18, ICU4X
   2.4.0, locale ru, `CollatorOptions::default()` (`research/records/sortkey-check/`, run of 2026-10-27, F-39, names
   "x×200" for each of the 1,110,876 scalar values x that are not Cc and are stable under NFC): name: 800 bytes of UTF-8;
   comparison key (NFC, full case folding, NFC): 1,000 bytes; sort key (`write_sort_key_to`): 8,513 bytes [observed
   maximum, the bound is not proven, see Q-103] (U+FDFA ×200; typically 410–630; the next ones go up to 3,652). Storage
   SHALL hold the name, the comparison key and the sort key of any valid name without truncation: truncating a key breaks
   the equality "equal alphabetically" (ADR 0036, item 2). Status of the caveat: for homogeneous names (x×200) the bound
   is proven by enumerating all characters; for mixed names it was checked by sampling and is not proven (F-48; review
   by run: alternating U+FDFA with 8 other characters, random mixes of the 40 "longest" characters, hill climbing: the
   maximum sort key 8,513, mixes at most 7,014; a single maximum of 46 bytes gives only an additive bound of 9,200). The
   implementation rechecks the bounds on mixed names and when the data version changes.

## Alternatives considered
- UTF-8 bytes: rejected for the visible bound, Cyrillic takes 2 bytes per letter, a Russian name would get half the cap
  of a Latin one; the user does not see bytes. A byte estimate of the name remains as a consequence (item 2).
- Grapheme clusters: rejected, the number of clusters depends on the segmentation rules of the Unicode version and does
  not limit size (a cluster can be arbitrarily long), a second, byte bound would be needed.
- Other values (100; 255; 1000): 100 may not hold long organization names; 255 is the historical habit of a byte field,
  with no argument for code points; 1000 bloats the word index. 200 is enough for long full names and titles. The
  conclusion "fits in a page with room to spare" by name size is not drawn: the sort key is up to 8,513 bytes (item 5).
- No cap: rejected by the owner (ADR 0003, item 4: there is a cap).
- A separate, larger cap for the query (for example, for long multi-word queries): rejected, a query longer than any
  valid name cannot find anything by exact search, and word search on such a query adds no meaning; one cap is simpler to
  explain and verify. Rejected by the owner within D-3 (OD-5).
- Counting the query length before NFC or before trimming: rejected, then a query of 200 characters with whitespace at
  the edges would be rejected while the same name is accepted; the rule "as for the name" loses its meaning.

## Consequences
- The owner confirmed 200 (for the name and for the query; OD-5, D-3); the unit is scalar values.
- Storage must hold the name (800 bytes), the comparison key (1,000 bytes) and the sort key (8,513 bytes [not proven, see
  Q-103]) of any valid name without truncation (item 5). The sort key size exceeds the name size by up to 10.6 times, so
  whether a 16 KiB page (ADR 0019, item 2, not yet decided) fits for the order index (sort key + date + number) is a
  question for area 2 (Q-48). If storage cannot hold the bounds, we return the question to the owner (a revision of this
  ADR, cap 200).
- Input before normalization is longer than the normalized text: one character after NFC may replace up to 4 input
  characters (the maximum NFD length of one scalar value, F-20), and the trimmed edges are unbounded. The semantics of
  records is unambiguous: a name or query that is valid by the cap is not rejected because of the input size before
  normalization. The specific byte cap on input (not less than 4 × 200 = 800 bytes for characters plus room for the
  edges [imprecise: the units are scalar values, not bytes; not proven, see Q-101]) and the form of refusal when it is
  exceeded belong to the API area (a requirement for other areas).
- Showing the cap to the user (error text, a counter in the form) belongs to the UI/API area.

## Sources
- ADR 0003, 0008, 0019, 0021, 0034; brief §5 ("name length"); Q-55, Q-48 (`docs/open-questions.md`).
- F-4, F-5, F-20, F-39, F-48 (`facts.md`): UAX #15; RFC 3629; Unicode 18 measurements (`research/records/`,
  `research/records/sortkey-check/`).
