#!/usr/bin/env python3
"""Print a path-and-content SHA-256 fingerprint for a media folder."""

import argparse
import hashlib
import json
from pathlib import Path
import sys


def fingerprint(root):
    if not root.is_dir():
        raise ValueError("folder does not exist")
    paths = sorted(root.rglob("*"), key=lambda path: path.relative_to(root).as_posix())
    if any(path.is_symlink() for path in paths):
        raise ValueError("folder contains a symbolic link")
    digest = hashlib.sha256()
    files = 0
    size = 0
    for path in paths:
        if not path.is_file():
            continue
        relative = path.relative_to(root).as_posix().encode()
        digest.update(len(relative).to_bytes(4, "big"))
        digest.update(relative)
        content = hashlib.sha256()
        with path.open("rb") as stream:
            for chunk in iter(lambda: stream.read(1024 * 1024), b""):
                content.update(chunk)
                size += len(chunk)
        digest.update(content.digest())
        files += 1
    return {"files": files, "bytes": size, "sha256": digest.hexdigest()}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("folder", type=Path)
    args = parser.parse_args()
    try:
        print(json.dumps(fingerprint(args.folder)))
    except (OSError, ValueError) as error:
        print("Media hash failed: " + (str(error) if isinstance(error, ValueError) else type(error).__name__), file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
