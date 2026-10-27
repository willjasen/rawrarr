#!/usr/bin/env python3
"""Copy one explicitly selected file collection into the Syncthing /shared/out tree.

Dry-run by default. --apply creates a copy; it never deletes or overwrites files.
"""

import argparse
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
        for name in list(directories):
            path = current_path / name
            if path.is_symlink():
                raise ValueError("source contains a symbolic link")
        for name in names:
            path = current_path / name
            if path.is_symlink() or not path.is_file():
                raise ValueError("source contains a non-regular file")
            relative = path.relative_to(source)
            size = path.stat().st_size
            files.append((path, relative, size))
            total_bytes += size
    return files, total_bytes


def destination_name(value: str) -> str:
    if not value or value in (".", "..") or "/" in value or "\\" in value or "\x00" in value:
        raise ValueError("name must be one directory name under /shared/out")
    return value


def is_within(path: Path, parent: Path) -> bool:
    try:
        path.relative_to(parent)
        return True
    except ValueError:
        return False


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


def inspect_destination(files, target: Path):
    missing = []
    conflicts = []
    for source_file, relative, _ in files:
        destination = target / relative
        for parent in destination.parents:
            if parent == target.parent:
                break
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
            missing.append((source_file, relative, destination))
    return missing, conflicts


def copy_file(source: Path, destination: Path):
    destination.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.NamedTemporaryFile(prefix=".rawrarr-publish-", dir=destination.parent, delete=False) as temporary:
        temporary_path = Path(temporary.name)
        try:
            with source.open("rb") as source_file:
                shutil.copyfileobj(source_file, temporary, length=1024 * 1024)
            temporary.flush()
            os.fsync(temporary.fileno())
            try:
                os.link(temporary_path, destination)
            except FileExistsError:
                if not destination.is_file() or not same_contents(source, destination):
                    raise ValueError("destination changed during copy; no existing file was overwritten")
        finally:
            temporary_path.unlink(missing_ok=True)


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--source", required=True, type=Path, help="selected source directory, usually beneath /media")
    parser.add_argument("--shared-dir", required=True, type=Path, help="host path mounted as Syncthing's /shared folder")
    parser.add_argument("--name", required=True, help="destination directory name under /shared/out")
    parser.add_argument("--apply", action="store_true", help="copy missing files after preflight")
    args = parser.parse_args(argv)

    try:
        name = destination_name(args.name)
        if args.source.is_symlink():
            raise ValueError("source must not be a symbolic link")
        source = args.source.resolve(strict=True)
        shared = args.shared_dir.resolve(strict=True)
        if not source.is_dir():
            raise ValueError("source must be an existing directory")
        if not shared.is_dir() or shared.is_symlink():
            raise ValueError("shared directory must be an existing directory")
        if is_within(source, shared) or is_within(shared, source):
            raise ValueError("source and shared directory must not overlap")
        output_root = shared / "out"
        if output_root.is_symlink() or (output_root.exists() and not output_root.is_dir()):
            raise ValueError("/shared/out must be a real directory when it exists")
        target = output_root / name
        if target.is_symlink() or (target.exists() and not target.is_dir()):
            raise ValueError("destination must be a directory, not a symbolic link or file")
        files, total_bytes = scan_tree(source)
        if not files:
            raise ValueError("source directory contains no regular files")
        missing, conflicts = inspect_destination(files, target)
        if conflicts:
            raise ValueError(f"preflight found {len(conflicts)} conflicting destination file(s); nothing was copied")
        copy_bytes = sum(size for _, relative, size in files if not (target / relative).exists())
        free_bytes = shutil.disk_usage(shared).free
        print(f"Source: selected directory ({len(files)} files, {total_bytes} bytes)")
        print(f"Destination: /shared/out/{name} ({len(missing)} new files, {copy_bytes} bytes)")
        print("Syncthing will replicate these files to every peer already sharing /shared.")
        print("Files are not encrypted at rest by Syncthing; ensure the selected scope is approved.")
        if copy_bytes > free_bytes:
            raise ValueError(f"shared filesystem has insufficient free space ({free_bytes} bytes available)")
        if not args.apply:
            print("Dry run; use --apply to copy the selected files.")
            return 0
        output_root.mkdir(parents=True, exist_ok=True)
        for source_file, relative, destination in missing:
            copy_file(source_file, destination)
        print(f"Published {len(missing)} files ({copy_bytes} bytes) to /shared/out/{name}.")
        return 0
    except (OSError, ValueError) as error:
        print(f"Publish failed: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
