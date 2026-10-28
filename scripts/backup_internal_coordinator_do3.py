#!/usr/bin/env python3
"""Create a protected, local recovery backup of the internal do3 coordinator."""

from __future__ import annotations

import argparse
import datetime as dt
import hashlib
import json
import os
import shutil
import subprocess
import sys
import tarfile
import tempfile
from pathlib import Path


ROOT = Path("/opt/rawrarr-internal-do3")
BACKUP_ROOT = ROOT / "backups"
CONTAINERS = {
    "roarlay": "rawrarr-internal-do3-roarlay",
    "postgres": "rawrarr-internal-do3-postgres",
    "tailscale": "rawrarr-internal-do3-tailscale",
    "umami": "rawrarr-internal-do3-umami-1",
    "umami_postgres": "rawrarr-internal-do3-umami-postgres-1",
    "analytics_caddy": "rawrarr-internal-do3-analytics-caddy",
}
VOLUMES = {
    "tailscale_state": "rawrarr-internal-do3-tailscale-state",
    "coordinator_ui": "rawrarr-internal-do3-ui",
}
COMPOSE_FILES = (
    "docker-compose.internal-coordinator.yml",
    "docker-compose.analytics.yml",
    "docker-compose.analytics-dashboard-internal.yml",
    "analytics-dashboard-internal.Caddyfile",
)
EXTERNAL_SECRET_FILES = {"ui-github-token": Path("/root/gh-token")}
DATABASES = (
    ("coordinator", CONTAINERS["postgres"], "rawrarr", "rawrarr"),
    ("analytics", CONTAINERS["umami_postgres"], "umami", "umami"),
)
HEALTH_REQUIRED = {CONTAINERS["roarlay"], CONTAINERS["postgres"], CONTAINERS["umami_postgres"]}


def run(args: list[str], *, stdout=None, stdin=None, check: bool = True) -> subprocess.CompletedProcess:
    capture = stdout is None
    return subprocess.run(
        args,
        stdin=stdin,
        stdout=subprocess.PIPE if capture else stdout,
        stderr=subprocess.PIPE,
        check=check,
        text=capture,
    )


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as source:
        for block in iter(lambda: source.read(1024 * 1024), b""):
            digest.update(block)
    return digest.hexdigest()


def copy_protected_secrets(destination: Path) -> list[str]:
    source = ROOT / "secrets"
    if not source.is_dir():
        raise RuntimeError(f"missing secrets directory: {source}")
    target = destination / "secrets"
    target.mkdir(mode=0o700)
    saved = []
    for entry in sorted(source.iterdir()):
        if entry.is_symlink() or not entry.is_file():
            raise RuntimeError(f"refusing non-regular secret entry: {entry.name}")
        output = target / entry.name
        shutil.copyfile(entry, output)
        os.chmod(output, 0o600)
        saved.append(entry.name)
    for archive_name, external_path in EXTERNAL_SECRET_FILES.items():
        if external_path.is_symlink() or not external_path.is_file():
            raise RuntimeError(f"missing or unsafe required mounted secret: {archive_name}")
        output = target / archive_name
        if output.exists():
            raise RuntimeError(f"duplicate secret filename: {archive_name}")
        shutil.copyfile(external_path, output)
        os.chmod(output, 0o600)
        saved.append(archive_name)
    if not saved:
        raise RuntimeError("no internal service secrets were found")
    return saved


def copy_deployment_files(destination: Path) -> list[str]:
    target = destination / "deployment"
    target.mkdir(mode=0o700)
    for name in COMPOSE_FILES:
        source = ROOT / name
        if not source.is_file() or source.is_symlink():
            raise RuntimeError(f"missing or unsafe deployment file: {name}")
        shutil.copyfile(source, target / name)
        os.chmod(target / name, 0o600)
    return list(COMPOSE_FILES)


