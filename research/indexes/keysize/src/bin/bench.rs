// Microbenchmark of the CPU cost of planner operations (ADR 0047): merging sorted key runs and
// sorting keys by memcmp. std only. The data is pseudo-random 40-byte keys (xorshift64*).
use std::time::Instant;
fn xs(s: &mut u64) -> u64 {
    *s ^= *s >> 12;
    *s ^= *s << 25;
    *s ^= *s >> 27;
    s.wrapping_mul(0x2545F4914F6CDD1D)
}
fn main() {
    let mut s = 0x5EED0001u64;
    for &n in &[1usize << 15, 1 << 17, 1 << 20] {
        let mut keys: Vec<[u8; 40]> = Vec::with_capacity(n);
        for _ in 0..n {
            let mut k = [0u8; 40];
            for c in k.chunks_mut(8) {
                c.copy_from_slice(&xs(&mut s).to_be_bytes());
            }
            keys.push(k);
        }
        let mut a = keys.clone();
        let t = Instant::now();
        a.sort_unstable();
        let d = t.elapsed();
        println!("sort_unstable n={n}: {:.2} ms ({:.0} ns/key)", d.as_secs_f64() * 1e3, d.as_nanos() as f64 / n as f64);
        // k runs of n/k keys, each sorted; heap merge.
        for &k in &[16usize, 256, 4096] {
            let per = n / k;
            let mut runs: Vec<Vec<[u8; 40]>> = Vec::new();
            for r in 0..k {
                let mut v = keys[r * per..(r + 1) * per].to_vec();
                v.sort_unstable();
                runs.push(v);
            }
            let t = Instant::now();
            let mut heap = std::collections::BinaryHeap::new();
            for (i, r) in runs.iter().enumerate() {
                heap.push(std::cmp::Reverse((r[0], i, 0usize)));
            }
            let mut out = 0usize;
            while let Some(std::cmp::Reverse((_, i, j))) = heap.pop() {
                out += 1;
                if j + 1 < runs[i].len() {
                    heap.push(std::cmp::Reverse((runs[i][j + 1], i, j + 1)));
                }
            }
            let d = t.elapsed();
            println!("heap merge n={n} k={k}: {:.2} ms ({:.0} ns/key) out={out}", d.as_secs_f64() * 1e3, d.as_nanos() as f64 / n as f64);
        }
    }
}
