# Improve AI answers and split chat and voice

## Objective and initial state

Respond to the owner's report that a Movies-versus-TV file-size percentage question received the generic review answer. Remove the repeated cloud-consent checkbox and give chat and voice their own tabs on the signed-in roarlay assistant page. Continue from the deployed OpenAI implementation in [the earlier runbook](2026-10-18-implement-openai-chat-and-voice.md). The dedicated `codex/ai-request-inbox` worktree was clean at `aa6e692`; the shared `main` checkout was separate.

The coordinator-local inbox showed the size-comparison question and a short follow-up as `needs_review`. The classifier accepts only movie year and genre filters, and the browser assistant holds only the decrypted movie list, so it could not calculate the requested TV share. The current page has one form and a required checkbox.

## Actions and findings

- Added `library_storage_share` to the existing strict OpenAI interpretation schema. The browser totals `sizeOnDisk` from its Movies records and Sonarr series records, reports each category's share of their combined bytes, and explains missing or zero size data without inventing a percentage. The assistant loads the TV records on demand for this question and clears cached values when the node or age ID changes.
- Split the assistant page into accessible Chat and Voice tabs with separate answer areas and a visible voice transcript. Removed the required cloud checkbox; a standing notice states that sending a question or recording goes to OpenAI. The existing authenticated, same-origin API and request logging remain in place.
- A live OpenAI structured-output request using the reported question classified it as `library_storage_share` with null movie filters. It contained the question only, not private library records.

## Verification and final state

- Focused JavaScript tests pass (five cases), including 40/60 percent reported-byte shares and missing TV data. JavaScript syntax, `git diff --check`, and Rust `cargo check` pass; the Rust check has the existing `read_shared_recipient` dead-code warning.
- Deployment and signed-in browser verification are pending.

## Recovery and remaining work

In progress. Keep the OpenAI key, private age ID, decrypted catalog, and local PostgreSQL data out of Git and this runbook.
