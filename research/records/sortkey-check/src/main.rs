// Measurement of the sizes of derived name keys for 200 scalar values (ADR 0022).
// For each scalar value x (not a surrogate, not Cc, NFC-stable: NFC(x×200) == x×200) the name x×200 is built and
// measured: name bytes (UTF-8), comparison key bytes (NFC -> fold -> NFC), ru-collator sort key bytes
// (`CollatorBorrowed::write_sort_key_to`, CollatorOptions::default()). Prints the maxima and the first five characters.
use icu_casemap::CaseMapper;
use icu_collator::options::CollatorOptions;
use icu_collator::{CollatorBorrowed, CollatorPreferences};
use icu_locale::locale;
use icu_normalizer::ComposingNormalizerBorrowed;

fn main() {
    let prefs: CollatorPreferences = locale!("ru").into();
    let col = CollatorBorrowed::try_new(prefs, CollatorOptions::default()).expect("ru data");
    let nfc = ComposingNormalizerBorrowed::new_nfc();
    let cm = CaseMapper::new();
    let n = 200usize;

    let mut rows: Vec<(usize, u32, usize, usize)> = Vec::new(); // (sortkey, cp, namebytes, cmpbytes)
    let mut counted = 0u32;
    for cp in 0u32..=0x10FFFF {
        let Some(ch) = char::from_u32(cp) else { continue };
        if ch.is_control() {
            continue;
        }
        let name: String = std::iter::repeat(ch).take(n).collect();
        if nfc.normalize(&name) != name {
            continue;
        }
        counted += 1;
        let mut key: Vec<u8> = Vec::new();
        col.write_sort_key_to(&name, &mut key).expect("key");
        let folded = nfc.normalize(&cm.fold_string(&name)).into_owned();
        rows.push((key.len(), cp, name.len(), folded.len()));
    }
    rows.sort_by(|a, b| b.cmp(a));
    println!("scalar values measured: {counted}");
    println!("max name bytes: {}", rows.iter().map(|r| r.2).max().unwrap());
    println!("max comparison-key bytes: {}", rows.iter().map(|r| r.3).max().unwrap());
    println!("max sort-key bytes: {}", rows[0].0);
    println!("min sort-key bytes: {}", rows.last().unwrap().0);
    println!("top 10 by sort-key bytes (U+cp sortkey namebytes cmpbytes):");
    for r in rows.iter().take(10) {
        println!("U+{:04X} {} {} {}", r.1, r.0, r.2, r.3);
    }
    let mut by_cmp = rows.clone();
    by_cmp.sort_by(|a, b| b.3.cmp(&a.3));
    println!("top 3 by comparison-key bytes:");
    for r in by_cmp.iter().take(3) {
        println!("U+{:04X} {} {} {}", r.1, r.0, r.2, r.3);
    }
}
