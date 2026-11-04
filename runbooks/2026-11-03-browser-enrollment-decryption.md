# Browser enrollment and metadata decryption

Tracking: [#390](https://github.com/willjasen/rawrarr/issues/390).

## Scope and design

Close the browser enrollment/decryption gap without changing production behavior.
Use the reusable OrbStack test image, snapshot the current task worktree, prepare
dependencies with the copied lockfile, then disconnect networking before starting
disposable PostgreSQL and the actual coordinator/metadata executables. PostgreSQL
shares the clone's network namespace and uses tmpfs. There are no published host
ports, bind mounts, production URLs, persistent volumes, or injected auth sessions.
Each service receives an explicit environment rather than inheriting host settings.

Chromium uses a virtual resident WebAuthn authenticator. The application creates
the passkey, age ID and encrypted signing-root backup. A real metadata process
requests enrollment; the application's approval dialog signs it in the browser.
The same browser subsequently performs passkey sign-in and the production
Settings export action over the coordinator relay. Synthetic metadata, group keys,
user envelope, credentials and HMAC filenames are seeded only in the harness.
This tests export of already-published ciphertext, not upstream ingestion.

The transparent observer forwards actual coordinator requests/responses without
replacing handlers. Generated private markers are checked against full observed
HTTP streams, browser requests, service logs and an in-memory PostgreSQL data
dump (literal, URL, base64 and hex forms). Capture buffers and secrets are never
printed or saved on the host. This establishes only marker non-disclosure on
these exercised paths, not a proof covering every encoding or application flow.

## Invocation and prerequisites

Existing `rawrarr-testing-base:local` and `postgres:17-alpine` in OrbStack, working
Docker, and sufficient clone/build storage are required. No test-base rebuild is
needed for source changes.

```sh
python3 scripts/tests/run_rawrarr_tests_in_docker.py --browser-enrollment-export
python3 scripts/tests/run_rawrarr_tests_in_docker.py --browser-enrollment-export --browser-enrollment-inject-failure
python3 -m unittest discover -s scripts/tests -p test_browser_enrollment_docker.py
```

The second command must exit nonzero after real enrollment approval. The outer
runner removes only its owned containers in a `finally` block on either outcome;
the browser helper also reaps its child processes and removes its temporary
files. A cleanup failure is reported rather than silently ignored. Unexpected
termination of the host runner itself is outside Python's `finally` guarantee.

## Coverage and limitations

- Successful owner registration, encrypted root creation, signed metadata approval,
  fresh passkey sign-in, and exact three-movie JSON/artwork ZIP contents.
- Valid but unrelated age key fails decryption with no download, then the correct
  key succeeds at desktop 1280x800 and mobile 375x812. Export controls must fit
  within the viewport with no page-wide horizontal overflow.
- Anonymous browser fetches and anonymous/invalid-cookie HTTP requests to index,
  config, group/node recipients, user envelope, text ciphertext and artwork
  ciphertext reject with HTTP 401, through the relay and directly on the node.
- Both viewport flows must transfer exact harness-seeded ciphertext through the
  real coordinator relay; downloaded manifest paths, titles, JSON and artwork
  must match exactly. Scanner self-controls reject deliberately exposed markers.
- Focused Rust export tests and existing Node workspace/Playwright mini-export
  regressions run inside the same offline clone, not on the host.
- No production deployment, storage node, real hardware authenticator, HTTPS/
  Tailscale, multi-user account enrollment, direct-route selection, key rotation,
  upstream Radarr ingestion, or whole-project security claim is included.

## Adoption, recovery and rollback

Test-only changes; no persistent schema, runtime or protocol change, migration,
deployment adoption step or rollback is necessary. A failed assertion must not
be worked around by weakening application authorization or substituting a fake
export server. Re-run with a new disposable clone after correcting the harness.

## Validation

2026-11-03, reusable `amd64` OrbStack base, Rust 1.85.1, Node 26.13.0,
age 1.3.2 and Chromium:

- `PYTHONDONTWRITEBYTECODE=1 python3 scripts/tests/run_rawrarr_tests_in_docker.py --browser-enrollment-export`
  passed. The runner built the real executable offline, passed the one focused
  Rust `metadata_export_tests` test, passed 37 existing Node/Chromium tests
  (`test_rawrarr_coordinator_workspace.mjs` and `test_rawrarr_mini_export.mjs`),
  and passed real enrollment, passkey sign-in, both viewport exports/negative
  controls and coordinator HTTP/log/database marker audits.
- `python3 scripts/tests/run_rawrarr_tests_in_docker.py --browser-enrollment-export --browser-enrollment-inject-failure`
  exited 1 as intended after real signed approval; the outer verification
  confirmed no remaining containers with either dedicated browser-tier service
  label. Both the success run and failure run printed owned-resource cleanup.
- `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest discover -s scripts/tests -p test_browser_enrollment_docker.py`
  passed seven runner lifecycle/dispatch tests.
- `git diff --check` passed.

Early harness runs caught an incorrect expected approval status (the real route
returns 204) and a recursively placed scanner self-control; both were fixed in
test code. No production defect or runtime fix was needed. No full Rust suite,
full browser suite, live deployment, production database or real user data was
used or claimed. `PYTHONDONTWRITEBYTECODE=1` avoids altering tracked legacy Python
bytecode during host-side harness imports.

## Integration

No dependency on the parent task's unpushed Rust fixture/module is imported.
Shared integration paths: `scripts/tests/run_rawrarr_tests_in_docker.py` (new
dispatch flag) and `scripts/tests/README.md` (coverage index). Keep the parent
metadata-export and concurrent AI runner flags when integrating these changes.
