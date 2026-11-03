#!/usr/bin/env python3
"""Refresh repo-declared third-party support images on do3."""

from __future__ import annotations

import argparse
import contextlib
import json
import os
import subprocess
import sys
import time
from pathlib import Path
from typing import Iterator


PROJECTS = {
    "rawrarr-internal-do3": {
        "directory": Path("/opt/rawrarr-internal-do3"),
        "compose": "docker-compose.internal-support-images.yml",
        "health_url": "https://rawrarr.risk-mermaid.ts.net/health",
        "tailscale_container": "rawrarr-internal-do3-tailscale",
    },
    "rawrarr-roarlay-prod": {
        "directory": Path("/opt/rawrarr-roarlay-prod"),
        "compose": "docker-compose.public-support-images.yml",
        "health_url": "https://roarlay.rawrarr.net/health",
        "tailscale_container": "rawrarr-roarlay-prod-tailscale",
    },
}

TARGETS = (
    ("rawrarr-internal-do3", "rawrarr-internal-do3-postgres", "postgres", "postgres:17-alpine"),
    ("rawrarr-internal-do3", "rawrarr-internal-do3-tailscale", "tailscale", "tailscale/tailscale:stable"),
    ("rawrarr-internal-do3", "rawrarr-internal-do3-analytics-caddy", "analytics-caddy", "caddy:2-alpine"),
    ("rawrarr-internal-do3", "rawrarr-internal-do3-umami-postgres-1", "umami-postgres", "postgres:17-alpine"),
    ("rawrarr-roarlay-prod", "rawrarr-roarlay-prod-postgres", "postgres", "postgres:17-alpine"),
    ("rawrarr-roarlay-prod", "rawrarr-roarlay-prod-tailscale", "tailscale", "tailscale/tailscale:stable"),
    ("rawrarr-roarlay-prod", "rawrarr-roarlay-prod-caddy", "caddy", "caddy:2-alpine"),
    ("rawrarr-roarlay-prod", "rawrarr-roarlay-prod-cloudflared", "cloudflared",
     "cloudflare/cloudflared@sha256:faa8e0a1475f137c910be6025f43c8e4092fdd35461b31e19fd1390657a1ff39"),
    ("rawrarr-roarlay-prod", "rawrarr-roarlay-prod-crowdsec-1", "crowdsec", "crowdsecurity/crowdsec:v1.8.2"),
    ("rawrarr-roarlay-prod", "rawrarr-roarlay-prod-headscale", "headscale", "headscale/headscale:0.29.5"),
    ("rawrarr-roarlay-prod", "rawrarr-roarlay-prod-umami-1", "umami",
     "ghcr.io/umami-software/umami:3.4.1"),
    ("rawrarr-roarlay-prod", "rawrarr-roarlay-prod-umami-postgres-1", "umami-postgres",
     "postgres:17-alpine"),
)
HEALTH_TIMEOUT_SECONDS = 180
LOCK_RETRY_SECONDS = 60


def command(args: list[str], *, cwd: Path | None = None, capture: bool = True) -> subprocess.CompletedProcess:
    return subprocess.run(
        args, cwd=cwd, check=True, text=True,
        stdout=subprocess.PIPE if capture else None,
        stderr=subprocess.PIPE if capture else None,
    )


def docker_container(container: str) -> dict:
    values = json.loads(command(["docker", "inspect", container]).stdout)
    if not values:
        raise RuntimeError(f"Container is missing: {container}")
    return values[0]


