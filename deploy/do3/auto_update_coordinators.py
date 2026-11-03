#!/usr/bin/env python3
"""Poll GitHub main and roll coordinator changes through do3's local image cache."""

from __future__ import annotations

import contextlib
import hashlib
import json
import os
import shutil
import subprocess
import sys
import tarfile
import time
import urllib.error
import urllib.request
from datetime import datetime, timezone
from pathlib import Path
from typing import Iterator


REPOSITORY = "willjasen/rawrarr"
INTERNAL = {
    "container": "rawrarr-int-roarlay",
    "lock": Path("/opt/rawrarr-internal-do3/.container-rebuild-roarlay.lock"),
    "project_dir": Path("/opt/rawrarr-internal-do3"),
    "health_url": "http://127.0.0.1:8080/health",
    "version_url": "http://127.0.0.1:8080/api/version",
    "image_env": "RAWARR_INTERNAL_IMAGE",
    "image_base": "rawrarr:internal-base-amd64",
}
PUBLIC = {
    "container": "rawrarr-prod-roarlay",
    "lock": Path("/opt/rawrarr-roarlay-prod/.container-rebuild-roarlay.lock"),
    "project_dir": Path("/opt/rawrarr-roarlay-prod"),
    "version_url": "http://127.0.0.1:8080/api/version",
    "health_url": "http://127.0.0.1:8080/health",
    "image_env": None,
    "image_base": None,
}
INTERNAL_TAILSCALE_CONTAINER = "rawrarr-int-tailscale"
INTERNAL_HEALTH_URL_PUBLIC = "https://rawrarr.risk-mermaid.ts.net/health"
PUBLIC_HEALTH_URL = "https://roarlay.rawrarr.net/health"
MIN_FREE_BYTES = 5 * 1024**3
HEALTH_TIMEOUT_SECONDS = 150

NON_RUNTIME_PREFIXES = ("docs/", "runbooks/", "scripts/", ".github/", "website/")
NON_RUNTIME_FILES = {
    "docker-compose.macbook-storage.yml",
    "docker-compose.tailscale-mini.yml",
    "docker-compose.mini-relays.yml",
    "storage-proxy.Caddyfile",
    "AGENTS.md",
    "README.md",
    "ROADMAP.md",
    "TESTING.md",
    "docker-compose.orbstack-test.yml",
    "docker-compose.internal-mini.yml",
    "docker-compose.tailscale-serve.yml",
    "docker-compose.headscale-macbook.yml",
    "docker-compose.headscale-mini.yml",
    "docker-compose.tailscale-macbook.yml",
    "deploy/do3/docker-compose.internal-coordinator.yml",
    "deploy/do3/docker-compose.analytics-dashboard-internal.yml",
    "deploy/do3/docker-compose.production-container-names.yml",
}


def command(args: list[str], *, cwd: Path | None = None, env: dict[str, str] | None = None,
            capture: bool = True, check: bool = True) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        args,
        cwd=cwd,
        env=env,
        check=check,
        text=True,
        stdout=subprocess.PIPE if capture else None,
        stderr=subprocess.PIPE if capture else None,
    )


def github_json(token: str, endpoint: str) -> dict:
    request = urllib.request.Request(
        f"https://api.github.com/repos/{REPOSITORY}/{endpoint}",
        headers={"Accept": "application/vnd.github+json", "Authorization": f"Bearer {token}",
                 "X-GitHub-Api-Version": "2022-11-28", "User-Agent": "rawrarr-do3-deployer"},
    )
    with urllib.request.urlopen(request, timeout=30) as response:
        return json.load(response)


def get_main_sha(token: str) -> str:
    value = github_json(token, "commits/main").get("sha", "")
    if len(value) != 40 or any(character not in "0123456789abcdef" for character in value):
        raise RuntimeError("GitHub returned an invalid main commit SHA")
    return value


def changed_paths(token: str, base: str, head: str) -> list[str]:
    result = github_json(token, f"compare/{base}...{head}")
    files = result.get("files")
    if not isinstance(files, list):
        raise RuntimeError("GitHub compare response did not contain a file list")
    if len(files) >= 300:
        return ["rust-arr/unknown-change"]
    return [entry["filename"] for entry in files if isinstance(entry, dict) and "filename" in entry]


