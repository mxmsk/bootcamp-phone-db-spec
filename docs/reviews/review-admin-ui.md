# Final review: the "admin-ui" group

Capabilities: operator-control, operator-lifecycle, web-ui-shell, web-ui-flows. Overlaps checked against server-runtime, http-api, api-access, user-journeys, ADR 0054, 0075, 0077, 0081.

Total: 7 items: 5 contradictions, 2 BLOCKERs.

## 1. Contradiction: control channel under "storage error" caused by an unavailable settings directory
- Where: `operator-control`, "Channel works under load and storage failures"; `operator-lifecycle`, "Settings directory is not writable" (scenario "Settings directory is read-only after a crash"); `server-runtime`, "Startup order"; ADR 0081 item 3.
- Essence: operator-control requires (MUST) that the channel execute token and stop commands in the "storage error" state. But on a crash-recovery start with a read-only settings directory the server runs in "storage error" without `control.lock` and without a channel. Then `npdb stop` and `npdb backup`, per ADR 0081 item 3 (`ENOENT` -> lock is free), print «сервер не запущен» ("server not running") with code 20 although the server is running, and offline token commands bypass the running server.
- Fix: in operator-control narrow the requirement to "a storage error that occurred after the channel was opened". For this mode define separate observable command behavior, for example: the server takes the already existing `control.lock` for reading, and commands answer «сервер работает без канала команд; остановка — сигналом SIGTERM» ("server is running without a control channel; stop with SIGTERM"), with a separate code.

## 2. Contradiction: `serve` on an empty or nonexistent directory "does not change the directory" but takes `LOCK` first
- Where: `operator-lifecycle`, "Server does not start on a directory without a database" (the text "without changing the directory", scenarios "Empty directory": "the directory stayed empty", "No directory": code 5); `server-runtime`, "Startup order"; ADR 0054 item 1; ADR 0077 item 1 (refusal order `LOCK` (6) -> ... -> no database (5)).
- Essence: by the startup order the server first opens or creates the `LOCK` file in the database directory and writes the pid into it, and only then checks for the database. On an empty directory `LOCK` remains after the refusal, so the scenario "the directory stayed empty" cannot be met. On a nonexistent directory `LOCK` cannot be opened, and per ADR 0054 item 1 that is an "other lock error" with its own cause, not code 5.
- Fix: either check for `control` (read-only, creating no files) before taking `LOCK` and record that in "Startup order", or allow the leftover `LOCK` in the scenario ("only `LOCK` in the directory") and explicitly assign the missing directory to code 5 before the `LOCK` step.

## 3. Contradiction: an interrupted response stream is "no connection" or "storage error"
- Where: `web-ui-shell`, `"Server stopping" and connection drop` (a failure of search or number history after "number known" -> «... не выполнено: нет связи с сервером» and «Повторить»), against `web-ui-shell`, `"Read-only" and "storage error" texts on action failure` (scenario "Search under storage error"), `web-ui-flows`, "Search by number (CUJ4)" (scenario "Incomplete stream") and "Known number is determined by the server response" (scenario "History interrupted"), where a stream without a completeness marker -> «ошибка хранилища, обратитесь к оператору».
- Essence: a connection dropped in the middle of a `by-number` response body fits both rules, and implementations will show different texts. Moreover, per `http-api` ("snapshot revoked" after the stream has started, server shutdown) a drop without a marker also happens without a storage error, so the user would get «обратитесь к оператору» (contact the operator) where a retry is acceptable.
- Fix: separate the cases explicitly. No response received (no status) -> "no connection". Status 200 received but the body ended without `complete` -> one chosen text, preferably with «Повторить» (Retry), not "storage error". Record this in both capabilities.

## 4. Contradiction: texts when a re-read after an action fails
- Where: `web-ui-shell`, `"Server stopping" and connection drop` (a "re-read" failure -> «<Действие> не выполнено: нет связи с сервером» and «Повторить»); `web-ui-flows`, "Stale data (CUJ12)" (scenario "Re-read failed" -> «Обновить список не удалось»), "Re-query after delete" (scenario "Re-query failed" -> «Запись удалена, но список обновить не удалось» and «Обновить»), "What to show after saving" (scenario "History failed": no text defined).
- Essence: for one and the same event (a connection drop during a re-read) two capabilities define different texts and buttons. And if the shell rule with the action «Сохранение» (Saving) is applied to the number history after a successful save (`web-ui-flows`, "Flow message texts"), the result is «Сохранение не выполнено» (Saving failed) although the save succeeded.
- Fix: in web-ui-shell exclude the re-read after an action from the general rule and refer to web-ui-flows. In "History failed" define an exact text that does not speak of a failed save.

## 5. BLOCKER: "outcome unknown": what the UI does with the re-read history and "a retry is safe"
- Where: `web-ui-flows`, "Entered input is kept when a write fails" (scenario "Outcome unknown": "the UI re-reads the number history; a retry by the user is safe"); `web-ui-shell`, `"Server stopping" and connection drop` («Проверьте поиском по номеру» (Check with a search by number), no automatic actions); `http-api`, the requirement about the absence of idempotency keys.
- Essence: it is not defined where and how the UI shows the automatically re-read history, nor whether it concludes "saved" or "not saved" from it. The shell meanwhile sends the user to check on their own. The claim "a retry is safe" diverges from http-api: if the original add without a date went through, the retry gets `409 number_known`, and the UI (CUJ2) asks for a date for a record it has just created itself. If a replacement went through, a retry with the old version gives `version_conflict` and «данные устарели» (data is stale).
- Fix: define the display of the re-read history (as a by-number result) and the texts, and reconcile «Проверьте поиском...» with the automatic re-read. Remove "a retry is safe" or clarify that on a retry of an add without a date the UI shows the history in which the record may already exist.

## 6. BLOCKER: exit code of `npdb backup` on a connection break to the server
- Where: `operator-control`, "Backup cancellation" (connection break to the server -> cancellation, code 33) versus "Backup outcome: success" (scenario "Failure before completion": server crashed -> code 24) and "Command wait timeouts" (code 24).
- Essence: the command sees a server crash as a connection break. One implementation returns 33 ("cancelled"), another 24 ("outcome unknown"), and the operator's script in CUJ15 behaves differently.
- Fix: clarify that "connection break" in "Backup cancellation" is a cancellation cause on the server side (the command disappeared), while for the command itself a drop without a final message is always code 24.

## 7. Contradiction: scale of the load-generation tool in Purpose
- Where: `operator-lifecycle`, Purpose ("a test load-generation tool for verifying 10^8").
- Essence: OD-6 item 1 replaced the scale with 10^7, and the requirements of the same capability ("Load-generation tool is a separate program", "Load profile and reproducibility") speak of 10^7.
- Fix: in Purpose replace 10^8 with 10^7.
