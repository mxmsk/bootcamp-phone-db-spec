// Behavior probe of ICU4X 2.4.0 (compiled_data) for the name-search spec.
// Run: see README.md. Modes: wbt <WordBreakTest.txt> | cases | fold <CaseFolding.txt>
use icu_casemap::CaseMapper;
use icu_segmenter::options::WordBreakInvariantOptions;
use icu_segmenter::WordSegmenter;
use std::collections::BTreeMap;
use std::env;
use std::fs;

fn segments(seg: &icu_segmenter::WordSegmenterBorrowed<'_>, s: &str) -> Vec<(String, bool)> {
    let mut out = Vec::new();
    let mut prev = 0;
    let mut it = seg.segment_str(s).iter_with_word_type();
    while let Some((b, t)) = it.next() {
        if b > prev {
            out.push((s[prev..b].to_string(), t.is_word_like()));
            prev = b;
        }
    }
    out
}

fn breaks(seg: &icu_segmenter::WordSegmenterBorrowed<'_>, s: &str) -> Vec<usize> {
    seg.segment_str(s).filter(|&b| b != 0).collect()
}

fn parse_wbt_line(line: &str) -> Option<(String, Vec<usize>)> {
    let line = line.split('#').next()?.trim();
    if line.is_empty() {
        return None;
    }
    let mut s = String::new();
    let mut brk = Vec::new();
    for tok in line.split_whitespace() {
        match tok {
            "÷" => {
                if !s.is_empty() {
                    brk.push(s.len());
                }
            }
            "×" => {}
            hex => s.push(char::from_u32(u32::from_str_radix(hex, 16).ok()?)?),
        }
    }
    Some((s, brk))
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let non_complex = WordSegmenter::new_for_non_complex_scripts(WordBreakInvariantOptions::default());
    let auto = WordSegmenter::new_auto(WordBreakInvariantOptions::default());
    match args.get(1).map(|s| s.as_str()) {
        Some("wbt") => {
            let text = fs::read_to_string(&args[2]).unwrap();
            for (name, seg) in [("new_for_non_complex_scripts", &non_complex), ("new_auto", &auto)] {
                let (mut total, mut ok) = (0, 0);
                let mut bad = Vec::new();
                for line in text.lines() {
                    if let Some((s, exp)) = parse_wbt_line(line) {
                        total += 1;
                        let got = breaks(seg, &s);
                        if got == exp {
                            ok += 1;
                        } else {
                            bad.push((line.to_string(), got));
                        }
                    }
                }
                println!("{name}: {ok}/{total} matched WordBreakTest-18.0.0");
                for (l, got) in bad.iter().take(30) {
                    println!("  MISMATCH: {l}\n    ICU4X boundaries (byte offsets): {got:?}");
                }
            }
        }
        Some("cases") => {
            let cases: Vec<(&str, &str)> = vec![
                ("colon between letters (ASCII)", "Smith:John"),
                ("colon between letters (Cyrillic)", "Ann:Иван"),
                ("U+FE55 between letters", "ab\u{FE55}cd"),
                ("U+FF1A between letters", "ab\u{FF1A}cd"),
                ("underscore", "john_smith"),
                ("U+00B7 between letters", "Ann\u{B7}Bob"),
                ("U+2018 between letters", "ab\u{2018}cd"),
                ("U+2019 between letters", "O\u{2019}Brien"),
                ("U+0027 between letters", "O'Brien"),
                ("U+FF07 between letters", "ab\u{FF07}cd"),
                ("dot between letters", "Smith.John"),
                ("U+2027 between letters", "ab\u{2027}cd"),
                ("U+0387 between letters", "αβ\u{387}γδ"),
                ("hyphen", "Anna-Maria"),
                ("digits+letters", "Agent 007b"),
                ("R2-D2", "R2-D2"),
                ("numbers 3,456.7", "3,456.7"),
                ("CJK 李雷", "李雷"),
                ("CJK 李雷 韩梅梅", "李雷 韩梅梅"),
                ("hiragana+kanji", "こんにちは世界"),
                ("only hiragana", "こんにちは"),
                ("katakana", "コンピュータ"),
                ("Thai", "สวัสดีครับ"),
                ("Hebrew", "שלום עולם"),
                ("Hebrew with apostrophe", "ג'ינס"),
                ("Hebrew gershayim", "צה\"ל"),
                ("Cyrillic", "Иван Петров"),
                ("«й» and «ё»", "Ёлка Майя"),
                ("emoji", "ab😀cd"),
                ("CJK+Latin joined", "李雷Smith"),
                ("Latin+CJK joined", "Smith李雷"),
                ("CJK+digit", "李雷2"),
                ("kana+kanji+katakana", "東京タワー"),
                ("Thai+Latin", "Somchai สมชาย"),
                ("Lao", "ສະບາຍດີ"),
                ("Khmer", "សួស្តី"),
                ("Hangul", "김철수 이영희"),
            ];
            for (name, s) in &cases {
                for (m, seg) in [("non_complex", &non_complex), ("auto", &auto)] {
                    let segs = segments(seg, s);
                    let v: Vec<String> = segs
                        .iter()
                        .map(|(t, w)| format!("{:?}{}", t, if *w { "*" } else { "" }))
                        .collect();
                    println!("{name:40} [{m:11}] {}", v.join(" | "));
                }
            }
            println!("(* = word-like)");
        }
        Some("locale") => {
            // Effect of content_locale (sv/fi/ru/en) on boundaries: colon and the like.
            use icu_segmenter::options::WordBreakOptions;
            for tag in ["und", "en", "ru", "sv", "fi"] {
                let lid: icu_locale_core::LanguageIdentifier = tag.parse().unwrap();
                let mut o = WordBreakOptions::default();
                o.content_locale = Some(&lid);
                let seg = WordSegmenter::try_new_for_non_complex_scripts(o).unwrap();
                let seg = seg.as_borrowed();
                for s in ["Smith:John", "k:a", "Ann:Иван"] {
                    let v: Vec<String> = segments(&seg, s).iter().map(|(t, _)| format!("{t:?}")).collect();
                    println!("locale={tag:4} {s:12} {}", v.join(" | "));
                }
            }
        }
        Some("fold") => {
            // Check of CaseMapper::fold against the C+F statuses of CaseFolding.txt for all code points.
            let text = fs::read_to_string(&args[2]).unwrap();
            let mut cf: BTreeMap<u32, String> = BTreeMap::new();
            let mut s_only: BTreeMap<u32, String> = BTreeMap::new();
            let mut t_only: BTreeMap<u32, String> = BTreeMap::new();
            for line in text.lines() {
                let l = line.split('#').next().unwrap().trim();
                if l.is_empty() {
                    continue;
                }
                let p: Vec<&str> = l.split(';').map(|x| x.trim()).collect();
                let cp = u32::from_str_radix(p[0], 16).unwrap();
                let m: String = p[2]
                    .split_whitespace()
                    .map(|h| char::from_u32(u32::from_str_radix(h, 16).unwrap()).unwrap())
                    .collect();
                match p[1] {
                    "C" | "F" => {
                        cf.insert(cp, m);
                    }
                    "S" => {
                        s_only.insert(cp, m);
                    }
                    "T" => {
                        t_only.insert(cp, m);
                    }
                    _ => {}
                }
            }
            let cm = CaseMapper::new();
            let cmb = cm;
            let (mut checked, mut diff) = (0, 0);
            for cp in 0..=0x10FFFFu32 {
                let Some(c) = char::from_u32(cp) else { continue };
                let s = c.to_string();
                let expected = cf.get(&cp).cloned().unwrap_or(s.clone());
                let got = cmb.fold_string(&s).to_string();
                checked += 1;
                if got != expected {
                    diff += 1;
                    if diff <= 20 {
                        println!("MISMATCH U+{cp:04X}: ICU4X {:?}, C+F {:?}", got, expected);
                    }
                }
            }
            println!("fold: checked {checked} code points, mismatches with C+F: {diff}");
            println!("status S entries: {}, T entries: {}", s_only.len(), t_only.len());
            for cp in s_only.keys().take(3) {
                let c = char::from_u32(*cp).unwrap();
                println!("  S example U+{cp:04X}: fold={:?}, S={:?}, C+F={:?}", cmb.fold_string(&c.to_string()), s_only[cp], cf.get(cp));
            }
            for (name, s) in [("ß", "straße"), ("İ", "İ"), ("ẞ", "ẞ"), ("Ё", "Ё"), ("ΐ", "ΐ")] {
                println!("  {name}: {:?} -> {:?}", s, cmb.fold_string(s));
            }
        }
        _ => eprintln!("usage: wbt <file> | cases | fold <file>"),
    }
}
