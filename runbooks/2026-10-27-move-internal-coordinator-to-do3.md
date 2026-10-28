# Move the internal coordinator to do3

## Objective and issue

Move the internal coordinator from Brandon's Mac mini to do3 beside the public Rawrarr instance. Keep the public deployment and public Caddy routing unchanged, preserve the internal database and tailnet URL, and expose the new internal coordinator only through Tailscale. Tracked by [issue #246](https://github.com/willjasen/rawrarr/issues/246).

## Design

- The public Rawrarr service is `rawrarr-roarlay` at `rawrarr-roarlay.risk-mermaid.ts.net`; its Compose project remains `rawrarr-roarlay-prod`.
- The internal deployment uses the separate Compose project `rawrarr-internal-do3`, separate PostgreSQL and UI volumes, a private `control` network, and its own Tailscale sidecar.
- The internal Tailscale sidecar receives the existing `rawrarr` Tailscale state during cutover, preserving `rawrarr.risk-mermaid.ts.net` and the WebAuthn origin/RP ID.
- No coordinator port is published on a public interface. Port 7299 is loopback-only for host diagnostics; Tailscale Serve forwards HTTPS 443 to the app's `127.0.0.1:8080` inside its network namespace.
- The internal Compose project mounts only service secrets and `/ui`; it does not mount subscriber `/shared`, `/media`, or `/data`.

## Migration source and preservation

- Build or pull only the immutable image for the exact published `main` SHA recorded below; verify its architecture is `linux/amd64` and its OCI revision label matches.
- Before cutover, create and verify a PostgreSQL dump, copy the approved-node registry and required service secrets through authenticated SSH, and copy the Tailscale state into a new host-local volume. Do not place these runtime files in the repository.
- Keep the Mac mini database, service configuration, and original Tailscale volume intact for rollback. Stop the old coordinator and Tailscale sidecar only after the do3 database restore and local health check pass.
- The public Rawrarr PostgreSQL volume, public roarlay, Headscale, Tailscale, Stripe service, and Caddy configuration are outside this migration.

## Execution and verification

_To be completed with the published revision, backup checks, migration results, endpoint checks, and rollback state._

## Rollback

If do3 fails verification, stop only `rawrarr-internal-do3-roarlay` and `rawrarr-internal-do3-tailscale`, keep their data volumes, then atomically claim the Mac mini `roarlay` and `tailscale` deployment locks and restart those two services using the original volumes and Compose files. Confirm `tailscale serve status --json` routes `rawrarr.risk-mermaid.ts.net:443` to `http://127.0.0.1:8080` and HTTPS `/health` identifies roarlay before releasing the locks.
