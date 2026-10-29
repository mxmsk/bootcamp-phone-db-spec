# storage-format

## Purpose

This capability defines the layout of the database files: the directory contents, the header and state markers, the format version and its
compatibility in v1, pages and spilling of large keys, record encoding (number, date, name, number version), page checksums,
free space and the file size at 10^8 records. The WAL, indexes, snapshots and the integrity check are other capabilities of area 2;
the decisions are ADR 0040–0044.

## ADDED Requirements

### Requirement: Database directory contents and data file
A database SHALL be a directory with the files `data` (the only page file: all records and all index trees), `control`, `dwb`,
`LOCK` and the WAL directory `wal/`; the format of `control`, `dwb` and `wal/` is defined by `storage-durability`. The version generation counter,
network tokens and settings are not part of the database directory. There are no other mandatory files. Trace: CUJ13, CUJ15; X-2; item 1.6
of the original wording. Decisions: ADR 0040 (item 1), 0016 (item 5), 0051 (item 1), 0054.

#### Scenario: A freshly created database
- **WHEN** the operator creates an empty database with the create command (CUJ13)
- **THEN** the directory contains `data` with page 0 (META) and empty trees, `control`, `dwb`, `wal/` and `LOCK`
- **AND** the `db_id` in META is random and equals the `db_id` in `control`

#### Scenario: All data in one page file
- **WHEN** the server saves a record and updates the indexes
- **THEN** the pages of records and of all indexes are in `data`, not in separate per-index files

#### Scenario: The temporary retention file
- **WHEN** open snapshots need earlier page versions beyond the memory budget (`storage-snapshots`, "Snapshot retains earlier page versions")
- **THEN** a temporary file `retain` may exist: it is not part of the database, not included in a backup, and is deleted at start

#### Scenario: Tokens outside the directory
- **WHEN** the operator makes a backup (CUJ15)
- **THEN** the backup has no network tokens, server settings or version generation counter

### Requirement: Data file header
The `data` header SHALL be a single page 0 of type META — an ordinary page, changed by WAL entries and protected by
a checksum and the doublewrite buffer; there is no second copy. A file without the signature `89 4E 50 44 42 0D 0A 1A` SHALL
be rejected as "not a database"; unrecoverable corruption of page 0 is a start refusal. Trace: CUJ14. Decisions: ADR 0040 (items 2–3), 0014
(item 4), 0053 (items 1–2).

#### Scenario: A file not from this system
- **WHEN** the first bytes of `data` do not equal the signature
- **THEN** refusal to open as "not a database", without reading the other pages

#### Scenario: A torn write of page 0
- **WHEN** after a failure page 0 in `data` fails the checksum check and `dwb` has a valid copy of the same page
- **THEN** the page is restored from `dwb`, the database opens

#### Scenario: Page 0 is unrecoverable
- **WHEN** page 0 fails the check and there is no valid copy in `dwb`
- **THEN** the server does not start and tells the operator; `data` is not modified; restore is from a backup (CUJ15)

### Requirement: Header fields
META SHALL contain: the signature, `format_version`, `page_size`, `features_incompat`, `features_compat`, `db_id`, `page_count`,
`freelist_head`, `free_page_count`, tree descriptors (up to 32: `tree_id`, `kind`, `root_page`, `height`, `agg_len`,
`entry_count`, `key_format_label`) at the offsets of ADR 0040 item 3; numbers are little-endian, the rest of the page is zeros. There are no
shutdown or backup markers in META: they are in `control`. Trace: CUJ14, CUJ15. Decisions: ADR 0040 (items 3, 5).

#### Scenario: Layout by offsets
- **WHEN** page 0 of a created database is read
- **THEN** the signature is at offset 32, `format_version` at 40, `page_size` at 44, `db_id` at 64, `page_count` at 80, the tree
  descriptors from 96 at 32 bytes per descriptor

#### Scenario: Key format label
- **WHEN** the order tree descriptor is read at start
- **THEN** it contains `key_format_label`; on a mismatch with the program's label the derived trees are rebuilt before
  accepting operations (ADR 0045 item 8), records and number headers are not rebuilt