def container_metadata(name: str) -> dict:
    result = run([
        "docker", "inspect", "--format",
        "{{.Name}}|{{.Image}}|{{.Config.Image}}|{{index .Config.Labels \"org.opencontainers.image.revision\"}}|{{.State.Status}}|{{if .State.Health}}{{.State.Health.Status}}{{end}}",
        name,
    ])
    fields = result.stdout.rstrip("\n").split("|", 5)
    if len(fields) != 6:
        raise RuntimeError(f"unable to read safe container metadata: {name}")
    return {
        "name": fields[0].lstrip("/"),
        "image_id": fields[1],
        "image_ref": fields[2],
        "revision": fields[3] or None,
        "status": fields[4],
        "health": fields[5] or None,
    }


def dump_database(destination: Path, name: str, container: str, user: str, database: str) -> int:
    output = destination / f"{name}.dump"
    with output.open("wb") as dump:
        os.chmod(output, 0o600)
        run(
            ["docker", "exec", container, "pg_dump", "--username", user,
             "--dbname", database, "--format=custom", "--no-owner", "--no-acl"],
            stdout=dump,
        )
    if output.stat().st_size == 0:
        raise RuntimeError(f"database dump is empty: {name}")
    with output.open("rb") as dump:
        catalog = run(["docker", "exec", "-i", container, "pg_restore", "--list"], stdin=dump)
    if not catalog.stdout.strip():
        raise RuntimeError(f"database dump catalog is empty: {name}")
    catalog_path = destination / f"{name}.catalog.txt"
    catalog_path.write_text(catalog.stdout, encoding="utf-8")
    os.chmod(catalog_path, 0o600)
    return output.stat().st_size


def volume_mountpoint(name: str) -> Path:
    result = run(["docker", "volume", "inspect", "--format", "{{.Mountpoint}}", name])
    path = Path(result.stdout.strip()).resolve(strict=True)
    expected = Path("/var/lib/docker/volumes").resolve()
    if expected not in path.parents or path.name != "_data":
        raise RuntimeError(f"unexpected Docker volume mountpoint for {name}")
    return path


def validate_archive(path: Path) -> None:
    with tarfile.open(path, "r:gz") as archive:
        if not archive.getmembers():
            raise RuntimeError(f"Docker volume archive is empty: {path.name}")
        for member in archive.getmembers():
            if member.name.startswith("/") or ".." in Path(member.name).parts:
                raise RuntimeError(f"unsafe archive path in {path.name}")
            if (member.issym() or member.islnk()) and (
                member.linkname.startswith("/") or ".." in Path(member.linkname).parts
            ):
                raise RuntimeError(f"unsafe archive link in {path.name}")


def archive_volume(destination: Path, archive_name: str, volume_name: str) -> int:
    source = volume_mountpoint(volume_name)
    output = destination / archive_name
    with tarfile.open(output, "w:gz", format=tarfile.PAX_FORMAT) as archive:
        archive.add(source, arcname=".", recursive=True)
    os.chmod(output, 0o600)
    validate_archive(output)
    return output.stat().st_size


def write_checksums(destination: Path) -> None:
    lines = []
    for path in sorted(destination.rglob("*")):
        if path.is_file() and path.name != "SHA256SUMS":
            relative = path.relative_to(destination).as_posix()
            lines.append(f"{sha256(path)}  {relative}")
    checksums = destination / "SHA256SUMS"
    checksums.write_text("\n".join(lines) + "\n", encoding="utf-8")
    os.chmod(checksums, 0o600)


