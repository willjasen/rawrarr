# Repair missing movie metadata cache

Issue [#379](https://github.com/willjasen/rawrarr/issues/379).

## Initial state and root cause

The Movies page loads 1,955 Radarr entries but reports 0 B of movie and total
metadata. Read-only inspection on the internal MacBook confirmed that
`rawrarr-node-internal-fossil-1` is healthy and `/metadata` is a dangling symlink
to `/shared/in/metadata`. The mounted `/shared` volume has no `in` directory.
The encrypted config, group-recipient markers, and node-local identities remain
present; no private file contents were inspected.

The image creates the alias, but the mounted volume hides the image's shared
directories. The application creates the metadata target only when copying
legacy metadata. A fresh volume without legacy metadata therefore cannot write
through `/metadata`. The storage endpoint treats missing directories and other
filesystem errors identically as zero.

## Repair and safeguards

Create only `/shared/in/metadata` in the existing internal MacBook fossil
container, as its normal application user. Preserve all existing files,
identities, credentials, volumes, other containers, and browser windows. Do not
restart or deploy services, trigger a Radarr library refresh, or access the
public roarlay. Observe whether the existing worker begins writing encrypted
metadata after its normal retry.

## Permanent fix and adoption

Subscriber startup and vault opening initialize the shared metadata target even
when no legacy metadata exists. Existing files are preserved, and unexpected
files or symlinks in the shared directory path fail explicitly. Unlocking the
vault wakes both metadata workers rather than waiting for their retry timer.
The storage endpoint now propagates filesystem and worker failures as HTTP 500;
the existing browser error state renders unavailable sizes instead of 0 B.

This is an additive, repeatable directory initialization, not a data-format or
protocol migration. Existing ciphertext, keys, and filenames must remain
unchanged. Existing deployments adopt the backend repair through a normal
published application release; a UI-only update is insufficient. The internal
directory repair can be exercised before public rollout. Rollback must retain
the created directories and any recovered ciphertext.

## Outcome

- Created `/shared/in/metadata` and set directory permissions to `0755`, as
  application UID 10001, in the existing internal MacBook fossil container.
- The existing worker resumed automatically. Read-back found all **1,955 movie
  text records** (10,284,846 bytes) and **17 artwork files** (10,287,911 bytes),
  with artwork still increasing. Every checked file had the age ciphertext
  header; no plaintext or private identities were read.
- Fossil remains running and healthy with its original start time
  `2026-11-03T01:56:04.872595807Z`. No services were restarted or deployed.
- A later read-back found 94 artwork files (57,154,773 bytes), confirming the
  sequential backfill was still making progress.
- No Radarr library scan was needed. Reloading Movies reads the recovered cache
  size; the browser card itself has not been independently read back.
- `python3 scripts/tests/run_rawrarr_tests_in_docker.py` ran the current source
  in an isolated offline OrbStack clone, reusing the existing testing base.
  Node.js passed all 153 tests; Rust passed all 121 tests, including the five
  new metadata initialization, encrypted-write, preservation, and size/error
  checks. The broader Playwright suite reported one timeout in
  `test_rawrarr_coordinator_sonarr_settings.mjs:103`, waiting for a visible
  Radarr settings note. That mocked browser test and the UI source it executes
  were unchanged by this backend repair; no unrelated test fix was included.
- `git diff --check` passed. The full multi-node environment test and browser
  read-back of the live metadata counter were not performed.
- The source change is not deployed; the live repair used only directory
  creation against the existing published application.
