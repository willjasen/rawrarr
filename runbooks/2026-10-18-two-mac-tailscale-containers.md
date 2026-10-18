# Plan Tailscale containers on both Macs

## Objective

Plan separate Tailscale containers on the MacBook Pro and Mac mini, with the MacBook container using Tailscale Serve for the Rawrarr coordinator. No tailnet or container configuration is changed in this planning task.

## Initial state

- MacBook checkout contains separate coordinator and metadata Compose roles; the coordinator's internal Docker listener is port 8080 and its diagnostic host port is loopback-only 7298. Metadata remains at loopback 7297.
- The user reports OrbStack was started on the Mac mini. Read-only SSH preflight confirmed macOS 27.0.1, ARM64, OrbStack Docker context, Docker 29.8.2; no containers were running during that check.
- Existing Tailscale integration documentation described optional transport but did not state the explicit requirement to use dedicated containers on both Macs and avoid native Tailscale clients.

## Plan recorded

- Add one independently enrolled Tailscale node per Mac, each with a unique stable hostname and persistent node-local state directory.
- Keep `/shared`, media, PostgreSQL, passkeys, age identities, and Rawrarr node tokens outside Tailscale state and enrollment material.
- On the MacBook, connect the Tailscale container to the coordinator over a dedicated internal Docker network and use Serve to expose only the coordinator over HTTPS within the tailnet. Preserve the local metadata UI and do not enable Funnel.
- On the Mini, connect the Tailscale and storage containers over an internal Docker network. Storage reaches the coordinator at its HTTPS Serve name; its media mount remains read-only and local.
- Before implementation, validate the Serve sidecar's ingress networking under OrbStack. Prefer default userspace mode unless testing demonstrates it cannot provide required Serve ingress; kernel mode has extra TUN/capability requirements and should not be enabled without need.
- Enroll both nodes interactively or through protected secret injection. Keep authentication material out of Compose text, Git, logs, and `/shared`.
- Verify from a second tailnet device that the coordinator endpoint uses valid HTTPS and is private; verify HTTPS and coordinator health from inside the Mini storage container; then provision the Mini's distinct Rawrarr node credential and test restart/outage behavior.

## Verification and remaining work

Confirmed the mini's OrbStack Docker engine responds over SSH. Documentation diff and whitespace checks remain to be run. Actual Tailscale enrollment, Serve configuration, storage node provisioning, and end-to-end network verification remain unperformed and require a separate deployment task.

## References

- [Tailscale Docker parameters](https://tailscale.com/docs/features/containers/docker/docker-params) — persistent `TS_STATE_DIR`; userspace default and TUN/capability requirements for kernel mode.
- [Tailscale Serve](https://tailscale.com/docs/reference/tailscale-cli/serve) — tailnet service proxying and configuration.
- [Two-host topology plan](../docs/tailscale-integration-plan.md#two-host-container-deployment-2026-10-18).
