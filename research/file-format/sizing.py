#!/usr/bin/env python3
"""Database file size calculation for 10^8 records (file-format subtask, ADR 0041-0044).
Usage: python3 sizing.py   (standard library, no dependencies)

Part A: B+-tree leaf fill measured in a model, on a 16 KiB page under random inserts.
Part B: size under the "cell = header + key + value" model for a set of assumed indexes (see notes).
Part C: page capacity for keys of different lengths (how many cells fit when an overflow spill is needed).
All input numbers are profile parameters; the profile is an assumption, not a measurement on real data (there is no real data).
"""
import bisect, random

PAGE = 16384
HDR = 40                 # common header 32 B + tree header 8 B
CAP = PAGE - HDR         # usable capacity of a leaf/inner page
CELL_HDR = 3             # prefix_len, suffix_len, value_len (1 varint byte each for typical values)
SLOT = 2 / 16            # a u16 slot per every 16th cell (restart points)

# ---------- Part A: fill under random inserts ----------
def fill_sim(n, mean, sd, seed=1):
    rnd = random.Random(seed)
    leaves_keys = [[]]      # keys (float) per leaf
    leaves_sizes = [[]]
    mins = [-1.0]           # lower bound of the leaf
    used = [0]
    for _ in range(n):
        k = rnd.random()
        s = max(8, int(rnd.gauss(mean, sd)) + CELL_HDR)
        i = bisect.bisect_right(mins, k) - 1
        ks, ss = leaves_keys[i], leaves_sizes[i]
        j = bisect.bisect_left(ks, k)
        ks.insert(j, k); ss.insert(j, s); used[i] += s
        if used[i] > CAP:   # split in the middle by bytes
            half, acc, cut = used[i] / 2, 0, 0
            for cut, x in enumerate(ss):
                acc += x
                if acc >= half:
                    break
            cut += 1
            nk, ns = ks[cut:], ss[cut:]
            leaves_keys[i], leaves_sizes[i] = ks[:cut], ss[:cut]
            used[i] = sum(leaves_sizes[i])
            leaves_keys.insert(i + 1, nk); leaves_sizes.insert(i + 1, ns)
            mins.insert(i + 1, nk[0]); used.insert(i + 1, sum(ns))
    pages = len(leaves_keys)
    return sum(used) / (pages * CAP), pages

# ---------- Part B: size under the model ----------
def model(R=10**8, h=1.25, name_bytes=42.1, sort_key=35.2, words=2.5, word_key=16.0, distinct_words=1e7,
          fill=0.67, compress=0.0, inner=0.015, seq_bytes=5):
    """Bytes per record for the ADR 0045 layout (R, O, W, D); cell = header + slot + key + value.
    compress - the share of bytes saved by prefix compression in the "compressible" part of keys (sort key, word key, number/date of R).
    R: key number(8)+date(3), value is the name; number header: key 11, value seq.
    O: OrderKey = esc(sort key) + 00 01 + date(3) + number(8); value is the word keys (length 1 B + bytes).
    W: word: 0x01 + word key + 00 01 + OrderKey (one per distinct word); whole name: 0x02 + H64(8) + OrderKey.
    D: word dictionary: word key -> u32 (distinct_words entries)."""
    nums = R / h
    c = 1 - compress
    order_key = sort_key * c + 2 + 3 + 8
    hdr = CELL_HDR + SLOT
    r_rec = hdr + 11 * c + name_bytes
    r_hdr = (hdr + 11 * c + seq_bytes) * (nums / R)
    o = hdr + order_key + words * (1 + word_key)
    w_words = words * (hdr + 1 + word_key * c + 2 + order_key)
    w_exact = hdr + 1 + 8 + order_key
    d = distinct_words * (hdr + word_key * c + 4) / R
    per_rec = {'R records': r_rec, 'R number headers': r_hdr, 'O order': o, 'W words': w_words,
               'W whole name': w_exact, 'D dictionary': d}
    data = sum(per_rec.values())
    total = data * R / fill * (1 + inner)
    return per_rec, data, total

def gb(x):
    return x / 1e9

