## 1. Decisions and facts
- [x] 1.1 ADR 0096–0101 ([OURS], Accepted (OD-7)); `docs/adr/README.md`
- [x] 1.2 Item "Changed by OD-7" in ADR 0045, 0048, 0051, 0052, 0056, 0057, 0059, 0062, 0067, 0070, 0071, 0072, 0074, 0080, 0083, 0092, 0095
- [x] 1.3 `facts.md`: F-120…F-127 (hyper, tokio/mio, socket2, rustix, rustls, RFC 9112 §6.1)
- [x] 1.4 `docs/open-questions.md`: Q-163 closed, (T) Q-168…Q-180, section "OD-7", totals row

## 2. Delta specs
- [x] 2.1 `server-runtime`: slots and limits, TE+CL, shutdown, by-number stream
- [x] 2.2 `http-api`, `api-access`, `web-ui-shell`: `too_many_connections` removed, shutdown, TE+CL
- [x] 2.3 `storage-indexes`: dictionary D, search methods
- [x] 2.4 `storage-durability`, `storage-snapshots`, `operator-control`: backup without retention, modified-page cap
- [x] 2.5 `web-ui-flows`: number history up to about 10^4

## 3. At archiving (manual, not expressible by deltas)
- [ ] 3.1 `server-runtime` Purpose: remove "too many connections" — `too_many_connections` from the list of refusal names
- [ ] 3.2 `storage-snapshots` Purpose: "backup without stopping the server" — without words about a backup snapshot (as needed)
- [ ] 3.3 `storage-format`, scenario "Retention temporary file": check against the decision on Q-168
- [ ] 3.4 `user-journeys`, CUJ15: "backup point" is the commit number (Q-83 closed by ADR 0057, 0096)
