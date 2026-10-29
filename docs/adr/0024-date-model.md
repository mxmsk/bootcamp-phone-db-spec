# 0024. Date: calendar day, bounds, zero date, technical limit
- Status: Accepted
- Label: [OURS] (items 1–4, 6, 7); [USER] (item 5: OD-5, D-5; item 3 for rename and delete: OD-5, Q-90)
- Date: 2026-10-27
- CUJ: CUJ1, CUJ2, CUJ9; item 1.3 of the original wording

## Context
ADR 0002 (an owner decision): the date is a calendar day, any date not earlier than 1876-03-07 is valid, future dates
are allowed, there is no product upper bound; when the date is missing on add, the zero date 1876-03-07 is substituted
(OD-5: for rename and delete the date is mandatory). Q-64 asks for a technical limit and how it is visible to the user.

## Decision
1. [OURS] A date is a day of the Gregorian calendar: a triple (year, month, day) with no time of day and no time zone;
   the same day for any observer. A nonexistent day (30 February, 29 February in a non-leap year, month 13, day 0) is
   invalid. A leap year is one divisible by 4 and not by 100, or divisible by 400 (RFC 3339, appendix C; F-7).
2. [OURS] The lower bound is 1876-03-07 inclusive (ADR 0002, item 3). A date earlier than it is invalid.
3. [OURS] The zero date 1876-03-07 is substituted if the date is not passed on add (ADR 0002, item 4); for rename and
   delete the date is mandatory, and its absence is a value error (date, "missing"; [USER], OD-5, Q-90; ADR 0025 item
   6). An explicitly passed 1876-03-07 is the same value and the same key (ADR 0002, items 4–5). After saving, an
   explicit date cannot be told from a substituted one on add.
4. [OURS] The validity of a date does not depend on the current time: a "future" date is not a special case, and the
   server's current date is not read during the check.
5. [USER, OD-5, D-5] The technical upper limit is 9999-12-31 inclusive (a four-digit year, as in RFC 3339
   "date-fullyear = 4DIGIT", F-7). A later date is invalid, an error of the same kind "date out of range" as for a date
   earlier than the lower bound; this is the only visible loss of "any future date".
6. [OURS] Two kinds of date error and their order: "nonexistent day", then "out of range" (calendar correctness is a
   property of the triple (year, month, day), the range is policy). The existence of a day is determined by the
   proleptic Gregorian calendar for any year on input (item 7): (0, 2, 29) and (10000, 2, 29) exist, so they give "out
   of range"; (1875, 2, 30) and (10000, 2, 30) do not exist in any year, so they give "nonexistent day". Writing a
   five-digit year in the protocol is the API area (Q-67); if it cannot be written, such input does not reach these
   rules. The reason "missing" (item 3) is only for rename and delete; it is checked before these two (the date was not
   passed, there is no value).
7. [OURS] The year on input is a signed 32-bit integer (−2,147,483,648…2,147,483,647), year 0 exists (astronomical
   numbering), leap-ness is by the formula of item 1 with a remainder of division (for negative years too: −4 is a leap
   year, −100 is not, −400 is); negative years give "out of range" (or "nonexistent day" if the day does not exist).
   Input with a year outside 32 bits does not reach `records`: it is rejected by the protocol layer (a requirement for
   the API, Q-67). The type bound is technical, not policy; 32 bits cover the valid years with room to spare, and input
   with a larger year has no practical meaning.

## Alternatives considered
- A wider limit (for example, year up to 262,143 or 2^31): rejected, it needs an extended year notation that familiar
  date formats lack (RFC 3339: 4 digits); there is no practical benefit for phone numbers.
- A limit "today + N years": rejected, it turns validity into a function of the clock, breaking item 4 and the
  repeatability of checks.
- The Julian calendar or another system: rejected, the patent date 1876-03-07 (F-1, only the issue date) is a day of the
  Gregorian calendar: the USA, as British colonies, switched to it in 1752 (F-27, a secondary source); the UI/API
  calendar is not chosen by the user.
- Storing the time of day and zone: rejected, the date is a calendar day (ADR 0002); how the API/UI accept strings with
  time is Q-67, Q-62.
- A limit at year 3000 or 2999: rejected as arbitrary with no gain.

## Consequences
- An explicit date 1876-03-07 cannot be told from a substituted one in results (Q-1 is only about display).
- Out-of-range values and nonexistent days are two different kinds in the error dictionary (ADR 0020, item 7); how they
  are expressed in the protocol is Q-49. Scenarios with "year 10000" are verifiable only on input that the protocol can
  carry (Q-67).
- Storage must represent any day from 1876-03-07 to 9999-12-31 (2,967,164 days inclusive; a requirement for storage).
- The owner confirmed 9999-12-31 (OD-5, D-5).

## Sources
- ADR 0002; F-1 (facts.md; only the patent issue date), F-27 (the switch to the Gregorian calendar in 1752); brief
  §3/Data; Q-64.
- F-7 (`facts.md`): RFC 3339.
