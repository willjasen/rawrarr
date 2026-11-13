# Pause the standalone admin-console preview

Issue: [#459](https://github.com/willjasen/rawrarr/issues/459).

## Decision

Pause the standalone sample admin-console preview instead of changing its
container user. The Docker default-root finding applied to `admin/Dockerfile`,
not to the Rawrarr coordinator or subscriber image. The preview has no live
account connection, but there is no current need to run it.

## Changes and adoption

- Removed `admin/Dockerfile` and `docker-compose.admin.yml`, eliminating the
  preview's build and launch paths.
- Retained static mockup files as inactive design source; no application serves
  them.
- Updated the admin-console design, deployment guidance, and Compose inventory
  to describe the paused state and the requirements for reactivation.
- No schema, persistent data, secret, or protocol changes; no migration is
  required.
- A source update does not stop containers created from an older checkout.
  Inspect labels and stop only an existing standalone preview on any host; do
  not touch coordinator, subscriber, or Tailscale services.

At implementation time, the local OrbStack context had no container named
`rawrarr-admin`, including stopped containers. Other hosts were not inspected.
Reactivation requires a reviewed non-root image and a deliberately restored
launch path; do not restore the removed root-running Dockerfile.

## Verification

- `node --test scripts/tests/test_rawrarr_admin_preview.mjs` in the existing
  `rawrarr-testing-base:local` image with copied source and network disabled
- `python3 -m unittest discover -s scripts/tests -p test_compose_layout.py`
