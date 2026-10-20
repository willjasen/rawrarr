# One-line subscriber node deployment plan

**Status:** Planned; this document does not implement or publish an installer.
**Date:** 2026-10-20

## Goal and current foundation

After preparing a node in Rawrarr, let the owner copy one curl-based command to that node's terminal. It downloads a deployment-specific script containing the generated Compose definition, checks or helps install Docker and Compose, writes the local deployment files, and starts the node. One line starts a guided installation; it does not imply permission to change the host without confirmation.

The authenticated `/setup/node` preview already generates fossil (`metadata`) and boneyard (`storage`) Compose files in `rust-arr/web-export/subscriber-compose.js`. It downloads YAML, uses `rawrarr:local`, publishes `127.0.0.1:7297`, and requires manual Docker/image preparation. Joining nodes generate their own credential and require explicit owner approval in roarlay. Public images, installer delivery, and automatic Docker setup are not implemented.

## Proposed owner experience

1. In **Set up a node**, choose fossil or boneyard and review the coordinator, role, image release, local storage, and host changes. Keep **Download Compose** and add **Copy install command** plus **Download/review script**.
2. Run the copied command on the destination node, not on roarlay. Fetch the complete script successfully before executing it; do not stream a partially downloaded script into a privileged shell.
3. The script detects the platform, Docker Engine availability, daemon access, Compose v2, image architecture, coordinator reachability, and port conflicts. Explain any required action before changing the machine.
4. Reuse a working Docker installation. If Docker is absent on a supported Linux host, offer an explicit installation confirmation. On desktop hosts, guide the owner through the supported Docker installation/startup flow and allow a safe rerun.
5. For boneyard, ask for the absolute media directory locally and verify that it exists and is readable. Show the deployment summary and obtain confirmation before starting containers.
6. Save the Compose file and local deployment manifest, pull the pinned image, validate Compose, and start only this deployment. Show progress and actionable failures.
7. Report **Started; waiting for owner approval**, with the trusted coordinator URL. The owner signs in to roarlay and approves the pending node. Report application readiness separately from container liveness; do not call an unapproved node fully configured.

Illustrative one-line command shape only; the reserved `.invalid` hostname below is not a working installer:

```sh
curl --fail --silent --show-error --proto '=https' --tlsv1.2 'https://install.rawrarr.invalid/deployments/<download-id>/install.sh' -o rawrarr-install.sh && bash ./rawrarr-install.sh
```

The final copied command must use a collision-safe local download path, avoid overwriting an existing file or following a local symlink, quote generated values, and propagate download/installer failure. Keep prompts on the terminal and fail explicitly if interaction is required but unavailable. Do not prepend `sudo` to the whole script. A separately documented unattended mode may come later.

## Script generation and delivery

- Reuse the existing subscriber Compose generator and its role/input validation. Generate the manual YAML and embedded script from the same deployment model, not independently maintained templates. Continue rejecting roarlay/coordinator, PostgreSQL, and operator-service deployments.
- Embed the Compose definition as literal data in the script, using a collision-safe quoted heredoc or an equivalently safe encoding. Never evaluate YAML, concatenate user input into executable shell, or use `eval`. Handle shell metacharacters, newlines, quotes, Unicode, and Compose `$` interpolation deliberately.
- Keep host media paths out of hosted artifacts. The embedded definition must support a validated node-local media-path input; resolve it locally into the final Compose configuration without shell evaluation or unsafe text replacement. Manual Compose download can continue to include a path chosen locally in the browser.
- Proposed delivery is a short-lived HTTPS download created by the authenticated setup flow. The server renders a vetted installer from an allowlisted deployment model; it must not host or execute arbitrary browser-supplied shell. Allow only operational setup fields already needed for enrollment, such as generated node ID, role, coordinator origin, release, and Compose project identifier.
- The download identifier grants access only to this non-secret setup artifact, never enrollment approval, an account session, or node authority. Apply expiry, explicit invalidation, issuance/download limits, bounded storage, and `Cache-Control: no-store`. Redact download identifiers from access logs and analytics. Permit bounded retries until expiry; download alone must not consume approval or prevent recovery after a failed transfer.
- Do not upload private age identities, encrypted root backups, root private keys, passkeys, credentials, subscriber configuration/content, or user-chosen device names to the installer distributor. Do not persist setup artifacts in `/shared` or relay subscriber files to serve them. The short-lived record contains only the allowlisted non-secret deployment parameters.
- Version the installer and deployment schema, pin application images by immutable digest, and show the selected release before execution. Publish release provenance and verification instructions; a checksum delivered by the same server is corruption detection, not independent publisher authentication. Do not advertise arbitrary generated scripts as release-signed artifacts.
- HTTPS certificate validation is required. Reject redirects to unapproved origins and never use `curl -k`. Establish how the destination reaches the download origin and coordinator before offering a working command: a private tailnet URL cannot install its own missing network access.