#### Scenario: Shutdown markers are not duplicated
- **WHEN** the contents of META are checked
- **THEN** it has no `clean`, `running`, `backup` markers: the kind of shutdown is determined only by `run_state` in `control`

### Requirement: Format version and opening earlier v1 databases
A v1 program SHALL open a database of any data format version from 1 to its current one by itself, without operator action and without a
conversion command, converting it to the current version at start before accepting the first operation by ordinary WAL entries; the conversion
survives a failure (an interrupted one is continued or repeated). A backup made before an upgrade opens the same way. Trace: CUJ11,
CUJ14, CUJ15. Decisions: ADR 0013 (items 2–4), 0040 (item 4).

#### Scenario: A program upgrade within v1
- **WHEN** a new v1 version of the program starts on a database with a lower `format_version`
- **THEN** the database opens without an operator command, is converted to the current version before accepting the first operation, data and result
  order do not change

#### Scenario: Failure during conversion
- **WHEN** the process crashes in the middle of a format conversion
- **THEN** at the next start the conversion is completed or repeated, no record is lost, and at no moment does the file
  become unreadable for either the earlier or the new program version

#### Scenario: A backup from before an upgrade
- **WHEN** the operator starts the new version on a backup made by the earlier version before the upgrade
- **THEN** the backup is opened and converted under the same rules; the `backup` marker is cleared under the rules of ADR 0054 items 5, 9

### Requirement: Refusal to open a database of a newer version
The program SHALL refuse to open a database whose `format_version` is greater than it knows or in which an unknown `features_incompat` bit is set,
with a clear message to the operator; an unknown `features_compat` bit SHALL be ignored. There is no rollback to
an earlier program version: the earlier program is not required to open a database after the new one. Trace: CUJ11, CUJ14. Decisions:
ADR 0013 (item 4), 0040 (item 4).

#### Scenario: A database newer than the program
- **WHEN** an old program starts on a database converted by the new program
- **THEN** the start is rejected with a message that the database was created by a newer version; the file is not modified

#### Scenario: An unknown compatible feature
- **WHEN** a bit unknown to the program is set in `features_compat`
- **THEN** the database opens, the bit is ignored

### Requirement: Shutdown and backup markers are not stored in the data file
The storage SHALL keep the shutdown kind marker (`clean`, `running`, `backup`) only in `control`; `data` neither stores nor
duplicates it; what `data` and `control` share is `db_id`. The rules for setting and clearing the marker (durably, before the first operation is accepted;
`backup` — set by the backup command, cleared only by a database started from the backup) — ADR 0054 items 4–6, 9; ADR 0028 item 10. Trace:
CUJ14, CUJ15. Decisions: ADR 0040 (item 5), 0054.

#### Scenario: A copy of the data file only
- **WHEN** the operator copies only `data` from the directory of a stopped database not with the backup command
- **THEN** the start kind is determined from `control` (ADR 0028 item 7); `data` has no markers that could pass for a "backup"

#### Scenario: A single source of truth
- **WHEN** `run_state` in `control` is `clean`, while `data` contains pages changed after the checkpoint
- **THEN** the start kind is still determined by `control` and the WAL under ADR 0054; `data` does not participate in the decision

### Requirement: Checks when opening the file
When opening `data` the following SHALL be checked: the signature, `format_version`, `features_incompat`, `page_size` = 16,384, a `db_id`
equal to the `db_id` in `control`, and a file length no less than `page_count` × 16,384 as of the last checkpoint; a violation is a refusal with a message to the operator. A tail of the
file beyond `page_count` is allowed. The other start checks are in ADR 0053. Trace: CUJ14. Decisions: ADR 0040 (item 6), 0014
(items 4–5), 0053 (item 1).

#### Scenario: A truncated file
- **WHEN** the length of `data` is less than `page_count` × 16,384
- **THEN** the server does not start and says the file is truncated

#### Scenario: File growth did not reach the disk
- **WHEN** the WAL contains page changes with numbers beyond the end of the file (the extension was not flushed before the failure)
- **THEN** during recovery these pages are applied as zero ones, the file is extended, the database opens

