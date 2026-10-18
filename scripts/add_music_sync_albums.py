#!/usr/bin/env python3
"""Add exact artist/album paths to an existing Syncthing music filter."""

import argparse
import os
from pathlib import Path
import stat
import tempfile


def album_patterns(album):
    parts = album.split("/")
    if len(parts) != 2 or any(not part or part in (".", "..") for part in parts):
        raise ValueError("album must be an artist/album path")
    if any(any(char in part for char in "*?[]!\\\r\n") for part in parts):
        raise ValueError("album path contains a Syncthing pattern character")
    artist, title = parts
    # A directory pattern covers its contents; including the parent would
    # include every sibling album beneath that artist as well.
    return (f"!/{artist}/{title}",)


def updated_lines(original, albums):
    lines = original.splitlines()
    if not lines or lines[-1] != "*":
        raise ValueError("existing .stignore must end with a catch-all * pattern")
    additions = []
    for album in albums:
        for pattern in album_patterns(album):
            if pattern not in lines and pattern not in additions:
                additions.append(pattern)
    return lines[:-1] + additions + ["*"], additions


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, required=True, help="host music root containing .stignore")
    parser.add_argument("--album", action="append", required=True, help="exact artist/album path; repeat for each album")
    parser.add_argument("--source", action="store_true", help="require each selected album to exist at this root")
    parser.add_argument("--apply", action="store_true", help="write the filter; default is dry-run")
    args = parser.parse_args()

    try:
        root = args.root.resolve(strict=True)
        if not root.is_dir():
            raise ValueError("root is not a directory")
        ignore = root / ".stignore"
        if ignore.is_symlink() or not ignore.is_file():
            raise ValueError("root must contain a regular .stignore file")
        if not (root / ".stfolder").is_dir():
            raise ValueError("root must contain a .stfolder marker")
        for album in args.album:
            album_patterns(album)
            if args.source:
                artist, title = album.split("/")
                if (root / artist).is_symlink() or (root / artist / title).is_symlink():
                    raise ValueError(f"source album contains a symlink: {album}")
                if not (root / artist / title).is_dir():
                    raise ValueError(f"source album does not exist: {album}")
        original = ignore.read_text(encoding="utf-8")
        updated, additions = updated_lines(original, args.album)
        if not additions:
            print("No filter changes needed")
            return
        print("Add these exact allow patterns before the final *:")
        for pattern in additions:
            print(pattern)
        if not args.apply:
            print("Dry-run only; use --apply to write .stignore")
            return

        mode = stat.S_IMODE(ignore.stat().st_mode)
        fd, temporary = tempfile.mkstemp(prefix=".stignore.", dir=root)
        try:
            with os.fdopen(fd, "w", encoding="utf-8") as stream:
                stream.write("\n".join(updated) + "\n")
                stream.flush()
                os.fsync(stream.fileno())
            os.chmod(temporary, mode)
            os.replace(temporary, ignore)
        finally:
            if os.path.exists(temporary):
                os.unlink(temporary)
        print(f"Updated {ignore}")
    except (OSError, ValueError) as exc:
        parser.exit(1, f"error: {exc}\n")


if __name__ == "__main__":
    main()