The download origin, endpoint contract, expiry, and supported release versions must be finalized during implementation. Do not publish a copy button until its actual URL works from supported destination nodes.

## Docker and platform policy

| Destination | Planned first-release behavior |
| --- | --- |
| Debian/Ubuntu Linux, supported releases | Detect architecture and package conflicts. With consent, use Docker's documented signed package repository to install Engine and Compose v2 and enable/start the daemon where supported. Publish an exact tested OS/version matrix before release. |
| macOS | Reuse working Docker Desktop or OrbStack with Compose v2. If missing or stopped, give explicit installation/start instructions and resume on rerun; do not silently install licensed desktop software or alter host integrations. |
| Windows | Keep manual Compose deployment available initially. A Bash command is not a native PowerShell installer; test and document a specific WSL2 + Docker Desktop path before advertising one-line support. |
| Other/unsupported systems | Exit before mutation with requirements and manual deployment instructions. No speculative package-manager fallback. |

Check the Docker client, server, selected context, permissions, and minimum supported Compose version independently. Refuse a remote Docker context in the first release: media paths, loopback checks, and node-local state must refer to the destination host.

Install only missing prerequisites after confirmation; do not replace a healthy engine, remove competing packages, restart another workload's daemon, modify firewall rules, or add the owner to the root-equivalent Docker group automatically. Explain requested privilege escalation and restrict it to approved package/service steps. If a logout, reboot, desktop GUI action, or administrator decision is required, stop with a resumable next step rather than claiming success.

## Local deployment and data contract

- Use an owner-controlled installation directory outside the source checkout, synchronized folders, and media. Apply restrictive permissions to local files. Persist a small manifest recording node ID, role, project, image digest, installer version, and managed paths, but no credentials.
- Use a stable, explicit Compose project per deployment and reuse it on rerun. On a fresh supported host, publish only `127.0.0.1:7297:8080` and retain `http://localhost:7297` as the matching local WebAuthn origin. If another deployment/process owns the port, identify the conflict and stop; never choose another port, stop that workload, or take over an existing `rawrarr-local` project.
- Keep `/data` node-local and persistent, `/shared` in its dedicated application-data volume, and `/media` a boneyard-only, read-only bind with missing-path creation disabled. Fossil has no media mount. Do not install PostgreSQL, roarlay, or an operator service on the subscriber's behalf.
- Let the node create and retain its credential in `/data`; never place credentials in the script, command URL, manifest, YAML, logs, or shell history. Age identity provisioning remains a separate explicit local action, not a bootstrap prerequisite.
- Inspect and validate the resolved Compose definition before starting it. Pull the selected release image and use scoped `docker compose -p <project> -f <file> up -d`, without a source build, `down -v`, orphan removal, or changes to other Compose projects.
- Use an atomic node-local installer lock, atomic file replacement, and compatibility checks for reruns. Preserve node identity, credentials, volumes, and local edits. If an existing installation cannot be identified confidently, stop for review rather than adopting or overwriting it.
- Do not configure Syncthing peers, auto-accept folders, unlock a vault, mount an age identity, or approve enrollment. Network reachability and successful script download confer no trust. Combined dino roles and tarpit remain separate future work.

## Failure handling, readiness, and lifecycle

Use bounded network/health waits and explicit phase output: preflight, prerequisite setup, files saved, image pulled, container started, awaiting approval, and ready. A successful `/health` response proves only liveness. Verify the expected node ID/role and authenticated enrollment state before claiming it is the intended usable node; show coordinator-unreachable, pending, denied, expired, and revoked states without exposing secrets.

