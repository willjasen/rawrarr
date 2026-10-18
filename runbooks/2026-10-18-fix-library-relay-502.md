# Fix the encrypted library relay 502

## Objective and initial state

Restore the coordinator Movies library request after the browser reported HTTP 502. The metadata node was healthy as `rawrarr-local-metadata-recovery`, but the coordinator's configured target still used the Docker hostname `metadata`, which no longer resolved after metadata entered recovery mode.

## Actions

- Confirmed the coordinator could not resolve `metadata`; confirmed `rawrarr-local-metadata-recovery:8080/health` returned HTTP 200 from the coordinator's shared Tailscale network namespace.
- Recreated only the coordinator using the existing `rawrarr-local` Compose topology and configured the trusted `macbook-metadata` target to `http://rawrarr-local-metadata-recovery:8080`. Kept the storage target unchanged.
- Preserved the recovery metadata container, its existing `/data` volume, `/shared` mount, and node-token mount.

## Verification

- Coordinator, metadata loopback, and coordinator HTTPS health returned HTTP 200; coordinator and metadata containers reported healthy.
- Tailscale Serve still maps HTTPS 443 `/` to `http://127.0.0.1:8080`.
- Direct upstream health from the coordinator network namespace returned HTTP 200.
- Coordinator workspace Node checks passed (5/5); `git diff --check` passed.
- An authenticated browser library request was not available for a live end-to-end exercise during this repair. The corrected hostname and earlier focused POST-key validation establish the fixed network hop and relay allowlist behavior.

## Recovery and remaining work

The standard `rawrarr-local-metadata-1` container remains stopped with restart disabled. `rawrarr-local-metadata-recovery` continues to serve port 7297. Preserve it and its mounts until the standard metadata Compose service is safely restored with the original volume and secret mounts. The coordinator currently targets the recovery hostname; update that target when the normal metadata service returns.
