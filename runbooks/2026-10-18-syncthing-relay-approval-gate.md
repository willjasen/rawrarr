# Syncthing public relay fallback and node approval gate

## Objective and initial state

Use Syncthing's public relay service as the fallback when direct peer connections fail, remove the Mini SOCKS5 dependency and the MacBook Tailscale Serve TCP forward, and document that production storage and metadata nodes must be approved before Syncthing is configured. Relevant design context: [Mini storage-node plan](../docs/mac-mini-storage-node-plan.md), [container deployment](../docs/container-deployment.md), and [Tailscale integration](../docs/tailscale-integration-plan.md).

The MacBook and Mini Syncthing instances were paired for an operational relay test. This test pairing does not authorize using automatic Syncthing setup for subsequent production nodes. Production order is enrollment, explicit owner approval, then deliberate peer/folder configuration. PostgreSQL stays on the coordinator's private Docker network and is never exposed through Tailscale.

## Actions

- Enabled global discovery, NAT traversal, and the public relay pool on the two test Syncthing instances; changed the test peer address to `dynamic` so it can use discovery instead of a fixed Tailscale address.
- Removed the Mini Syncthing SOCKS5 proxy environment dependency and its Mini Tailscale sidecar. Preserved the sidecar's persistent state directory and unrelated containers/volumes.
- Removed only the MacBook Serve TCP 22000 forward to Syncthing. Preserved the HTTPS coordinator handler.
- Updated the deployment/design docs to gate peer/folder setup on explicit owner approval. The current enrollment flow does not automatically provision Syncthing peers; the production gate is documented as a required manual step until an approved, tested automation exists.

## Verification

The MacBook Syncthing log reported a `relay-client` connection after restart. The local Syncthing status endpoint showed the peer connected, and its relay-enabled settings were read back. The MacBook Tailscale Serve status retained the coordinator HTTPS handler and no longer had the Syncthing TCP handler. The coordinator HTTPS health route returned successfully after a transient restart response. The Mini Syncthing container was healthy after recreation without SOCKS5 configuration.

The final Tailscale Serve read-back retained only HTTPS 443 to `http://127.0.0.1:8080`, and the tailnet `/health` request returned HTTP 200 with the coordinator page. The Mini Syncthing container reports healthy and its environment has no SOCKS proxy variable. A fresh authenticated Syncthing API query could not be completed: the local GUI API rejected the config-derived key on the MacBook and was not listening on the Mini loopback at the attempted address. The earlier MacBook log evidence of `relay-client` remains the live proof of relay use; current connection type can change dynamically. A successful folder state is not proof that a production node was owner-approved; production onboarding must use the order above.

## Recovery and remaining work

Protected pre-change Syncthing config and deployment file backups are stored outside the repository in the respective Macs' Application Support directories. Restore only the specific backed-up configuration if needed; do not reset Serve globally or remove persistent Syncthing state. No PostgreSQL settings or routes were changed.

Automatic post-approval Syncthing provisioning is not implemented. Any future automation must bind the approved Rawrarr node identity to verified Syncthing device IDs, share only the intended folder, and reject unapproved peers/folders.
