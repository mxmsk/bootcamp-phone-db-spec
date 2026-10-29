//! Name order corpus (ADR 0039 item 5 (b)). Builds a deterministic set of names, sorts it with the ru comparison
//! function (`icu_collator`, default parameters, ADR 0037/0038) and prints the order: one line per name, format
//! `<code points in hex separated by spaces>\t<relation to the previous: '<' or '='>`. The hash of the output (SHA-256) is checked against the snapshot
//! `order-corpus.sha256` by the script `check-corpus.sh`.
//!
//! Corpus: (A) single characters from ranges; (B) base + combining mark and base + two marks in all orders;
//! (C) 20,000 pseudo-random strings of length 1..=8 from a fixed pool (xorshift64* generator, seed below). Names
//! are normalized to NFC and deduplicated. Names equal under the collator are ordered by code points ONLY for
//! output reproducibility (the product has no such key, ADR 0036 item 2); the '=' relation shows equality.
use icu_collator::options::CollatorOptions;
use icu_collator::CollatorBorrowed;
use icu_locale::locale;
use icu_normalizer::ComposingNormalizerBorrowed;
use std::cmp::Ordering;
use std::collections::BTreeSet;

const SEED: u64 = 0x5EED_0001;
const RANDOM_COUNT: usize = 20_000;
const RANDOM_MAX_LEN: u64 = 8;

fn scalars(lo: u32, hi: u32) -> Vec<char> {
    (lo..=hi).filter_map(char::from_u32).collect()
}

struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        self.0.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }
}

fn main() {
    // (A) ranges of single characters.
    let mut singles: Vec<char> = Vec::new();
    for (lo, hi) in [
        (0x20, 0x7E),     // printable ASCII
        (0xA0, 0x24F),    // Latin-1 and extended Latin
        (0x300, 0x36F),   // combining marks
        (0x400, 0x52F),   // Cyrillic and supplement
        (0x2000, 0x206F), // general punctuation, spaces, ignorables
        (0x3040, 0x30FF), // hiragana, katakana
        (0x4E00, 0x4E7F), // Han ideographs (start of the block)
    ] {
        singles.extend(scalars(lo, hi));
    }
    let mut names: Vec<String> = singles.iter().map(|c| c.to_string()).collect();

    // (B) base + combining marks.
    let mut bases: Vec<char> = Vec::new();
    bases.extend(scalars(0x410, 0x44F));
    bases.extend(['\u{401}', '\u{451}']);
    bases.extend(scalars('A' as u32, 'Z' as u32));
    bases.extend(scalars('a' as u32, 'z' as u32));
    bases.extend(scalars('0' as u32, '9' as u32));
    bases.extend([' ', '-', '_', '.', '\u{AD}']);
    let marks = ['\u{300}', '\u{301}', '\u{306}', '\u{308}', '\u{323}', '\u{327}', '\u{345}'];
    for b in &bases {
        for m in marks {
            names.push(format!("{b}{m}"));
        }
    }
    for b in ['а', 'е', 'и', 'a', 'e', 'o'] {
        for m1 in marks {
            for m2 in marks {
                names.push(format!("{b}{m1}{m2}"));
            }
        }
    }

    // (C) pseudo-random strings from the pool.
    let mut pool: Vec<char> = Vec::new();
    pool.extend(scalars(0x410, 0x44F));
    pool.extend(['\u{401}', '\u{451}']);
    pool.extend(scalars('A' as u32, 'Z' as u32));
    pool.extend(scalars('a' as u32, 'z' as u32));
    pool.extend(scalars('0' as u32, '9' as u32));
    pool.extend([' ', ' ', '-', '_', '.', '\'', '\u{AD}', '\u{2019}', '\u{200B}', '\u{E9}', '\u{FC}']);
    pool.extend(marks);
    pool.extend(scalars(0x4E00, 0x4E1F));
    let mut rng = Rng(SEED);
    for _ in 0..RANDOM_COUNT {
        let len = 1 + rng.next() % RANDOM_MAX_LEN;
        let mut s = String::new();
        for _ in 0..len {
            s.push(pool[(rng.next() % pool.len() as u64) as usize]);
        }
        names.push(s);
    }

    // NFC and deduplication.
    let nfc = ComposingNormalizerBorrowed::new_nfc();
    let set: BTreeSet<String> = names.iter().map(|n| nfc.normalize(n).into_owned()).collect();
    let mut sorted: Vec<String> = set.into_iter().collect();

    let c = CollatorBorrowed::try_new(locale!("ru").into(), CollatorOptions::default()).expect("ru data");
    // The input is already sorted by code points (BTreeSet), and sort_by is stable: names equal under the collator
    // remain in code point order.
    sorted.sort_by(|a, b| c.compare(a, b));

    let mut out = String::new();
    for (i, name) in sorted.iter().enumerate() {
        let rel = if i == 0 {
            '-'
        } else if c.compare(&sorted[i - 1], name) == Ordering::Equal {
            '='
        } else {
            '<'
        };
        let hex: Vec<String> = name.chars().map(|ch| format!("{:04X}", ch as u32)).collect();
        out.push_str(&format!("{}\t{}\n", hex.join(" "), rel));
    }
    print!("{out}");
    eprintln!("names: {}", sorted.len());

    // Second pass (ADR 0039 item 9 (d)): the index is built from sort key bytes (`write_sort_key_to`), not from
    // `compare()`. For every adjacent pair the order by key bytes must match `compare()`: '<' gives a strictly
    // smaller key, '=' gives equal keys. A mismatch means exit code 1.
    let mut prev: Vec<u8> = Vec::new();
    let mut key_mismatches = 0usize;
    for (i, name) in sorted.iter().enumerate() {
        let mut key: Vec<u8> = Vec::new();
        let Ok(()) = c.write_sort_key_to(name, &mut key);
        if i > 0 {
            let by_compare = c.compare(&sorted[i - 1], name);
            if prev.cmp(&key) != by_compare {
                key_mismatches += 1;
            }
        }
        prev = key;
    }
    eprintln!("sort-key mismatches: {key_mismatches}");
    if key_mismatches != 0 {
        std::process::exit(1);
    }
}
