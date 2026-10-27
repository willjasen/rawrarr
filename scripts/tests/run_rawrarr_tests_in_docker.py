#!/usr/bin/env python3
"""Build and start one uniquely named Rawrarr test container in OrbStack."""

from __future__ import annotations

import argparse
from datetime import datetime, timezone
from pathlib import Path
import os
import shutil
import subprocess
import sys
import tempfile
import time


ROOT = Path(__file__).resolve().parents[2]
TESTS = ROOT / "scripts" / "tests"
PROJECT = "rawrarr-test-regression"
IMAGE = "rawrarr-testing-base:local"
BASE36 = "0123456789abcdefghijklmnopqrstuvwxyz"
CLEANUP_INTERVAL_SECONDS = 60 * 60
CLEANUP_STATE = Path.home() / "Library" / "Application Support" / "Rawrarr" / ".test-container-cleanup-last-run"


def timestamp_code() -> str:
    value = time.time_ns() // 1_000_000
    encoded = ""
    while value:
        value, digit = divmod(value, len(BASE36))
        encoded = BASE36[digit] + encoded
    return encoded or BASE36[0]


def docker(context: str, *args: str, check: bool = True) -> str:
    result = subprocess.run(
        ["docker", "--context", context, *args],
        check=check,
        capture_output=True,
        text=True,
    )
    return result.stdout.strip()


def copy_project_into_container(context: str, name: str, destination: Path) -> None:
    rust_source = destination / "rust-arr"
    tests_source = destination / "tests"
    shutil.copytree(ROOT / "rust-arr", rust_source, ignore=shutil.ignore_patterns("target", ".DS_Store", "__pycache__"))
    shutil.copytree(TESTS, tests_source, ignore=shutil.ignore_patterns(".DS_Store", "__pycache__"))
    subprocess.run(["docker", "--context", context, "cp", f"{rust_source}/.", f"{name}:/workspace/rust-arr/"], check=True)
    subprocess.run(["docker", "--context", context, "cp", f"{tests_source}/.", f"{name}:/workspace/scripts/tests/"], check=True)

    # The Rust source tree was copied above, including the current UI assets.
    # Copy only the root-level fixtures needed by source-structure tests.
    for compose_file in sorted(ROOT.glob("docker-compose*.yml")):
        subprocess.run(["docker", "--context", context, "cp", str(compose_file), f"{name}:/workspace/{compose_file.name}"], check=True)
    for fixture in (ROOT / ".github", ROOT / "admin", ROOT / "docs", ROOT / "storage-proxy.Caddyfile"):
        if not fixture.exists():
            continue
        subprocess.run(["docker", "--context", context, "cp", str(fixture), f"{name}:/workspace/"], check=True)




def cleanup_candidates(context: str) -> list[tuple[str, str, str]]:
    ids = docker(context, "ps", "-aq", "--filter", f"label=com.docker.compose.project={PROJECT}").splitlines()
    candidates = []
    cutoff = time.time() - CLEANUP_INTERVAL_SECONDS
    for container_id in ids:
        created, status, name, service = docker(
            context,
            "inspect", "--format", "{{.Created}}|{{.State.Status}}|{{.Name}}|{{index .Config.Labels \"com.docker.compose.service\"}}",
            container_id,
        ).lstrip("/").replace("|/", "|").split("|", 3)
        created_epoch = datetime.fromisoformat(created.replace("Z", "+00:00")).timestamp()
        # Only remove stopped containers created by this test runner, never the shared base image container.
        is_test_runner = service.startswith("tests-") or name.startswith(f"{PROJECT}-tests-")
        if status != "running" and is_test_runner and created_epoch < cutoff:
            candidates.append((container_id, name, created))
    return candidates


