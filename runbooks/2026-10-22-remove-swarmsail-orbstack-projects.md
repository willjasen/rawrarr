# Remove Swarmsail OrbStack projects

## Objective and initial state

Remove the `swarmsail-local` and `swarmsail-e2e-2fef67d5767b` Docker Compose projects from OrbStack at the owner's request. The local Rawrarr subscriber (`fossil`) is a separate project and uses its own Tailscale sidecar. The main checkout contained unrelated modifications and merge conflicts, so work is isolated on branch `codex/remove-swarmsail-orbstack`.

The local Swarmsail project had five containers (four running, one stopped), with state stored in host bind mounts under `~/Library/Application Support/Swarmsail/`. Preserve these directories for future recreation. The E2E project had six running containers, three isolated Docker networks, and nine project-owned Docker volumes plus generated test files under `~/.local/state/swarmsail/e2e/`; these are disposable test state and are in scope for removal.

The required GitHub issue search and creation both failed with API errors (422 search validation and 404 create). Proposed issue text:

> Title: Remove inactive Swarmsail OrbStack projects
>
> Objective: Remove the `swarmsail-local` and `swarmsail-e2e-*` Docker Compose projects from OrbStack at the owner's request.
>
> Acceptance criteria: Remove containers and project-owned Docker networks for both named projects; preserve host bind-mounted Swarmsail configuration and data for future recreation; remove the disposable E2E volumes; verify the Rawrarr internal subscriber remains healthy; record scope and verification in this runbook.

## Actions

- Removed the five `swarmsail-local` containers: control plane, Caddy, local Headscale, CrowdSec, and stopped npmplus.
- Removed the local project's private and default Docker networks. Preserved the external `swarmsail-rawrarr-ingress` network; it is empty and may be reused when Swarmsail is recreated.
- Removed all six `swarmsail-e2e-2fef67d5767b` containers, its three isolated Docker networks, and its nine project-owned test volumes.
- Removed only the matching generated E2E run directory under `~/.local/state/swarmsail/e2e/`.
- Preserved the local Swarmsail host bind-mounted configuration and data under `~/Library/Application Support/Swarmsail/`, including Headscale and Caddy state.
- Left Rawrarr projects, containers, networks, and volumes untouched.

## Verification

- `docker compose ls --all` no longer lists either Swarmsail project.
- Docker container, network, and volume listings contain no objects for either named project; the E2E generated run directory is absent.
- Confirmed the retained host data directories for local Headscale, Caddy, and CrowdSec still exist.
- Confirmed `rawrarr-node-internal-fossil-1` remains running and healthy; its Tailscale sidecar remains running.
- Confirmed the external `swarmsail-rawrarr-ingress` network remains present with zero containers.
- No tests were run; this was a local container cleanup operation.

## Recovery and remaining work

The local Swarmsail project can be recreated from its Compose files and retained host data. The E2E stack can be recreated by rerunning the Swarmsail E2E harness, which will generate new isolated state. The external ingress network remains available. No GitHub issue was created because GitHub issue search/create returned API errors; the proposed issue text is recorded above.
