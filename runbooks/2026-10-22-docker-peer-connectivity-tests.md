# Docker peer connectivity tests by node type

## Objective and initial state

Capture the requirement for Docker environment tests that check connectivity among Rawrarr nodes of the same type. Fossil nodes should be tested against other fossil nodes, and boneyard nodes against other boneyard nodes. Existing [Syncthing peer sync work](2026-10-17-rawrarr-syncthing-peer-sync.md) validates an isolated web/storage pair and file transfer; this request extends the test direction to same-type peers.

## Test direction

- Keep fossil and boneyard peer groups separate; do not infer connectivity between unlike roles from these checks.
- For each peer in the test group, verify its container is running and its Syncthing service/API responds.
- Verify each intended same-type peer is present in Syncthing's live connections and reports connected, rather than treating a healthy container or configured device entry as proof of connectivity.
- Where the test scope includes folder sharing, verify the expected folder is shared and reaches the expected state. A transfer check can establish data-plane behavior beyond peer connectivity.
- Use isolated Docker networks/configuration and disposable test state. Never include production identities, API keys, subscriber app-data, or media.
- Keep any topology assumptions explicit: whether every same-type node must connect to every other peer, or whether a selected peer path is sufficient, remains to be specified during implementation.

## Initial state and remaining work

The existing isolated Syncthing integration covers a metadata/web node paired with a storage node, not same-type fossil or boneyard groups. No Docker environment tests were added in this task. Implementation should extend the Docker test harness with distinct same-type groups and prove Syncthing process/API health plus live peer connectivity; add folder-state or transfer assertions where appropriate.