#### Scenario: A growth margin in the tail
- **WHEN** the file is longer than `page_count` × 16,384
- **THEN** the database opens; the tail is not read

#### Scenario: A foreign page size
- **WHEN** `page_size` in the header is not equal to 16,384
- **THEN** refusal to open, the database is not modified

#### Scenario: A data file of another database
- **WHEN** the `db_id` in META is not equal to the `db_id` in `control`
- **THEN** refusal to open with a message about files of different databases being mixed

### Requirement: Page size 16 KiB
A page SHALL be 16,384 bytes, constant in v1 and recorded in the header; page N lies in the file at offset
N × 16,384. The value is a default, confirmed by the owner (brief §4; ADR 0041 item 1). Trace: X-1; brief §4. Decisions: ADR 0041
(item 1), 0019 (item 2).

#### Scenario: Page position
- **WHEN** page 5 is read
- **THEN** it occupies bytes 81,920…98,303 of the file and its `page_no` is 5

### Requirement: Common page header and types
Every page SHALL begin with a 32-byte header: `checksum`, `page_type`, `flags`, `page_layout`, `page_no`,
`page_lsn`, `cell_count`, `cells_start`; types 0 (invalid, zeros), 1 META, 2 leaf, 3 inner, 4 overflow, 5
free list. A page with a type outside the set, of all zeros, or with a `page_no` not equal to its position is corruption. Trace: CUJ14.
Decisions: ADR 0041 (items 2–3).

#### Scenario: A page not in its place
- **WHEN** a page written to position 9 is read but its `page_no` is 7
- **THEN** the page is invalid: a storage error, data from it is not used

#### Scenario: A zero page
- **WHEN** a tree or list refers to a page all 16,384 bytes of which are zero
- **THEN** it is corruption: a storage error

### Requirement: Tree page layout
A tree page SHALL store after the 40-byte header an array of `u16` restart-point slots and cells from the end of the page
downward; a cell's key is given by a prefix shared with the previous cell (`prefix_len`) and its own part; every 16th cell must be a restart point
(`prefix_len` = 0), as well as a cell with a spill and the one following it. Trace: CUJ1, CUJ5, X-1. Decisions:
ADR 0041 (item 4).

#### Scenario: Key reconstruction
- **WHEN** a cell with `prefix_len` = 6 and its own part of 4 bytes is read
- **THEN** its key is the first 6 bytes of the previous cell's key and the 4 own bytes

#### Scenario: Restart points
- **WHEN** a leaf has 40 cells without spills
- **THEN** the restart points are at least cells 0, 16 and 32, and the slots point to them in ascending key order

### Requirement: Key and value: limits, spill to overflow
The storage SHALL store a key up to 65,535 bytes and a value up to 2^32 − 1 bytes whole, without truncation: a cell payload of no more than 2,048 bytes lies in the page, otherwise the page holds the first
256 bytes, `total_len` and the number of the first page of a chain of type 4. A key longer than the cap SHALL lead to a refusal of a single operation without changing data and without a "storage error" (default, D-36).
Trace: CUJ1, CUJ3, CUJ5, CUJ7; ADR 0041 (items 5–6), 0045 (item 6).

#### Scenario: A name of 200 U+FDFA characters
- **WHEN** a record with a name of 200 U+FDFA characters is saved (name 800 bytes, sort key 8,513 bytes)
- **THEN** the record and all its keys are stored whole, the order key goes through overflow, the operation does not end with a "does not fit"
  error, keys are not truncated

#### Scenario: Ordinary keys lie in the page
- **WHEN** a record with a name up to 800 bytes and a comparison key up to 1,000 bytes is saved
- **THEN** the cells of the record and of the exact name lie in the page whole, without overflow pages

#### Scenario: A key above the cap
- **WHEN** a key of 65,536 bytes is fed to the storage layer (unreachable for valid names: the additive bound is ≈ 20,400 bytes)
- **THEN** the operation is rejected entirely, data and version are unchanged, the key is not truncated, the "storage error" state did not occur; the kind of refusal for the client belongs to area 3, an internal error