def create_backup(output_root: Path) -> Path:
    if os.geteuid() != 0:
        raise RuntimeError("run this command as root on do3")
    if not output_root.is_absolute() or output_root.resolve() != BACKUP_ROOT.resolve():
        raise RuntimeError(f"backup destination must be {BACKUP_ROOT}")
    if not output_root.is_dir() or output_root.is_symlink():
        raise RuntimeError(f"backup directory must already exist and be a directory: {output_root}")
    if output_root.stat().st_mode & 0o077:
        raise RuntimeError("backup directory must not grant group or other permissions")
    if shutil.disk_usage(output_root).free < 512 * 1024 * 1024:
        raise RuntimeError("less than 512 MiB is free on the backup filesystem")

    lock = output_root / ".internal-coordinator-backup.lock"
    lock_fd = os.open(lock, os.O_CREAT | os.O_EXCL | os.O_WRONLY, 0o600)
    os.close(lock_fd)
    stage = None
    try:
        now = dt.datetime.now(dt.timezone.utc)
        backup_id = f"internal-coordinator-{now.strftime('%Y%m%dT%H%M%SZ')}"
        final = output_root / backup_id
        stage = Path(tempfile.mkdtemp(prefix=f".{backup_id}.", suffix=".incomplete", dir=output_root))
        os.chmod(stage, 0o700)
        services = {}
        for role, container in CONTAINERS.items():
            metadata = container_metadata(container)
            if metadata["status"] != "running":
                raise RuntimeError(f"required internal service is not running: {container}")
            if container in HEALTH_REQUIRED and metadata["health"] != "healthy":
                raise RuntimeError(f"required internal service is not healthy: {container}")
            services[role] = metadata

        sizes = {}
        for name, container, user, database in DATABASES:
            sizes[f"{name}.dump"] = dump_database(stage, name, container, user, database)
        for archive_name, volume in (("tailscale-state.tar.gz", VOLUMES["tailscale_state"]),
                                     ("coordinator-ui.tar.gz", VOLUMES["coordinator_ui"])):
            sizes[archive_name] = archive_volume(stage, archive_name, volume)

        saved_files = copy_deployment_files(stage)
        saved_secrets = copy_protected_secrets(stage)
        serve_status = run(["docker", "exec", CONTAINERS["tailscale"], "tailscale", "serve", "status", "--json"])
        serve_path = stage / "tailscale-serve-status.json"
        serve_path.write_text(serve_status.stdout, encoding="utf-8")
        os.chmod(serve_path, 0o600)

        manifest = {
            "backup_id": backup_id,
            "created_at_utc": now.isoformat(),
            "host": "do3",
            "scope": "internal Rawrarr coordinator and its private analytics services only",
            "services": services,
            "databases": {"coordinator": "rawrarr", "analytics": "umami"},
            "volumes": VOLUMES,
            "deployment_files": saved_files,
            "secret_filenames": saved_secrets,
            "artifact_sizes_bytes": sizes,
            "consistency": {
                "postgresql": "transaction-consistent logical pg_dump archives",
                "volumes": "live read-only filesystem archives; restore only while their services are stopped",
            },
            "images_saved": False,
        }
        manifest_path = stage / "manifest.json"
        manifest_path.write_text(json.dumps(manifest, indent=2, sort_keys=True) + "\n", encoding="utf-8")
        os.chmod(manifest_path, 0o600)
        write_checksums(stage)
        if final.exists():
            raise RuntimeError(f"backup already exists: {backup_id}")
        os.rename(stage, final)
        os.chmod(final, 0o700)
        return final
    except BaseException:
        if stage is not None:
            shutil.rmtree(stage, ignore_errors=True)
        raise
    finally:
        lock.unlink(missing_ok=True)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output-dir", type=Path, default=BACKUP_ROOT)
    args = parser.parse_args()
    try:
        result = create_backup(args.output_dir)
    except (OSError, RuntimeError, subprocess.CalledProcessError, json.JSONDecodeError) as exc:
        detail = exc.stderr.strip() if isinstance(exc, subprocess.CalledProcessError) and exc.stderr else str(exc)
        print(f"backup failed: {detail}", file=sys.stderr)
        return 1
    print(f"backup created: {result}")
    print(f"verify checksums: cd {result} && sha256sum --check SHA256SUMS")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
