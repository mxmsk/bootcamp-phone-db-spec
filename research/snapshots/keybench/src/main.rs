//! Measurement of the speed of building a name sort key (ADR 0039 item 3; Q-82): how long rebuilding the
//! order index from the stored names will take. The same pinned `icu_*` versions as in ../../ordering/icu-pin.
//! Names: pseudo-random, Cyrillic and Latin, 2 words (first name + last name) of 4..=10 letters each; the length of a typical
//! name in records is an estimate, not product data. Prints: threads, names, seconds, names/s, average key length.
use icu_collator::options::CollatorOptions;
use icu_collator::CollatorBorrowed;
use icu_locale::locale;
use std::time::Instant;

struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        self.0.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }
}

fn word(r: &mut Rng, cyr: bool) -> String {
    let n = 4 + (r.next() % 7) as usize;
    let mut s = String::with_capacity(n * 2);
    for i in 0..n {
        let c = if cyr {
            // а..я (U+0430..U+044F), the first letter is uppercase (А..Я U+0410..U+042F)
            let base = if i == 0 { 0x0410 } else { 0x0430 };
            char::from_u32(base + (r.next() % 32) as u32).unwrap()
        } else {
            let base = if i == 0 { b'A' } else { b'a' };
            (base + (r.next() % 26) as u8) as char
        };
        s.push(c);
    }
    s
}

fn main() {
    let threads: usize = std::env::args().nth(1).and_then(|a| a.parse().ok()).unwrap_or(1);
    let per_thread: usize = std::env::args().nth(2).and_then(|a| a.parse().ok()).unwrap_or(1_000_000);
    let start = Instant::now();
    let handles: Vec<_> = (0..threads)
        .map(|t| {
            std::thread::spawn(move || {
                let c: CollatorBorrowed<'static> =
                    CollatorBorrowed::try_new(locale!("ru").into(), CollatorOptions::default()).unwrap();
                let mut r = Rng(0x5EED_0001 + t as u64 * 7919);
                // Prepare the names in advance to measure only the key construction.
                let names: Vec<String> = (0..per_thread.min(200_000))
                    .map(|_| {
                        let cyr = r.next() % 4 != 0;
                        format!("{} {}", word(&mut r, cyr), word(&mut r, cyr))
                    })
                    .collect();
                let t0 = Instant::now();
                let mut key: Vec<u8> = Vec::with_capacity(256);
                let mut total_key = 0usize;
                for i in 0..per_thread {
                    key.clear();
                    let Ok(()) = c.write_sort_key_to(&names[i % names.len()], &mut key);
                    total_key += key.len();
                }
                (t0.elapsed().as_secs_f64(), total_key)
            })
        })
        .collect();
    let mut max_t: f64 = 0.0;
    let mut total_key = 0usize;
    for h in handles {
        let (t, k) = h.join().unwrap();
        max_t = max_t.max(t);
        total_key += k;
    }
    let n = threads * per_thread;
    let _ = start;
    println!(
        "threads={threads} names={n} seconds={max_t:.2} names_per_sec={:.0} avg_key_bytes={:.1}",
        n as f64 / max_t,
        total_key as f64 / n as f64
    );
}
