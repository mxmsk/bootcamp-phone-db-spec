// Variant B of ADR 0030: the UAX #29 word boundary rules (WB3–WB999) on top of the Word_Break and Extended_Pictographic data from
// icu_properties 2.4.0 (Unicode 18), without WordSegmenter. Modes:
//   wbt <WordBreakTest.txt>                  — check against WordBreakTest-18.0.0
//   data <WordBreakProperty.txt> <emoji-data.txt> — check of the icu_properties data against the UCD files for all scalars
//   diff                                     — where WordSegmenter::new_for_non_complex_scripts differs from the UAX #29 rules
//   points                                   — point cases (Han, kana, Thai, Lao, Myanmar, etc.)
//   golden                                   — stored reference of variant B (UAX #29 rules only, without WordSegmenter)
//   gc <UnicodeData.txt> <PropList.txt>      — General_Category (all 30 values) and White_Space of icu_properties against UCD 18
//   word                                     — the "word" predicate (not only Z*/P*/Cc/Cf, ADR 0030 item 3, OD-5) against WordType::is_word_like
use icu_properties::props::{ExtendedPictographic, GeneralCategoryGroup, Script, WordBreak as W};
use icu_properties::{CodePointMapData, CodePointSetData, PropertyNamesLong, PropertyNamesShort};
use icu_segmenter::options::WordBreakInvariantOptions;
use icu_segmenter::WordSegmenter;
use std::collections::BTreeMap;
use std::env;
use std::fs;

struct Seg {
    wb: icu_properties::CodePointMapDataBorrowed<'static, W>,
    ep: icu_properties::CodePointSetDataBorrowed<'static>,
}

impl Seg {
    fn new() -> Self {
        Seg {
            wb: CodePointMapData::<W>::new(),
            ep: CodePointSetData::new::<ExtendedPictographic>(),
        }
    }

    // Byte offsets of boundaries within the string (without 0, with len).
    fn breaks(&self, s: &str) -> Vec<usize> {
        // Units after WB4: (start, base class, last raw class, first raw class, first is ExtPict)
        struct U {
            start: usize,
            base: W,
            last_raw: W,
            first_ep: bool,
        }
        let mut us: Vec<U> = Vec::new();
        for (i, c) in s.char_indices() {
            let cl = self.wb.get(c);
            let ign = cl == W::Extend || cl == W::Format || cl == W::ZWJ;
            let merge = ign
                && match us.last() {
                    Some(p) => !(p.base == W::CR || p.base == W::LF || p.base == W::Newline),
                    None => false,
                };
            if merge {
                us.last_mut().unwrap().last_raw = cl;
            } else {
                us.push(U {
                    start: i,
                    base: cl,
                    last_raw: cl,
                    first_ep: self.ep.contains(c),
                });
            }
        }
        let ahl = |c: W| c == W::ALetter || c == W::HebrewLetter;
        let mnlq = |c: W| c == W::MidNumLet || c == W::SingleQuote;
        let mut out = Vec::new();
        for k in 1..us.len() {
            let (p, c) = (&us[k - 1], &us[k]);
            let (pb, cb) = (p.base, c.base);
            let prev2 = if k >= 2 { Some(us[k - 2].base) } else { None };
            let next = us.get(k + 1).map(|u| u.base);
            let no_break = if pb == W::CR && cb == W::LF {
                true // WB3
            } else if pb == W::Newline || pb == W::CR || pb == W::LF || cb == W::Newline || cb == W::CR || cb == W::LF {
                false // WB3a, WB3b
            } else if p.last_raw == W::ZWJ && c.first_ep {
                true // WB3c
            } else if p.last_raw == W::WSegSpace && p.base == W::WSegSpace && cb == W::WSegSpace {
                true // WB3d (adjacency of raw characters: p has no Extend tail)
            } else if ahl(pb) && ahl(cb) {
                true // WB5
            } else if ahl(pb) && (cb == W::MidLetter || mnlq(cb)) && next.map_or(false, ahl) {
                true // WB6
            } else if prev2.map_or(false, ahl) && (pb == W::MidLetter || mnlq(pb)) && ahl(cb) {
                true // WB7
            } else if pb == W::HebrewLetter && cb == W::SingleQuote {
                true // WB7a
            } else if pb == W::HebrewLetter && cb == W::DoubleQuote && next == Some(W::HebrewLetter) {
                true // WB7b
            } else if prev2 == Some(W::HebrewLetter) && pb == W::DoubleQuote && cb == W::HebrewLetter {
                true // WB7c
            } else if pb == W::Numeric && cb == W::Numeric {
                true // WB8
            } else if ahl(pb) && cb == W::Numeric {
                true // WB9
            } else if pb == W::Numeric && ahl(cb) {
                true // WB10
            } else if prev2 == Some(W::Numeric) && (pb == W::MidNum || mnlq(pb)) && cb == W::Numeric {
                true // WB11
            } else if pb == W::Numeric && (cb == W::MidNum || mnlq(cb)) && next == Some(W::Numeric) {
                true // WB12
            } else if pb == W::Katakana && cb == W::Katakana {
                true // WB13
            } else if (ahl(pb) || pb == W::Numeric || pb == W::Katakana || pb == W::ExtendNumLet) && cb == W::ExtendNumLet {
                true // WB13a
            } else if pb == W::ExtendNumLet && (ahl(cb) || cb == W::Numeric || cb == W::Katakana) {
                true // WB13b
            } else if pb == W::RegionalIndicator && cb == W::RegionalIndicator {
                // WB15/16: odd number of RI before the boundary
                let mut n = 0;
                let mut j = k;
                while j > 0 && us[j - 1].base == W::RegionalIndicator {
                    n += 1;
                    j -= 1;
                }
                n % 2 == 1
            } else {
                false // WB999
            };
            if !no_break {
                out.push(c.start);
            }
        }
        if !s.is_empty() {
            out.push(s.len());
        }
        out
    }

