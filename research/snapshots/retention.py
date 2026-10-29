#!/usr/bin/env python3
"""Calculation of the cost of snapshots, backup and index rebuild (ADR 0055-0059). Arithmetic over explicit assumptions only;
measurements are keybench (sort key) and crcbench (crc32c). The output is saved in out-retention.txt."""

KIB = 1024
MIB = 1024 * KIB
GIB = 1024 * MIB

PAGE = 16 * KIB          # assumption: 16 KiB page (ADR 0019 item 2, not decided; Q-48)
FILE = 40 * 10**9        # owner's target "file <= 40 GB per 10^8 records" (brief §4; Q-47 recomputes)
STREAM_MAX_RECORDS = 2_967_164   # ADR 0035 item 6, F-7
STREAM_BYTES = 300 * 10**6       # ADR 0035 item 6: ~100 bytes per record, arithmetic

# Pages that one record changes under physical (page-level) retention of earlier versions:
# a number index leaf, an order index leaf, an exact-search leaf, a data/versions leaf, word index leaves.
# Words per name: 2-4 occurrences per record (estimate 2-4×10^8 occurrences per 10^8 records, design.md); we take 4.
PAGES_PER_WRITE = 1 + 1 + 1 + 1 + 4
print(f"Assumptions: page {PAGE // KIB} KiB, file {FILE / 1e9:.0f} GB, pages changed by one write: {PAGES_PER_WRITE}")
print(f"Upper bound on retention = W * T * {PAGES_PER_WRITE} * {PAGE // KIB} KiB = W * T * {PAGES_PER_WRITE * PAGE // KIB} KiB")
print("(upper bound: repeated edits of the same pages and shared upper tree levels are not accounted for; not more than the file size)")
print()


def fmt(b):
    if b >= GIB:
        return f"{b / GIB:.1f} GiB"
    return f"{b / MIB:.0f} MiB"


def retention(w, t):
    return min(w * t * PAGES_PER_WRITE * PAGE, FILE)


print("Retention of old page versions while one snapshot is alive (W - committed writes per second):")
rows = [
    ("search query, 30 s ceiling", 30),
    ("stream by number, 120 s ceiling", 120),
]
for rate_mib in (32, 64, 128, 256):
    t = FILE / (rate_mib * MIB)
    rows.append((f"copy of 40 GB at {rate_mib} MiB/s ({t / 60:.1f} min)", t))
print(f"{'snapshot':48s}" + "".join(f"W={w:<6d}" for w in (1, 10, 100, 1000)).rjust(0))
for name, t in rows:
    print(f"{name:48s}" + "".join(f"{fmt(retention(w, t)):<9s}" for w in (1, 10, 100, 1000)))
print()

print("Serving by number: worst-case history (2,967,164 records, ~300 MB, ADR 0035 item 6) and snapshot lifetime")
for rate_mbit in (10, 100, 1000):
    secs = STREAM_BYTES * 8 / (rate_mbit * 1e6)
    print(f"  client {rate_mbit:>4d} Mbit/s: {secs:6.1f} s; fits in 120 s: {'yes' if secs <= 120 else 'no'}")
min_rate = STREAM_BYTES * 8 / 120 / 1e6
print(f"  minimum client rate for 120 s: {min_rate:.1f} Mbit/s")
print()

print("Rebuild of the order index from names for 10^8 records (Q-82). Sort key speed: keybench measurement,")
print("1 thread 1.96 M names/s, 4 threads 8.42 M/s, 8 threads 14.8 M/s (Apple M3 Pro; average key 24.2 bytes).")
N = 10**8
for threads, rate in ((1, 1.96e6), (4, 8.42e6), (8, 14.8e6)):
    print(f"  key construction only, {threads} threads: {N / rate:6.1f} s")
# I/O: we read names and numbers (estimate 60 bytes per data record: name ~30, number, date), we write the index (key 24-25 bytes +
# date 4 + number ~8 + page overhead ~ 1.3).
READ = N * 60
WRITE = int(N * (24.2 + 4 + 8) * 1.3)
SORT_PASSES = 2  # external sort: writing and reading runs; does not fit in memory - the "worst" case assumption
print(f"  data read ~{READ / 1e9:.1f} GB, index write ~{WRITE / 1e9:.1f} GB (assumption: 60 bytes of data/record, key+date+number, "
      f"overhead 1.3)")
for mbs in (200, 500, 2000):
    io = (READ + WRITE * (1 + SORT_PASSES)) / (mbs * 1e6)
    print(f"  I/O at {mbs:>4d} MB/s (external sort, {SORT_PASSES} extra passes): {io:6.0f} s")
print("  total for the order index only: ~20 s (fast disk, 8 threads) ... ~2.5 min (200 MB/s disk, 1 thread: 51 s keys + 101 s I/O);")
print("  word index and exact-search index -")
print("  separate, not measured (case folding and UAX #29 over ~2-4 words per name): add during implementation.")
print()

print("Page checksum while copying: crcbench measurement (crc32c, 1 thread) 8.4-12.2 GiB/s;")
print(f"  40 GB => {FILE / (8.4 * GIB):.0f} s on one thread, i.e. much less than the copy I/O (minutes); not a bottleneck.")
