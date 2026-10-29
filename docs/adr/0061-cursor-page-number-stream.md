# 0061. Cursor, page and number results stream
- Status: Accepted ([OURS]: items 1–4, 6); Accepted (OD-6) ([DEFAULT]: item 5 — page maximum, D-38)
- Label: [OURS] (items 1–4, 6); [DEFAULT] (item 5)
- Date: 2026-10-27
- CUJ: CUJ4, CUJ5, CUJ7, CUJ8, CUJ11, CUJ14

## Context
ADR 0032 leaves to the API area the cursor encoding, the fingerprint algorithm and the vectors; ADR 0033 — the allowed page sizes; ADR 0035 item 6
and 0056 require a number stream with a completeness marker.

## Decision
1. [OURS] The cursor is a base64url string without padding (RFC 4648 §5) of a binary structure (numbers are big-endian): format byte `0x01`; search
   kind (`0x01` words, `0x02` exact); fingerprint 16 bytes; position: name length `u16`, name UTF-8 (NFC), date `u32` (days from 1876-03-07;
   maximum 2,967,163), number of number digits `u8`, ASCII digits; CRC-32C (ADR 0043, F-55) over all preceding bytes, 4 bytes.
   Maximum: 2+16+2+800+4+1+15+4 = 844 bytes → 1126 characters (verified by calculation, F-89); longer — `invalid_cursor`. Only the form
   equal to the re-encoding of the decoded bytes is canonical (RFC 4648 §3.5: padding bits are zero); "=" padding is invalid. The position
   is checked against `records` (ADR 0032 item 4). The format byte allows adding a new format without changing old ones; a new format — only
   for new cursors, the old one is always accepted.
2. [OURS] Fingerprint: the first 16 bytes of SHA-256 (FIPS 180-4; the checksum of "abc" = ba7816bf…15ad verified by a run) over
   `"phonedb-cursor-v1" ‖ 0x00 ‖ kind ‖ u32(number of keys) ‖ (u32(length) ‖ UTF-8 key)…`; keys without repeats in ascending UTF-8 byte order
   (= by code points). For exact search — one query comparison key. SHA-256 is chosen as an algorithm fixed by a standard
   without a secret (not `DefaultHasher`); 128 bits — ADR 0032 item 3. The crate (`sha2`) is a decision of the build area (cargo deny).
3. [OURS] Reference vectors (keys given after the `name-search` stage; position: "Ann Smith", 2024-05-01, +79991234567; calculation Python 3.x,
   `hashlib`, own CRC-32C verified on "123456789" = 0xE3069283, F-57):
   - words "ann smith", "SMITH  Ann", "ann ann smith" (keys ann, smith): fingerprint `5646b76be8265f28547d98fa34ebc340`, cursor
     `AQFWRrdr6CZfKFR9mPo068NAAAlBbm4gU21pdGgAANNfCzc5OTkxMjM0NTY3HHIJVg`;
   - exact "ann smith" (key "ann smith"): `541085902fbeb42537b129ecefb02cc5`, cursor `AQJUEIWQL760JTexKezvsCzFAAlBbm4gU21pdGgAANNfCzc5OTkxMjM0NTY3gZMoBw`;
   - words "елка" (key "елка"): `4e1251e7217357263165eaf96013d4cf`, cursor `AQFOElHnIXNXJjFl6vlgE9TPAAlBbm4gU21pdGgAANNfCzc5OTkxMjM0NTY3CMlOBg`;
   - words "ё" + U+0308 (after NFC and folding the key is "е"; the "query → key" step was not verified here by an ICU4X run): `107250e9c42be5e8cabd905d9493beb2`,
     cursor `AQEQclDpxCvl6Mq9kF2Uk76yAAlBbm4gU21pdGgAANNfCzc5OTkxMjM0NTY3RgZPHQ`.
   The vectors are immutable within v1; the product checks them with a golden test, the key step — by a run through ICU4X at build.
4. [OURS] Number stream: a 200 response is one JSON object with members in order `number`, `version`, `records`, `count`, `complete`
   (`count` is the number of records, `complete: true`); the server writes `records` as the snapshot is read (ADR 0056), without accumulating. An object without
   `count` and `complete`, or a cut one, is incomplete. New members only between `version` and `records`. One JSON object, not
   NDJSON: ordinary clients parse it whole, while streaming ones see the marker at the end; a cut object is not parseable anyway.
   An error after the status has been sent does not change it (ADR 0035 item 6). For other responses the format is the same in spirit — an object.
5. [DEFAULT] Page: `page_size` is an integer 1…100, default 50 (ADR 0033). Rationale: an exact search page is ≈ 50 point reads
   (ADR 0047 item 4), 100 gives ≤ 2× the D-23 target (the ADR 0049 item 2 targets refer to 50), the body ≤ ≈ 100 KB. Alternatives: 200 or 1000 — a longer
   page and a larger response; no size choice — extra requests for programs. Latency targets at 100 are not set (a new question).
6. [OURS] The cursor does not depend on `page_size`; an empty `cursor=` is invalid, absence is the first page; `next_cursor` is a string or `null`;
   there is no total count.

## Alternatives considered
- NDJSON for the stream: rejected — see item 4 (another content type, clients do not parse it by default).
- A record count at the start: rejected — the number is unknown until the snapshot is read to the end without a second pass.
- A BLAKE3/SipHash hash with fixed keys: rejected — a NIST standard is easier to verify with independent implementations.
- Signing the cursor: rejected in ADR 0032.

## Consequences
- Word and exact queries have the same shape; a pagination client needs no knowledge of the cursor format.
- A format-1 cursor is issued until the end of v1.

## Sources
- ADR 0032, 0033, 0035, 0043, 0047, 0049, 0056; RFC 4648 §3.5, §5 (F-88); FIPS 180-4 (not opened, SHA-256 checked against the test vector "abc").
