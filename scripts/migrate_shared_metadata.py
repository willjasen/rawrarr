#!/usr/bin/env python3
"""Copy legacy /shared/metadata files into /shared/in/metadata without deleting sources.

Dry-run by default. Stop metadata writers and wait for Syncthing to become idle
before using --apply on an existing shared folder.
"""

import argparse
import hashlib
import os
from pathlib import Path
import shutil
import sys
import tempfile


def scan_tree(source: Path):
    files = []
    total_bytes = 0
    for current, directories, names in os.walk(source, followlinks=False):
        current_path = Path(current)
        if current_path.is_symlink():
            raise ValueError("legacy metadata contains a symbolic link")
        for name in list(directories):
            if (current_path / name).is_symlink():
                raise ValueError("legacy metadata contains a symbolic link")
        for name in names:
            path = current_path / name
            if path.is_symlink() or not path.is_file():
                raise ValueError("legacy metadata contains a non-regular file")
            relative = path.relative_to(source)
            size = path.stat().st_size
            files.append((path, relative, size))
            total_bytes += size
    return files, total_bytes


def same_contents(left: Path, right: Path) -> bool:
    if left.stat().st_size != right.stat().st_size:
        return False
    with left.open("rb") as left_file, right.open("rb") as right_file:
        while True:
            left_bytes = left_file.read(1024 * 1024)
            right_bytes = right_file.read(1024 * 1024)
            if left_bytes != right_bytes:
                return False
            if not left_bytes:
                return True


def tree_manifest(files):
    manifest = []
    for source_file, relative, size in files:
        digest = hashlib.sha256()
        with source_file.open("rb") as stream:
            for block in iter(lambda: stream.read(1024 * 1024), b""):
                digest.update(block)
        manifest.append((relative.as_posix(), size, digest.hexdigest()))
    return sorted(manifest)


def plan(files, target: Path):
    missing = []
    conflicts = []
    for source_file, relative, size in files:
        destination = target / relative
        parent = target
        for part in relative.parts[:-1]:
            parent = parent / part
            if parent.is_symlink() or (parent.exists() and not parent.is_dir()):
                conflicts.append(relative)
                break
        if relative in conflicts:
            continue
        if destination.is_symlink():
            conflicts.append(relative)
        elif destination.exists():
            if not destination.is_file() or not same_contents(source_file, destination):
                conflicts.append(relative)
        else:
            missing.append((source_file, relative, size))
    return missing, conflicts


def copy_file(source: Path, destination: Path):
    destination.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.NamedTemporaryFile(prefix=".rawrarr-migrate-", dir=destination.parent, delete=False) as temporary:
        temporary_path = Path(temporary.name)
        try:
            with source.open("rb") as source_file:
                shutil.copyfileobj(source_file, temporary, length=1024 * 1024)
            temporary.flush()
            os.fsync(temporary.fileno())
            os.chmod(temporary_path, 0o644)
            try:
                os.link(temporary_path, destination)
            except FileExistsError:
                if not destination.is_file() or not same_contents(source, destination):
                    raise ValueError("destination changed during migration; no existing file was overwritten")
        finally:
            temporary_path.unlink(missing_ok=True)


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--shared-dir", required=True, type=Path, help="existing folder mounted as Syncthing's /shared")
    parser.add_argument("--apply", action="store_true", help="copy missing metadata after preflight")
    args = parser.parse_args(argv)
    try:
        shared = args.shared_dir.resolve(strict=True)
        if args.shared_dir.is_symlink() or not shared.is_dir():
            raise ValueError("shared directory must be a real directory")
        source = shared / "metadata"
        incoming = shared / "in"
        target = incoming / "metadata"
        if source.is_symlink() or not source.is_dir():
            raise ValueError("legacy /shared/metadata directory does not exist")
        if incoming.is_symlink() or (incoming.exists() and not incoming.is_dir()):
            raise ValueError("/shared/in must be a real directory when it exists")
        if target.is_symlink() or (target.exists() and not target.is_dir()):
            raise ValueError("/shared/in/metadata must be a real directory when it exists")
        files, total_bytes = scan_tree(source)
        if not files:
            raise ValueError("legacy metadata directory contains no regular files")
        missing, conflicts = plan(files, target)
        if conflicts:
            raise ValueError(f"preflight found {len(conflicts)} conflicting destination file(s); nothing was copied")
        copy_bytes = sum(size for _, _, size in missing)
        free_bytes = shutil.disk_usage(shared).free
        print(f"Legacy source: /shared/metadata ({len(files)} files, {total_bytes} bytes)")
        print(f"New destination: /shared/in/metadata ({len(missing)} files to copy, {copy_bytes} bytes)")
        print("Legacy files will remain in place; no existing destination file will be overwritten.")
        print(f"Legacy SHA-256 manifest: {hashlib.sha256(repr(tree_manifest(files)).encode()).hexdigest()}")
        if copy_bytes > free_bytes:
            raise ValueError(f"shared filesystem has insufficient free space ({free_bytes} bytes available)")
        if not args.apply:
            print("Dry run; stop metadata writers, wait for Syncthing to become idle, then use --apply to migrate.")
            return 0
        for source_file, relative, _ in missing:
            copy_file(source_file, target / relative)
        print(f"Copied {len(missing)} files into /shared/in/metadata; the legacy source is preserved.")
        return 0
    except (OSError, ValueError) as error:
        print(f"Migration failed: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
