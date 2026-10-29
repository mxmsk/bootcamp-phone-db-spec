#!/usr/bin/env python3
"""Measure the latency of a durable WAL write: append N bytes + fsync / F_FULLFSYNC (macOS) / fdatasync (Linux).
Usage: python3 fsync-bench.py [directory] [iterations]. The directory must be on the volume under test."""
import os, sys, time, fcntl, platform, statistics, json

d = sys.argv[1] if len(sys.argv) > 1 else os.environ.get("TMPDIR", ".")
n = int(sys.argv[2]) if len(sys.argv) > 2 else 1000
path = os.path.join(d, "wal-bench.tmp")

def pct(xs, p):
    xs = sorted(xs)
    return xs[min(len(xs) - 1, int(len(xs) * p))]

def run(name, sync, size):
    fd = os.open(path, os.O_RDWR | os.O_CREAT | os.O_TRUNC, 0o600)
    # a pre-filled 64 MiB file of zeros (the WAL does not grow on every commit)
    os.write(fd, b"\0" * (64 << 20))
    os.fsync(fd)
    os.lseek(fd, 0, os.SEEK_SET)
    buf = os.urandom(size)
    lat = []
    for _ in range(n):
        t = time.perf_counter()
        os.write(fd, buf)
        sync(fd)
        lat.append((time.perf_counter() - t) * 1000)
    os.close(fd)
    os.unlink(path)
    r = dict(mode=name, bytes=size, n=n, p50=round(statistics.median(lat), 3), p90=round(pct(lat, .9), 3),
             p99=round(pct(lat, .99), 3), max=round(max(lat), 3))
    print(json.dumps(r))

print(json.dumps(dict(platform=platform.platform(), machine=platform.machine(), dir=d)))
modes = [("fsync", os.fsync)]
if hasattr(fcntl, "F_FULLFSYNC"):
    modes.append(("F_FULLFSYNC", lambda fd: fcntl.fcntl(fd, fcntl.F_FULLFSYNC)))
if hasattr(os, "fdatasync"):
    modes.append(("fdatasync", os.fdatasync))
for size in (4096, 65536):
    for name, sync in modes:
        run(name, sync, size)
