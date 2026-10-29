// Measurement of key sizes for the index calculation (ADR 0045-0049): typical names, not x×200 (F-39).
// For each name: bytes of the name (UTF-8), of the comparison key (NFC, fold, NFC, ё->е), of the ru sort key
// (`write_sort_key_to`, CollatorOptions::default()), and the number of 0x00 and 0x01 bytes in the sort key.
// Additionally: the key structure property (exactly two 0x01 bytes, no 0x00) on all single scalar values
// and on all pairs from a pool of characters.
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

    let names: &[(&str, &str)] = &[
        ("ru", "Иванов Иван Иванович"),
        ("ru", "Петрова Мария Сергеевна"),
        ("ru", "Сидоров Александр Владимирович"),
        ("ru", "Ёлкина Анна"),
        ("ru", "Мария"),
        ("ru", "Иван"),
        ("ru", "Кузнецова Екатерина Дмитриевна"),
        ("ru", "ООО Ромашка"),
        ("ru", "Морозов Павел Викторович"),
        ("ru", "Смирнов А. В."),
        ("ru", "Попов Иван"),
        ("ru", "Новикова-Петрова Ольга Игоревна"),
        ("la", "John Smith"),
        ("la", "Anna-Maria O'Brien"),
        ("la", "Ann Smith"),
        ("la", "Michael Johnson"),
        ("la", "Elizabeth Alexandra Mary Windsor"),
        ("la", "José García"),
        ("la", "Bob"),
        ("la", "Müller Hans-Peter"),
        ("cjk", "李雷"),
        ("cjk", "王小明"),
        ("cjk", "田中太郎"),
        ("cjk", "김민준"),
        ("ar", "محمد علي"),
        ("th", "สมชาย ใจดี"),
        ("mix", "Pizza Ёлка 24/7"),
        ("mix", "Agent 007b"),
    ];
    println!("# name\tscript\tchars\tname_bytes\tcmpkey_bytes\tsortkey_bytes\tzero\tones");
    let mut tot: std::collections::BTreeMap<&str, (usize, usize, usize, usize, usize)> = Default::default();
    for (sc, n) in names {
        let n = nfc.normalize(n).into_owned();
        let mut key = Vec::new();
        col.write_sort_key_to(&n, &mut key).unwrap();
        let folded = nfc.normalize(&cm.fold_string(&n)).into_owned();
        let chars = n.chars().count();
        let zeros = key.iter().filter(|&&b| b == 0).count();
        let ones = key.iter().filter(|&&b| b == 1).count();
        println!("{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}", n, sc, chars, n.len(), folded.len(), key.len(), zeros, ones);
        let e = tot.entry(sc).or_default();
        e.0 += 1;
        e.1 += chars;
        e.2 += n.len();
        e.3 += folded.len();
        e.4 += key.len();
    }
    println!("# script\tcount\tavg_chars\tavg_name_bytes\tavg_cmpkey\tavg_sortkey\tsortkey_per_char");
    for (sc, (c, ch, nb, cb, kb)) in &tot {
        println!(
            "{}\t{}\t{:.1}\t{:.1}\t{:.1}\t{:.1}\t{:.2}",
            sc, c, *ch as f64 / *c as f64, *nb as f64 / *c as f64, *cb as f64 / *c as f64,
            *kb as f64 / *c as f64, *kb as f64 / *ch as f64
        );
    }
    println!("sortkey example Ann: {:02x?}", {
        let mut k = Vec::new();
        col.write_sort_key_to("Ann", &mut k).unwrap();
        k
    });

    // Key structure on single characters.
    let mut bad_zero = 0u32;
    let mut not_two_ones = 0u32;
    let mut total = 0u32;
    let mut ones_hist: std::collections::BTreeMap<usize, u32> = Default::default();
    let mut pool: Vec<char> = Vec::new();
    for cp in 0u32..=0x10FFFF {
        let Some(ch) = char::from_u32(cp) else { continue };
        let s = ch.to_string();
        let mut k = Vec::new();
        col.write_sort_key_to(&s, &mut k).unwrap();
        total += 1;
        if k.contains(&0) {
            bad_zero += 1;
        }
        let ones = k.iter().filter(|&&b| b == 1).count();
        *ones_hist.entry(ones).or_default() += 1;
        if ones != 2 {
            not_two_ones += 1;
        }
        if pool.len() < 400 && (cp < 0x250 || (0x400..0x500).contains(&cp) || (0x300..0x370).contains(&cp)) {
            pool.push(ch);
        }
    }
    println!("single scalar values: total {total}, with 0x00 in key {bad_zero}, ones!=2: {not_two_ones}, hist {ones_hist:?}");
    // All pairs from the pool.
    let mut pairs = 0u64;
    let mut pz = 0u64;
    let mut pn2 = 0u64;
    for a in &pool {
        for b in &pool {
            let s: String = [*a, *b].iter().collect();
            let mut k = Vec::new();
            col.write_sort_key_to(&s, &mut k).unwrap();
            pairs += 1;
            if k.contains(&0) {
                pz += 1;
            }
            if k.iter().filter(|&&x| x == 1).count() != 2 {
                pn2 += 1;
            }
        }
    }
    println!("pairs from pool of {}: {pairs}, with 0x00: {pz}, ones!=2: {pn2}", pool.len());
}
