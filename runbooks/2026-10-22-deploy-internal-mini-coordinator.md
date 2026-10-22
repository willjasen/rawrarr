# Deploy internal Mac mini coordinator

## Objective and starting state

Deploy a private Rawrarr coordinator on the Mac mini, reachable by nodes on the owner's Tailscale tailnet. The mini already runs a separate Rawrarr metadata subscriber and has a native Tailscale identity with HTTPS Serve handlers on ports 3875, 8096, and 8384. Port 7297 is owned by the existing subscriber sidecar. Its separate Rawrarr Tailscale sidecar is registered to an offline Headscale control server. Preserve that project, its volumes, and all existing Serve handlers.

The mini has no `tmux`; remote work proceeds over ordinary SSH as the user requested. The coordinator uses a new `rawrarr-internal-mini` Compose project, new PostgreSQL and Tailscale state volumes, and loopback diagnostics on port 7298. A dedicated userspace Tailscale sidecar named `rawrarr` shares its network namespace only with roarlay; Serve will publish `https://rawrarr.risk-mermaid.ts.net/` to `http://127.0.0.1:8080`. The RP ID is `risk-mermaid.ts.net`. Tailscale authentication uses a protected auth-key file on the mini; the key must be entered by the owner directly and never copied into Compose or chat.

## Work performed

- Added `docker-compose.internal-mini.yml` for a distinct `rawrarr-internal-mini` project, with isolated PostgreSQL and Tailscale state volumes. Only roarlay shares the Tailscale namespace; the existing subscriber remains separate.
- On the Mac mini, archived source from the task-owned Git HEAD to `~/Library/Application Support/rawrarr/internal-coordinator/source`. Created a random PostgreSQL password and approved-node registry under its protected `secrets/` directory (both files mode 600). The registry reuses the existing mini metadata node's verifier; its token was hashed inside the container and never printed or copied.
- Built `rawrarr:internal-mini`; started only the new project's `postgres` and `roarlay` services. The existing `rawrarr-subscriber` project, volumes, and native Tailscale Serve configuration remain untouched.
- Remote `tmux` is unavailable; SSH commands ran directly as requested by the user.

## Verification

- The new project's PostgreSQL is healthy; roarlay is healthy with zero restarts and logs `rawrarr coordinator listening on 0.0.0.0:8080`.
- `http://127.0.0.1:7298/health` on the mini returned HTTP 200.
- The mini native Tailscale identity and its Serve handlers on ports 3875, 8096, and 8384 are unchanged. The dedicated sidecar authenticated from the owner-provided `~/tailscale-authkey` file, with the local file restricted to mode 600 and the mini copy stored mode 600. Tailscale reports `rawrarr.risk-mermaid.ts.net.` online. Serve JSON confirms HTTPS port 443 `/` proxies to `http://127.0.0.1:8080`. HTTPS `/health` returned HTTP 200 from both the mini and the deploying workstation and identified Roarlay. The coordinator is healthy with zero restarts. The mini native Serve handlers remain on 3875, 8096, and 8384; host diagnostics remain bound to `127.0.0.1:7298`.

## Recovery and remaining work

The existing boneyard/metadata node is not migrated to the new coordinator or to the owner's Tailscale tailnet by this deployment. Do not stop the existing subscriber project or remove its volumes. If coordinator startup or Serve verification fails, stop only `rawrarr-internal-mini` services and preserve its database volume and secret files for diagnosis. Existing HTTPS Serve ports must remain configured. The task-owned deployment markers were removed after health and tailnet route verification. The existing boneyard/metadata node is not migrated to the new coordinator or to the owner's Tailscale tailnet by this deployment. Preserve the coordinator database, Tailscale state, and secret files during future maintenance.