def compare_revisions(token: str, base: str, head: str) -> dict:
    return github_json(token, f"compare/{base}...{head}")


def affects_coordinator(paths: list[str]) -> bool:
    """Conservatively select coordinator builds for runtime and unknown app paths."""
    for path in paths:
        if path in NON_RUNTIME_FILES or path.startswith(NON_RUNTIME_PREFIXES):
            continue
        if path.startswith(("rust-arr/", "docker-compose.", "deploy/do3/docker-compose.")):
            return True
    return False


def docker_container(container: str) -> dict:
    result = command(["docker", "inspect", container])
    values = json.loads(result.stdout)
    if not values:
        raise RuntimeError(f"Container is missing: {container}")
    return values[0]


def deployed_revision(container: str) -> str:
    data = docker_container(container)
    labels = data.get("Config", {}).get("Labels") or {}
    return labels.get("org.opencontainers.image.revision", "")


def image_matches(tag: str, revision: str) -> bool:
    result = command(["docker", "image", "inspect", tag], check=False)
    if result.returncode:
        return False
    image = json.loads(result.stdout)[0]
    labels = image.get("Config", {}).get("Labels") or {}
    return (image.get("Os") == "linux" and image.get("Architecture") == "amd64"
            and labels.get("org.opencontainers.image.revision") == revision)


def write_override(path: Path, image: str, deployed_at: str) -> None:
    path.parent.mkdir(mode=0o700, parents=True, exist_ok=True)
    content = (
        "services:\n  roarlay:\n"
        f"    image: {image}\n"
        "    pull_policy: never\n"
        f"    environment:\n      RAWARR_DEPLOYED_AT: {json.dumps(deployed_at)}\n"
    )
    path.write_text(content, encoding="utf-8")
    path.chmod(0o600)


def compose_files(container: str, override: Path) -> tuple[str, Path, str, list[Path]]:
    data = docker_container(container)
    labels = data.get("Config", {}).get("Labels") or {}
    project = labels.get("com.docker.compose.project")
    project_dir = Path(labels.get("com.docker.compose.project.working_dir", ""))
    service = labels.get("com.docker.compose.service")
    raw_files = labels.get("com.docker.compose.project.config_files", "")
    files = [Path(item) for item in raw_files.split(",") if item]
    files = [item for item in files if item.name != "deployment-info.override.yml"]
    if not project or not service or not project_dir.is_dir() or not files:
        raise RuntimeError(f"Cannot recover the active Compose definition for {container}")
    files.append(override)
    return project, project_dir, service, files


def compose_command(project: str, project_dir: Path, files: list[Path], service: str) -> list[str]:
    env_file = project_dir / "secrets" / "analytics.env"
    if not env_file.is_file():
        raise RuntimeError(f"Coordinator Compose env file is missing: {env_file}")
    args = [
        "docker", "compose", "--project-directory", str(project_dir),
        "--env-file", str(env_file), "-p", project,
    ]
    for compose_file in files:
        args.extend(("-f", str(compose_file)))
    args.extend(("up", "-d", "--no-deps", "--pull", "never", service))
    return args


@contextlib.contextmanager
def deployment_lock(path: Path, revision: str) -> Iterator[None]:
    path.parent.mkdir(parents=True, exist_ok=True)
    while True:
        try:
            descriptor = os.open(path, os.O_CREAT | os.O_EXCL | os.O_WRONLY, 0o600)
            with os.fdopen(descriptor, "w", encoding="utf-8") as marker:
                marker.write(f"pid={os.getpid()} revision={revision}\n")
            break
        except FileExistsError:
            print(f"{path} is held by another deployment; retrying in 60 seconds", flush=True)
            time.sleep(60)
    try:
        yield
    finally:
        path.unlink(missing_ok=True)


def wait_healthy(container: str) -> None:
    deadline = time.monotonic() + HEALTH_TIMEOUT_SECONDS
    while time.monotonic() < deadline:
        data = docker_container(container)
        state = data.get("State", {})
        health = state.get("Health", {}).get("Status", state.get("Status", "unknown"))
        if health == "healthy":
            return
        if health in {"unhealthy", "exited", "dead"}:
            raise RuntimeError(f"{container} entered {health}")
        time.sleep(3)
    raise RuntimeError(f"{container} did not become healthy within {HEALTH_TIMEOUT_SECONDS}s")


