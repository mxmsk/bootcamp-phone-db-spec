// Check of the records facts on Unicode 18: White_Space, NFC and the "trim / NFC" order, hyphen names.
// Run: cargo run --release -- <directory with PropList.txt and UnicodeData.txt of Unicode 18.0.0>
use icu_normalizer::{ComposingNormalizerBorrowed, DecomposingNormalizerBorrowed};
use icu_properties::props::{CanonicalCombiningClass, WhiteSpace};
use icu_properties::{CodePointMapData, CodePointSetData};
use std::collections::BTreeSet;

fn is_ws(set: &icu_properties::CodePointSetDataBorrowed, c: char) -> bool {
    set.contains(c)
}

fn trim<'a>(set: &icu_properties::CodePointSetDataBorrowed, s: &'a str) -> &'a str {
    s.trim_matches(|c| is_ws(set, c))
}

fn main() {
    let dir = std::env::args().nth(1).expect("dir with PropList.txt, UnicodeData.txt");
    let proplist = std::fs::read_to_string(format!("{dir}/PropList.txt")).unwrap();
    let unidata = std::fs::read_to_string(format!("{dir}/UnicodeData.txt")).unwrap();
    println!("PropList header: {}", proplist.lines().next().unwrap());

    // 1. White_Space per PropList-18.0.0 and per ICU4X data.
    let mut file_ws: BTreeSet<u32> = BTreeSet::new();
    for line in proplist.lines() {
        let body = line.split('#').next().unwrap().trim();
        let Some((range, prop)) = body.split_once(';') else { continue };
        if prop.trim() != "White_Space" {
            continue;
        }
        let (a, b) = match range.trim().split_once("..") {
            Some((a, b)) => (a, b),
            None => (range.trim(), range.trim()),
        };
        let (a, b) = (u32::from_str_radix(a, 16).unwrap(), u32::from_str_radix(b, 16).unwrap());
        file_ws.extend(a..=b);
    }
    let ws = CodePointSetData::new::<WhiteSpace>();
    let icu_ws: BTreeSet<u32> = (0..=0x10FFFFu32)
        .filter_map(char::from_u32)
        .filter(|c| ws.contains(*c))
        .map(|c| c as u32)
        .collect();
    println!("White_Space: PropList={} ICU4X={} equal={}", file_ws.len(), icu_ws.len(), file_ws == icu_ws);
    assert_eq!(file_ws, icu_ws);

    // 2. Names of U+2010/U+2011/U+2012/U+2212 per UnicodeData.txt of Unicode 18.
    for cp in ["2010", "2011", "2012", "2212", "002D"] {
        let line = unidata.lines().find(|l| l.starts_with(&format!("{cp};"))).unwrap();
        let f: Vec<&str> = line.split(';').collect();
        println!("U+{cp}: name={} gc={} decomp={:?}", f[1], f[2], f[5]);
    }

    // 3. NFC/NFD/ccc on all scalar values.
    let nfc = ComposingNormalizerBorrowed::new_nfc();
    let nfd = DecomposingNormalizerBorrowed::new_nfd();
    let ccc = CodePointMapData::<CanonicalCombiningClass>::new();
    let all: Vec<char> = (0..=0x10FFFFu32).filter_map(char::from_u32).collect();
    let wsl: Vec<char> = all.iter().copied().filter(|c| is_ws(&ws, *c)).collect();

    let mut changed = vec![];
    for &w in &wsl {
        let s = w.to_string();
        let n = nfc.normalize(&s).to_string();
        assert!(n.chars().all(|c| is_ws(&ws, c)), "NFC(ws) not all ws: {:x}", w as u32);
        assert_eq!(ccc.get(w), CanonicalCombiningClass::NotReordered, "ccc != 0 for {:x}", w as u32);
        if n != s {
            changed.push(format!("U+{:04X}->{}", w as u32, n.chars().map(|c| format!("U+{:04X}", c as u32)).collect::<String>()));
        }
    }
    println!("NFC changes whitespace: {changed:?}");
    let mut bad_nonws = 0;
    for &d in &all {
        if is_ws(&ws, d) {
            continue;
        }
        let s = d.to_string();
        let a = nfd.normalize(&s);
        let b = nfc.normalize(&s);
        if a.chars().any(|c| is_ws(&ws, c)) || b.chars().any(|c| is_ws(&ws, c)) {
            bad_nonws += 1;
        }
    }
    println!("non-whitespace whose NFC/NFD contains whitespace: {bad_nonws}");
    assert_eq!(bad_nonws, 0);

    // 4. trim(NFC(s)) == NFC(trim(s)) for s from all pairs/triples with a whitespace character at the edge.
    let mut checked = 0u64;
    let mut check = |s: &str| {
        let left = trim(&ws, &nfc.normalize(s)).to_string();
        let right = nfc.normalize(trim(&ws, s)).to_string();
        assert_eq!(left, right, "order differs for {:?}", s.chars().map(|c| c as u32).collect::<Vec<_>>());
        checked += 1;
    };
    for &x in &all {
        for &w in &wsl {
            let (mut a, mut b, mut c) = (String::new(), String::new(), String::new());
            a.push(w);
            a.push(x);
            b.push(x);
            b.push(w);
            c.push(w);
            c.push(x);
            c.push(w);
            check(&a);
            check(&b);
            check(&c);
        }
    }
    // Pseudo-random strings of "dangerous" characters: whitespace, combining, hangul, leading/trailing.
    let pool: Vec<char> = wsl
        .iter()
        .copied()
        .chain(['e', 'a', '\u{0301}', '\u{0308}', '\u{0323}', '\u{1100}', '\u{1161}', '\u{11A8}', '\u{0F71}', '\u{0F72}', '\u{0F80}', '\u{3099}', 'Я'])
        .collect();
    let mut st: u64 = 0x9E3779B97F4A7C15;
    for _ in 0..2_000_000 {
        let mut s = String::new();
        st = st.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        let len = 1 + (st >> 60) as usize % 8;
        for _ in 0..len {
            st = st.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            s.push(pool[(st >> 33) as usize % pool.len()]);
        }
        check(&s);
    }
    println!("trim(NFC(s)) == NFC(trim(s)): strings checked {checked}, mismatches 0");

    // 5. NFC idempotence on all scalar values.
    for &c in &all {
        let s = c.to_string();
        let once = nfc.normalize(&s).to_string();
        assert_eq!(nfc.normalize(&once), once.as_str());
    }
    println!("NFC is idempotent on all scalar values");

    // 6. Is the length after NFC never greater than before NFC? (for reference: the maximum growth in the number of scalar values)
    let mut max_growth = 1usize;
    for &c in &all {
        let n = nfc.normalize(&c.to_string()).chars().count();
        max_growth = max_growth.max(n);
    }
    println!("max scalar values after NFC of one scalar value: {max_growth}");

    // 7. Sizes (for the storage requirements; measurement only, not an estimate).
    // 7a. maximum NFD length of one scalar value (the bound for "how many input scalar values yield one NFC character").
    let mut max_nfd = (0usize, 'a');
    for &c in &all {
        let n = nfd.normalize(&c.to_string()).chars().count();
        if n > max_nfd.0 {
            max_nfd = (n, c);
        }
    }
    println!("max NFD length of one scalar value: {} (U+{:04X})", max_nfd.0, max_nfd.1 as u32);
    // 7b. full case folding (C+F) of one character: growth in scalar values and in UTF-8 bytes.
    let cm = icu_casemap::CaseMapper::new();
    let (mut max_chars, mut max_bytes) = ((1usize, 'a'), (1f64, 'a'));
    for &c in &all {
        let f = cm.fold_string(&c.to_string()).to_string();
        let g = nfc.normalize(&f).to_string();
        let (n, b) = (g.chars().count(), g.len() as f64 / c.len_utf8() as f64);
        if n > max_chars.0 {
            max_chars = (n, c);
        }
        if b > max_bytes.0 {
            max_bytes = (b, c);
        }
    }
    println!("fold+NFC of one char: max {} scalar values (U+{:04X}); max byte growth {:.3}x (U+{:04X})",
        max_chars.0, max_chars.1 as u32, max_bytes.0, max_bytes.1 as u32);
}
