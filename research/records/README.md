# research/records

Reproducible checks of the facts of the `records` capability (ADR 0021–0024).

## unicode18-check

Checks against Unicode 18.0.0 data (ICU4X 2.4.0, `icu_normalizer`/`icu_properties`/`icu_casemap` with compiled data;
the data is built for Unicode 18.0.0 and CLDR 49.0.0, see the README of the `icu_properties_data-2.4.0` crate):

1. White_Space in ICU4X equals the White_Space section of PropList-18.0.0.txt (25 code points).
2. The names of U+2010 (HYPHEN), U+2011 (NON-BREAKING HYPHEN), U+2012, U+2212 per UnicodeData.txt 18.0.0.
3. The NFC of every whitespace character consists only of whitespace (U+2000, U+2001 change); no non-whitespace character
   has whitespace in its NFC/NFD; ccc of whitespace = 0.
4. `trim(NFC(s)) == NFC(trim(s))`: all strings "w+x", "x+w", "w+x+w" for every whitespace w and every Unicode scalar value x
   (83,404,800 strings) and 2,000,000 pseudo-random strings made of "dangerous" characters.
5. NFC is idempotent on all Unicode scalar values.
6. Measurements for the storage requirements: the maximum number of Unicode scalar values after NFC of one character; the maximum NFD length of one
   character; the maximum growth in Unicode scalar values and bytes under NFC → full case folding → NFC of one character.

Run (requires the source UCD files of Unicode 18.0.0):

```
mkdir -p $TMPDIR/ucd && cd $TMPDIR/ucd
curl -O https://www.unicode.org/Public/18.0.0/ucd/PropList.txt -O https://www.unicode.org/Public/18.0.0/ucd/UnicodeData.txt
cd <project root>/research/records/unicode18-check
CARGO_TARGET_DIR=$TMPDIR/t18 cargo run --release -- $TMPDIR/ucd
```

For the result of the 2026-10-27 run see `facts.md`, F-4 and F-6.

## sortkey-check

For names "x×200" (x is each Unicode scalar value that is not Cc and is stable under NFC; 1,110,876 values) it measures the size of the name
(UTF-8), of the comparison key (NFC, full case folding, NFC) and of the ru-collator sort key
(`CollatorBorrowed::write_sort_key_to`, `CollatorOptions::default()`, ICU4X 2.4.0). The expected output is `output-2026-10-27.txt`
(maximum sort key 8,513 bytes, U+FDFA; comparison key 1,000; name 800).

```
cd <project root>/research/records/sortkey-check
CARGO_TARGET_DIR=$TMPDIR/sk cargo run --release
```
