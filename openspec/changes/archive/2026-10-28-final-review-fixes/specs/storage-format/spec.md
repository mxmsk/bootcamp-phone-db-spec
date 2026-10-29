## MODIFIED Requirements

### Requirement: Database directory contents and data file
A database SHALL be a directory with the files `data` (the single page file: all records and all index trees), `control`, `dwb`,
`LOCK` and the WAL directory `wal/`; the format of `control`, `dwb` and `wal/` is defined by `storage-durability` ("Layout of control, dwb and WAL"). Only a backup has one more file, the backup descriptor `backup.desc` (`storage-snapshots`); a working database does not have it. The version generation counter,
network tokens and settings are not part of the database directory. There are no other mandatory files. Trace: CUJ13, CUJ15; X-2; item 1.6
of the original wording. Decisions: ADR 0040 (item 1), 0016 (item 5), 0051 (item 1), 0054.

#### Scenario: Freshly created database
- **WHEN** the operator creates an empty database with the creation command (CUJ13)
- **THEN** the directory contains `data` with page 0 (META) and empty trees, `control`, `dwb`, `wal/` and `LOCK`
- **AND** `db_id` in META is random and equal to `db_id` in `control`

#### Scenario: All data in one page file
- **WHEN** the server saves a record and updates the indexes
- **THEN** the pages of records and of all indexes are in `data`, not in separate per-index files

#### Scenario: Retention temporary file
- **WHEN** open snapshots need earlier page versions beyond the memory budget (`storage-snapshots`, "Snapshot retains earlier page versions")
- **THEN** a temporary file `retain` may exist: it does not form the database, is not included in a backup, and is deleted at start

#### Scenario: Tokens outside the directory
- **WHEN** the operator makes a backup (CUJ15)
- **THEN** the backup contains no network tokens, server settings or version generation counter

### Requirement: Data file header
The `data` header SHALL be a single page 0 of type META: an ordinary page, changed by WAL entries and protected by the
checksum and the doublewrite buffer; there is no second copy. A file without the signature `89 4E 50 44 42 0D 0A 1A` SHALL
be rejected as "not a database"; unrecoverable corruption of page 0 is a start refusal. Trace: CUJ14. Decisions: ADR 0040 (items 2–3), 0014
(item 4), 0053 (items 1–2).

#### Scenario: A file not from this system
- **WHEN** bytes 32…39 of `data` (the `signature` field) are not equal to the signature
- **THEN** refusal to open as "not a database", without reading other pages

#### Scenario: Torn write of page 0
- **WHEN** after a crash page 0 in `data` fails the checksum check, and `dwb` has a valid copy of the same page
- **THEN** the page is restored from `dwb`, the database opens

#### Scenario: Page 0 is unrecoverable
- **WHEN** page 0 fails the check and there is no valid copy in `dwb`
- **THEN** the server does not start and reports to the operator; `data` is not changed; recovery is from a backup (CUJ15)

### Requirement: Checks when opening the file
When opening `data`, these SHALL be checked: the signature, `format_version`, `features_incompat`, `page_size` = 16,384, `db_id`
equal to `db_id` in `control`, and a file length of at least `page_count` × 16,384 as of the last checkpoint; a violation is a refusal with a message to the operator. A file tail
beyond `page_count` is allowed. Other start checks are in ADR 0053. Trace: CUJ14. Decisions: ADR 0040 (item 6), 0014
(items 4–5), 0053 (item 1).

#### Scenario: Truncated file
- **WHEN** the length of `data` is less than `page_count` × 16,384
- **THEN** the server does not start and reports that the file is truncated

#### Scenario: File growth did not reach the disk
- **WHEN** the WAL contains page changes with numbers beyond the end of the file (the extension was not flushed before the crash) or inside the file but zeros on disk
- **THEN** during recovery these pages are applied as zero pages (if the entry initializes the page in full), the file is extended, the database opens

#### Scenario: Growth reserve in the tail
- **WHEN** the file is longer than `page_count` × 16,384
- **THEN** the database opens; the tail is not read

#### Scenario: Foreign page size
- **WHEN** `page_size` in the header is not equal to 16,384
- **THEN** refusal to open, the database is not changed

#### Scenario: Data file of another database
- **WHEN** `db_id` in META is not equal to `db_id` in `control`
- **THEN** refusal to open with a message about mixed files of different databases