def preserve_image(container: str, role: str, revision: str) -> tuple[str, str]:
    image_id = docker_container(container).get("Image", "")
    if not image_id:
        raise RuntimeError(f"Could not read current image ID for {container}")
    stable_tag = f"rawrarr:rollback-{role}-before-{revision[:7]}-amd64"
    existing = command(["docker", "image", "inspect", stable_tag], check=False)
    if existing.returncode:
        command(["docker", "tag", image_id, stable_tag])
    # Preserve a stable first-before tag across retries and a separate exact
    # rollback point for the image that is running on this specific attempt.
    attempt_tag = f"rawrarr:rollback-{role}-attempt-{revision[:7]}-{image_id.removeprefix('sha256:')[:12]}-amd64"
    attempt = command(["docker", "image", "inspect", attempt_tag], check=False)
    if attempt.returncode:
        command(["docker", "tag", image_id, attempt_tag])
    elif json.loads(attempt.stdout)[0]["Id"] != image_id:
        raise RuntimeError(f"Attempt rollback tag already points to an unexpected image: {attempt_tag}")
    return image_id, attempt_tag


def preserve_tag(source: str, target: str) -> str:
    image = json.loads(command(["docker", "image", "inspect", source]).stdout)[0]
    image_id = image["Id"]
    existing = command(["docker", "image", "inspect", target], check=False)
    if existing.returncode:
        command(["docker", "tag", image_id, target])
    else:
        # This tag is intentionally immutable: a retry keeps the original
        # pre-deployment base image rather than replacing it with the new base.
        return json.loads(existing.stdout)[0]["Id"]
    return image_id


def verify_service(container: str, image_id: str, revision: str, version_url: str,
                   health_url: str) -> None:
    wait_healthy(container)
    data = docker_container(container)
    if data.get("Image") != image_id:
        raise RuntimeError(f"{container} is not running the expected immutable image ID")
    if deployed_revision(container) != revision:
        raise RuntimeError(f"{container} revision label does not match {revision}")
    image = json.loads(command(["docker", "image", "inspect", image_id]).stdout)[0]
    if image.get("Os") != "linux" or image.get("Architecture") != "amd64":
        raise RuntimeError(f"{container} image is not linux/amd64")
    version = json.loads(command(["docker", "exec", container, "curl", "--fail", "--silent",
                                  "--show-error", version_url]).stdout)
    if version.get("revision") != revision or not version.get("deployedAt"):
        raise RuntimeError(f"{container} version endpoint does not report the expected revision/time")
    command(["docker", "exec", container, "curl", "--fail", "--silent", "--show-error", health_url])


def deploy_role(role: str, target: dict, image: str, image_id: str, revision: str) -> None:
    container = target["container"]
    old_image, old_rollback_tag = preserve_image(container, f"{role}-roarlay", revision)
    build_dir = target["project_dir"] / "builds" / f"github-main-{revision[:7]}-auto"
    override = build_dir / "deployment-info.override.yml"
    timestamp = datetime.now(timezone.utc).replace(microsecond=0).isoformat().replace("+00:00", "Z")
    write_override(override, image, timestamp)
    project, project_dir, service, files = compose_files(container, override)
    env = os.environ.copy()
    if target["image_env"]:
        env[target["image_env"]] = target["image_base"]
    compose = compose_command(project, project_dir, files, service)
    # Validate the complete merged Compose configuration before taking the lock.
    config = compose[:compose.index("up")] + ["config", "--quiet"]
    command(config, cwd=project_dir, env=env)
    with deployment_lock(target["lock"], revision):
        command(compose, cwd=project_dir, env=env, capture=False)
        try:
            if role == "internal":
                verify_service(container, image_id, revision, target["version_url"], target["health_url"])
                serve = command(["docker", "exec", INTERNAL_TAILSCALE_CONTAINER,
                                 "tailscale", "serve", "status", "--json"]).stdout
                status = json.loads(serve)
                proxy = status["Web"]["rawrarr.risk-mermaid.ts.net:443"]["Handlers"]["/"]["Proxy"]
                if proxy != "http://127.0.0.1:8080":
                    raise RuntimeError("Internal Serve route no longer proxies to coordinator port 8080")
                command(["curl", "--fail", "--silent", "--show-error", INTERNAL_HEALTH_URL_PUBLIC])
            else:
                verify_service(container, image_id, revision, target["version_url"], target["health_url"])
                edge = command(["curl", "--silent", "--show-error", "--output", "/dev/null",
                                "--write-out", "%{http_code}", PUBLIC_HEALTH_URL], check=False)
                if edge.returncode or edge.stdout.strip() != "200":
                    print(f"warning: public edge health check returned {edge.stdout.strip() or 'no response'}; local service is healthy", flush=True)
        except Exception:
            print(f"verification failed for {role}; restoring the saved image", file=sys.stderr, flush=True)
            rollback = build_dir / "rollback.override.yml"
            write_override(rollback, old_rollback_tag, timestamp)
            rollback_files = [path for path in files if path != override] + [rollback]
            rollback_command = compose_command(project, project_dir, rollback_files, service)
            command(rollback_command, cwd=project_dir, env=env, capture=False)
            wait_healthy(container)
            if docker_container(container).get("Image") != old_image:
                raise RuntimeError(f"Rollback verification failed for {role}")
            raise


