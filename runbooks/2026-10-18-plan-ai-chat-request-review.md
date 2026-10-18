# Plan AI chat, voice, and request review

## Objective and initial state

Record the proposed roarlay cloud-AI chat and owner-run request-review workflow. At task start, the shared `main` checkout had unrelated subscriber-onboarding edits; this task used a clean dedicated worktree from `main` at `57b5ecf`. Existing design keeps passkeys and PostgreSQL on roarlay, age-encrypted movie metadata on the metadata path, and plaintext library records out of roarlay. See the [current-state review](2026-10-18-current-state-documentation-review.md) and [metadata role separation](2026-10-18-metadata-external-coordinator.md).

## Decisions and actions

- Added the proposed user experience to `docs/DESIGN.md` and marked implementation outstanding in `ROADMAP.md`.
- A 1990s question should use the existing movie release-year field and cite actual matching records and freshness. Missing fields or unavailable nodes produce an honest incomplete answer and a review item.
- The PostgreSQL inbox holds submitted questions, interpretation, outcomes, and proposed actions for owner review. An owner-run agent may read this inbox and propose a metadata refresh, catalog repair, or configuration change. Mutations need explicit owner approval and an outcome audit.
- Cloud AI receives minimal, explicitly enabled context. The design preserves roarlay's blind-content boundary and keeps age identities, credentials, plaintext config, and media out of the operator service.
- Added a signed-in browser voice conversation using the same read-only answers and review inbox. A later Twilio phone path needs caller authentication before private answers; caller ID alone is insufficient. Retain a consented transcript for review, not raw audio by default. Stretchie's current Twilio interface informed the call-button idea, while Rawrarr's goal is a two-way talking assistant.

## Verification and final state

Documentation-only planning; no code, database migration, cloud call, or container deployment was performed. Confirmed the proposed boundary against `docs/technical-status.md`, `docs/device-architecture.md`, and `rust-arr/migrations/`. Checked the Markdown diff and links before committing.

## Remaining work and recovery

Design the request schema, retention and deletion controls, safe query path, cloud consent, scoped agent API, browser voice session, optional Twilio call route, caller authentication, and approval UI; implement and test each before deployment. Revert this task's documentation commits if the proposal changes. No runtime rollback is needed.
