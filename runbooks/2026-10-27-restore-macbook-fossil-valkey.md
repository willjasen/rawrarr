# Restore the MacBook fossil Valkey service

## Objective and tracking

Restore the local Valkey service expected by the MacBook `fossil` subscriber. Valkey holds disposable ciphertext cache data only; the active shared config remains authoritative. This operational repair is tracked under issue [#100](https://github.com/willjasen/rawrarr/issues/100).

## Initial state

- The MacBook `rawrarr-node-internal` project had healthy `fossil` and Tailscale containers but no Valkey container.
- Its live `~/rawrarr-node-internal/compose.yml` defined only `fossil` and `tailscale`, and fossil had no `RAWARR_CONFIG_CACHE_URL`.
- Fossil used the published `main` image for revision `80d963bd1463429c5a760aa0e410d0abca89e67f`, reported as `linux/amd64`.
- The `fossil` deployment lock was clear before work began.

## Actions

- Atomically claimed the MacBook `fossil` deployment lock and backed up its Compose file to `~/Library/Application Support/Rawrarr/backups/pre-valkey-node-2026-10-27/compose.yml`.
- Added a `valkey/valkey:8-alpine` `artwork-cache` service to the existing subscriber project. Valkey shares the Tailscale network namespace, binds to loopback, enables protected mode, disables persistence, caps memory at 2 GB, and evicts least-recently-used keys. The service has a `PING` health check and no persistent volume because its cache is rebuildable.
- Configured fossil's `RAWARR_CONFIG_CACHE_URL` as `redis://127.0.0.1:6379/` and made fossil wait for a healthy Valkey service.
- Started Valkey, waited for its health check, then recreated only fossil with its existing published image. The Tailscale sidecar, node data volumes, shared volume, and other services were not recreated.
- Kept cache values private; no Valkey key names or values, config ciphertext, credentials, or age identities were read.

## Verification

- `docker compose -p rawrarr-node-internal -f compose.yml config --quiet` passed on the MacBook.
- The Valkey container reported healthy and `valkey-cli ping` returned `PONG`.
- Fossil reported healthy, its `/health` endpoint returned HTTP 200, and its configured cache URL points to loopback Valkey.
- Valkey contained four `rawrarr:config:*` keys, checked as a count only to confirm the config ciphertext mirror initialized.
- The fossil image remained the published `linux/amd64` image at revision `80d963bd1463429c5a760aa0e410d0abca89e67f`.
- Tailscale remained running with zero restarts. The fossil lock was released after verification.
