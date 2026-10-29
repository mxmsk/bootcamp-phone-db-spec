#!/usr/bin/env python3
"""Cold read of 16 KiB pages without the OS cache (F_NOCACHE on macOS): random QD1 and sequential.
Usage: python3 read-bench.py [directory] [file size in MiB]."""
import os, sys, time, fcntl, random, json, statistics
d = sys.argv[1] if len(sys.argv) > 1 else os.environ.get("TMPDIR", ".")
mib = int(sys.argv[2]) if len(sys.argv) > 2 else 2048
P = 16384
path = os.path.join(d, "read-bench.tmp")
fd = os.open(path, os.O_RDWR | os.O_CREAT | os.O_TRUNC, 0o600)
if hasattr(fcntl, 'F_NOCACHE'): fcntl.fcntl(fd, fcntl.F_NOCACHE, 1)  # before writing: the file does not enter the cache
chunk = os.urandom(1 << 20)
for _ in range(mib): os.write(fd, chunk)
os.fsync(fd)
if hasattr(fcntl, "F_NOCACHE"): fcntl.fcntl(fd, fcntl.F_NOCACHE, 1)
n = (mib << 20) // P
def pct(xs, p): xs = sorted(xs); return xs[min(len(xs)-1, int(len(xs)*p))]
lat = []
for _ in range(5000):
    off = random.randrange(n) * P
    t = time.perf_counter(); os.pread(fd, P, off); lat.append((time.perf_counter() - t) * 1000)
print(json.dumps(dict(random_16k_QD1_ms=dict(n=5000, p50=round(statistics.median(lat),3), p99=round(pct(lat,.99),3), max=round(max(lat),3)))))
t = time.perf_counter(); tot = 0; off = 0
while off < (mib << 20):
    tot += len(os.pread(fd, 4 << 20, off)); off += 4 << 20
dt = time.perf_counter() - t
print(json.dumps(dict(sequential_MBps=round(tot / dt / 1e6), bytes=tot)))
os.close(fd); os.unlink(path)
