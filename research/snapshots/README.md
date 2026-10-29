# research/snapshots

Calculations and measurements of the "snapshots" subtask (area 2; ADR 0055-0059, capability `storage-snapshots`).

- `retention.py` -> `out-retention.txt`: retention of earlier page versions under snapshots, backup duration, and an estimate of the
  order index rebuild. The assumptions (16 KiB page, 8 pages per record, 40 GB) are set at the start of the script; this is
  arithmetic, not a measurement of the product.
- `keybench/`: speed measurement of `write_sort_key_to` (`icu_collator` =2.4.0, pinned versions as in `../ordering/icu-pin`).
  Run: `cargo build --release --locked && ./target/release/keybench <threads> <names per thread>`; result of 2026-10-27, Apple M3 Pro:
  1 thread 1.96 M names/s, 4 threads 8.42 M/s, 8 threads 14.8 M/s, average key 24.2 bytes (the names are synthetic: two words
  of 4-10 letters, 3/4 Cyrillic).
- `crcbench/`: measurement of crc32c (`crc32c` =0.6.8) over 16 KiB pages, 1 thread: 8.4-12.2 GiB/s (2026-10-27). Order of magnitude
  only: the choice of checksum crate is not made here.
