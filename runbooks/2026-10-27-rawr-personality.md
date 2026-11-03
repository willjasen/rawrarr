# Give Rawrarr's voice agent a warm, playful personality

Implement issue [#233](https://github.com/willjasen/rawrarr/issues/233): make Rawr a helpful, innocent, curious, patient, gently upbeat, playfully earnest dino helper across text and voice. Keep grounded answers candid about which library records were checked; do not imply human childhood, personal experiences, or unchecked results.

## Changes

- Added the shared character description to the OpenAI intro and question-classification prompts without changing the allowed intent schema.
- Updated locally generated library answers and unsupported responses to use a warmer, gently playful tone while retaining their local-data grounding and existing filters.
- Introduced Rawr on the assistant page and clarified the communication-style boundary in the design guide.
- Added a focused regression for the grounded answer style.

## Verification

- `node --test scripts/tests/test_rawrarr_ai_chat.mjs`: passed (9 tests).
- `git diff --check`: passed.

## Limits

- This change affects the signed-in Roarlay assistant. It does not change the local speech synthesis voice or make the assistant a general-purpose agent.
