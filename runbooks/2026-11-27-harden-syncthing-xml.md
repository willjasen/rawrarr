# Harden Syncthing XML parsing

Issue: [#510](https://github.com/willjasen/rawrarr/issues/510)

## Objective and initial state

Address scanner findings for XML parsing in `setup_local_syncthing.py` and `lock_syncthing_to_tailscale.py`. Review found a third runtime parse in `share_storage_media.py`. The scripts read the local Syncthing container configuration; no Python dependency manifest is used for these scripts. The task started from clean `origin/main` at `566310a` in a dedicated worktree on `codex/harden-syncthing-xml`.

## Change

Added `scripts/safe_xml.py`, which validates XML with Python's Expat parser and rejects DTD/entity declarations before passing valid input to ElementTree. Updated all three runtime Syncthing config readers. Added sanitized positive and hostile-input coverage, and documented the shared helper and focused checks. No data, protocol, or deployment compatibility change is involved.

## Verification

- `python3 scripts/tests/test_setup_local_syncthing.py` — 6 tests passed.
- `python3 scripts/tests/test_share_storage_media.py` — 5 tests passed.
- `python3 scripts/tests/test_lock_syncthing_to_tailscale.py` — 2 tests passed.
- `python3 -m py_compile` for the helper and three callers — passed.
- Source scan confirmed every runtime Syncthing config parse uses `safe_fromstring`; the sole remaining ElementTree parse is behind the helper's declaration-rejection pass.
- `git diff --check` — passed.

## Final state

The implementation and focused verification are complete. No runtime deployment or live Syncthing configuration was changed. No migration is needed. The change is local to the task branch; no pull request or deployment was requested.
