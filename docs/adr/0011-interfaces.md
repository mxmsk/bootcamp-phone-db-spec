# 0011. Interfaces: UI, programmatic API, server process
- Status: Accepted
- Label: [USER]
- Date: 2026-10-27
- CUJ: CUJ7, CUJ8, CUJ10, CUJ13, CUJ16, X-2; item 1.2 of the original wording ("Visual User Interface and programmatic API")

## Context
The original wording requires a visual interface and a programmatic API. The brief defines where the UI is available,
how it is entered, what the API can do and who owns the database.

## Decision
1. The UI is a web page in a browser on the same machine; available only through loopback (brief §3/Interfaces).
2. Sign-in to the UI is by a link with an embedded token that the server prints; the token changes at every start
   (brief §3/Interfaces). After a server restart the old link stops working, and a new one from the server output is
   needed (brief §2, CUJ13).
3. The UI token is accepted only from loopback (OD-3, Q-27).
4. The programmatic API is HTTP + JSON; the same operations as in the UI, plus exact whole-name search; there is no
   administration through the API (brief §3/Interfaces).
5. One server process owns the database; the UI and the API are its clients (brief §3/Interfaces).
6. The UI shows server problems (read-only, storage error) only when an action fails (brief §3/Interfaces): «Сохранение не выполнено: база доступна только для чтения» ("Saving failed: the database is read-only") or «ошибка хранилища, обратитесь к оператору» (storage error, contact the
   operator); the UI does not show the server state in advance (brief §2, CUJ16).

## Alternatives considered
- Administration through the API: excluded, "there is no administration through the API" (brief §3/Interfaces; ADR
  0018).
- Showing the server state in the UI in advance: excluded, the UI shows problems only when an action fails (brief
  §3/Interfaces; brief §2, CUJ16).
- No other alternatives were considered in the sources.

## Consequences
- The behavior of the interface (what to show after save, date format, undoing a delete, result composition) belongs to
  the UI iteration (brief §5; OD-1).
- Open questions: the UI token and the local-program token, keeping the sign-in (Q-38); an open tab after a restart
  (Q-39); verbatim UI messages (Q-65); result composition (Q-29); what to show after save (Q-61); the date format in
  the UI (Q-62); undoing a delete (Q-63); TLS on loopback (Q-70); API codes for "read-only" and "storage error" (Q-44);
  see `docs/open-questions.md`.

## Sources
- `docs/source/brief.md`, section 3 "Interfaces"; section 2 (CUJ13, CUJ16).
- Owner decision of 2026-10-27 (OD-3), Q-27. Place of record: docs/owner-decisions.md, OD-3.
