#!/usr/bin/env python3
"""Create a protected, local recovery backup of the internal do3 coordinator."""

from __future__ import annotations

import argparse
import datetime as dt
import hashlib
import json
import os
import re
import shutil
import subprocess
import sys
import tarfile
import tempfile
from pathlib import Path


ROOT = Path("/opt/rawrarr-internal-do3")
BACKUP_ROOT = ROOT / "backups"
STAGING_ROOT = Path("/run")
AGE_HEADER = b"age-encryption.org/v1\n"
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


def require_tmpfs_staging() -> None:
    mounts = Path("/proc/mounts").read_text(encoding="utf-8")
    best_mount = None
    best_type = None
    for line in mounts.splitlines():
        fields = line.split()
        if len(fields) < 3:
            continue
        mountpoint = Path(fields[1].replace("\\040", " "))
        try:
            STAGING_ROOT.resolve().relative_to(mountpoint.resolve())
        except ValueError:
            continue
        if best_mount is None or len(str(mountpoint)) > len(str(best_mount)):
            best_mount = mountpoint
            best_type = fields[2]
    if best_type != "tmpfs":
        raise RuntimeError("/run must be a tmpfs so plaintext staging stays in memory")
    if shutil.disk_usage(STAGING_ROOT).free < 512 * 1024 * 1024:
        raise RuntimeError("less than 512 MiB is free in /run tmpfs for plaintext staging")


def load_recipient() -> tuple[str, str]:
    result = run([
        "docker", "exec", CONTAINERS["postgres"], "psql", "-X", "-qAt",
        "--set", "ON_ERROR_STOP=1", "--username", "rawrarr", "--dbname", "rawrarr",
        "--command",
        "BEGIN READ ONLY; SELECT age_recipient FROM rawrarr_environment_owner "
        "WHERE singleton=TRUE; COMMIT;",
    ])
    value = result.stdout.strip()
    if not re.fullmatch(r"age1pq1[a-z0-9]{13,4089}", value):
        raise RuntimeError("provision one valid environment owner public age recipient in the internal coordinator database")
    return value, hashlib.sha256(value.encode("ascii")).hexdigest()


def validate_age_recipient(recipient: str) -> str:
    with tempfile.TemporaryFile(dir=STAGING_ROOT) as plaintext, tempfile.TemporaryFile(dir=STAGING_ROOT) as ciphertext:
        plaintext.write(b"Rawrarr backup recipient preflight\n")
        plaintext.seek(0)
        run(
            ["docker", "exec", "-i", CONTAINERS["roarlay"], "age", "--recipient", recipient],
            stdin=plaintext,
            stdout=ciphertext,
        )
        ciphertext.seek(0)
        if ciphertext.read(len(AGE_HEADER)) != AGE_HEADER:
            raise RuntimeError("age recipient preflight did not produce a valid encrypted-file header")
    return run(["docker", "exec", CONTAINERS["roarlay"], "age", "--version"]).stdout.strip()


def verify_plain_backup(source: Path) -> None:
    checksum_file = source / "SHA256SUMS"
    if not checksum_file.is_file() or checksum_file.is_symlink():
        raise RuntimeError("existing backup has no safe SHA256SUMS file")
    entries = checksum_file.read_text(encoding="utf-8").splitlines()
    if not entries:
        raise RuntimeError("existing backup checksum manifest is empty")
    expected_paths = set()
    for line in entries:
        try:
            expected, relative = line.split("  ", 1)
        except ValueError as exc:
            raise RuntimeError("existing backup has an invalid checksum manifest") from exc
        path_part = Path(relative)
        if path_part.is_absolute() or ".." in path_part.parts:
            raise RuntimeError("existing backup checksum manifest contains an unsafe path")
        artifact = source / path_part
        if artifact.is_symlink() or not artifact.is_file() or sha256(artifact) != expected:
            raise RuntimeError(f"existing backup checksum failed: {relative}")
        expected_paths.add(path_part.as_posix())
    actual_paths = set()
    for item in source.rglob("*"):
        if item.is_symlink():
            raise RuntimeError(f"refusing symlink in existing backup: {item.name}")
        if item.is_file() and item.name != "SHA256SUMS":
            actual_paths.add(item.relative_to(source).as_posix())
    if actual_paths != expected_paths:
        raise RuntimeError("existing backup file set does not match its checksum manifest")


def tar_directory(source: Path, destination: Path) -> None:
    with tarfile.open(destination, "w:gz", format=tarfile.PAX_FORMAT) as archive:
        for item in sorted(source.rglob("*")):
            if item.is_symlink():
                raise RuntimeError(f"refusing symlink in backup staging tree: {item.name}")
            archive.add(item, arcname=item.relative_to(source).as_posix(), recursive=False)
    os.chmod(destination, 0o600)
    validate_archive(destination)


