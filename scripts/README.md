# Scripts

This project keeps a local Radarr inspection helper and repeatable checks under [`tests/`](tests/). See the [test index](tests/README.md) for coverage, dependencies, and invocation.

- `local_radarr.py` — read Radarr status and library totals using the node-local plaintext mirror in the local Docker container; never prints the API key or movie paths. Requires Python 3 and Docker access. Run `python3 scripts/local_radarr.py`.

- `setup_local_syncthing.py` — Bootstrap the existing MacBook `rawrarr-local-syncthing-1` for local review with global discovery and public relays enabled as connection fallback. Requires Python 3 and Docker OrbStack context. Dry-run by default; `--apply` backs up protected config and restarts only this container. Use after `docker compose -p rawrarr-local -f docker-compose.syncthing.yml up -d syncthing`; never add `--remove-orphans`. This bootstrap does not add remote peers or authorize production sharing: configure peers and folders only after owner approval. GUI: http://127.0.0.1:8385; set owner-selected credentials there before broader access.
- `provision_node.py` — Prepare a trusted storage/metadata node credential and coordinator registry entry; Python 3, dry-run by default, `--apply` writes protected node state outside the checkout. See [container deployment](../docs/container-deployment.md).
