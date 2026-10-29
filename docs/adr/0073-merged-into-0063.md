# 0073. Mapping storage states to server responses (merged into 0063)
- Status: Superseded by 0063
- Label: [OURS]
- Date: 2026-10-27
- CUJ: CUJ16, X-5

## Context
The decision about which condition corresponds to which refusal and what the server does not show to clients coincided in topic with ADR 0063 and diverged from it
in the check order (the token check while the server is starting).

## Decision
The content has been moved to ADR 0063: the kinds of refusals and their correspondence to conditions — item 2; the state is not cached — item 3; environment details are not
reported to the client — item 4; a refusal before the response begins and a stream cut — item 5; start and stop — item 6; the check order — item 7. There is no separate decision here.

## Consequences
- References to "ADR 0073" should be read as ADR 0063.

## Sources
- ADR 0063.
