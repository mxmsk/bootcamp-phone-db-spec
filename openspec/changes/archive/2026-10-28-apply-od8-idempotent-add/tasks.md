## 1. Spec

- [x] 1.1 `records`: the 1876-03-07 substitution and the "unchanged" exception
- [x] 1.2 `write-operations`: add, error kinds, check order, retry after an unknown outcome, adding without a date for a known number
- [x] 1.3 `http-api`: 200 `unchanged` response to an add without a date, retry after a connection drop, `date: null`
- [x] 1.4 `web-ui-flows`: CUJ1, keeping entered input on an unknown outcome, determining a known number
- [x] 1.5 `user-journeys`: CUJ1, CUJ7, CUJ9

## 2. Documents

- [x] 2.1 Close Q-180 in `docs/open-questions.md`, update the totals
- [x] 2.2 Item "Changed by OD-8" in ADR 0002, 0005, 0025, 0026, 0060, 0062, 0090; mark in `docs/adr/README.md`

## 3. Verification

- [x] 3.1 `openspec validate apply-od8-idempotent-add --strict`