def compose_context(project: str, container: str, service: str, override: Path) -> tuple[Path, list[Path]]:
    data = docker_container(container)
    labels = data.get("Config", {}).get("Labels") or {}
    project_dir = Path(labels.get("com.docker.compose.project.working_dir", ""))
    raw_files = labels.get("com.docker.compose.project.config_files", "")
    files = [Path(item) for item in raw_files.split(",") if item]
    if labels.get("com.docker.compose.project") != project:
        raise RuntimeError(f"{container} is not in the expected Compose project {project}")
    if labels.get("com.docker.compose.service") != service:
        raise RuntimeError(f"{container} is not the expected Compose service {service}")
    if project_dir != PROJECTS[project]["directory"] or not project_dir.is_dir() or not files:
        raise RuntimeError(f"Cannot recover the active Compose definition for {container}")
    project_root = project_dir.resolve()
    if any(not item.is_file() or not item.resolve().is_relative_to(project_root) for item in files):
        raise RuntimeError(f"An active Compose file for {container} is missing or outside its project")
    if not override.is_file():
        raise RuntimeError(f"A Compose file required to refresh {container} is missing")
    files = [item for item in files if item != override]
    files.append(override)
    return project_dir, files


def compose_args(project: str, project_dir: Path, files: list[Path], action: list[str]) -> list[str]:
    args = ["docker", "compose", "--project-directory", str(project_dir), "-p", project]
    for compose_file in files:
        args.extend(("-f", str(compose_file)))
    args.extend(action)
    return args


@contextlib.contextmanager
def deployment_lock(path: Path, container: str) -> Iterator[None]:
    path.parent.mkdir(parents=True, exist_ok=True)
    while True:
        try:
            descriptor = os.open(path, os.O_CREAT | os.O_EXCL | os.O_WRONLY, 0o600)
            with os.fdopen(descriptor, "w", encoding="utf-8") as marker:
                marker.write(f"pid={os.getpid()} container={container}\n")
            break
        except FileExistsError:
            print(f"{path} is held by another deployment; retrying in 60 seconds", flush=True)
            time.sleep(LOCK_RETRY_SECONDS)
    try:
        yield
    finally:
        path.unlink(missing_ok=True)


def write_override(path: Path, service: str, image: str) -> None:
    path.parent.mkdir(mode=0o700, parents=True, exist_ok=True)
    path.write_text(
        f"services:\n  {service}:\n    image: {image}\n    pull_policy: never\n",
        encoding="utf-8",
    )
    path.chmod(0o600)


def image_id(reference: str) -> str:
    result = command(["docker", "image", "inspect", reference])
    return json.loads(result.stdout)[0]["Id"]


def wait_running(container: str) -> None:
    deadline = time.monotonic() + HEALTH_TIMEOUT_SECONDS
    while time.monotonic() < deadline:
        state = docker_container(container).get("State", {})
        status = state.get("Health", {}).get("Status", state.get("Status", "unknown"))
        if status in {"healthy", "running"}:
            return
        if status in {"unhealthy", "exited", "dead"}:
            raise RuntimeError(f"{container} entered {status}")
        time.sleep(3)
    raise RuntimeError(f"{container} did not become ready within {HEALTH_TIMEOUT_SECONDS}s")


def verify_route(project: str) -> None:
    settings = PROJECTS[project]
    deadline = time.monotonic() + HEALTH_TIMEOUT_SECONDS
    last_error: Exception | None = None
    while time.monotonic() < deadline:
        try:
            if project == "rawrarr-internal-do3":
                status = command([
                    "docker", "exec", settings["tailscale_container"],
                    "tailscale", "serve", "status", "--json",
                ]).stdout
                serve = json.loads(status)
                proxy = serve["Web"]["rawrarr.risk-mermaid.ts.net:443"]["Handlers"]["/"]["Proxy"]
                if proxy != "http://127.0.0.1:8080":
                    raise RuntimeError(
                        "Internal Tailscale Serve route no longer targets coordinator port 8080"
                    )
            command(["curl", "--fail", "--silent", "--show-error", "--output", "/dev/null",
                     settings["health_url"]])
            return
        except (subprocess.CalledProcessError, json.JSONDecodeError, KeyError, RuntimeError) as error:
            last_error = error
            time.sleep(3)
    raise RuntimeError(f"{project} route verification timed out: {last_error}")


