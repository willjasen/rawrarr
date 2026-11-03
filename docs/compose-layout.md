# Compose inventory and cleanup

Tracked by [#154](https://github.com/willjasen/rawrarr/issues/154).
Inventory baseline: 2026-11-03, commit `7be69d7`. This maps repository
definitions and callers, not the current contents of remote Docker labels.
Lack of a repository caller is not evidence that a deployed overlay is unused.

## Current layout

- `docker-compose.yml` and the remaining root overlays retain their paths for
  compatibility. They are not all part of one stack.
- [`deploy/testing/compose.yml`](../deploy/testing/compose.yml) is the isolated
  coordinator/two-fossil environment, with its Caddy and Headscale fixtures beside
  it. It replaces `docker-compose.orbstack-test.yml`.
- `deploy/do3/` contains environment-specific coordinator/support definitions.
  Leave these separate from subscriber definitions and from each other.
- `rust-arr/web-export/subscriber-compose.js` generates downloaded subscriber
  Compose. Its output is not a checked-in deployment stack.
- Protected host-local Compose overrides and runtime data are not repository
  templates; do not copy them into this tree.

## Root-file inventory

There were 22 root Compose files before the first increment; 19 remain after
the second increment retired the former Mini coordinator and empty Serve overlay.
Paths in the caller column are relative to the repository. Test names are under
`scripts/tests/`. Historical runbooks record past usage, not current deployment
authority. All root `docker-compose*.yml` files are also copied into regression
containers by `scripts/tests/run_rawrarr_tests_in_docker.py`.

| File | Purpose / combination | Known repository callers and evidence |
| --- | --- | --- |
| `docker-compose.yml` | Combined local coordinator/PostgreSQL/fossil base; historical Mac topology, not the current internal environment. | `docs/container-deployment.md`, `docs/user-guide.md`, `docs/production-deployment-plan.md`, `docs/admin-console.md`, `rust-arr/README.md`; `scripts/classify_internal_deploy_targets.py`; passkey, workspace, healthcheck, isolation, and trial tests. |
| `docker-compose.coordinator.yml` | Standalone coordinator/PostgreSQL base; also combined with the old MacBook base and transport overlay in historical deployments. | Deployment/production guides; deploy classifier; analytics, passkey, and trial tests; October 18 deployment runbooks. |
| `docker-compose.metadata.yml` | Standalone fossil plus private Valkey cache, against an existing coordinator. | Deployment guide; deploy classifier and its test; node-network-isolation test; metadata/cache runbooks. |
| `docker-compose.storage.yml` | Standalone boneyard using an existing Tailscale container namespace; not a coordinator stack. | Deployment guide; deploy classifier; workspace, direct-route, and network-isolation tests. |
| `docker-compose.internal-mini.yml` | Retired: former private Mac mini coordinator/PostgreSQL/Tailscale topology, superseded internally by do3. | Former analytics/workspace/trial callers now cover the do3 definition. October 22-27 runbooks remain historical; deployment classifiers ignore the retired path. |
| `docker-compose.tailscale-macbook.yml` | Transport/cache overlay for the old combined MacBook base; modifies roarlay and fossil networking. | Deployment/admin guides; deploy classifier; workspace, direct-route, and isolation tests; October 18 and 25 runbooks. |
| `docker-compose.tailscale-mini.yml` | Separate Mac mini Tailscale companion used by storage/relay definitions. | Deployment guide. Requires inspection of host-local active files before consolidation. |
| `docker-compose.macbook-storage.yml` | Additional MacBook boneyard using an existing namespace and protected node-local mounts. | Deploy classifier; `runbooks/2026-10-18-storage-media-replication.md`. |
| `docker-compose.media.yml` | Changes the base fossil into a storage-role node with a read-only media bind. | Deployment/user guides; `rust-arr/README.md`; local-library-scan runbook. |
| `docker-compose.age-identity.yml` | Optional fossil private identity secret mount; never a coordinator overlay. | `docs/user-guide.md`. |
| `docker-compose.admin.yml` | Private sample admin preview, combined with base + MacBook Tailscale overlay. | `docs/admin-console.md`; admin-preview test; admin-preview runbook. |
| `docker-compose.analytics.yml` | Coordinator settings overlay only; starts no Umami services. | `docs/coordinator-analytics.md`; analytics test. |
| `docker-compose.analytics-services.yml` | Explicit opt-in Umami/PostgreSQL services on the coordinator's existing private control network. | Analytics guide/test; analytics opt-in runbook. Not equivalent to the settings-only overlay. |
| `docker-compose.openai.yml` | Optional coordinator OpenAI file-secret integration. | October 18 OpenAI implementation and October 27 internal OpenAI configuration runbooks. No current script caller found. |
| `docker-compose.syncthing.yml` | Separate Syncthing companion for approved subscribers; no app or PostgreSQL recreation. | `scripts/README.md`; deploy classifier; MacBook Syncthing runbook. |
| `docker-compose.syncthing-media.yml` | Adds a node-local media bind to an existing Syncthing service. | User guide; deploy classifier; storage-media-replication runbook. |
| `docker-compose.syncthing-tailnet.yml` | Syncthing SOCKS/tailnet routing overlay; requires established TCP forwarding. | `scripts/README.md`; deploy classifier; storage-media-replication runbook. |
| `docker-compose.headscale-macbook.yml` | Separate staged MacBook Headscale client; not a replacement for the app's transport. | Deployment guide; deploy classifier. |
| `docker-compose.headscale-mini.yml` | Separate staged Mac mini Headscale client. | Deployment guide. |
| `docker-compose.mini-relays.yml` | Caddy storage relay in the existing Mini Tailscale namespace. | Node-network-isolation test; mounts root `storage-proxy.Caddyfile`. No current script caller found. |
| `docker-compose.tailscale-serve.yml` | Retired: empty compatibility overlay after removing the TCP proxy. | `runbooks/2026-10-18-remove-syncthing-proxy.md` is historical. No executable caller found; deployment classifiers ignore the retired path. |

The moved `docker-compose.orbstack-test.yml` was referenced by
`test_orbstack_arr_connectivity.py`, `test_rawrarr_node_network_isolation.mjs`, and
the October 21, 25, and 26 environment-test runbooks. The two executable callers
now use `deploy/testing/compose.yml`; historical command transcripts remain
historical. The original October 21 runbook links to the current paths.
Regression snapshots already copy `deploy/`, including the relocated fixtures.
No exact root-file references were found in `.github/`; the regression fixture
copy is a glob rather than an exact filename caller.

## Consolidation decisions and next groups

Implemented: separate the isolated test environment first. Keep its explicit
`rawrarr-test-environment` project, service names, named volumes, networks, secret
paths, listeners, and image settings. Rebase relative paths, not runtime data.

Implemented next: retire the former Mini coordinator and empty Serve overlay
using the operator's confirmation and the audit evidence below. Keep the
MacBook transport overlay for its still-documented admin preview dependency.

Planned, not yet implemented:

1. Establish canonical role-specific bases under `deploy/compose/` for the
   coordinator and subscriber roles. Review the duplicated PostgreSQL/roarlay
   settings in the combined, coordinator-only, and former Mini stacks before
   choosing whether to share definitions. Similar text alone does not establish
   equivalent volumes, ports, enrollment, or transport behavior.
2. Group optional app and transport companions with documented base/overlay
   combinations. Keep analytics opt-in and private identity mounts role-specific.
   The two staged Headscale clients may share a template only if their hostname
   and persistent-state differences remain explicit.
3. Retire remaining obsolete Mac coordinator/transport definitions only
   after comparing host-local Compose labels, persistent resource names, and
   remaining tests/docs. Do not retain obsolete examples as the default install
   path. Decide whether the root combined stack remains a development entry point
   in that increment, rather than silently changing the default deployment.

Every group must update scripts, tests, documentation, regression snapshot
fixtures, and deployment-path classifiers together. Moving a file changes
Compose's relative-path base and can change default project/resource names:
compare resolved configuration with an explicit project, never just YAML text.
Do not use a directory move as a substitute for reducing duplicated behavior.

## Using the relocated test environment

From the repository root, validate without creating services:

```sh
docker --context orbstack compose --env-file /dev/null \
  -p rawrarr-test-environment -f deploy/testing/compose.yml config --quiet
```

The definition also defaults to `name: rawrarr-test-environment` to prevent a
directory-derived project. The integration runner resolves its path independently
of the invoking working directory and continues to pass `-p` explicitly. See
[Testing Rawrarr](../TESTING.md#isolated-coordinator-and-two-fossil-environment)
for prerequisites and the full integration procedure.

Existing deployments need no database, `/data`, or `/shared` migration. For the
test environment, replace the old `-f docker-compose.orbstack-test.yml` argument
with `-f deploy/testing/compose.yml`, preserving its project name, protected
environment settings and any additional overrides. The only resolved mount
changes are the two checked-in static fixture paths; runtime mounts are unchanged.
Do not use `down`, remove volumes, or adopt an old `rawrarr-orbstack-test` stack as
the current environment. If existing labels name the old file, update the
operator's command before the next deliberate recreation; moving source does not
reconfigure running containers. Old external command lines fail with a missing
file until updated; no compatibility wrapper is left at the root.

Rollback is a checkout of the prior published revision plus the prior file
argument, preserving the same project and volumes. No state was rewritten.
Actual starts/recreations must still obey the deployment locks and published-main
requirements; a successful `config` check is not a multi-node runtime test.

## Retired Mac coordinator definitions

On 2026-11-03 the operator explicitly confirmed the old Mac coordinator stacks
can be treated as retired. Removed `docker-compose.internal-mini.yml` and the
empty `docker-compose.tailscale-serve.yml`. This is a source-definition retirement,
not a remote teardown or an automatic migration to a new coordinator.

Read-only inspection of local OrbStack containers, including stopped containers,
found the internal subscriber uses host-local `rawrarr-node-internal/compose.yml`
and its image override. do3's current internal coordinator uses its host-local
internal-coordinator definition; public services use their separate production
files and accumulated overlays. Neither audit found the former Mini or empty
Serve definition in container labels. Remote audits of both Macs could not be
completed: the saved Mac mini key was unavailable, and Tailscale SSH stopped at
strict host-key verification. Retirement relies on operator confirmation for
that gap, not an assertion that all hosts were independently verified.

The analytics, billing-audience, and relay/direct-route tests now inspect the
do3 coordinator definition instead of keeping a duplicate obsolete stack alive
as a test fixture. Current coordinator/subscriber templates remain supported;
`docker-compose.yml` is not retired. The MacBook transport overlay remains
because `docs/admin-console.md` still combines it with the private admin preview.
Its replacement needs a separate caller migration before removal.

Existing do3 services and Mac subscriber deployments require no adoption action.
There is no persistent-state or protocol migration and no container recreation.
For external commands that explicitly include the empty Serve overlay, remove
only that `-f` argument and preserve the rest of the project's files/settings.
Commands for the old Mini coordinator must stop being used; replacing its file
with the do3 definition is **not** a safe database/passkey/Tailscale migration.
If an unexpected host still relies on the retired stack, stop the cleanup there
and inspect its labels and state before making a deployment change. Recover the
prior definition from Git commit `26c2a3f` if necessary; preserve all databases,
volumes, credentials, and tailnet identities. Do not run `down` or remove data.
