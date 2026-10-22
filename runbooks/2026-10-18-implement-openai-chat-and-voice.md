# Implement OpenAI chat and voice

## Objective and initial state

Build a usable roarlay AI assistant using OpenAI, with a PostgreSQL request inbox and five questions per UTC day for the current trial-stage product. Continue the [AI design](2026-10-18-plan-ai-chat-request-review.md) and [quota decision](2026-10-18-trial-ai-chat-daily-limit.md). Work began in the clean dedicated `codex/ai-request-inbox` worktree at `079ae6a`; the shared `main` checkout contains unrelated changes. No AI code, trial enforcement, or OpenAI key was present. Roarlay currently receives encrypted movie responses; the browser decrypts them.

## Actions and findings

- Reviewed current OpenAI Responses structured-output, speech-to-text, and WebRTC documentation. The first release keeps library matching in the authorized browser to avoid sending decrypted movie records to roarlay.
- Added a coordinator-local PostgreSQL question table and authenticated status, inbox, submit, and transcription endpoints. Submissions use a per-account row lock and stable request IDs for a shared five-per-UTC-day text/voice allowance. Failed OpenAI interpretations release the allowance; unsupported questions remain for review and count as accepted requests.
- Added an optional Compose secret mount for the owner-created OpenAI key. Corrected its file mode to 0600 without reading its contents. The roarlay handler uses the Responses API with strict JSON schema and `store: false`; microphone audio uses `gpt-4o-mini-transcribe` and is not retained by Rawrarr. The interface requests explicit cloud consent.
- Added Ask rawrarr to the workspace. Questions and recordings go to OpenAI; the browser filters decrypted movie records and displays matched titles. Press-to-talk lasts at most 15 seconds. A browser voice marked local speaks the full answer when available; otherwise the spoken message is generic. Personality is warm, innocent, and curious, with factual answers grounded in the loaded records.
- Current licensing has no trial/paid entitlement, so the five-question limit applies to every account. Search, deletion, retention controls, owner-run configuration proposals, Twilio, and full-duplex voice are subsequent work.

## Verification and final state

- `PATH="$HOME/.cargo/bin:$PATH" cargo check --manifest-path rust-arr/Cargo.toml` passed with one existing dead-code warning.
- `node --test scripts/tests/test_rawrarr_ai_chat.mjs` passed all three focused cases; JavaScript syntax and `git diff --check` passed.
- A live OpenAI Responses request with a synthetic 1990s question returned HTTP 200 and the expected 1990–1999 structured intent. No private library records were sent.
- A synthetic spoken question sent to OpenAI's transcription endpoint returned HTTP 200 and the expected text. The roarlay voice endpoint returned HTTP 403 without a signed-in session and required same-origin consent for authenticated requests; an actual browser microphone conversation was not exercised.
- Rebased the isolated branch onto the newer `main` subscriber-onboarding commit to avoid deploying an older UI. Resolved only overlapping runbook/test index entries. The full Compose file set plus the optional OpenAI overlay validated.
- Rebuilt only roarlay in `rawrarr-local` under its task-owned deployment marker. The container is healthy; PostgreSQL migration `0006_ai_requests.sql` exists; the non-root roarlay process can read the OpenAI secret mount. Local `127.0.0.1:7297/health` and tailnet `/health` both returned the Roarlay health page. Tailscale Serve JSON still maps HTTPS `/` to `http://127.0.0.1:8080` and preserves `/direct-api` at `http://127.0.0.1:8082`.
- In the signed-in Chrome workspace, loaded the actual decrypted Movies library, opened Ask rawrarr, and submitted “Which of my movies were made in the 90s?” The answer counted 276 movies from 1990–1999, displayed actual matching titles, changed the allowance from five to four, and showed a completed request in the inbox. PostgreSQL read-back showed `text|completed|1990-1999`. The deployment marker was removed after health and route checks.

## Recovery and remaining work

Do not put an OpenAI key, age identity, or decrypted library data in Git, this runbook, or a Compose environment value. To disable OpenAI, redeploy roarlay without `docker-compose.openai.yml`; the PostgreSQL inbox remains local for review. Actual microphone capture and spoken playback were not verified in the user's browser. The full-duplex voice agent, owner-run review/configuration workflow, retention controls, and entitlement-specific quota remain future work.