#### Scenario: Order of spilled keys
- **WHEN** two keys longer than 2,048 bytes coincide in the first 256 bytes and differ in the tail
- **THEN** their order is determined by the full keys, as for keys in the page

### Requirement: Page capacity and check in every release
Every tree page SHALL fit at least 7 cells of the largest size without a spill, so that a page split is always
possible; every release SHALL check capacity on names of "character × 200" for each character, on alternating pairs
of "long" characters and on random mixes. The storage does not need a proven upper bound on the sort key. Trace: CUJ1,
CUJ5, X-1. Decisions: ADR 0041 (items 7–8), 0022 (item 5), 0045 (item 6); Q-103.

#### Scenario: Capacity
- **WHEN** cells with a payload of exactly 2,048 bytes are inserted into a page
- **THEN** 7 cells fit, and a page split always gives two pages with at least one cell each

#### Scenario: Long key check in a release
- **WHEN** the release check is run
- **THEN** insertion, search, deletion and order are checked on keys from 0 to at least 16,000 bytes long, including names of
  alternating pairs of "long" characters (on the ADR 0041 sample the maximum is 7,297 bytes for a U+FDFA/U+FDFB mix)

### Requirement: Record key is number and date
The record key in the record tree R SHALL be 11 bytes: the number in 8 bytes (the digits of the canonical number by nibbles, digit d being
the value d + 1, missing nibbles 0) and the date in 3 bytes big-endian: 2,967,164 − d, where d is the number of days since 1876-03-07
(values 1…2,967,164; ascending key means descending date). The encoding is one-to-one; the number preserves the order of
canonical strings. Trace: CUJ1, CUJ2, CUJ4; item 1.3 of the original wording. Decisions: ADR 0042 (items 1–3), 0045 (item 4),
0023 (item 4), 0024.

#### Scenario: Number encoding
- **WHEN** "+79991234567" (11 digits) is encoded
- **THEN** the number takes 8 bytes `8A AA 23 45 67 80 00 00`

#### Scenario: Extreme numbers
- **WHEN** "+1" and "+123456789012345" are encoded
- **THEN** the result is `20 00 00 00 00 00 00 00` and `23 45 67 89 A1 23 45 60`; a short number comes before its continuation

#### Scenario: Date encoding
- **WHEN** 1876-03-07 (d = 0) and 9999-12-31 (d = 2,967,163) are encoded
- **THEN** the date keys are 2,967,164 (`2D 46 7C`) and 1 (`00 00 01`)

#### Scenario: Reading in ascending key order gives dates in descending order
- **WHEN** a number has records on the dates 2001-01-01, 2010-06-15 and 1999-12-31 and they are read in ascending key order
- **THEN** the order is 2010-06-15, 2001-01-01, 1999-12-31 (CUJ4: date DESC), without a reverse traversal

### Requirement: Record value is the name, bytewise
The value of a record cell SHALL be the name as it is saved (NFC, trimmed, UTF-8), from 1 to 800 bytes, with no separate length
field other than the cell's `value_len`; the storage returns it bytewise, and "the same name" is bytewise equality. Trace: CUJ1, CUJ3.
Decisions: ADR 0042 (item 4), 0003, 0021.

#### Scenario: The name is returned as saved
- **WHEN** the name "Ann" is saved and then read
- **THEN** "Ann" is returned bytewise, not "ANN" or another spelling

#### Scenario: The cap
- **WHEN** a valid name of 200 characters of 4 bytes each is saved
- **THEN** the value takes 800 bytes and is stored without truncation

### Requirement: Number header and number version
For every number with at least one record the storage SHALL keep a number header (key — the number and `00 00 00`, value —
the sequence part of the version S, a u64 varint, S ≥ 1) immediately before its records; a record change, an S change, the creation or deletion
of the header and the changes of the derived trees SHALL be one indivisible WAL entry. Trace: CUJ1, CUJ4, CUJ6, CUJ9, CUJ12, CUJ14; X-5. Decisions: ADR 0042 (items 5–8), 0027,
0028 (item 6), 0029, 0051 (item 7).

