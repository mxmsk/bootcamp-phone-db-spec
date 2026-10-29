# 0043. Page checksums: CRC-32C and the `crc32c` crate
- Status: Accepted
- Label: [OURS]
- Date: 2026-10-27
- CUJ: CUJ14, CUJ16; X-4; item 1.6 of the original wording ("store reliably")

## Context
ADR 0014 item 5 (owner decision, OD-4, item 3): on start it is checked that the database opens consistent; corruption of the other
data is detected on read (checksums) and leads to the "storage error" state; detection of a "lying" disk
is not promised. ADR 0017 item 3 explicitly names CRC among the allowed libraries, each with an ADR and a cargo deny check.
We need to choose the algorithm, the scope of the sum and the crate; checks are as in Fact Grounding.

## Decision
1. [OURS] The algorithm is CRC-32C (Castagnoli): polynomial `0x1EDC6F41` (in reflected form `0x82F63B78`), initial value
   and final XOR `0xFFFFFFFF`, reflected input and output (the parameters of "CRC-32/ISCSI", `check("123456789") = 0xE3069283`).
   The algorithm, not the crate, is part of the v1 format: the crate can be replaced without a format upgrade (item 6).
2. [OURS] The scope is the whole 16 384-byte page, in which the `checksum` field (bytes 0…3) is treated as zero during
   computation; the value is written as a u32 little-endian into bytes 0…3. Every page of the file of any type has a checksum,
   including META, overflow pages and free list pages (ADR 0041). The page number is part of its
   content (`page_no`), so a page written to the wrong place does not pass the check.
3. [OURS] When it is checked. The sum is computed immediately before the page is written to the file. Every page is checked (the sum,
   `page_type` from the set, `page_no` = position) on every read from the file into memory; a hit in the page
   cache does not require rechecking. A page of all zeros is invalid (type 0).
4. [OURS] A mismatch is corruption: a read that needed this page ends with a storage error (CUJ16,
   ADR 0014 item 6): data from an invalid page is never returned or used; the operator is told the page
   number, its type and the tree (if known). A page with a wrong sum for which `dwb` holds a valid copy of the same page is restored on start
   (ADR 0051 item 5 (b)); an unrepairable META page or tree root is a start refusal (ADR 0053 item 1 (e), item 2). There is no repair of corruption "in place": restore is from a backup (CUJ15).
5. [OURS] The crate is `crc32c` 0.6.8 (license `Apache-2.0/MIT`, one build-time dependency, `rustc_version`, and no
   dependencies at run time; F-55): hardware CRC-32C instructions on x86-64 (SSE 4.2) and aarch64, a software fallback
   elsewhere. Measurement (F-58): 16 KiB in ≈ 0.7 µs, ≈ 21–23 GiB/s on arm64 macOS; a cross-check against the `crc` crate
   (`CRC_32_ISCSI`, Table<16>, ≈ 4.6 GiB/s, ≈ 3.3 µs per page) gives the same values. Checking all pages of a 40 GB
   file is on the order of 2 seconds of computation with `crc32c` against about 8 seconds with `crc` (arithmetic from the measurement; disk
   reads are not counted).
6. [OURS] The fallback crate is `crc` 3.4.0 with `CRC_32_ISCSI` (license `MIT OR Apache-2.0`, pure Rust, 4.6 GiB/s, F-57): if
   `crc32c` stops being maintained (the last release 2024-06-09, F-55) or fails cargo deny, the replacement does not change the
   format. Test vectors of the format: `CRC-32C("123456789") = 0xE3069283`; 16 384 zero bytes → `0x94640B85`; a page
   where byte i equals `i mod 251` and bytes 0…3 are zero → `0x1D167853` (verified by two implementations, `crc32c` and an
   independent one in Python, F-58). A release does not pass if the check of even one vector does not match.
7. [OURS] The cargo deny check is a build requirement (ADR 0017 item 3): the licenses `Apache-2.0`/`MIT` are allowed; known
   vulnerabilities per RustSec for `crc32c` were not checked (`cargo-deny` is not installed in this environment; F-55, "not verified").
8. [OURS] What is not promised: detection of a "lying" disk, loss of a whole acknowledged record without page corruption, and
   corruption in which the sum coincidentally matched (the probability for random page corruption is 2^-32 ≈ 2.3 × 10^-10 per corruption;
   arithmetic). The details of the guarantee boundary are ADR 0014, items 3, 5.

## Alternatives considered
- CRC-32 IEEE (`crc32fast` 1.5.3: "Fast, SIMD-accelerated CRC32 (IEEE) checksum computation", F-56): also fast
  (≈ 18.7 GiB/s on 16 KiB by measurement versus 22.7 for `crc32c`) and actively maintained (release 2026-10-20), but
  it computes a different polynomial. Both are fine for detecting page corruption; CRC-32C was chosen for its higher speed on the
  measured machine and because `crc32c` implements exactly it with hardware support on x86-64 and aarch64. Rejected
  without a significant argument "against" IEEE; the choice can be revisited before the v1 release, and after the release it is part of the format.
- A 64-bit hash (xxHash and others) or a cryptographic one (SHA-256): ADR 0017 item 3 names CRC; the goal is to detect media corruption,
  not an attacker; 4 bytes per 16 384-byte page versus 8–32. For 2.5 million pages the probability of missing random
  corruption of one page, 2^-32, is acceptable. Rejected.
- A checksum per record (cell): localizes corruption more precisely, but +4 bytes on ≈ 10^9 cells ≈ 4 GB and a check on every
  cell read. Rejected: the unit of reading and writing is the page.
- No checksums, structural checks only: violates ADR 0014 item 5. Rejected.
- A sum seeded by the page number instead of a `page_no` field: functionally the same, but `page_no` in the page is visible when
  debugging and is checked separately. Rejected.

## Consequences
- A storage error on a page read: a search that needs this page answers with a storage error; the others
  keep working (ADR 0014 item 7).
- A full file check (ADR 0053 item 1 does not require it on start): sequential reading and sum verification; the time is determined by the disk, not by the
  processor (≈ 2 s of computation per 40 GB).
- The checksums of WAL entries (CRC-32C of the whole entry, ADR 0051 item 2) are part of the WAL format; they use the same algorithm and crate and are not
  defined here. The check order on read is ADR 0053 item 4 (the same rule: sum, number, type).
- A page with a wrong sum after a torn write on failure is a normal case; it is restored by `dwb` (ADR 0051 item 5 (b)).

## Sources
- ADR 0014 items 3, 5, 6, 7; ADR 0017 item 3; `docs/source/brief.md`, section 3 ("Implementation"), section 2 (CUJ14, CUJ16).
- F-55…F-59 (`facts.md`): the crates.io pages and the contents of the crates `crc32c` 0.6.8, `crc32fast` 1.5.3,
  `crc` 3.4.0, the measurement and vectors (`research/file-format/crc-bench/`, output `crc-bench-output-2026-10-27.txt`).
