# Rawrarr device model

## Objective and initial state

Introduce a first-class concept for computers that hold media and run scanners, separately from the rawrarr web process. Today the web process owns the configured filesystem path and performs scans itself. Related context: [local library scan](2026-10-17-local-library-scan.md), [metadata matching](2026-10-17-movie-metadata-matching.md), and [Radarr UI configuration](2026-10-17-rawrarr-radarr-ui-config.md).

## Actions

- Added a device domain model and `GET /api/devices`; the current in-process scanner appears as `local-scanner`, named by `RAWARR_DEVICE_NAME`, with readiness based on the configured local library root.
- Added a Devices panel in the dashboard and documented the web-service/scanner-agent ownership boundaries, device/library/file identifiers, stale inventory behavior, and secure outbound pairing direction in [the device architecture](../docs/device-architecture.md).
- Documented that remote device registration and scan transport are not implemented yet; this UI/model increment does not claim remote connectivity.

## Verification

- `cargo fmt`, locked release build, Docker Compose config, and `git diff --check` passed.
- Local `/api/devices` response and browser dashboard show the local scanner identity and its setup state.
- No media root was configured or scanned; no remote agent request or device credential was created.

## Final state and remaining work

The app now has a visible local scanner device model and a documented plan for separating the web coordinator from media-host scanner agents. Next work is persistent device/library storage, one-time pairing and revocation, and an outbound remote scanner agent. The web service still performs the configured local scan itself.
