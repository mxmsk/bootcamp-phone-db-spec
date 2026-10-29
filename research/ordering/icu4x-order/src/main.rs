//! Reference name sort orders for the specification (ADR 0036-0039).
//! Prints for each set the order by icu_collator =2.4.0, locale ru. Names that are equal under the collator are not
//! distinguished in the order (ADR 0036: there is no code-point key, their order is decided by date and number); in the output they stand side by side,
//! joined by " = " and, for output reproducibility, arranged by code points.
//! Output format: one line per variant: `<set> | <variant> | name1 < name2 = name3 < ...`.
//! The same set of variants is computed by research/ordering/icu4c-check (ICU4C) for an independent cross-check.

use std::cmp::Ordering;

use icu_collator::options::{AlternateHandling, CollatorOptions, Strength};
use icu_collator::preferences::{CollationCaseFirst, CollationNumericOrdering};
use icu_collator::CollatorPreferences;
use icu_collator::CollatorBorrowed;
use icu_locale::locale;

fn sorted(c: &CollatorBorrowed, names: &[&str]) -> String {
    let mut v = names.to_vec();
    // Code points are only for reproducible printing of names equal under the collator, not an order key of the results.
    v.sort_by(|a, b| match c.compare(a, b) {
        Ordering::Equal => a.chars().cmp(b.chars()),
        o => o,
    });
    let mut out = String::new();
    for (i, name) in v.iter().enumerate() {
        if i > 0 {
            out.push_str(if c.compare(v[i - 1], name) == Ordering::Equal { " = " } else { " < " });
        }
        out.push_str(name);
    }
    out
}

fn main() {
    let sets: Vec<(&str, Vec<&str>)> = vec![
        ("script", vec!["Ann", "Яков", "Бобр", "Zed", "Анна"]),
        ("yo-tail", vec!["Ежи", "Ёж", "Еж", "Её", "Жук"]),
        ("yo-pair", vec!["Ёлка", "Елка", "ЕЛКА"]),
        ("case", vec!["Антон", "АННА", "Анна", "анна"]),
        ("case-yo", vec!["Её", "Еж", "Ёж", "ёж", "Ежи"]),
        ("punct", vec!["AnnZed", "Anna", "Ann-Zed", "Ann Zed"]),
        ("punct2", vec!["Ann-Smith", "Ann Smith", "AnnSmith"]),
        ("numeric", vec!["Ann 10", "Ann 2"]),
        ("numeric2", vec!["Ann 2", "Ann 10", "Ann 1"]),
        ("invisible", vec!["A\u{00AD}nn", "Ann"]),
        ("smith", vec!["Bob Smith", "Ann Smith"]),
    ];
    // (name, strength, alternate_handling, case_first = upper, numeric)
    let variants: Vec<(&str, Strength, AlternateHandling, bool, bool)> = vec![
        ("tertiary+nonignorable (default)", Strength::Tertiary, AlternateHandling::NonIgnorable, false, false),
        ("secondary", Strength::Secondary, AlternateHandling::NonIgnorable, false, false),
        ("primary", Strength::Primary, AlternateHandling::NonIgnorable, false, false),
        ("tertiary+shifted", Strength::Tertiary, AlternateHandling::Shifted, false, false),
        ("tertiary+case_first upper", Strength::Tertiary, AlternateHandling::NonIgnorable, true, false),
        ("tertiary+numeric", Strength::Tertiary, AlternateHandling::NonIgnorable, false, true),
    ];
    for (vname, strength, alt, upper, numeric) in &variants {
        let mut o = CollatorOptions::default();
        o.strength = Some(*strength);
        o.alternate_handling = Some(*alt);
        let mut prefs: CollatorPreferences = locale!("ru").into();
        if *upper {
            prefs.case_first = Some(CollationCaseFirst::Upper);
        }
        if *numeric {
            prefs.numeric_ordering = Some(CollationNumericOrdering::True);
        }
        let c = CollatorBorrowed::try_new(prefs, o).expect("ru data");
        for (sname, names) in &sets {
            println!("{} | {} | {}", sname, vname, sorted(&c, names));
        }
    }
    // Comparison of ru and root: the Cyrl reordering is set by the locale.
    let root = CollatorBorrowed::try_new(locale!("und").into(), CollatorOptions::default()).expect("root");
    let ru = CollatorBorrowed::try_new(locale!("ru").into(), CollatorOptions::default()).expect("ru");
    println!("ru: Яков vs Ann = {:?}; root: Ann vs Бобр = {:?}", ru.compare("Яков", "Ann"), root.compare("Ann", "Бобр"));
    println!("equal: Ann vs A\\u00ADnn = {:?}", ru.compare("Ann", "A\u{00AD}nn"));
    println!("equal: NFD и+U+0306 vs й = {:?}", ru.compare("и\u{0306}", "й"));
}
