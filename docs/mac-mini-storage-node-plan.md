# Mac mini storage-node deployment plan

Status: planned, 2026-10-18. No remote deployment or drive inspection has been performed for this task.

## Confirmed scope

Deploy a subscriber storage node on the Mac mini using OrbStack. Its read-only media root is `/Volumes/silo/Media/data`. Join the existing Rawrarr instance, preserving its shared identity, configuration, and encryption recipient. The Mac mini gets its own device identity, local database, and sessions. It is not an operator service node.

## Placement

| Component | Mac mini location | Container location and access |
| --- | --- | --- |
| Media | `/Volumes/silo/Media/data` on the attached drive | `/media`, read-only |
| Shared application data | `~/Library/Application Support/rawrarr/shared-data` | `/shared`, writable replica of the existing instance |
| Metadata | Within that shared folder | `/metadata` aliases `/shared/metadata` |
| Device state | OrbStack named volume owned by `rawrarr-local` | `/data`, node-local |
| Authentication database | Separate OrbStack PostgreSQL named volume | Node-local; never synchronized |
| Database password | Protected Mac mini file outside the checkout | Compose secret, never printed or committed |

Use Compose project `rawrarr-local` on the Mac mini. The same project name on a different Docker engine does not join instances; shared-instance membership is a separate operation. Confirm the Docker context targets the Mac mini's OrbStack engine before any deployment command.

Publish the app on `127.0.0.1:7297`; use `http://localhost:7297` in the Mac mini browser with RP ID `localhost` and that exact WebAuthn origin. A browser on another machine interprets localhost as itself. Remote browser access needs a separately verified tunnel or HTTPS/origin plan; do not expose HTTP on the LAN or switch to an arbitrary OrbStack hostname.

## Current capability and gaps

The current `docker-compose.yml` builds from source; `docker-compose.media.yml` mounts one read-only root and sets `RAWARR_LIBRARY_PATH=/media`. A role flag such as `RAWRARR_MODE=storage`, published release images, and automated claim/enrollment remain planned. Do not configure a nonexistent role flag.

The scanner supports one mixed root, skips symlinks and hidden entries, and caps traversal at depth 64, 250,000 visited entries, and 10,000 media items. Check scan issues and truncation before treating results as a complete inventory. Encrypted sidecars go into shared metadata, never beside the media.

Joining the current instance means reusing its existing shared manifest and recipient, not generating an independent instance and merging it afterward. Folder synchronization is available, but portable passkey accounts, remote scan commands, scan reconciliation, and a combined cross-node catalog remain planned. Shared metadata arriving on another node does not establish that its UI can consume the Mac mini inventory. The initial milestone is a functioning local storage-node scan within the intended shared namespace; the complete combined catalog is a later implementation milestone.

## Deployment sequence

