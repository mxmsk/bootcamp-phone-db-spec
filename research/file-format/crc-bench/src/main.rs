// Vector check and CRC throughput measurement for a page (file-format subtask, ADR 0043).
use std::time::Instant;

fn bench<F: Fn(&[u8]) -> u32>(name: &str, page: usize, f: F) {
    let buf: Vec<u8> = (0..page).map(|i| (i.wrapping_mul(2654435761) >> 7) as u8).collect();
    let mut sink = 0u32;
    for _ in 0..2000 { sink ^= f(&buf); }
    let iters = 200_000usize;
    let t = Instant::now();
    for _ in 0..iters { sink ^= f(&buf); }
    let el = t.elapsed().as_secs_f64();
    let gib = (iters * page) as f64 / el / (1u64 << 30) as f64;
    println!("{name:<28} page {page:>6} B: {:>7.0} ns/page, {gib:>6.2} GiB/s (sink {sink})", el / iters as f64 * 1e9);
}

fn main() {
    let v = b"123456789";
    let isc = crc::Crc::<u32, crc::Table<16>>::new(&crc::CRC_32_ISCSI);
    println!("check vectors on \"123456789\": crc32c crate {:#010x} (expect 0xe3069283), crc ISCSI {:#010x}, crc32fast {:#010x} (IEEE, expect 0xcbf43926)",
        crc32c::crc32c(v), isc.checksum(v), crc32fast::hash(v));
    // Vectors for the specification scenarios: a 16 KiB page, the checksum field (bytes 0..4) zeroed.
    let zero = vec![0u8; 16384];
    let mut pat: Vec<u8> = (0..16384usize).map(|i| (i % 251) as u8).collect();
    for b in pat.iter_mut().take(4) { *b = 0; }
    println!("vector: CRC-32C(16384 zero bytes) = {:#010x}; CRC-32C(bytes i%251, first 4 zeroed) = {:#010x}",
        crc32c::crc32c(&zero), crc32c::crc32c(&pat));
    for page in [4096usize, 8192, 16384, 32768] {
        bench("crc32c (hw)", page, |b| crc32c::crc32c(b));
        bench("crc32fast (IEEE)", page, |b| crc32fast::hash(b));
        bench("crc Table<16> ISCSI", page, |b| isc.checksum(b));
    }
}
