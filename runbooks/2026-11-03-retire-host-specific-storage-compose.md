# Retire host-specific Mac storage and relay templates

Issue: [#154](https://github.com/willjasen/rawrarr/issues/154).
Continuation from published main `f67b072` in the existing task worktree.

## Decision and evidence

The supplied MacBook OrbStack screenshot shows the `rawrarr-node-internal`
project with fossil, artwork-cache, and tailscale. Read-only inspection of local
running/stopped container labels agrees: the internal fossil uses host-local
`compose.yml` and `compose.internal-base.yml`, not these old repository templates.
Remote Mini authentication still fails because the saved identity is unavailable;
the MacBook alias fails strict host-key verification. Neither was bypassed.

The operator explicitly selected: "Remove the MacBook storage and old Mini
transport/relay templates; I confirm both old setups are obsolete".
This supersedes the earlier decision to wait for remote audits, but does not
constitute an independently completed remote audit.

## Scope and compatibility

Removed `docker-compose.macbook-storage.yml`, `docker-compose.tailscale-mini.yml`,
`docker-compose.mini-relays.yml`, and the latter's `storage-proxy.Caddyfile`.
Updated deployment classifiers, test-fixture copying, retirement guards, and
network-isolation tests. Replaced the old Mini setup recipe with explicit
retirement and generic storage prerequisites. Root Compose count is now 13,
from 16 before this increment and 22 originally.

No retained Compose definition or application source changes. Generic storage,
metadata, coordinator, combined, Syncthing, and do3 stacks remain supported.
Current host-local subscriber deployment files are not changed. No deployment,
route edit, service stop, identity revocation, or data deletion is requested.

No schema, protocol, encrypted configuration, or data migration is needed.
Current deployments have no adoption action. Old commands referencing the
removed filenames fail explicitly; inspect unexpected dependent stacks rather
than substituting templates or dropping overlays before recreation. The generic
storage template still requires a separately managed, verified existing
Tailscale namespace; its legacy fallback name does not create that namespace.

All removed files are recoverable from published `f67b072`. Preserve original
Compose project/resource names, node-local `/data`, shared folders, media mounts,
credentials, and identities during any separately authorized recovery.
Restoring source changes no live container. Do not use `down`, remove volumes,
or create replacement identities as part of this cleanup.

## Verification

- Passed 33 focused Python checks: layout (6), internal deploy targets (11),
  coordinator updater (13), and regression snapshot fixtures (3).
- Passed 8 focused Node checks across network isolation and direct routing.
  These source-backed tests require no browser or service fixtures.
- Compared all 26 retained deployment/test YAML definitions against published
  starting main: byte-identical. Confirmed 13 root Compose files remain.
- Retained do3/test models passed `docker compose config --quiet`; the do3 check
  supplied an interpolation-only image placeholder, not a build/deployment.
- Remaining retired-path references in scripts/docs are guards, classifiers,
  inventory, or recovery text, not executable file readers/commands.
  `git diff --check` passed.

The full Rust/Node/Playwright suite, multi-node startup, live routing, and remote
Mac Docker audit were not run for this source-only retirement. No runtime source
or retained Compose definition changed; focused checks cover the altered callers.
Installed deployment automation is not upgraded by editing its source here and
may retain older path-selection behavior until separately updated.