### Requirement: Common page header and types
Every page SHALL begin with a 32-byte header: `checksum`, `page_type`, `flags`, `page_layout`, `page_no`,
`page_lsn`, `cell_count`, `cells_start`; types 0 (invalid, zeros), 1 META, 2 leaf, 3 internal, 4 overflow, 5
free list. A page with a type outside the set, all zeros, or with `page_no` not equal to its position is corruption; the exception is an all-zero page that the first applied WAL entry initializes in full (the first change after allocation): during recovery it is accepted as empty. Trace: CUJ14.
Decisions: ADR 0041 (items 2–3).

#### Scenario: Page not in its place
- **WHEN** a page written at position 9 is read, but its `page_no` is 7
- **THEN** the page is invalid: storage error, data from it is not used

#### Scenario: Zero page
- **WHEN** a tree or list references a page whose all 16,384 bytes are zero, and there is no WAL entry initializing it in full
- **THEN** this is corruption: storage error

#### Scenario: Zero page inside the file after growth
- **WHEN** after a crash a page within the file length consists of zeros (growth happened, the page write did not reach the disk), and the WAL contains an entry initializing it in full
- **THEN** during recovery it is accepted as empty and overwritten; the start is not rejected

### Requirement: Number header and number version
For every number with at least one record, the storage SHALL keep a number header (key — the number and `00 00 00`, value —
the ordinal part of the version S, u64 varint, S ≥ 1) immediately before its records; a record change, an S change, header creation or deletion
and changes to derived trees SHALL be one indivisible WAL entry. Trace: CUJ1, CUJ4, CUJ6, CUJ9, CUJ12, CUJ14; X-5. Decisions: ADR 0042 (items 5–8), 0027,
0028 (item 6), 0029, 0051 (item 7).

#### Scenario: Adding a number's first record
- **WHEN** a number's first record is added
- **THEN** in one WAL entry the number header with its S and the record cell are created; there is no partial result either after a crash
  or for a reader

#### Scenario: Deleting the last record
- **WHEN** a number's last record is deleted
- **THEN** in the same WAL entry the header is deleted; no data remains about the number, search returns the initial version

#### Scenario: Reading a number's history
- **WHEN** a number search is performed
- **THEN** the header and all the number's records are read consecutively from one consistent state; the version comes from the header

#### Scenario: What is not stored in `data`
- **WHEN** the contents of number headers and META are checked
- **THEN** a number without records has no header, the generation (G, E) and the `next_S` counter are not stored in `data` (they are in `control` and the WAL)
- **AND** S in the header is a varint, at most 5 bytes for S < 2^35; the version (G, E, S) of 32 bytes exists only in an API response

#### Scenario: Generation change without rewriting headers
- **WHEN** the server starts after a crash restart and receives a new generation (G, E)
- **THEN** the observable versions of all numbers changed, while the headers in `data` were not rewritten

#### Scenario: Header invariant
- **WHEN** an integrity check walks tree R
- **THEN** every record has the header of its number, every header has at least one record, the header's S is less than `next_S`

### Requirement: File growth and out of space
The file SHALL grow in chunks of 4,096 pages (64 MiB), only if there is no suitable free page; being unable to extend the
file or write a page (including at a checkpoint: extension does not reserve blocks, `ENOSPC` may show up later, ADR 0095) SHALL give the "read-only" or "storage error" state per ADR 0053 items 5–6 without a partial
operation result. Trace: CUJ16; X-1. Decisions: ADR 0044 (items 3–4), 0014
(items 6–7), 0029 (item 1), 0053.

#### Scenario: Disk full
- **WHEN** when adding a record the file cannot be extended
- **THEN** the record is not created and the number version is unchanged; the user gets a refusal per CUJ16 ("read-only" or "storage
  error" by `errno`, ADR 0053), search works

#### Scenario: Disk full at a checkpoint
- **GIVEN** the file is extended but the volume space is exhausted
- **WHEN** the checkpoint could not write pages (`ENOSPC`) after already acknowledged operations
- **THEN** the acknowledged data is preserved in the WAL, the state is "read-only", search works; new operations are rejected (`storage-durability`, "Checkpoints bound the WAL")

#### Scenario: Extended region
- **WHEN** a page from the extended, not yet written region of the file is read
- **THEN** it consists of zeros (type 0) and is invalid: it cannot be referenced until it is written with a valid checksum

#### Scenario: Growth chunk
- **WHEN** there are no free pages and a new one is needed
- **THEN** the file length grows by 4,096 pages, `page_count` by the number allocated