#### Scenario: Adding the first record of a number
- **WHEN** the first record of a number is added
- **THEN** in one WAL entry a number header with its S and a record cell are created; there is no partial result either after a failure
  or for a reader

#### Scenario: Deleting the last record
- **WHEN** the last record of a number is deleted
- **THEN** in the same WAL entry the header is deleted; no data remains about the number, search returns the initial version

#### Scenario: Reading number history
- **WHEN** a search by number is run
- **THEN** the header and all records of the number are read consecutively from one consistent state; the version is from the header

#### Scenario: What is not stored in `data`
- **WHEN** the contents of number headers and META are checked
- **THEN** a number without records has no header, the generation (G, E) and the counter `next_S` are not stored in `data` (they are in `control` and the WAL)
- **AND** S in the header is a varint, at most 5 bytes at S < 2^35; the version (G, E, S) of 32 bytes exists only in the API response

#### Scenario: Generation change without rewriting headers
- **WHEN** the server starts after a crash restart and gets a new generation (G, E)
- **THEN** the observable versions of all numbers changed, while the headers in `data` were not rewritten

#### Scenario: The header invariant
- **WHEN** the consistency check walks the tree R
- **THEN** every record has the header of its number, every header has at least one record, the S of a header is no greater than `next_S`

### Requirement: Page checksum
Every page of the file SHALL carry a CRC-32C (polynomial 0x1EDC6F41, initial value and final XOR 0xFFFFFFFF, reflected)
over all 16,384 bytes with the `checksum` field zero, written as a little-endian u32 into bytes 0…3; the sum is computed before
the page is written. Trace: CUJ14; item 1.6 of the original wording. Decisions: ADR 0043 (items 1–2), 0014 (item 5).

#### Scenario: Reference vectors
- **WHEN** the CRC-32C is computed of "123456789", of 16,384 zero bytes, and of a page where byte i equals i mod 251 and bytes 0…3
  are zero
- **THEN** the results are 0xE3069283, 0x94640B85 and 0x1D167853

#### Scenario: All page types
- **WHEN** a page of any type is written, including META, overflow and free list
- **THEN** its `checksum` is filled by this rule

### Requirement: Checksum check on read and storage error
Every page SHALL be checked (the sum, a type from the set, `page_no` equal to the position) on every read from the file into memory;
a mismatch SHALL end the read that needs the page with a storage error; data from an invalid page is not
returned or used; the operator is told the page number and its type. Detection of the loss of an acknowledged write without
page corruption is not promised. Trace: CUJ14, CUJ16. Decisions: ADR 0043 (items 3–4, 8), 0014 (items 3, 5–7).

#### Scenario: One-bit corruption
- **WHEN** one bit in a tree leaf is changed and the page is read from the file
- **THEN** the read that needs the page ends with a storage error; searches that do not touch the page work

#### Scenario: A page from the cache
- **WHEN** a page is already checked and lies in the cache
- **THEN** recomputing the sum on cache access is not required

#### Scenario: The error is not silent
- **WHEN** a page checksum did not match
- **THEN** changes are rejected as in the "storage error" state (CUJ16), the operator is given a message; restore is from a
  backup (CUJ15)

### Requirement: CRC crate and fallback
The implementation SHALL compute the sum with the `crc32c` 0.6.8 crate (license Apache-2.0/MIT); the format fixes the algorithm, not the crate, and
replacing it with `crc` 3.4.0 (`CRC_32_ISCSI`) does not change the format. A release SHALL pass the cargo deny check and the reference vectors.
Trace: X-4. Decisions: ADR 0043 (items 5–7), 0017 (item 3).

#### Scenario: Implementations agree
- **WHEN** the sums are computed with `crc32c` and `crc` with `CRC_32_ISCSI` on the same pages
- **THEN** the values match

#### Scenario: A build without license checks
- **WHEN** cargo deny is not passed for `crc32c`
- **THEN** the release is not published; replacing with `crc` is allowed without converting the format

