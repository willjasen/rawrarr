# Set up MacBook Syncthing for review

## Objective and initial state

Deploy only the MacBook Rawrarr Syncthing companion and let the owner review it before any Mini work. Related: [shared-folder plan](2026-10-18-plan-two-mac-syncthing.md).

Verified OrbStack context, the current Rawrarr `/shared` bind at `~/Library/Application Support/rawrarr/shared-data`, and existing native Syncthing on localhost:8384 with unrelated folders. Port 8385 is free. The existing official Syncthing 2.1.5 image is available locally and pinned by digest. Repository has unrelated changes; preserve them. No remote is configured.

## Actions

- Added standalone `docker-compose.syncthing.yml` in project `rawrarr-local`: localhost:8385 GUI, dedicated local state volume, exact `/shared` bind, UID/GID 10001 matching Rawrarr, and a dedicated bridge network.
- Leave native Syncthing, Rawrarr app/database, and browser windows intact. No remote peers or sync ports published. No Mac mini access or deployment.

## Verification and completion

- Compose validation and three sanitized setup tests passed. The bootstrap script defaults to dry-run and refuses existing remote peers or unrelated folders; `--apply` was verified twice, including restart persistence.
- Initial internal-network publication was unavailable in OrbStack. Switched only the new companion to a dedicated bridge network, with GUI still published solely on 127.0.0.1:8385. Disabled discovery, relays, NAT traversal, and bound the sync listener to container loopback before enabling that network. No peers exist.
- Corrected config ownership after the first copied file was unreadable. The reusable script now copies an archive with explicit UID/GID 10001 and mode 0600. Local protected pre-change config backups live outside the repository and shared folder.
- REST and GUI confirmed folder `rawrarr-shared` at `/shared`, send-and-receive, unshared, idle, 2,121 files and 17 directories, zero errors. Counts are this task's snapshot, not a fixed expected size. API key was read only in memory and never printed.
- Verified runtime UID 10001 can read existing encrypted config and write the shared directory. Container and HTTP health checks pass; the existing Rawrarr endpoint remains HTTP 200 on port 7297.
- Opened localhost:8385 for owner review. GUI reports no username/password configured because it listens on the container interface; the host publication is loopback-only. Owner should set their chosen GUI credentials in Settings before any broader access. No credentials were invented or entered on their behalf.
- No Mac mini work or peer transfer performed. Rawrarr's own Syncthing API integration has not been wired up; this is the standalone companion for review.

## Reusable commands

```sh
docker compose -p rawrarr-local -f docker-compose.syncthing.yml up -d syncthing
python3 scripts/setup_local_syncthing.py
python3 scripts/setup_local_syncthing.py --apply
python3 scripts/tests/test_setup_local_syncthing.py
```

The standalone Compose file intentionally sees the app/database as other services in the same project. Never use `--remove-orphans`; it would remove those unrelated containers. The bootstrap temporarily stops only the companion and writes no media or app configuration.

## Recovery

Stop only this companion with `docker compose -p rawrarr-local -f docker-compose.syncthing.yml stop syncthing`. Retain its state volume and the shared folder. Do not use `down -v` or reset the native Syncthing instance. Before future peer enrollment, review network access and GUI authentication; the initial GUI is reachable only through the loopback binding for owner review.
