# Promote a fossil with boneyard storage

Issue: [#481](https://github.com/willjasen/rawrarr/issues/481)

## Objective and design

Enable a fossil to be promoted with media storage while keeping one enrolled node, one identity, and one Rawrarr container. The metadata role remains `metadata`; configuring `RAWARR_LIBRARY_PATH` adds storage scans and scheduled scans. The optional host media directory is mounted read-only at `/media`. Existing standalone `storage` role deployments retain their storage-only behavior and media requirement.

No coordinator database, protocol, encrypted configuration, or synchronized data migration is required. Existing standalone boneyards remain unchanged. To promote a fossil, regenerate its Compose configuration with the same node identity and existing volumes, add the optional media mount, and recreate only the fossil service after reviewing its Compose diff. Preserve `/data`, `/shared`, `/ui`, Tailscale state, and credentials.

## Verification

- Rust role tests confirm metadata nodes can add optional storage while legacy storage nodes retain their media requirement and role behavior. Coordinator relay tests allow only validated scan operations for either subscriber role; the promoted node uses metadata role.
- Subscriber Compose tests confirm one `fossil` service with metadata role and optional read-only media mount.
- First full `python3 scripts/tests/run_rawrarr_tests_in_docker.py` run: onboarding Chromium passed, but exposed stale auth fixture routes, the expanded storage controls pushed Fossils card actions below the viewport, two role-source checks still expected metadata-only startup, a stale owner-recipient role assertion, and mixed-case Rayfish runtime strings. These were corrected.
- Second full `python3 scripts/tests/run_rawrarr_tests_in_docker.py` run: **passed** — Node 155/155, Chromium suite passed including onboarding, ENS login, passkey layout, and Fossils desktop/mobile action visibility; Rust 145 passed, 1 ignored, 0 failed. Python integration fixtures were skipped by the standard runner.
- Focused role/Compose/source checks also passed after the fixes.
- Live deployment and existing-node promotion were not performed.