def cleanup_if_due(context: str, *, dry_run: bool = False, force: bool = False) -> int:
    try:
        last_cleanup = float(CLEANUP_STATE.read_text(encoding="ascii").strip())
    except (FileNotFoundError, ValueError):
        last_cleanup = 0
    remaining = CLEANUP_INTERVAL_SECONDS - (time.time() - last_cleanup)
    if remaining > 0 and not dry_run and not force:
        print(f"TEST CLEANUP: skipped; next cleanup eligible in {int(remaining)} seconds")
        return 0

    candidates = cleanup_candidates(context)
    mode = "DRY RUN" if dry_run else "CLEANUP"
    print(f"TEST {mode}: {len(candidates)} stopped test container(s) older than one hour")
    for container_id, name, created in candidates:
        print(f"  {name} ({container_id[:12]}) created {created}")
    if dry_run:
        return len(candidates)

    for container_id, name, _created in candidates:
        docker(context, "rm", container_id)
        print(f"  removed {name}")
    CLEANUP_STATE.parent.mkdir(parents=True, exist_ok=True)
    temp_state = CLEANUP_STATE.with_name(f"{CLEANUP_STATE.name}.{os.getpid()}.tmp")
    temp_state.write_text(f"{time.time():.6f}\n", encoding="ascii")
    temp_state.replace(CLEANUP_STATE)
    return len(candidates)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--context", default="orbstack", help="Docker context (default: orbstack)")
    parser.add_argument("--remove-after", action="store_true", help="remove the completed test container after showing its logs")
    parser.add_argument("--rebuild-base", action="store_true", help="rebuild the dependency image (for toolchain or dependency updates)")
    parser.add_argument("--cleanup-dry-run", action="store_true", help="list stopped test containers older than one hour without removing them")
    parser.add_argument("--cleanup-now", action="store_true", help="remove eligible old stopped test containers now, without starting a test run")
    args = parser.parse_args()

    context = docker(args.context, "context", "show")
    if context != args.context:
        raise RuntimeError(f"expected Docker context '{args.context}', got '{context}'")

    if args.cleanup_dry_run:
        cleanup_if_due(args.context, dry_run=True)
        return 0
    if args.cleanup_now:
        cleanup_if_due(args.context, dry_run=False, force=True)
        return 0

    cleanup_if_due(args.context)

    run_id = timestamp_code()
    name = f"{PROJECT}-tests-{run_id}"
    image_exists = subprocess.run(
        ["docker", "--context", args.context, "image", "inspect", IMAGE],
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
        check=False,
    ).returncode == 0
    if args.rebuild_base or not image_exists:
        subprocess.run(
            ["docker", "--context", args.context, "build", "--tag", IMAGE,
             "--file", str(TESTS / "Dockerfile"), str(ROOT)],
            check=True,
        )
    docker(
        args.context,
        "create",
        "--entrypoint", "/bin/sh",
        "--name", name,
        "--label", f"com.docker.compose.project={PROJECT}",
        "--label", f"com.docker.compose.service=tests-{run_id}",
        "--env", "HOME=/tmp/rawrarr-test-home",
        "--network", "bridge",
        IMAGE,
        "-c", "sh /workspace/scripts/tests/wait-for-test-trigger.sh",
    )
    with tempfile.TemporaryDirectory(prefix="rawrarr-test-source-") as temp_dir:
        copy_project_into_container(args.context, name, Path(temp_dir))

    print(f"TEST CONTAINER: {name}", flush=True)
    docker(args.context, "start", name)
    try:
        # Refresh this container's writable linked-clone layer from the copied
        # source and lockfile before test networking is disabled.
        subprocess.run(
            ["docker", "--context", args.context, "exec", name,
             "sh", "/workspace/scripts/tests/prepare-test-clone.sh"],
            check=True,
        )
        docker(args.context, "network", "disconnect", "bridge", name)
        docker(args.context, "exec", name, "touch", "/tmp/rawrarr-test-ready")
        subprocess.run(["docker", "--context", args.context, "logs", "--follow", name], check=False)
        exit_code = int(docker(args.context, "inspect", "--format", "{{.State.ExitCode}}", name))
    except subprocess.CalledProcessError:
        docker(args.context, "stop", name, check=False)
        if args.remove_after:
            docker(args.context, "rm", name, check=False)
            print(f"REMOVED TEST CONTAINER: {name}")
        else:
            print(f"TEST LOGS RETAINED: docker --context {args.context} logs {name}")
        raise
    print(f"TEST CONTAINER EXIT: {exit_code}")
    if args.remove_after:
        docker(args.context, "rm", name)
        print(f"REMOVED TEST CONTAINER: {name}")
    else:
        print(f"TEST LOGS RETAINED: docker --context {args.context} logs {name}")
    return exit_code


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (OSError, RuntimeError, subprocess.CalledProcessError) as error:
        print(f"FAIL Rawrarr Docker test runner: {type(error).__name__}: {error}", file=sys.stderr)
        raise SystemExit(1)
