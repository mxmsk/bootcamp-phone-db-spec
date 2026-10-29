//! Build check of the pinned set: the constructors the specification needs build and work
//! without `auto`/`lstm`/`unstable` (icu_segmenter is not used in the product, ADR 0039 item 4 (c)).
use icu_casemap::CaseMapper;
use icu_collator::options::CollatorOptions;
use icu_collator::CollatorBorrowed;
use icu_locale::locale;
use icu_normalizer::DecomposingNormalizerBorrowed;
use icu_properties::props::{ExtendedPictographic, WordBreak};
use icu_properties::{CodePointMapData, CodePointSetData};

fn main() {
    let c = CollatorBorrowed::try_new(locale!("ru").into(), CollatorOptions::default()).expect("ru data");
    let m = CaseMapper::new();
    let nfd = DecomposingNormalizerBorrowed::new_nfd();
    let wb = CodePointMapData::<WordBreak>::new();
    let ep = CodePointSetData::new::<ExtendedPictographic>();
    println!(
        "{:?} {} {} {:?} {}",
        c.compare("Яков", "Ann"),
        m.fold_string("ÁNN"),
        nfd.normalize("й").chars().count(),
        wb.get('a'),
        ep.contains('\u{1F600}')
    );
}
