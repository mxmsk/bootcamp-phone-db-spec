// Key sizes for the storage calculation (file-format subtask).
// Part 1: realistic names (Russian and Latin personal names, organizations) - bytes of the name, comparison key, sort key.
// Part 2: mixed names of 200 "long" characters - checking additivity of the sort key length (Q-103):
//   pairs (a,b) of the K longest characters; names a*100+b*100 and (ab)*100; compared with 200*max(key(x*200))/200.
use icu_casemap::CaseMapper;
use icu_collator::options::CollatorOptions;
use icu_collator::{CollatorBorrowed, CollatorPreferences};
use icu_locale::locale;
use icu_normalizer::ComposingNormalizerBorrowed;

fn sortkey(col: &CollatorBorrowed, s: &str) -> usize {
    let mut k: Vec<u8> = Vec::new();
    col.write_sort_key_to(s, &mut k).expect("key");
    k.len()
}

fn main() {
    let prefs: CollatorPreferences = locale!("ru").into();
    let col = CollatorBorrowed::try_new(prefs, CollatorOptions::default()).expect("ru data");
    let nfc = ComposingNormalizerBorrowed::new_nfc();
    let cm = CaseMapper::new();

    // Part 1
    let ru_first = ["Александр", "Дмитрий", "Максим", "Сергей", "Андрей", "Алексей", "Артём", "Илья", "Кирилл", "Михаил",
        "Анна", "Мария", "Елена", "Ольга", "Наталья", "Екатерина", "Татьяна", "Ирина", "Светлана", "Юлия"];
    let ru_last = ["Иванов", "Смирнов", "Кузнецов", "Попов", "Васильев", "Петров", "Соколов", "Михайлов", "Новиков",
        "Фёдоров", "Морозов", "Волков", "Алексеев", "Лебедев", "Семёнов", "Егоров", "Павлов", "Козлов", "Степанов",
        "Николаев", "Орлов", "Андреев", "Макаров", "Никитин", "Захаров"];
    let ru_pat = ["Александрович", "Дмитриевич", "Сергеевич", "Андреевич", "Алексеевич", "Михайлович", "Ивановна",
        "Петровна", "Сергеевна", "Александровна"];
    let en_first = ["John", "Mary", "James", "Patricia", "Robert", "Jennifer", "Michael", "Linda", "William", "Elizabeth"];
    let en_last = ["Smith", "Johnson", "Williams", "Brown", "Jones", "Garcia", "Miller", "Davis", "Rodriguez", "Martinez"];
    let orgs = ["ООО «Ромашка и партнёры»", "Pizza Express Moscow Central Branch", "Служба доставки Быстрый курьер",
        "Dr. Hans-Joachim von Hohenzollern-Sigmaringen", "Иванов Иван Иванович (рабочий)", "Mama"];
    let mut names: Vec<String> = Vec::new();
    for f in ru_first { for l in ru_last { names.push(format!("{f} {l}")); } }
    for f in ru_first { for l in ru_last { for p in ru_pat.iter().take(3) { names.push(format!("{l} {f} {p}")); } } }
    for f in en_first { for l in en_last { names.push(format!("{f} {l}")); } }
    for o in orgs { names.push(o.to_string()); }
    let mut tot = [0usize; 4];
    let mut mx = [0usize; 4];
    for n in &names {
        let n = nfc.normalize(n).into_owned();
        let chars = n.chars().count();
        let folded = nfc.normalize(&cm.fold_string(&n)).into_owned();
        let v = [chars, n.len(), folded.len(), sortkey(&col, &n)];
        for i in 0..4 { tot[i] += v[i]; mx[i] = mx[i].max(v[i]); }
    }
    let c = names.len();
    println!("realistic names: {c}");
    println!("avg chars {:.1}, avg name bytes {:.1}, avg cmp-key bytes {:.1}, avg sort-key bytes {:.1}",
        tot[0] as f64 / c as f64, tot[1] as f64 / c as f64, tot[2] as f64 / c as f64, tot[3] as f64 / c as f64);
    println!("sortkey bytes per char: {:.2}", tot[3] as f64 / tot[0] as f64);
    println!("max chars {} name {} cmp {} sort {}", mx[0], mx[1], mx[2], mx[3]);

    // Part 2
    let mut singles: Vec<(usize, char)> = Vec::new();
    for cp in 0u32..=0x10FFFF {
        let Some(ch) = char::from_u32(cp) else { continue };
        if ch.is_control() { continue; }
        let s: String = std::iter::repeat(ch).take(200).collect();
        if nfc.normalize(&s) != s { continue; }
        singles.push((sortkey(&col, &s), ch));
    }
    singles.sort_by(|a, b| b.cmp(a));
    let k = 80usize;
    let top: Vec<char> = singles.iter().take(k).map(|x| x.1).collect();
    println!("single x*200 max sort key: {} (U+{:04X})", singles[0].0, singles[0].1 as u32);
    let mut worst = 0usize; let mut worst_desc = String::new();
    let mut viol = 0usize; let mut checked = 0usize;
    let mut worst_over = i64::MIN; let mut worst_over_desc = String::new();
    let mut worst_mixed = 0usize; let mut worst_mixed_desc = String::new();
    for &a in &top {
        for &b in &top {
            let ka = singles.iter().find(|x| x.1 == a).unwrap().0;
            let kb = singles.iter().find(|x| x.1 == b).unwrap().0;
            let cand: [String; 3] = [
                format!("{}{}", a.to_string().repeat(100), b.to_string().repeat(100)),
                format!("{}", format!("{a}{b}").repeat(100)),
                format!("{}{}", a.to_string().repeat(150), b.to_string().repeat(50)),
            ];
            for (i, s) in cand.iter().enumerate() {
                if nfc.normalize(s) != *s { continue; }
                checked += 1;
                let key = sortkey(&col, s);
                // additive per-character estimate: shares of a and b of the uniform keys
                let bound = match i { 0 | 1 => (ka + kb) / 2, _ => (ka * 3 + kb) / 4 };
                if key > bound + 8 { viol += 1; }
                if a != b {
                    let over = key as i64 - ka.max(kb) as i64;
                    if over > worst_over { worst_over = over; worst_over_desc = format!("U+{:04X}/U+{:04X} variant {i} key {key} max(ka,kb) {}", a as u32, b as u32, ka.max(kb)); }
                    if key > worst_mixed { worst_mixed = key; worst_mixed_desc = format!("U+{:04X}/U+{:04X} variant {i}", a as u32, b as u32); }
                }
                if key > worst { worst = key; worst_desc = format!("U+{:04X}/U+{:04X} variant {i}", a as u32, b as u32); }
            }
        }
    }
    println!("pairs checked {checked}, max mixed key {worst} ({worst_desc}); exceeding mean-of-singles+8: {viol}");
    println!("a!=b: max mixed key {worst_mixed} ({worst_mixed_desc}); max excess over max(ka,kb): {worst_over} ({worst_over_desc})");
}
