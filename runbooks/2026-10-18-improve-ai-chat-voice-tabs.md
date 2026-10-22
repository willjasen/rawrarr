# Improve AI answers and split chat and voice

## Objective and initial state

Respond to the owner's report that a Movies-versus-TV file-size percentage question received the generic review answer. Remove the repeated cloud-consent checkbox and give chat and voice their own tabs on the signed-in roarlay assistant page. Continue from the deployed OpenAI implementation in [the earlier runbook](2026-10-18-implement-openai-chat-and-voice.md). The dedicated `codex/ai-request-inbox` worktree was clean at `aa6e692`; the shared `main` checkout was separate.

The coordinator-local inbox showed the size-comparison question and a short follow-up as `needs_review`. The classifier accepts only movie year and genre filters, and the browser assistant holds only the decrypted movie list, so it could not calculate the requested TV share. The current page has one form and a required checkbox.

## Actions and findings

- Added `library_storage_share` to the existing strict OpenAI interpretation schema. The browser totals `sizeOnDisk` from its Movies records and Sonarr series records, reports each category's share of their combined bytes, and explains missing or zero size data without inventing a percentage. The assistant loads the TV records on demand for this question and clears cached values when the node or age ID changes.
- Split the assistant page into accessible Chat and Voice tabs with separate answer areas and a visible voice transcript. Removed the required cloud checkbox; a standing notice states that sending a question or recording goes to OpenAI. The existing authenticated, same-origin API and request logging remain in place.
- Renamed the sidebar and page heading to `rawrs` at the owner's request.
- Enlarged the chat panel and answer area, with the request inbox collapsed below it. The voice tab is now labeled `rawrs`; the text tab stays `Chat` until the owner chooses its name.
- Text chat now requests a second OpenAI Responses call with `stream: true`. Roarlay relays introduction deltas over SSE, then the browser appends the answer calculated from decrypted local records. The cloud never receives catalog rows. Voice retains its turn-based path.
- Added an account-specific `RAWARR_AI_UNLIMITED_USER_ID` override to bypass the five-per-day limit for both text and voice. The private Compose override is at `~/Library/Application Support/rawrarr/ai-testing-unlimited.yml`, mode `0600`, and is outside Git; no secret value or account ID is recorded here. Redeploy without that override to restore the normal limit.
- A live OpenAI structured-output request using the reported question classified it as `library_storage_share` with null movie filters. It contained the question only, not private library records.

## Verification and final state

- Focused JavaScript tests pass (six cases), including 40/60 percent reported-byte shares, missing TV data, and SSE frames split across chunks. JavaScript syntax, `git diff --check`, and Rust `cargo check` pass; the Rust check has the existing `read_shared_recipient` dead-code warning.
- Rebuilt `rawrarr-local` roarlay with the private account override while holding its deployment marker. The container became healthy; local `127.0.0.1:7297/health` and tailnet `/health` returned the roarlay page. Tailscale Serve still routes HTTPS `/` to `http://127.0.0.1:8080`, preserves `/direct-api` at `8082`, and preserves the Syncthing TCP forward.
- In the signed-in Chrome page, the Chat and `rawrs` tabs, large answer area, collapsed inbox, no checkbox, and “Unlimited questions while you test” appeared. A live text question about Movies versus TV produced a streamed OpenAI introduction followed by the local size result: Movies 8.0 TB (44.2%) and TV 10.1 TB (55.8%) of combined 18.1 TB. The voice tab opened with an enabled Talk button. Microphone transcription was not exercised because this browser had not granted microphone access for this test.

## Recovery and remaining work

The local override remains active for this account while testing. To restore the default five-per-day limit, redeploy roarlay with the normal four Compose files and omit `~/Library/Application Support/rawrarr/ai-testing-unlimited.yml`; remove the private override afterward. Keep the OpenAI key, private age ID, decrypted catalog, and local PostgreSQL data out of Git and this runbook. The text tab still has the working name `Chat` until the owner names it. Browser voice recording and transcription remain unverified here.