1. **Read-only host preflight.** Confirm the exact Mac mini, CPU architecture, macOS and OrbStack versions, Docker context, available internal disk space, existing containers/volumes, and port 7297 ownership. Check the attached volume identity and filesystem, actual mounted path, permissions, sleep/reboot behavior, and whether the selected root includes symlinks or unrelated content. No remote state has been verified yet.
2. **Protect existing state.** Inventory the current instance manifest/recipient without exposing private material. Take a consistent shared-folder backup and a PostgreSQL backup for each existing node being changed; retain the current image/source revision and Compose settings. Preserve all existing media, services, and browser windows. Do not clone `/data`, sessions, or a live PostgreSQL datadir onto the Mini.
3. **Prepare one-instance bootstrap.** Choose and verify the existing Syncthing peer/folder mapping, or set up the app-data-only transport if none exists. Bootstrap a consistent copy of the existing `/shared` while Rawrarr on the Mini is stopped; preserve the instance ID and public recipient. Keep live concurrent configuration writes disabled until conflict handling is checked. Never include `/media`, private identities, PostgreSQL, or `/data` in the sync folder. Use unique node-local storage so the Mini generates its own device ID. New-node passkey enrollment must be tested against its own local database; existing passkeys are not automatically replicated. The user supplies the existing age identity through the app's masked flow if unlock is required. Do not generate a replacement identity for this instance.
4. **Prepare deployment configuration.** Use a reviewed source revision with the base Compose file and media overlay, setting `RAWARR_HOST_MEDIA_PATH=/Volumes/silo/Media/data`, `RAWARR_DEVICE_NAME=Mac mini`, `RAWARR_LIBRARY_NAME=Mac mini media`, and `RAWARR_HOST_PORT=7297`. Prefer `RAWARR_LOCAL_PLAINTEXT_CONFIG=false` on this storage deployment. Provision or reuse the Mini's protected PostgreSQL password file; the documented local password-file path is only a convention until verified on that host. Do not copy or disclose a password from another machine.
5. **Guard the external mount before launch.** Add `bind.create_host_path: false` to the final media mount configuration before deployment; the current media overlay does not specify it. Verify both the volume identity and an expected existing media entry, not just that the directory exists. A directory can exist while the intended disk is absent. Confirm the rendered Compose mount is read-only and points to the exact root. This guard prevents automatic source-directory creation; it does not detect all disconnects or stale mounts.
6. **Start and validate.** On the Mini's OrbStack engine, validate the composed configuration without printing secret values, then build/start the explicitly named project. Verify PostgreSQL health, app container health, and `/health`. Sign in and unlock through the intended browser origin, then scan a bounded sample before attempting the full root. Confirm access as the app user (UID 10001), read-only mount metadata, expected files/counts, no unexpected access issues, and encryption of emitted sidecars. Do not test read-only enforcement by writing to real media.
7. **Verify shared-instance behavior separately.** Confirm matching instance IDs, distinct device IDs, and delivery of a test encrypted metadata record between nodes. Verify local passkey sign-in separately on both nodes. Record catalog limitations explicitly rather than treating successful Syncthing transfer as end-to-end catalog success.
8. **Exercise recovery.** First use a disposable fixture for missing-drive startup, remount, and app restart tests. Schedule any real drive disconnect test around other consumers of silo; do not unplug it during normal service use. Check that unavailable media is reported as unavailable rather than reconciled as deletion. This is a deployment acceptance requirement, not a claim that the current app implements that protection. Restart/recreate only the app if required after remount; revalidate paths before scanning. Verify a planned Mac reboot/login and OrbStack restart before calling the node unattended-ready.

## Acceptance and rollback

The local milestone requires a healthy app/database, successful passkey and age flow, a correct read-only scan of known media, persistent unique device identity across restart, and verified same-instance shared records. A full-library scan must disclose any cap/truncation. Distributed catalog use and unattended drive-loss handling remain incomplete until demonstrated or implemented.

If validation fails, stop the new Mini app and pause only its new sync relationship; retain its volumes and evidence. Do not run `docker compose down -v`, delete shared records, or change the existing node. Restore from the recorded backup only with synchronization paused and a reviewed recovery scope. No media rollback should be needed because its mount is read-only.

## Next decisions and implementation work

- Perform the Mac mini preflight and verify the existing shared-folder transport.
- Resolve the same-instance new-node authentication/bootstrap path before enabling normal shared writes.
- Add fail-closed mount configuration and a reusable dry-run-by-default deployment preflight if deployment automation is implemented.
- Test scanner performance and completeness on this library; implement pagination or larger-library handling if the current cap is reached.
- Implement missing-drive status/reconciliation protections and cross-node catalog consumption before promising unattended distributed operation.

## References

- [Node types](node-types.md), [device architecture](device-architecture.md), and [data domains](data-domains.md).
- [OrbStack volumes and mounts](https://docs.orbstack.dev/docker/file-sharing): macOS folders can be bind-mounted into containers.
- [Compose service volumes](https://docs.docker.com/reference/compose-file/services/#volumes): long bind syntax supports read-only mounts and `create_host_path: false`.