On interruption or failure, return nonzero and identify the failed phase, preserved deployment directory, and safe rerun/log command. Do not automatically remove an installed Docker engine, erase volumes, mint a new identity, or approve a replacement node. Expired script downloads can be regenerated for the same pending deployment without creating duplicate node IDs; revoked nodes still require the explicit re-enrollment policy.

Initial installation must not silently become an upgrade on rerun. If versions or local configuration differ, explain the difference and stop for an explicit update decision. Save the prior managed Compose/manifest before an approved update, preserve volumes, and document release-specific rollback compatibility. Stopping/removing containers must be distinct from an explicitly confirmed destructive data-removal operation. No automatic host Docker uninstallation.

## Delivery phases

1. **Release prerequisites:** publish tested Linux `amd64`/`arm64` application images, immutable digests, provenance, and upgrade guidance. Replace the `rawrarr:local` preview dependency; keep the preview labeled until this is complete.
2. **Shared deployment model and embedded script:** extend the current generator, keep the manual YAML path, define local path handling and manifest/rerun behavior, and test both fossil and boneyard output.
3. **Host bootstrap:** implement the tested Linux path and existing-Docker macOS path, scoped privilege consent, Docker/context checks, locking, port/media preflight, persistent file writes, pull, validation, and startup.
4. **Setup UI and download delivery:** add review/copy actions, authenticated artifact issuance, restricted rendering, expiry/invalidation, redacted logging, and real destination-network verification.
5. **Enrollment and recovery:** wire accurate pending/readiness reporting, safe retries, and recovery instructions without bypassing current approval. Future multi-account claims must use owner-issued invitations rather than a node-supplied user ID.
6. **Release gate:** complete the acceptance matrix below, document exact supported environments, and only then advertise one-line deployment.

## Acceptance criteria and planned evidence

Add focused tests under `scripts/tests/` and update their index. Extend `test_rawrarr_subscriber_compose.mjs` and `test_rawrarr_subscriber_onboarding_browser.mjs` where appropriate.

- On clean disposable supported Linux hosts without Docker, one copied line prompts for approved host changes, installs prerequisites, and starts each supported role using a published image. Repeat on existing-Docker Linux and macOS hosts without changing other workloads.
- Test missing client/plugin, stopped daemon, denied permissions/sudo, remote contexts, unsupported OS/architecture, package conflicts, absent terminal, reboot-required cases, occupied 7297, and missing/unreadable media mounts. Fail before unrelated mutation; never silently select another port or create an empty media directory.
- Validate every rendered Compose shape with Compose v2. Exercise paths with spaces, quotes, `$`, and shell metacharacters; reject invalid control characters. Verify no supplied value executes as shell and that interpolation preserves the exact intended path.
- Exercise cancelled confirmation, partial downloads, TLS failures, altered artifacts, wrong role/ID, expired downloads, rate limits, concurrent runs, interrupted pulls/startup, and failed coordinator access. No download failure may execute a stale or partial script.
- Verify repeat runs preserve IDs, credentials, data, project ownership, and local edits. Confirm explicit upgrades preserve volumes and failures leave a documented recovery path.
- Verify only the allowlisted role runs, the listener stays loopback-only, boneyard media stays read-only, fossil has no media, and no coordinator/PostgreSQL service appears.
- Demonstrate pending approval, owner approval, denial/expiry/revocation, restart continuity, and coordinator outages against an isolated test coordinator. Confirm `/health` alone never produces a fully-ready claim and no peer/folder is auto-approved.
- Inspect generated commands/scripts, download records, logs, Compose files, and manifests for forbidden secrets, host media paths in hosted artifacts, and subscriber content. Verify wrong-owner artifact invalidation and protected issuance, while anonymous downloads grant no account or node permissions.

No installer, Docker installation, live node deployment, or support-matrix verification is performed by this planning change.

Related: [container deployment](container-deployment.md), [coordinator linking](coordinator-linking-plan.md), [node types](node-types.md), and [roadmap](../ROADMAP.md).
