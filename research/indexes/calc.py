#!/usr/bin/env python3
"""Calculation of index size and page-read counts for 10^8 records (ADR 0045-0049).
All input numbers are assumptions listed in PARAMS; measured (keysize/output-2026-10-27.txt): average ru name length
33.7 UTF-8 bytes, sort key 30.2 bytes (12 names), 1.7 bytes per character. The rest is a model, not a measurement."""
import math, sys

N = 10**8
PARAMS = dict(
    page=16384, page_hdr=64, name_b=34, sortkey_b=30, words=2.5, wordkey_b=12, wordkeys_val=34,
    version_b=32, rec_per_number=1.25, distinct_words=1e7, hash_b=8, entry_ovh=2, child_ptr=8, agg=8, sep_b=24,
)
def sizes(p, lcp_save, fill, rec_per_number=None):
    rpn = rec_per_number or p['rec_per_number']
    U = p['page'] - p['page_hdr']
    order_tail = 2 + 3 + 8                      # 00 01 + DateKey + NumKey
    orderkey = p['sortkey_b'] + order_tail
    # R: record 11 + name; number header 11 + version
    r_rec = 11 - 3 + p['name_b'] + p['entry_ovh']
    r_hdr = 11 - 3 + p['version_b'] + p['entry_ovh']
    R = N * r_rec + (N / rpn) * r_hdr
    # O: orderkey with common-prefix compression of the sort key + value (word keys)
    O = N * (orderkey - lcp_save * p['sortkey_b'] + p['entry_ovh'] + p['wordkeys_val'])
    # W (words): wordkey is compressed to ~2 bytes (runs of one word); the order key with half the compression (rare words)
    w_entry = 2 + (orderkey - 0.6 * lcp_save * p['sortkey_b']) + p['entry_ovh']
    Wp = N * p['words'] * w_entry
    # E: 1 + hash 8 + orderkey; equal names share a hash and a sort key (repeat share 0.4)
    e_entry = 1 + p['hash_b'] + orderkey + p['entry_ovh'] - 0.4 * lcp_save * (p['hash_b'] + p['sortkey_b'])
    E = N * e_entry
    D = p['distinct_words'] * (p['wordkey_b'] * 0.7 + 4 + p['entry_ovh'])
    leaf = dict(R=R, O=O, W=Wp + E, D=D)
    tot = {k: v / fill for k, v in leaf.items()}
    # inner pages: separator + pointer (+ counters for D)
    internal = {}
    levels = {}
    ent = dict(R=N + N / rpn, O=N, W=N * p['words'] + N, D=p['distinct_words'])
    for k in leaf:
        leaves = leaf[k] / fill / U
        per = p['sep_b'] + p['child_ptr'] + (2 * p['agg'] if k == 'D' else 0)
        f = U * fill / per
        lv, cnt, pages_int = 1, leaves, 0
        while cnt > 1:
            cnt = math.ceil(cnt / f); pages_int += cnt; lv += 1
        internal[k] = pages_int * p['page']; levels[k] = lv
    return leaf, tot, internal, levels, dict(order=orderkey, r_rec=r_rec, r_hdr=r_hdr, w=w_entry, e=e_entry)

def gb(x): return f"{x/1e9:6.1f}"
print("== Index size, 10^8 records, 16 KiB page (GB = 1e9 bytes) ==")
for page in (8192, 16384):
    for lcp, nm in ((0.25, 'weak prefix compression'), (0.45, 'medium'), (0.60, 'strong')):
        for fill, fn in ((0.69, 'random inserts (0.69)'), (0.90, 'after rebuild (0.90)')):
            p = dict(PARAMS, page=page)
            leaf, tot, internal, levels, e = sizes(p, lcp, fill)
            s = sum(tot.values()) + sum(internal.values())
            print(f"page={page//1024:2d}K {nm:24s} {fn:26s} R{gb(tot['R'])} O{gb(tot['O'])} W+E{gb(tot['W'])} D{gb(tot['D'])} inner{gb(sum(internal.values()))} TOTAL{gb(s)}  levels {levels}")
print()
print("== Sensitivity to records per number and to name length (page 16K, medium compression, fill 0.69) ==")
for rpn in (1.0, 1.25, 2.0, 4.0):
    leaf, tot, internal, levels, e = sizes(PARAMS, 0.45, 0.69, rpn)
    print(f"records per number {rpn}: total {gb(sum(tot.values())+sum(internal.values()))} GB")
for nm_b, sk, wk, vl in ((20, 18, 18, 20), (34, 30, 30, 34), (68, 60, 60, 68), (120, 100, 105, 120)):
    p = dict(PARAMS, name_b=nm_b, sortkey_b=sk, wordkeys_val=vl)
    leaf, tot, internal, levels, e = sizes(p, 0.45, 0.69)
    print(f"name {nm_b} B, sort key {sk} B: total {gb(sum(tot.values())+sum(internal.values()))} GB")
