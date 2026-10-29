# 0100. Number history: UI responsiveness up to ≈ 10^4 records, stream buffer is bounded
- Status: Accepted (OD-7)
- Label: [OURS] (owner decision OD-7 item 7 — [USER])
- Date: 2026-10-28
- CUJ: CUJ4, CUJ8, X-1

## Context
A number can have up to 2,967,164 records, the UI re-reads the full history after saving, deletion and conflict; hundreds of MB of JSON and millions of DOM rows hang the tab (`docs/reviews/rust-expert.md`, item 12; not verified). On the server hyper bounds the stream by the size of the write buffer, not by "the socket accepting" (F-123).

## Decision
1. (Q-163 closed.) The UI guarantees responsiveness of the number history up to ≈ 10^4 records; beyond that it is not guaranteed (a number usually has few records, edge cases are neglected — the owner, discussion of Q-128). The results are not truncated, an incomplete stream is not taken for a complete one. There is no new API operation; incremental rendering (a virtual list) is implementation, not a requirement.
2. Server: the number stream is bounded by the buffer ("buffer is bounded"): a small hyper `max_buf_size` (the value is implementation), the next portion is read from the engine after the buffer has accepted the previous one. An idle time of 120 s means the client is not accepting data.

## Alternatives considered
- Pagination or truncation of the history: rejected, it changes visible behavior (ADR 0035 item 6, ADR 0061).

## Consequences
- "Changed by OD-7" in ADR 0092 ("Consequences"), 0072 item 4; `web-ui-flows` and `server-runtime` are changed.

## Sources
- OD-7 item 7; `docs/reviews/rust-expert.md` item 12; F-123.
