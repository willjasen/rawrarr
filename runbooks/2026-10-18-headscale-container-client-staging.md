# Headscale container client staging

> Historical experiment, not a current deployment procedure. The operator
> confirmed both staged clients obsolete on 2026-11-03. Their Compose definitions
> and the MacBook coordinator overlay are retired. The internal coordinator stays
> on do3; do not replay the old enrollment or Mac coordinator-route instructions.
> No live sidecar, state, or enrollment was removed. See the
> [retirement record](2026-11-03-retire-staged-headscale-compose.md).

## Objective and initial state

Connect Docker Tailscale clients on the MacBook and Mac mini to the Swarmsail Headscale instance at `https://rawrarr.willjasen.net`. Do not alter native Tailscale on either Mac. Use `https://swarmsail.willjasen.net` for Caddy-routed coordinator calls; do not use Tailscale Serve with Headscale.

The existing app client states were backed up and verified before this transition:

- MacBook: `~/Library/Application Support/rawrarr/backups/tailscale-identities-20261018T222233Z/macbook`
- Mac mini: `~/Library/Application Support/rawrarr/backups/tailscale-identities-20261018T222233Z/mac-mini`

Use separate new state directories named `tailscale/headscale-macbook` and `tailscale/headscale-mini`, preserving the original directories for rollback. Headscale is v0.29.5 and has the existing Rawrarr account user. Use a separate, single-use, short-lived pre-auth key for each client, passed only through a mode-600 node-local file and a temporary Compose override. The first connection uses a separate `rawrarr-headscale` Compose project so the existing app sidecars and their active Tailscale Serve and Syncthing routes stay intact until their replacement paths are ready.

Completed on 2026-10-18: the MacBook and Mac mini Headscale sidecars are online as `rawrarr-headscale-macbook` (`100.100.25.129`) and `rawrarr-headscale-mini` (`100.100.25.130`). Both pre-auth keys and temporary Compose overrides were removed after enrollment; the enrolled sidecars were recreated from their persistent Compose files and verified to have no `TS_AUTHKEY` environment setting or temporary secret mount. Both nodes see one peer, and bidirectional `tailscale ping` succeeds over DERP region NYC. Direct connectivity was not established. Neither host-level Tailscale client was changed.

The Mac mini storage service now calls the coordinator at `https://swarmsail.willjasen.net` directly over the LAN, with its SOCKS5 override removed. Its existing `/data` volume and old app Tailscale namespace were preserved. The container returned HTTP 200 from `/health` and reached `swarmsail.willjasen.net` after recreation; it is healthy. The coordinator database reports its `mac-mini-storage` row online after the restart, confirming an application heartbeat through Caddy. Its current network identity remains on the previous Tailscale tailnet until the app-sidecar migration is completed.

## Caddy coordinator ingress

Caddy proxies `swarmsail.willjasen.net` to `coordinator:8080` over the external bridge `swarmsail-rawrarr-ingress`. Only the Swarmsail Caddy container and the MacBook Rawrarr Tailscale sidecar join this network. Keep it separate from `rawrarr-local_default`, which contains the rest of the application. The coordinator call URL for storage containers is `https://swarmsail.willjasen.net`. The DNS-only record points to the MacBook LAN address `192.168.128.210`; clients on the Mini LAN reached the endpoint successfully.

The existing coordinator WebAuthn origin and RP ID remain `https://rawrarr.risk-mermaid.ts.net` and `rawrarr.risk-mermaid.ts.net`. A passkey created for that RP ID cannot sign in at `swarmsail.willjasen.net`; perform passkey migration before retiring the old browser origin. Health/API reachability at the new hostname does not verify browser authentication.

## Enrollment procedure

Before recreating a sidecar, verify its existing identity backup and claim the corresponding node-local container deployment marker. Create a mode-700 new state directory without copying old state into it. Create a distinct Headscale pre-auth key with a short expiry, write its value directly from CLI JSON into a mode-600 local file without printing it, and mount it read-only into the sidecar as `TS_AUTHKEY=file:/run/secrets/headscale-preauth-key` using a temporary Compose override. Set `TS_EXTRA_ARGS=--login-server=https://rawrarr.willjasen.net`, `TS_AUTH_ONCE=true`, and point `TS_STATE_DIR` at the new directory. Recreate only the Docker sidecar; do not run `tailscale up` on the Mac host.

After enrollment, remove the temporary key file and override. Confirm each new container's `tailscale status` reports the Headscale control server and the node appears under the Rawrarr Headscale user. Repeat separately on the other Mac. Verify peer connectivity between the two Headscale clients and coordinator health over Caddy. The original app sidecars remain on their existing tailnet until the WebAuthn origin and Syncthing direct-path migration are prepared.

## Verification and rollback

Check the Caddy certificate and DNS-only record for `swarmsail.willjasen.net`, confirm `https://swarmsail.willjasen.net/health` returns HTTP 200 from both Headscale client containers, and verify coordinator calls from the Mini storage service. Confirm exactly two intended Headscale nodes, connected state, and a successful peer request. Verify both native Mac Tailscale states remain unchanged.

If an enrollment or route check fails, stop only the corresponding `rawrarr-headscale` sidecar and retain its new state for diagnosis. The original Rawrarr sidecars and state directories remain untouched. Revoke any unused pre-auth key and remove only the task's temporary secret file. The old identity backups remain available for a later in-place migration or rollback.

## Remaining work

The Headscale Caddy ingress at `rawrarr.willjasen.net` remains the control-plane endpoint. The selected `swarmsail.willjasen.net` coordinator route uses the MacBook LAN address and is LAN-reachable only. The original Rawrarr app sidecars remain on the previous tailnet; their HTTPS Serve and Syncthing TCP routes are still in use. Browser passkey migration to the new RP domain and a direct Syncthing path between the Headscale clients are still required before replacing those app sidecars.
