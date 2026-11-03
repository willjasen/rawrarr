# Retire staged Headscale clients and Mac coordinator transport

Issue: [#154](https://github.com/willjasen/rawrarr/issues/154).
Third increment, following `5286e30` in the same dedicated worktree.

## Decisions and evidence

The two staged Headscale definitions differed only in container name, hostname/
TS_HOSTNAME, and state path. Repository callers were the deployment guide,
inventory, historical staging runbook, and a MacBook deploy-classifier entry.
The operator chose: "Retire both staged-client definitions; I confirm they are
obsolete". The remote Mac audit limitation from the
[prior record](2026-11-03-retire-legacy-coordinator-compose.md) still applies.
No new successful remote live-state check is claimed.

The operator then clarified: "Anything in regards to deploying a coordinator to
either the MacBook or the Mac mini is no longer needed. The internal coordinator
lives on do3 and will stay there." The remaining MacBook coordinator overlay
was retained solely for old callers, including the admin preview. The operator
selected "Keep the preview as a standalone localhost-only container".

The clean task branch/worktrees were inspected and `origin/main` fetched.
Newer upstream changes were observed, not merged into this task. No other task's
checkout or commits were modified.

## Changes and boundaries

- Removed both `docker-compose.headscale-*.yml` staged-client templates and
  `docker-compose.tailscale-macbook.yml`. Root definitions decrease from 19 to
  16 (22 before this issue's increments).
- Removed stale deploy-classifier entries, added retired-path exclusions, and
  guarded that mixed changes still select real runtime updates.
- The sample admin template now publishes `127.0.0.1:7299:8090` directly from its
  own namespace, with no coordinator/Tailscale dependency. Sample UI, image,
  container name, health check, and absence of volumes/production data remain.
- Moved relevant workspace/direct-route assertions to do3 and retained
  isolation/cache coverage on supported subscriber/test definitions.
- Replaced current Mac coordinator deployment instructions and marked historical
  staging procedures as such. Generic self-hosting templates remain available,
  but are not internal Mac deployment recipes. The internal coordinator stays
  on do3; both Macs remain subscribers.

No running/stopped container, host-local definition, Tailscale state, credential,
enrollment, database, or volume is changed. Subscriber transport, production
Headscale, and test Headscale remain unchanged.

## Adoption, failure, and rollback

Current do3/subscriber deployments require no migration. No persistent schema,
encrypted config, or protocol changed. Retired staging workflows have no new
template: stop using their filenames. An unexpected caller must pause for
inspection of its project and identity mounts, not enroll a new identity.
Definitions are recoverable from `5286e30` with their original resource settings.

The admin preview has no persistent state and needs no data migration. For new
local previews, use the separate `rawrarr-admin-preview` project after checking
port 7299 and `rawrarr-admin` name ownership. Existing previews are not silently
moved between projects or recreated. Read their labels and preserve the project
until deliberate migration. A legacy sidecar may still own port 7299; do not
reconfigure/restart it automatically. Tailnet access supplied by the retired
namespace is intentionally no longer included. See
[admin adoption](../docs/admin-console.md#local-preview).

Do not merely remove the old MacBook overlay from a combined deployment's
command: it changes more than admin networking. Any unexpected dependent stack
needs a separate reviewed migration. Source rollback to `5286e30` restores the
old templates but does not restore routes or change running containers.
Retirement never authorizes identity deletion, key revocation, or data cleanup.

## Verification

- Focused Python checks passed: Compose layout (5), internal deploy targets (10),
  and do3 coordinator updater (12). Bytecode writes were disabled.
- Resolved the standalone admin model with an empty env file and verified it
  contains only `admin`, publishes exactly `127.0.0.1:7299:8090`, preserves the
  `admin/` build context, and has no dependencies, shared namespace, volumes,
  or secrets. Confirmed 16 root Compose files remain.
- Retained test and do3 coordinator definitions passed `config --quiet`.
  The coordinator check supplied `RAWARR_INTERNAL_IMAGE=rawrarr:config-validation-only`
  solely for required interpolation; no image was built or deployed.
- The source-snapshot Docker regression runner passed Node.js, Playwright, and
  Rust (123 Rust tests) in clone `rawrarr-test-regression-tests-mw15m7dy`.
  The reusable test base was not rebuilt. Full runner output is retained as a
  session artifact, `staged-client-retirement-regression.log`.
- Reviewed remaining retired-filename callers: only retirement guards,
  classifiers, inventory/recovery documentation, and historical records remain.
  `git diff --check` passed.

No remote Mac Docker audit, full multi-node startup, deployment, or live cleanup
was performed in this increment. Admin networking was verified in the resolved
model, not by starting a preview or exercising a live published port. The
regression runner explicitly skips Python integration and peer-container sync;
those environment checks were not run.
