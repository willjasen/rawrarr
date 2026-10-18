#!/usr/bin/env python3
"""Build and start one uniquely named Rawrarr test container in OrbStack."""

from __future__ import annotations

import argparse
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import time


ROOT = Path(__file__).resolve().parents[2]
TESTS = ROOT / "scripts" / "tests"
PROJECT = "rawrarr-testing"
IMAGE = "rawrarr-testing-base:local"
BASE36 = "0123456789abcdefghijklmnopqrstuvwxyz"


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
    subprocess.run(["docker", "--context", context, "cp", str(ROOT / "docker-compose.yml"), f"{name}:/workspace/docker-compose.yml"], check=True)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--context", default="orbstack", help="Docker context (default: orbstack)")
    parser.add_argument("--remove-after", action="store_true", help="remove the completed test container after showing its logs")
    parser.add_argument("--rebuild-base", action="store_true", help="rebuild the dependency image (for toolchain or dependency updates)")
    args = parser.parse_args()

    context = docker(args.context, "context", "show")
    if context != args.context:
        raise RuntimeError(f"expected Docker context '{args.context}', got '{context}'")

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
        "--network", "none",
        IMAGE,
        "-c", "sh /workspace/scripts/tests/run-tests-in-container.sh",
    )
    with tempfile.TemporaryDirectory(prefix="rawrarr-test-source-") as temp_dir:
        copy_project_into_container(args.context, name, Path(temp_dir))

    print(f"TEST CONTAINER: {name}", flush=True)
    docker(args.context, "start", name)
    subprocess.run(["docker", "--context", args.context, "logs", "--follow", name], check=False)
    exit_code = int(docker(args.context, "inspect", "--format", "{{.State.ExitCode}}", name))
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
