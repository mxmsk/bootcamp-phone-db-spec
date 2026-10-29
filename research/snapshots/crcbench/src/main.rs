//! Speed of crc32c over a buffer of 16 KiB pages (one thread). Prints MiB/s.
use std::time::Instant;

fn main() {
    let page = 16 * 1024;
    let pages = 64 * 1024; // 1 GiB buffer
    let mut buf = vec![0u8; page * pages];
    let mut x: u64 = 0x9E37_79B9_7F4A_7C15;
    for b in buf.iter_mut() {
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        *b = x as u8;
    }
    let t0 = Instant::now();
    let mut acc = 0u32;
    for p in buf.chunks(page) {
        acc ^= crc32c::crc32c(p);
    }
    let s = t0.elapsed().as_secs_f64();
    println!("pages={pages} page_bytes={page} seconds={s:.3} MiB_per_s={:.0} acc={acc}", 1024.0 / s);
}