for w in (1.5, 2.5, 4.0):
    p = dict(PARAMS, words=w)
    leaf, tot, internal, levels, e = sizes(p, 0.45, 0.69)
    print(f"words per name {w}: total {gb(sum(tot.values())+sum(internal.values()))} GB")
print()
print("== Worst case of a single record (maximum allowed name is 200 chars) ==")
for nm, chars, name_b, sk, words in (("200 Han chars, each one a word", 200, 600, 200 * 4.42 + 5, 200),
                                   ("200 chars U+FDFA (one word)", 200, 800, 8513, 1),
                                   ("200 Cyrillic chars in 40 words", 200, 380, 200 * 1.71 + 5, 40)):
    ok = sk + 13
    per_post = 2 + ok + 2
    tot = name_b + 11 + ok + 2 + 40 + per_post * words + (1 + 8 + ok + 2) + 32 / 1.25
    print(f"{nm}: order key {ok:.0f} B, postings {words}, total per record without compression ~{tot/1024:.1f} KiB (typical ~{(sum([83.6, 65.5-0, 38*2.5, 39]) ):.0f} B)")
print()
print("== Page reads (cold cache; internal pages are cached) ==")
p = PARAMS; U = p['page'] - p['page_hdr']; fill = 0.69
leaf, tot, internal, levels, e = sizes(p, 0.45, fill)
print("levels:", levels, " internal pages, MiB:", {k: round(v / 2**20) for k, v in internal.items()})
Wl = (N * p['words'] * e['w'] / fill) / U
per_w = U * fill / e['w']; per_o = U * fill / (e['order'] + p['wordkeys_val'] - 0.45*30); per_r = U*fill/ (e['r_rec'])
print(f"W postings per page ~{per_w:.0f}; O records per page ~{per_o:.0f}; R records per page ~{per_r:.0f}")
M1 = 2**17
print(f"M1={M1}: G1 reads {M1/per_w:.0f} pages ({M1*e['w']/2**20:.1f} MiB); G2 boundary: expected {50*N/M1:.0f} O records = {50*N/M1/per_o:.0f} pages")
t_seq, t_rand, par = 8e-6, 100e-6, 8
print("\ntime model: sequential page 8 us, random read 100 us (QD1), 8 in parallel; CPU: merge/check ~100 ns per record")
def row(name, seq, rnd, cpu_entries):
    t_ser = seq * t_seq + rnd * t_rand + cpu_entries * 100e-9
    t_par = seq * t_seq + rnd * t_rand / par + cpu_entries * 100e-9
    print(f"{name:62s} seq.pages {seq:9.0f} rand.rd {rnd:7.0f}  time {t_par*1e3:8.1f}..{t_ser*1e3:8.1f} ms")
d = levels
row("Lookup by number, <=10 records", 1, d['R'] - 2, 0)
row("Number with 2,967,164 records (stream)", 2967164 * e['r_rec'] / U / fill, 0, 2967164)
row("Exact search, page of 50 (W descent + 50 R reads)", 2, 50 + d['W'] - 2, 100)
row("Word, C=100 (G1)", 1, 50 + 2 + 2, 200)
row("Word, C=10^4 (G1)", 10**4 / per_w, 50 + 4, 10**4)
row("Word, C=M1=131072 (G1, boundary)", M1 / per_w, 50 + 4, M1)
row("Word, C=M1+1 (G2, boundary)", 50 * N / M1 / per_o, 50 + 4, 50 * N / M1)
row("Word, C=10^6 (G2)", 50 * N / 1e6 / per_o, 50 + 4, 50 * N / 1e6)
row("Prefix of 1 char, C=1.7e7 (G2)", 50 * N / 1.7e7 / per_o, 50 + 4, 50 * N / 1.7e7)
row("Two words, C=1e3 and 6e5 (G1 + check of 1000 O records)", 1e3 / per_w, 50 + 1000 + 6, 1e3)
row("Two frequent words, C=3e6+3e6, empty intersection (G3)", 6e6 / per_w, 100, 6e6 * 1)
row("All prefixes k>1024, empty intersection (whole O)", N / per_o, 0, N)
row("Query of 200 Han chars: count of 200 prefixes (D) + G1", 4 + 1, 200 * 2 + 50 + 5, 5000)
print("\nRebuild of O+W+E from R (estimate): CPU keys ~5 us per record /8 cores:", round(N * 5e-6 / 8 / 60, 1), "min;",
      "external sort ~", round((leaf['O'] + leaf['W']) * 2 * 2 / 2e9 / 60, 1), "min at 2 GB/s (2 read+write passes)")
