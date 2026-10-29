# 0069. Access crates: TLS, hash and token comparison
- Status: Accepted
- Label: [OURS]
- Date: 2026-10-27
- CUJ: CUJ7, CUJ8, CUJ10; item 1.2 of the original wording

## Context
ADR 0017 item 3 allows verified libraries, each with an ADR and a cargo deny check; HTTP and JSON — hyper and serde_json
(item 5). For TLS in the server itself (ADR 0012 item 1), the token hash and constant-time comparison crates are needed. Random bytes —
`getrandom` (F-78, the choice is a storage subtask).

## Decision
1. [OURS] TLS: `rustls` 0.23 (F-93): TLS 1.2 and 1.3, license `Apache-2.0 OR ISC OR MIT`; the default provider —
   `aws-lc-rs` (enabled by default), `ring` — an alternative (enabled by a feature). The provider choice is an implementation decision:
   building `aws-lc-rs` may require a C toolchain, the providers' licenses and RustSec were not checked. Connecting to tokio —
   `tokio-rustls` 0.26.6 (`Apache-2.0`/`MIT`, depends on `rustls` and `tokio`). PEM reading — `rustls-pki-types` 1.15.1 (`PemObject`,
   the `std` feature), a separate `rustls-pemfile` is not needed (F-94).
2. [OURS] The token hash — `sha2` 0.11.1 (SHA-256, pure Rust, `Apache-2.0`/`MIT`; F-100).
3. [OURS] Constant-time comparison — `subtle` 2.6.1 (`ConstantTimeEq`: "should execute in constant time" — a goal, not a hard
   guarantee; for slices of different length it returns earlier; F-100). The hashes have the same length of 32 bytes, so an early exit
   by length does not occur. The `subtle` license was not checked (the page source did not give it).
4. [OURS] The hexadecimal token form is written by hand; a `base64`/`hex` crate is not needed.
5. [OURS] The cargo deny check is a build requirement: `cargo-deny` is not installed in the environment, licenses and RustSec advisories for these
   crates were not checked; the `cargo tree` dependency graph was not obtained (the network to crates.io is closed from the environment). Not verified.
6. [OURS] The crates are replaceable without a change in behavior: the contract is the algorithm (SHA-256, TLS 1.2/1.3), not the crate.

## Alternatives considered
- `native-tls`/OpenSSL: rejected, an external C library and system differences; rustls is pure Rust (except the provider).
- Argon2 for tokens: ADR 0065 item 4.
- Comparison with `==` on bytes: rejected (P-7b).

## Consequences
- Versions are as of 2026-10-27; pinned in `Cargo.lock`; the provider choice and the cargo deny check are done by the build.

## Sources
- Facts F-93, F-94, F-100, F-78; ADR 0017.