def download_and_build(token: str, revision: str, internal_build_dir: Path) -> tuple[str, str]:
    tag = f"rawrarr:{revision}-amd64"
    existing = command(["docker", "image", "inspect", tag], check=False)
    if not existing.returncode:
        if not image_matches(tag, revision):
            raise RuntimeError(f"Immutable revision tag {tag} exists with the wrong revision or architecture")
        image_id = json.loads(existing.stdout)[0]["Id"]
        return tag, image_id
    available = shutil.disk_usage("/opt").free
    if available < MIN_FREE_BYTES:
        raise RuntimeError(f"Only {available // 1024**3} GiB is free under /opt; refusing to build")
    internal_build_dir.mkdir(mode=0o700, parents=True, exist_ok=False)
    archive = internal_build_dir / "source.tar.gz"
    environment = os.environ.copy()
    environment["GH_TOKEN"] = token
    with archive.open("wb") as output:
        subprocess.run(["gh", "api", f"repos/{REPOSITORY}/tarball/{revision}"],
                       check=True, stdout=output, stderr=subprocess.PIPE, env=environment)
    digest = hashlib.sha256(archive.read_bytes()).hexdigest()
    with archive.open("rb") as source_archive:
        verified_digest = hashlib.file_digest(source_archive, "sha256").hexdigest()
    if digest != verified_digest:
        raise RuntimeError("Downloaded GitHub archive failed its SHA-256 verification")
    (internal_build_dir / "source.sha256").write_text(f"{digest}  source.tar.gz\n", encoding="ascii")
    source = internal_build_dir / "source"
    source.mkdir(mode=0o700)
    with tarfile.open(archive, "r:gz") as bundle:
        members = bundle.getmembers()
        if any(Path(member.name).is_absolute() or ".." in Path(member.name).parts for member in members):
            raise RuntimeError("GitHub archive contains an unsafe path")
        bundle.extractall(source, members=members, filter="data")
    roots = list(source.iterdir())
    if len(roots) != 1 or not (roots[0] / "rust-arr" / "Dockerfile").is_file():
        raise RuntimeError("GitHub archive does not contain the expected rust-arr Docker build context")
    build_context = roots[0] / "rust-arr"
    command(["docker", "build", "--platform", "linux/amd64", "--build-arg",
             f"RAWARR_BUILD_REVISION={revision}", "--label",
             f"org.opencontainers.image.revision={revision}", "-t", tag,
             str(build_context)], capture=False)
    if not image_matches(tag, revision):
        raise RuntimeError("Built image does not match the requested revision/platform")
    image_id = json.loads(command(["docker", "image", "inspect", tag]).stdout)[0]["Id"]
    return tag, image_id


def load_token(path: Path) -> str:
    token = path.read_text(encoding="utf-8").strip()
    if not token or "\n" in token:
        raise RuntimeError("GitHub token file is empty or malformed")
    return token


