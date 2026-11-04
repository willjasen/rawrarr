#!/usr/bin/env python3
"""Classify changed Rawrarr paths into internal coordinator/node deploy targets."""

from __future__ import annotations

import argparse
import json
import subprocess
import sys


COORDINATOR_FILES = {
    "rust-arr/src/coordinator.rs",
    "rust-arr/src/coordinator_ai.rs",
    "rust-arr/src/coordinator_avatar.rs",
    "rust-arr/src/coordinator_health.html",
    "rust-arr/src/coordinator_workspace.html",
    "rust-arr/src/assets/device-signing-client.js",
    "rust-arr/web-export/device-signing-client.js",
    "rust-arr/src/coordinator-workspace.mjs",
    "rust-arr/src/coordinator-device-routes.mjs",
    "rust-arr/src/coordinator-passkey.mjs",
    "rust-arr/src/profile-color.mjs",
    "docker-compose.coordinator.yml",
}

NODE_FILES = {
    "rust-arr/src/node_auth.rs",
    "rust-arr/src/node_role.rs",
    "rust-arr/src/library.rs",
    "rust-arr/src/metadata.rs",
    "rust-arr/src/sonarr.rs",
    "rust-arr/src/quality_profiles.rs",
    "rust-arr/src/artwork_cache.rs",
    "rust-arr/src/movie_snapshot.rs",
    "rust-arr/src/storage_node.rs",
    "rust-arr/src/syncthing.rs",
    "rust-arr/src/subscriber_onboarding.html",
    "rust-arr/src/assets/subscriber-onboarding.js",
    "docker-compose.metadata.yml",
    "docker-compose.storage.yml",
    "docker-compose.syncthing.yml",
    "docker-compose.syncthing-media.yml",
    "docker-compose.syncthing-tailnet.yml",
}

COORDINATOR_PREFIXES = ("rust-arr/migrations/",)

BOTH_FILES = {
    "rust-arr/src/main.rs",
    "rust-arr/src/auth.rs",
    "rust-arr/src/secure_config.rs",
    "rust-arr/src/assets/metadata-export.js",
    "rust-arr/web-export/metadata-export.js",
    "rust-arr/Cargo.toml",
    "rust-arr/Cargo.lock",
    "rust-arr/Dockerfile",
    "docker-compose.yml",
}

NON_RUNTIME_PREFIXES = (
    "deploy/testing/",
    "docs/",
    "runbooks/",
    "scripts/",
    ".github/",
)
NON_RUNTIME_FILES = {
    "docker-compose.macbook-storage.yml",
    "docker-compose.tailscale-mini.yml",
    "docker-compose.mini-relays.yml",
    "storage-proxy.Caddyfile",
    "AGENTS.md", "README.md", "ROADMAP.md", "TESTING.md",
    "docker-compose.orbstack-test.yml",
    "docker-compose.internal-mini.yml",
    "docker-compose.tailscale-serve.yml",
    "docker-compose.headscale-macbook.yml",
    "docker-compose.headscale-mini.yml",
    "docker-compose.tailscale-macbook.yml",
}


def classify_paths(paths: list[str]) -> dict[str, object]:
    targets: set[str] = set()
    for path in paths:
        if path in NON_RUNTIME_FILES or path.startswith(NON_RUNTIME_PREFIXES):
            continue
        if path in BOTH_FILES:
            targets.update(("coordinator", "nodes"))
        elif path in COORDINATOR_FILES or path.startswith("rust-arr/src/coordinator_") or path.startswith(COORDINATOR_PREFIXES):
            targets.add("coordinator")
        elif path in NODE_FILES:
            targets.add("nodes")
        elif path.startswith("rust-arr/src/") or path.startswith("rust-arr/web-export/"):
            # Unclassified runtime code is shared until its role is explicitly mapped.
            targets.update(("coordinator", "nodes"))
        elif path.startswith("docker-compose."):
            targets.update(("coordinator", "nodes"))
        elif path.startswith("rust-arr/"):
            targets.update(("coordinator", "nodes"))

    ordered = [target for target in ("coordinator", "nodes") if target in targets]
    return {"targets": ordered, "coordinator": "coordinator" in targets, "nodes": "nodes" in targets}


def changed_paths(base: str, head: str) -> list[str]:
    result = subprocess.run(
        ["git", "diff", "--name-only", "-z", f"{base}...{head}"],
        check=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )
    return [path.decode("utf-8", errors="surrogateescape") for path in result.stdout.split(b"\0") if path]


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--base", required=True, help="base Git ref, usually the deployed GitHub revision")
    parser.add_argument("--head", default="HEAD", help="head Git ref to deploy (default: HEAD)")
    args = parser.parse_args()
    try:
        paths = changed_paths(args.base, args.head)
    except subprocess.CalledProcessError as error:
        detail = error.stderr.decode("utf-8", errors="replace").strip()
        print(f"Could not compare Git refs: {detail}", file=sys.stderr)
        return 2
    result = classify_paths(paths)
    result["changedFiles"] = paths
    print(json.dumps(result, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
