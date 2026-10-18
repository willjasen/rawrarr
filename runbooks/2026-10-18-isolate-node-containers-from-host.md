# Isolate metadata and storage containers from host access

## Objective and initial state

Prevent host processes on the MacBook and Mac mini from connecting directly to the metadata or storage app. Initially both apps published `127.0.0.1:7297`, and both were also reachable by connecting to their OrbStack container IPs from the host. The coordinator used a Docker bridge address for metadata; Tailscale Serve used a Caddy proxy for storage.

## Actions

- Removed metadata/storage host-port mappings from the Compose definitions.
- Moved the MacBook metadata process into the existing Tailscale sidecar network namespace, bound it to `127.0.0.1:8081`, and configured the coordinator to relay to that shared loopback address. Configured metadata's coordinator heartbeat to use the sidecar SOCKS5 listener and corrected its mounted node-token path.
- Moved the Mac mini storage process into its Tailscale sidecar network namespace and bound it to `127.0.0.1:8080`. Kept Tailscale Serve's HTTPS root handler on a loopback-only proxy at `127.0.0.1:8082`, explicitly using HTTP for the proxy listener.
- Bound the storage proxy to loopback inside the Tailscale namespace and its upstream to storage loopback. Kept the separate coordinator proxy and existing Tailscale Serve handlers.
- Preserved `/data`, `/shared`, the read-only `/media` mount, PostgreSQL, node tokens, Tailscale state, and the coordinator database.

## Verification

Compose validation passed for the MacBook stack and both Mac mini deployment files. The MacBook metadata and coordinator containers are healthy; metadata health succeeds inside its shared namespace. Host connections to MacBook `127.0.0.1:7297` and sidecar IP `192.168.147.3:8081` are refused, and the obsolete recovery container was removed without deleting its shared volume. On the Mac mini, storage health succeeds on namespace loopback; host connections to `127.0.0.1:7297` and sidecar IP `192.168.97.2` ports `8080` and `8082` are refused. Mini Tailscale `/health` and coordinator `/health` both return HTTP 200. Read-back confirms MacBook Serve `/` proxies to `http://127.0.0.1:8080` and Mini Serve `/` proxies to `http://127.0.0.1:8082`. The coordinator root returns HTTP 200 and the sign-in document to an unauthenticated HTTP probe. Chrome automation was blocked by an open extension UI, so the authenticated movie list was not inspected in this task. Focused isolation tests pass 3/3 and `git diff --check` passes.

## Recovery and remaining work

If the coordinator or tailnet path fails, inspect the sidecar SOCKS listener, namespace sharing, and loopback ports before restoring access. Do not republish a metadata or storage port on the host. Keep deployment markers outside synchronized shared data and clear them only after route and health verification.