    fn segments<'a>(&self, s: &'a str) -> Vec<&'a str> {
        let mut v = Vec::new();
        let mut prev = 0;
        for b in self.breaks(s) {
            v.push(&s[prev..b]);
            prev = b;
        }
        v
    }
}

// The "word" predicate (ADR 0030 item 3, owner decision OD-5, D-19): a segment does not consist solely of whitespace characters and
// punctuation, that is, it contains a character whose General_Category is not Z* (Zs, Zl, Zp), not P* (Pc, Pd, Ps, Pe, Pi, Pf, Po),
// not Cc and not Cf. Letters, numbers, M* marks, S* symbols (including emoji), Co and Cn are words.
fn is_blank_gc(gc: icu_properties::props::GeneralCategory) -> bool {
    use icu_properties::props::GeneralCategory as G;
    GeneralCategoryGroup::Separator.contains(gc)
        || GeneralCategoryGroup::Punctuation.contains(gc)
        || gc == G::Control
        || gc == G::Format
}

fn is_word(seg: &str) -> bool {
    let g = CodePointMapData::<icu_properties::props::GeneralCategory>::new();
    seg.chars().any(|c| !is_blank_gc(g.get(c)))
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

fn parse_ucd(path: &str) -> Vec<(u32, u32, String)> {
    let mut v = Vec::new();
    for l in fs::read_to_string(path).unwrap().lines() {
        let l = l.split('#').next().unwrap().trim();
        if l.is_empty() {
            continue;
        }
        let (r, p) = l.split_once(';').unwrap();
        let r = r.trim();
        let (a, b) = match r.split_once("..") {
            Some((a, b)) => (a, b),
            None => (r, r),
        };
        v.push((u32::from_str_radix(a, 16).unwrap(), u32::from_str_radix(b, 16).unwrap(), p.trim().to_string()));
    }
    v
}

fn wb_by_name(n: &str) -> W {
    match n {
        "ALetter" => W::ALetter,
        "Format" => W::Format,
        "Katakana" => W::Katakana,
        "MidLetter" => W::MidLetter,
        "MidNum" => W::MidNum,
        "Numeric" => W::Numeric,
        "ExtendNumLet" => W::ExtendNumLet,
        "CR" => W::CR,
        "Extend" => W::Extend,
        "LF" => W::LF,
        "MidNumLet" => W::MidNumLet,
        "Newline" => W::Newline,
        "Regional_Indicator" => W::RegionalIndicator,
        "Hebrew_Letter" => W::HebrewLetter,
        "Single_Quote" => W::SingleQuote,
        "Double_Quote" => W::DoubleQuote,
        "ZWJ" => W::ZWJ,
        "WSegSpace" => W::WSegSpace,
        other => panic!("unknown Word_Break value {other}"),
    }
}

fn show<S: AsRef<str>>(v: &[S], flags: bool) -> String {
    v.iter()
        .map(|x| {
            let x = x.as_ref();
            format!("{:?}{}", x, if flags && is_word(x) { "*" } else { "" })
        })
        .collect::<Vec<_>>()
        .join(" | ")
}

fn icu_segments<'a>(seg: &icu_segmenter::WordSegmenterBorrowed<'_>, s: &'a str) -> Vec<&'a str> {
    let mut v = Vec::new();
    let mut prev = 0;
    for b in seg.segment_str(s) {
        if b > prev {
            v.push(&s[prev..b]);
            prev = b;
        }
    }
    v
}

fn main() {
    let a: Vec<String> = env::args().collect();
    let me = Seg::new();
    let icu = WordSegmenter::new_for_non_complex_scripts(WordBreakInvariantOptions::default());
    match a.get(1).map(|s| s.as_str()) {
        Some("wbt") => {
            let (mut total, mut ok) = (0, 0);
            for line in fs::read_to_string(&a[2]).unwrap().lines() {
                if let Some((s, exp)) = parse_wbt_line(line) {
                    total += 1;
                    let got = me.breaks(&s);
                    if got == exp {
                        ok += 1;
                    } else if total - ok <= 20 {
                        println!("MISMATCH: {line}\n  own boundaries {got:?}");
                    }
                }
            }
            println!("UAX #29 rules on icu_properties data: {ok}/{total} matched WordBreakTest-18.0.0");
        }
        Some("data") => {
            let mut exp: BTreeMap<u32, W> = BTreeMap::new();
            for (lo, hi, n) in parse_ucd(&a[2]) {
                for c in lo..=hi {
                    exp.insert(c, wb_by_name(&n));
                }
            }
            let mut ep: std::collections::BTreeSet<u32> = Default::default();
            for (lo, hi, n) in parse_ucd(&a[3]) {
                if n == "Extended_Pictographic" {
                    for c in lo..=hi {
                        ep.insert(c);
                    }
                }
            }
            let (mut n, mut bad_wb, mut bad_ep) = (0, 0, 0);
            for cp in 0..=0x10FFFFu32 {
                let Some(c) = char::from_u32(cp) else { continue };
                n += 1;
                if me.wb.get(c) != exp.get(&cp).copied().unwrap_or(W::Other) {
                    bad_wb += 1;
                    if bad_wb <= 10 {
                        println!("Word_Break mismatch: U+{cp:04X}");
                    }
                }
                if me.ep.contains(c) != ep.contains(&cp) {
                    bad_ep += 1;
                    if bad_ep <= 10 {
                        println!("Extended_Pictographic mismatch: U+{cp:04X}");
                    }
                }
            }
            println!("checked {n} scalars; Word_Break mismatches: {bad_wb}; Extended_Pictographic: {bad_ep}");
        }
        Some("diff") => {
            // For each scalar c: the strings "a"+c, c+"a", c+c, c+c+c, "1"+c+"1", "ー"+c. A boundary mismatch is grouped by script.
            let sc = CodePointMapData::<Script>::new();
            let names = PropertyNamesLong::<Script>::new();
            let mut by_script: BTreeMap<String, (u32, u32, String, String, String)> = BTreeMap::new();
            let mut total_diff = 0u32;
            for cp in 0..=0x10FFFFu32 {
                let Some(c) = char::from_u32(cp) else { continue };
                let cs = c.to_string();
                let forms = [
                    format!("a{cs}"),
                    format!("{cs}a"),
                    format!("{cs}{cs}"),
                    format!("{cs}{cs}{cs}"),
                    format!("1{cs}1"),
                    format!("\u{30FC}{cs}"),
                ];
                let mut hit: Option<String> = None;
                for f in &forms {
                    let own = me.segments(f);
                    let ic = icu_segments(&icu, f);
                    if own != ic {
                        hit = Some(format!("{} ‖ ICU4X {}", show(&own, false), show(&ic, false)));
                        break;
                    }
                }
                if let Some(h) = hit {
                    if env::var("DIFF_LIST").is_ok() {
                        eprintln!("U+{cp:04X}\t{:?}\t{:?}", names.get(sc.get(c)).unwrap_or("?"), me.wb.get(c));
                    }
                    total_diff += 1;
                    let name = names.get(sc.get(c)).unwrap_or("?").to_string();
                    let e = by_script.entry(name).or_insert((0, cp, h, String::new(), String::new()));
                    e.0 += 1;
                    e.3 = format!("U+{cp:04X}");
                }
            }
            println!("scalars with a boundary mismatch (own UAX #29 vs ICU4X non_complex): {total_diff}");
            for (s, (n, first, h, last, _)) in &by_script {
                println!("  {s:24} {n:6} scalars; first U+{first:04X}, last {last}; example: {h}");
            }
        }
        Some("points") => {
            let cases: Vec<(&str, String)> = vec![
                ("Han", "李雷".into()),
                ("Han with space", "李雷 韩梅梅".into()),
                ("hiragana", "こんにちは".into()),
                ("hiragana+kanji", "こんにちは世界".into()),
                ("katakana", "コンピュータ".into()),
                ("kanji+katakana", "東京タワー".into()),
                ("Han+Latin", "李雷Smith".into()),
                ("Han+digit", "李雷2".into()),
                ("Hangul", "김철수".into()),
                ("Thai", "สวัสดีครับ".into()),
                ("Thai+Latin", "Somchai สมชาย".into()),
                ("Lao", "ສະບາຍດີ".into()),
                ("Myanmar", "မြန်မာ".into()),
                ("Myanmar 2", "မင်္ဂလာပါ".into()),
                ("Khmer", "សួស្តី".into()),
                ("Tibetan", "བཀྲ་ཤིས".into()),
                ("«_»", "_".into()),
                ("john_smith", "john_smith".into()),
                ("U+203F", "\u{203F}".into()),
                ("Ann<U+202F>Smith", "Ann\u{202F}Smith".into()),
                ("U+202F alone", "\u{202F}".into()),
                ("TAB between words", "Ann\tSmith".into()),
                ("«ё»+U+0308 (NFC)", "\u{451}\u{308}".into()),
                ("emoji", "ab😀cd".into()),
                ("only hyphen", "- '".into()),
                ("O'Brien", "O'Brien".into()),
                ("colon", "Smith:John".into()),
            ];
            for (n, s) in &cases {
                println!("{n:22} UAX#29: {}", show(&me.segments(s), true));
                println!("{:22} ICU4X : {}", "", show(&icu_segments(&icu, s), true));
            }
            println!("(* = word by the predicate: has a character outside Z*, P*, Cc, Cf)");
        }
        Some("golden") => {
            // Product reference (variant B): segments by the UAX #29 rules on icu_properties data; WordSegmenter is not involved.
            // Line: <spec scenario> | <string> | segments; "*" — a word by the predicate (has a character outside Z*, P*, Cc, Cf).
            let cases: Vec<(&str, String)> = vec![
                ("Han: 李雷", "李雷".into()),
                ("Han with space: 李雷 韩梅梅", "李雷 韩梅梅".into()),
                ("hiragana+kanji: こんにちは世界", "こんにちは世界".into()),
                ("katakana+kanji: 東京タワー", "東京タワー".into()),
                ("Han+Latin: 李雷Smith", "李雷Smith".into()),
                ("Han+digit", "李雷2".into()),
                ("Hangul", "김철수".into()),
                ("Thai: Somchai สมชาย", "Somchai สมชาย".into()),
                ("Thai: สวัสดี", "สวัสดี".into()),
                ("Lao: ສະບາຍດີ", "ສະບາຍດີ".into()),
                ("Myanmar: မြန်မာ", "မြန်မာ".into()),
                ("Khmer", "សួស្តី".into()),
                ("Tibetan: བཀྲ་ཤིས", "བཀྲ་ཤིས".into()),
                ("Cyrillic+Latin: Annиван", "Annиван".into()),
                ("apostrophe U+0027: O'Brien", "O'Brien".into()),
                ("apostrophe U+2019: O’Brien", "O\u{2019}Brien".into()),
                ("apostrophe U+2018: O‘Brien", "O\u{2018}Brien".into()),
                ("apostrophe at end: o'", "o'".into()),
                ("colon: Smith:John", "Smith:John".into()),
                ("middle dot U+00B7: Ann·Bob", "Ann\u{B7}Bob".into()),
                ("dot between letters: Smith.John", "Smith.John".into()),
                ("dot between digits: 3.14", "3.14".into()),
                ("comma between digits: 1,000", "1,000".into()),
                ("digits and letters: A1 2B", "A1 2B".into()),
                ("«_»: john_smith", "john_smith".into()),
                ("«_» alone", "_".into()),
                ("«+» alone (Sm)", "+".into()),
                ("«$» alone (Sc)", "$".into()),
                ("emoji alone: 😀", "😀".into()),
                ("emoji in a name: Ann 😀", "Ann 😀".into()),
                ("«C++»", "C++".into()),
                ("U+200B alone (Cf)", "\u{200B}".into()),
                ("U+0000 alone (Cc)", "\u{0}".into()),
                ("U+203F alone", "\u{203F}".into()),
                ("hyphen: Anna-Maria", "Anna-Maria".into()),
                ("only hyphen, space, apostrophe", "- '".into()),
                ("U+202F: Ann<U+202F>Smith", "Ann\u{202F}Smith".into()),
                ("U+202F alone", "\u{202F}".into()),
                ("TAB: Ann<TAB>Smith", "Ann\tSmith".into()),
                ("«ё»+U+0308", "\u{451}\u{308}".into()),
                ("emoji: ab😀cd", "ab😀cd".into()),
                ("space: Иван Петров", "Иван Петров".into()),
            ];
            for (n, s) in &cases {
                println!("{n:36} | {}", show(&me.segments(s), true));
            }
            println!("(* = word by the predicate: has a character outside Z*, P*, Cc, Cf)");
        }
        Some("gc") => {
            // General_Category from icu_properties against UnicodeData-18.0.0 on all scalars (Cn — those absent from the file;
            // ranges "<..., First>"/"<..., Last>"); White_Space — against PropList-18.0.0.
            let names = PropertyNamesShort::<icu_properties::props::GeneralCategory>::new();
            let mut exp: BTreeMap<u32, String> = BTreeMap::new();
            let mut first: Option<u32> = None;
            for line in fs::read_to_string(&a[2]).unwrap().lines() {
                let f: Vec<&str> = line.split(';').collect();
                if f.len() < 3 {
                    continue;
                }
                let cp = u32::from_str_radix(f[0], 16).unwrap();
                if f[1].ends_with(", First>") {
                    first = Some(cp);
                } else if f[1].ends_with(", Last>") {
                    for c in first.take().unwrap()..=cp {
                        exp.insert(c, f[2].to_string());
                    }
                } else {
                    exp.insert(cp, f[2].to_string());
                }
            }
            let mut ws: std::collections::BTreeSet<u32> = Default::default();
            for (lo, hi, n) in parse_ucd(&a[3]) {
                if n == "White_Space" {
                    for c in lo..=hi {
                        ws.insert(c);
                    }
                }
            }
            let g = CodePointMapData::<icu_properties::props::GeneralCategory>::new();
            let wsd = CodePointSetData::new::<icu_properties::props::WhiteSpace>();
            let (mut n, mut bad_gc, mut bad_ws, mut bad_ln, mut cc) = (0, 0, 0, 0, 0);
            for cp in 0..=0x10FFFFu32 {
                let Some(c) = char::from_u32(cp) else { continue };
                n += 1;
                let e = exp.get(&cp).map(|s| s.as_str()).unwrap_or("Cn");
                let got = names.get(g.get(c)).unwrap_or("?");
                if got != e {
                    bad_gc += 1;
                    if bad_gc <= 10 {
                        println!("General_Category mismatch: U+{cp:04X} icu={got} UCD={e}");
                    }
                }
                if e == "Cc" {
                    cc += 1;
                }
                let e_ln = !(e.starts_with('Z') || e.starts_with('P') || e == "Cc" || e == "Cf");
                let g_ln = !is_blank_gc(g.get(c));
                if e_ln != g_ln {
                    bad_ln += 1;
                }
                if wsd.contains(c) != ws.contains(&cp) {
                    bad_ws += 1;
                    if bad_ws <= 10 {
                        println!("White_Space mismatch: U+{cp:04X}");
                    }
                }
            }
            println!(
                "checked {n} scalars; General_Category mismatches: {bad_gc}; «word» predicate (not Z*/P*/Cc/Cf) mismatches: {bad_ln}; White_Space: {bad_ws}; Cc: {cc}"
            );
        }
        Some("word") => {
            // The "word" predicate (not Z*/P*/Cc/Cf) against ICU4X is_word_like on individual scalars
            let mut mism: BTreeMap<String, u32> = BTreeMap::new();
            let g = CodePointMapData::<icu_properties::props::GeneralCategory>::new();
            let mut n = 0;
            for cp in 0..=0x10FFFFu32 {
                let Some(c) = char::from_u32(cp) else { continue };
                let s = c.to_string();
                let mut it = icu.segment_str(&s).iter_with_word_type();
                let mut like = false;
                while let Some((b, t)) = it.next() {
                    if b > 0 {
                        like = t.is_word_like();
                    }
                }
                let mine = is_word(&s);
                n += 1;
                if mine != like {
                    *mism.entry(format!("{:?} word={} ICU4X word-like={}", g.get(c), mine, like)).or_default() += 1;
                }
            }
            println!("scalars {n}; mismatches of the «word» predicate (not Z*/P*/Cc/Cf) vs is_word_like (single scalar):");
            for (k, v) in mism {
                println!("  {k}: {v}");
            }
        }
        Some("key") => {
            // Comparison key: NFC → fold (C+F) → NFC → U+0451→U+0435; idempotence and the form on "ё"+U+0308
            let nfc = icu_normalizer::ComposingNormalizerBorrowed::new_nfc();
            let cm = icu_casemap::CaseMapper::new();
            let key = |s: &str| -> String {
                let a = nfc.normalize(s);
                let b = cm.fold_string(&a).to_string();
                nfc.normalize(&b).replace('\u{451}', "\u{435}")
            };
            let show_cps = |s: &str| s.chars().map(|c| format!("U+{:04X}", c as u32)).collect::<Vec<_>>().join(" ");
            for (n, s) in [
                ("Ёлка", "Ёлка"),
                ("«ё»+U+0308 (NFC: U+0451 U+0308)", "\u{451}\u{308}"),
                ("«Ё»+U+0308", "\u{401}\u{308}"),
                ("«е»+U+0308+U+0308", "\u{435}\u{308}\u{308}"),
            ] {
                let k = key(s);
                let kk = key(&k);
                println!("{n}: key = [{}]; NFC(key) == key: {}; key(key) == key: {}", show_cps(&k), nfc.normalize(&k) == k, kk == k);
            }
            let (mut n, mut bad) = (0, 0);
            for cp in 0..=0x10FFFFu32 {
                let Some(c) = char::from_u32(cp) else { continue };
                let s = c.to_string();
                let k = key(&s);
                n += 1;
                if key(&k) != k {
                    bad += 1;
                }
            }
            println!("single scalars: {n}, non-idempotent keys: {bad}");
            let mut bad2 = 0u32;
            let mut ex = String::new();
            for cp in 0..=0x10FFFFu32 {
                let Some(c) = char::from_u32(cp) else { continue };
                for tail in ['\u{308}', '\u{301}', '\u{323}'] {
                    let s: String = [c, tail].iter().collect();
                    let k = key(&s);
                    if key(&k) != k {
                        bad2 += 1;
                        if ex.is_empty() {
                            ex = show_cps(&s);
                        }
                    }
                }
            }
            println!("pairs «char + U+0308/U+0301/U+0323»: non-idempotent keys {bad2}; first example: {ex}");
        }
        _ => eprintln!("usage: wbt|data|diff|points|word|key"),
    }
}