def refresh_one(source_root: Path, target: tuple[str, str, str, str]) -> bool:
    project, container, service, image = target
    project_settings = PROJECTS[project]
    directory = project_settings["directory"]
    override = source_root / "deploy" / "do3" / project_settings["compose"]
    lock = directory / f".container-rebuild-{service}.lock"

    with deployment_lock(lock, container):
        project_dir, files = compose_context(project, container, service, override)
        config = compose_args(project, project_dir, files, ["config", "--format", "json"])
        configured = json.loads(command(config, cwd=project_dir).stdout)
        service_config = configured.get("services", {}).get(service, {})
        configured_image = service_config.get("image")
        if configured_image != image:
            raise RuntimeError(f"{project}/{service} resolves to unexpected image {configured_image!r}")
        if service_config.get("pull_policy") != "always":
            raise RuntimeError(f"{project}/{service} support overlay does not allow the requested pull")
        command(compose_args(project, project_dir, files, ["pull", service]),
                cwd=project_dir, capture=False)
        pulled_id = image_id(image)
        running_id = docker_container(container).get("Image")
        if not running_id:
            raise RuntimeError(f"Could not read current image ID for {container}")
        if pulled_id == running_id:
            print(f"{container}: already current ({image})")
            return False

        build_dir = directory / "builds" / "support-image-updates" / str(time.time_ns())
        build_dir.mkdir(mode=0o700, parents=True, exist_ok=False)
        temporary_override = build_dir / f"{project}-{service}.override.yml"
        rollback_tag = f"rawrarr:rollback-support-{project}-{service}-{running_id.removeprefix('sha256:')[:12]}"
        command(["docker", "tag", running_id, rollback_tag])
        timestamp = time.strftime("%Y%m%dT%H%M%SZ", time.gmtime())
        write_override(temporary_override, service, image)
        update_files = [path for path in files if path != override] + [temporary_override]
        up = compose_args(project, project_dir, update_files,
                          ["up", "-d", "--no-deps", "--pull", "never", service])
        try:
            command(up, cwd=project_dir, capture=False)
            wait_running(container)
            if docker_container(container).get("Image") != pulled_id:
                raise RuntimeError(f"{container} did not start the pulled image")
            verify_route(project)
        except Exception:
            print(f"Update failed for {container}; restoring {rollback_tag}", file=sys.stderr, flush=True)
            rollback = build_dir / f"{project}-{service}-{timestamp}-rollback.yml"
            write_override(rollback, service, rollback_tag)
            rollback_files = [path for path in files if path != override] + [rollback]
            command(compose_args(
                project, project_dir, rollback_files,
                ["up", "-d", "--no-deps", "--pull", "never", service],
            ), cwd=project_dir, capture=False)
            wait_running(container)
            if docker_container(container).get("Image") != running_id:
                raise RuntimeError(f"Rollback verification failed for {container}")
            verify_route(project)
            raise

        print(f"{container}: updated to {image} (rollback {rollback_tag})")
        return True


def run(source_root: Path, selected_projects: set[str]) -> int:
    platform = command(["docker", "info", "--format", "{{.OSType}}/{{.Architecture}}"]).stdout.strip()
    if platform not in {"linux/x86_64", "linux/amd64"}:
        raise RuntimeError(f"do3 must run as linux/amd64; Docker reports {platform}")
    if not source_root.is_dir():
        raise RuntimeError(f"Published source directory does not exist: {source_root}")
    targets = [target for target in TARGETS if target[0] in selected_projects]
    if not targets:
        raise RuntimeError("No support image targets were selected")
    for target in targets:
        refresh_one(source_root, target)
    for project in selected_projects:
        verify_route(project)
    return 0


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--source-root", required=True, type=Path,
                        help="Root of the verified published GitHub source archive")
    parser.add_argument("--project", choices=tuple(PROJECTS), action="append",
                        help="Limit to one Compose project; repeat to select both")
    args = parser.parse_args()
    try:
        return run(args.source_root, set(args.project or PROJECTS))
    except (OSError, RuntimeError, subprocess.CalledProcessError, json.JSONDecodeError,
            KeyError, ValueError) as error:
        print(f"support image update failed: {error}", file=sys.stderr, flush=True)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