if __name__ == '__main__':
    print('== A. Leaf fill factor under random inserts (a model, not a real DB) ==')
    for mean, sd in [(58, 25), (49, 20), (65, 30), (120, 80)]:
        f, p = fill_sim(600_000, mean, sd)
        print(f'  cell size ~N({mean},{sd}): fill factor {f:.3f}, leaves {p}')
    print()
    print('== B. Size for 10^8 records under the ADR 0045 layout, profile P0: name 22 chars (42.1 bytes), sort key 35.2, 2.5 words ==')
    print('   (fill 0.67; internal nodes +1.5%; number version 5 bytes)')
    pr, d, t = model()
    print(f'  without prefix compression: {d:.0f} B/record, file {gb(t):.1f} GB')
    for k, v in pr.items():
        print(f'      {k:<20} {v:6.1f} B/record')
    print()
    print('== B2. Sensitivity to fill factor and prefix compression, GB ==')
    print('  fill  | compr. 0   0.1    0.25   0.45   0.6')
    for fill in (0.67, 0.71, 0.90):
        print(f'  {fill:.2f}  | ' + '  '.join(f'{gb(model(fill=fill, compress=c)[2]):5.1f}' for c in (0.0, 0.1, 0.25, 0.45, 0.6)))
    lo, hi = 0.0, 0.9
    for _ in range(60):
        mid = (lo + hi) / 2
        if model(compress=mid)[2] > 50e9: lo = mid
        else: hi = mid
    print(f'  compression at which the file is exactly 50 GB (fill 0.67): {hi:.3f}')
    print()
    print('== B3. Sensitivity to average name length (fill 0.67, no compression), GB ==')
    print('  chars   name bytes  sort key    words | no compr.    compr. 0.25   compr. 0.45')
    for chars, nb, sk, w, wk in [(12, 22, 19.0, 1.5, 12), (22, 42, 35.2, 2.5, 16), (40, 76, 63.0, 4.0, 17), (100, 190, 158.0, 8.0, 20),
                                (200, 380, 316.0, 15.0, 22)]:
        a = [gb(model(name_bytes=nb, sort_key=sk, words=w, word_key=wk, compress=c)[2]) for c in (0.0, 0.25, 0.45)]
        print(f'  {chars:>5}  {nb:>9}  {sk:>9.1f}  {w:>4}  | {a[0]:9.1f}   {a[1]:10.1f}   {a[2]:10.1f}')
    print()
    print('== B4. Sensitivity to records per number (no compression, P0) ==')
    for h in (1.0, 1.25, 3.0, 10.0):
        print(f'  records per number {h:>5}: {gb(model(h=h)[2]):.1f} GB')
    print()
    print('== B5. Worst case of a single record under the ADR 0045 layout ==')
    hdr = CELL_HDR + SLOT
    # name of 200 U+FDFA characters: name 800 B, sort key 8,513 B (esc: no 0x00 -> no growth), the word is one (characters in a row) or 200 words of 1 character
    ok = 8513 + 2 + 3 + 8
    # (a) 200 U+FDFA characters without spaces: one word
    a = (hdr + 11 + 800) + hdr + 11 + (hdr + ok + (1 + 600)) + (hdr + 1 + 600 + 2 + ok) + (hdr + 1 + 8 + ok)
    # (b) 200 Han words of 1 character (3 B): Han sort key ~4.4 B/character -> ~880 B
    sk = 880
    okb = sk + 2 + 3 + 8
    b = (hdr + 11 + 600) + (hdr + okb + 200 * (1 + 3)) + 200 * (hdr + 1 + 3 + 2 + okb) + (hdr + 1 + 8 + okb)
    print(f'  (a) 200 chars U+FDFA, one word: ~{a/1024:.1f} KiB of indexes per record')
    print(f'  (b) 200 Han words of one char each (sort key ~880 B): ~{b/1024:.1f} KiB of indexes per record')
    print()
    print('== C. Page capacity 16 KiB (usable', CAP, 'B) ==')
    for ksz in (35, 630, 1000, 2000, 2048, 4000, 8513, 9200):
        n = CAP // (ksz + 11 + CELL_HDR)
        print(f'  key {ksz:>5} B (+11 B tail, +3 cell header): cells that fit {n}')
    print('  spill threshold 2048 B: cells >= ', CAP // 2048)
    print('== D. Volumes ==')
    print('  internal node: separator ~24 B + 4 B pointer + 3 -> children per node ~', int(CAP // (24 + 4 + 3)))
    for pages_target_gb in (40, 64):
        print(f'  {pages_target_gb} GB = {pages_target_gb*1e9/PAGE/1e6:.2f} M pages of 16 KiB')
    print(f'  max file with a u32 page number: {2**32 * PAGE / 2**40:.0f} TiB')