def encrypt_directory(source: Path, backup_root: Path, archive_name: str, recipient: str) -> Path:
    if not archive_name.endswith(".tar.gz.age"):
        archive_name += ".tar.gz.age"
    final = backup_root / archive_name
    if final.exists():
        raise RuntimeError(f"encrypted backup already exists: {final.name}")
    with tempfile.NamedTemporaryFile(prefix="rawrarr-backup-", suffix=".tar.gz", dir=STAGING_ROOT, delete=False) as tar_file:
        tar_path = Path(tar_file.name)
    with tempfile.NamedTemporaryFile(prefix=f".{archive_name}.", suffix=".incomplete", dir=backup_root, delete=False) as encrypted_file:
        encrypted_path = Path(encrypted_file.name)
        os.chmod(encrypted_path, 0o600)
    try:
        tar_directory(source, tar_path)
        with tar_path.open("rb") as plaintext, encrypted_path.open("wb") as ciphertext:
            run(
                ["docker", "exec", "-i", CONTAINERS["roarlay"], "age", "--recipient", recipient],
                stdin=plaintext,
                stdout=ciphertext,
            )
            ciphertext.flush()
            os.fsync(ciphertext.fileno())
        with encrypted_path.open("rb") as ciphertext:
            if ciphertext.read(len(AGE_HEADER)) != AGE_HEADER:
                raise RuntimeError("age did not produce a valid encrypted-file header")
        os.link(encrypted_path, final)
        directory_fd = os.open(backup_root, os.O_RDONLY)
        try:
            os.fsync(directory_fd)
        finally:
            os.close(directory_fd)
        return final
    finally:
        tar_path.unlink(missing_ok=True)
        encrypted_path.unlink(missing_ok=True)


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
    require_tmpfs_staging()
    recipient, recipient_digest = load_recipient()
    age_version = validate_age_recipient(recipient)

    lock = output_root / ".internal-coordinator-backup.lock"
    lock_fd = os.open(lock, os.O_CREAT | os.O_EXCL | os.O_WRONLY, 0o600)
    os.close(lock_fd)
    stage = None
    try:
        now = dt.datetime.now(dt.timezone.utc)
        backup_id = f"internal-coordinator-{now.strftime('%Y%m%dT%H%M%SZ')}"
        stage = Path(tempfile.mkdtemp(prefix=f".{backup_id}.", suffix=".plaintext", dir=STAGING_ROOT))
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
            "encryption": {
                "format": "age",
                "age_version": age_version,
                "recipient_sha256": recipient_digest,
            },
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
        return encrypt_directory(stage, output_root, backup_id, recipient)
    except BaseException:
        if stage is not None:
            shutil.rmtree(stage, ignore_errors=True)
        raise
    finally:
        if stage is not None:
            shutil.rmtree(stage, ignore_errors=True)
        lock.unlink(missing_ok=True)


def encrypt_existing_backup(source: Path, output_root: Path) -> Path:
    if os.geteuid() != 0:
        raise RuntimeError("run this command as root on do3")
    if source.is_symlink() or not source.is_dir() or source.parent.resolve() != output_root.resolve():
        raise RuntimeError("existing backup must be a direct, real directory under the backup directory")
    if not source.name.startswith("internal-coordinator-"):
        raise RuntimeError("existing backup directory name is not recognized")
    if source.stat().st_mode & 0o077:
        raise RuntimeError("existing backup directory must not grant group or other permissions")
    if output_root.stat().st_mode & 0o077:
        raise RuntimeError("backup directory must not grant group or other permissions")
    if not output_root.is_absolute() or output_root.resolve() != BACKUP_ROOT.resolve():
        raise RuntimeError(f"backup destination must be {BACKUP_ROOT}")
    if shutil.disk_usage(output_root).free < 512 * 1024 * 1024:
        raise RuntimeError("less than 512 MiB is free on the backup filesystem")
    require_tmpfs_staging()
    recipient, recipient_digest = load_recipient()
    age_version = validate_age_recipient(recipient)
    verify_plain_backup(source)
    lock = output_root / ".internal-coordinator-backup.lock"
    lock_fd = os.open(lock, os.O_CREAT | os.O_EXCL | os.O_WRONLY, 0o600)
    os.close(lock_fd)
    stage = None
    try:
        stage = Path(tempfile.mkdtemp(prefix=f".{source.name}.", suffix=".plaintext", dir=STAGING_ROOT))
        os.chmod(stage, 0o700)
        for item in sorted(source.rglob("*")):
            if item.is_symlink():
                raise RuntimeError(f"refusing symlink in existing backup: {item.name}")
            relative = item.relative_to(source)
            target = stage / relative
            if item.is_dir():
                target.mkdir(mode=0o700, parents=True, exist_ok=True)
            elif item.is_file():
                target.parent.mkdir(mode=0o700, parents=True, exist_ok=True)
                shutil.copyfile(item, target)
                os.chmod(target, 0o600)
        manifest_path = stage / "manifest.json"
        if manifest_path.is_file() and not manifest_path.is_symlink():
            manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
            manifest["encryption"] = {
                "format": "age",
                "age_version": age_version,
                "recipient_sha256": recipient_digest,
            }
            manifest_path.write_text(json.dumps(manifest, indent=2, sort_keys=True) + "\n", encoding="utf-8")
            os.chmod(manifest_path, 0o600)
            write_checksums(stage)
        return encrypt_directory(stage, output_root, source.name, recipient)
    finally:
        if stage is not None:
            shutil.rmtree(stage, ignore_errors=True)
        lock.unlink(missing_ok=True)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output-dir", type=Path, default=BACKUP_ROOT)
    parser.add_argument("--encrypt-existing", type=Path, help="make an encrypted copy of an existing plaintext backup directory")
    args = parser.parse_args()
    try:
        if args.encrypt_existing:
            result = encrypt_existing_backup(args.encrypt_existing, args.output_dir)
        else:
            result = create_backup(args.output_dir)
    except (OSError, RuntimeError, UnicodeError, subprocess.CalledProcessError, json.JSONDecodeError) as exc:
        detail = exc.stderr.strip() if isinstance(exc, subprocess.CalledProcessError) and exc.stderr else str(exc)
        print(f"backup failed: {detail}", file=sys.stderr)
        return 1
    print(f"encrypted backup created: {result}")
    print("The archive is age-encrypted. Verify decryption on a device with the matching private age identity.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
