# 0101. The rustix (or libc) and socket2 crates; what was removed because std lacks them
- Status: Accepted (OD-7)
- Label: [OURS] (owner decision OD-7 item 8 — [USER])
- Date: 2026-10-28
- CUJ: CUJ13, CUJ15, CUJ16, X-2

## Context
std 1.96 has no `statvfs`, no I/O priority and no `posix_fadvise` / `F_NOCACHE`; `tokio::net::TcpListener::bind` always sets `SO_REUSEADDR` (F-124), and on macOS/BSD binding to 127.0.0.1:P can succeed while a foreign *:P exists (not verified), in which case code 7 is not given (ADR 0066/0067). `umask 0177` "before threads are created" is incompatible with an already running tokio (`docs/reviews/rust-expert.md`, item 8).

## Decision
1. The dependency `rustix` (fs) — `statvfs` (F-126): free space for the backup target (ADR 0057 item 6) and for the retention ceiling (ADR 0056 item 5). Replacing it with `libc` is allowed; the choice of one of the two is made at implementation, one of them is in the ADR.
2. The dependency `socket2` (F-125): the listener is created as a `Socket`, without `SO_REUSEADDR` and `SO_REUSEPORT`, `bind`, `listen`, then handed to tokio; an occupied port is a start refusal (code 7, ADR 0066).
3. The control channel socket (ADR 0080 item 4): created in a directory with permissions 0700, after `bind` — `chmod 0600`; `umask` is not changed.
4. "Low backup priority" and "without evicting the OS cache working set" are removed: the speed limit (64 MiB/s) and our own page cache are enough.
5. Dependency versions and MSRV are pinned in `Cargo.lock` when introduced (as for `icu_*`, ADR 0039).

## Alternatives considered
- Calls through `std::process` (`df`): fragile, rejected.
- `libc` everywhere: acceptable, but the unsafe code is wider.
- Wait for `std` equivalents: not in stable.

## Consequences
- "Changed by OD-7" in ADR 0057 item 11, 0070 item 3, 0080 item 4. The `storage-snapshots` requirement "Latency growth during copying" is changed.

## Sources
- OD-7 item 8; `docs/reviews/rust-expert.md` item 8; F-124, F-125, F-126.
