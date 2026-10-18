# Scripts

This project currently keeps its repeatable checks under [`tests/`](tests/). See the [test index](tests/README.md) for coverage, dependencies, and invocation.

- `setup_local_syncthing.py` — Configure the existing MacBook `rawrarr-local-syncthing-1` for local review: exact shared mount, no remote peers, disabled discovery/relay/NAT. Requires Python 3 and Docker OrbStack context. Dry-run by default; `--apply` backs up protected config and restarts only this container. Use after `docker compose -p rawrarr-local -f docker-compose.syncthing.yml up -d syncthing`; never add `--remove-orphans`. GUI: http://127.0.0.1:8385; set owner-selected credentials there before broader access.