### Requirement: Free page list and reuse
Free pages SHALL be tracked by a chain of type 5 pages (an element is a page number and `freed_at`, the LSN of the WAL entry that freed it) with the head and count in the header; a freed
page SHALL NOT be handed out again while there is an open snapshot with a published LSN less than `freed_at`; a page is never in a tree and in the list at once.
Trace: CUJ14, CUJ15; X-1. Decisions: ADR 0044 (items 1–2), 0035 (item 6), 0055 (item 3).

#### Scenario: List elements
- **WHEN** a type 5 page is full
- **THEN** it carries 1,362 elements of 12 bytes (`page_no` u32, `freed_at` u64) after `next` and `count`

#### Scenario: Which snapshots retain a page
- **WHEN** a number history read, a backup cut or a results page read is open
- **THEN** each of them retains the pages freed after its published LSN

#### Scenario: A page needed by a snapshot
- **WHEN** an operation with LSN F frees a page, while an open snapshot with a published LSN less than F is still active
- **THEN** the page is not handed out for new data until the snapshot is closed

#### Scenario: A page after a durable change
- **WHEN** there are no snapshots with a published LSN less than `freed_at`
- **THEN** the page is handed out from the list before the file grows

#### Scenario: After a restart
- **WHEN** the server is restarted, and the free list still has pages with the `freed_at` of an earlier run
- **THEN** there are no open snapshots (snapshots live only in memory) and these pages are handed out immediately

#### Scenario: Freeing overflow pages
- **WHEN** a cell with a spilled key is deleted
- **THEN** all pages of its chain are returned to the free list

### Requirement: File growth and out of space
The file SHALL grow in chunks of 4,096 pages (64 MiB), only if there is no suitable free page; the inability to extend
the file or write a page SHALL give the "read-only" or "storage error" state under ADR 0053 items 5–6 with no partial
result of the operation. Trace: CUJ16; X-1. Decisions: ADR 0044 (items 3–4), 0014
(items 6–7), 0029 (item 1), 0053.

#### Scenario: Disk full
- **WHEN** the file cannot be extended when adding a record
- **THEN** the record is not created and the number version is not changed; the user gets a refusal per CUJ16 ("read-only" or "storage
  error" by `errno`, ADR 0053), search works

#### Scenario: An extended area
- **WHEN** a page from an extended, not yet written area of the file is read
- **THEN** it consists of zeros (type 0) and is invalid: it cannot be referenced until it is written with a valid sum

#### Scenario: A growth chunk
- **WHEN** there are no free pages and a new one is needed
- **THEN** the file length grows by 4,096 pages, `page_count` by the number handed out

### Requirement: File is not shrunk automatically
The storage SHALL reuse freed space and not shrink the file automatically; there is no "shrink file" operation in v1;
the file size is a high-water mark, not the volume of data. Trace: X-1; X-3. Decisions: ADR 0044 (item 5), 0018.

#### Scenario: Deleting a large number of records
- **WHEN** millions of records are deleted
- **THEN** the length of `data` does not decrease, the freed pages are used by subsequent changes

### Requirement: File size at 10^8 records
The `data` file (without `wal/`, `dwb` and backups) of a database of 10^8 records of the reference profile P0 SHALL be no more than 64 × 10^9 bytes.
Profile P0: an average name of 22 characters (42.1 bytes), 2.5 words per name, 1.25 records per number, inserts in random order,
the index layout of ADR 0045.
The value is a default (D-21), confirmed by the owner; it replaces "≤ 40 GB" from brief §4 (ADR 0044 item 6). Trace: X-1; brief §4. Decisions:
ADR 0044 (items 6–8), 0017, 0019 (item 1).

#### Scenario: A database of the reference profile
- **WHEN** the test load-generation tool (Q-53) creates 10^8 records of profile P0 by inserts in random order
- **THEN** the length of `data` is no more than 64 × 10^9 bytes (a calculated expectation of 46–53 GB with prefix compression of 25–45%, 61 GB without
  compression; ADR 0044 item 8)

#### Scenario: Names longer than the profile
- **WHEN** a database with a different average name length is checked
- **THEN** the size is estimated by the table of ADR 0044 item 8; the number 64 GB is not applied to such a database as a target