def run_once() -> int:
    platform = command(["docker", "info", "--format", "{{.OSType}}/{{.Architecture}}"]).stdout.strip()
    if platform not in {"linux/x86_64", "linux/amd64"}:
        raise RuntimeError(f"do3 must build natively as linux/amd64; Docker reports {platform}")
    token_path = Path(os.environ.get("RAWARR_GITHUB_TOKEN_FILE", "/root/gh-token"))
    token = load_token(token_path)
    head = get_main_sha(token)
    internal_rev = deployed_revision(INTERNAL["container"])
    public_rev = deployed_revision(PUBLIC["container"])
    if internal_rev == head and public_rev == head:
        print(f"both coordinators already run main {head}")
        return 0
    # If a prior run stopped between internal and public promotion, sync the
    # older coordinator to the newer already-published image before inspecting
    # commits after it. This also handles a docs-only commit arriving mid-rollout.
    force_latest = False
    paths: list[str] = []
    if internal_rev and public_rev and internal_rev != public_rev:
        try:
            relation = compare_revisions(token, internal_rev, public_rev)
        except urllib.error.URLError:
            relation = {}
        status = relation.get("status")
        if status == "ahead":
            newer_revision, older_role, older_target = public_rev, "internal", INTERNAL
        elif status == "behind":
            newer_revision, older_role, older_target = internal_rev, "public", PUBLIC
        else:
            newer_revision, older_role, older_target = "", "", {}
        if newer_revision:
            relation_to_main = compare_revisions(token, newer_revision, head)
            if relation_to_main.get("status") not in {"ahead", "identical"}:
                newer_revision = ""
                force_latest = True
        if newer_revision:
            image = f"rawrarr:{newer_revision}-amd64"
            if image_matches(image, newer_revision):
                image_id = json.loads(command(["docker", "image", "inspect", image]).stdout)[0]["Id"]
                print(f"promoting existing {newer_revision} image to the lagging {older_role} coordinator")
                deploy_role(older_role, older_target, image, image_id, newer_revision)
                internal_rev = deployed_revision(INTERNAL["container"])
                public_rev = deployed_revision(PUBLIC["container"])
                if internal_rev == public_rev:
                    if internal_rev == head:
                        return 0
            else:
                force_latest = True
        else:
            force_latest = True
    base = internal_rev
    if base and base != head and not force_latest:
        paths = changed_paths(token, base, head)
        if not affects_coordinator(paths):
            if internal_rev == public_rev:
                print(f"main {head} has no coordinator runtime changes since {base}")
            else:
                print("coordinator revisions differ and could not be ordered; refusing to leave them divergent")
                force_latest = True
        if internal_rev == public_rev and not affects_coordinator(paths) and not force_latest:
            return 0
    elif not base:
        force_latest = True
    if force_latest:
        paths = ["rust-arr/unknown-deployment-state"]
    elif not paths:
        paths = ["rust-arr/unknown-base"]
    print(f"coordinator changes on main {head}: {', '.join(paths[:12])}", flush=True)
    build_dir = INTERNAL["project_dir"] / "builds" / f"github-main-{head[:7]}-auto-{time.time_ns()}"
    image, image_id = download_and_build(token, head, build_dir)
    if get_main_sha(token) != head:
        print(f"main advanced while building {head}; leaving services unchanged for the next poll")
        return 0
    # Save rollback points before replacing the reusable cache tag with this image.
    preserve_tag(INTERNAL["image_base"], f"rawrarr:rollback-base-before-{head[:7]}-amd64")
    for role, target in (("internal", INTERNAL), ("public", PUBLIC)):
        preserve_image(target["container"], f"{role}-roarlay", head)
    command(["docker", "tag", image_id, INTERNAL["image_base"]])
    if get_main_sha(token) != head:
        print(f"main advanced before deployment of {head}; services will remain unchanged for the next poll")
        return 0
    if internal_rev != head:
        deploy_role("internal", INTERNAL, image, image_id, head)
    if get_main_sha(token) != head:
        print(f"main advanced after internal verification; public promotion will follow the next poll")
        return 0
    if public_rev != head:
        deploy_role("public", PUBLIC, image, image_id, head)
    print(f"both coordinator deployments finished for {head}")
    return 0


def main() -> int:
    try:
        return run_once()
    except (OSError, RuntimeError, subprocess.CalledProcessError, urllib.error.URLError,
            json.JSONDecodeError, KeyError, tarfile.TarError) as error:
        print(f"coordinator update failed: {error}", file=sys.stderr, flush=True)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
