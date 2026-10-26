#!/usr/bin/env python3
"""Download and atomically install the latest private Rawrarr UI release bundle.

Dry-run is the default; container entrypoints must pass --apply. Requires Python 3.
"""
from __future__ import annotations
import argparse, hashlib, json, os, pathlib, shutil, sys, tempfile, urllib.error, urllib.request, zipfile

MAX_BUNDLE = 32 * 1024 * 1024
MAX_FILE = 8 * 1024 * 1024
MIN_POLL_INTERVAL_SECONDS = 5 * 60
DEFAULT_POLL_INTERVAL_SECONDS = 5 * 60

def poll_interval_seconds(requested: int) -> int:
    return max(MIN_POLL_INTERVAL_SECONDS, requested)
def fetch(url: str, token: str, *, accept: str = "application/vnd.github+json") -> bytes:
    request = urllib.request.Request(url, headers={"Authorization": f"Bearer {token}", "Accept": accept, "X-GitHub-Api-Version": "2022-11-28", "User-Agent": "rawrarr-ui-updater"})
    with urllib.request.urlopen(request, timeout=30) as response:
        data = response.read(MAX_BUNDLE + 1)
    if len(data) > MAX_BUNDLE:
        raise ValueError("GitHub response exceeded the 32 MiB limit")
    return data

def update(repo: str, token_file: pathlib.Path, root: pathlib.Path, apply: bool) -> str:
    token = token_file.read_text(encoding="utf-8").strip()
    if not token: raise ValueError("GitHub token file is empty")
    release = json.loads(fetch(f"https://api.github.com/repos/{repo}/releases/latest", token))
    assets = {item["name"]: item for item in release.get("assets", [])}
    if not all(name in assets for name in ("ui-bundle.zip", "ui-bundle.zip.sha256")):
        raise ValueError("latest UI release is missing its bundle or checksum")
    def asset(item):
        return fetch(f"https://api.github.com/repos/{repo}/releases/assets/{item['id']}", token, accept="application/octet-stream")
    archive, checksum = asset(assets["ui-bundle.zip"]), asset(assets["ui-bundle.zip.sha256"])
    expected = checksum.decode("ascii").strip().split()[0]
    if hashlib.sha256(archive).hexdigest() != expected: raise ValueError("UI bundle checksum mismatch")
    with zipfile.ZipFile(__import__("io").BytesIO(archive)) as bundle:
        entries = bundle.infolist()
        if not entries or len(entries) > 512: raise ValueError("UI bundle has an invalid file count")
        seen = set()
        total = 0
        for entry in entries:
            p = pathlib.PurePosixPath(entry.filename)
            if p.is_absolute() or ".." in p.parts or not p.parts or entry.is_dir(): raise ValueError("UI bundle contains an unsafe path")
            if entry.filename in seen: raise ValueError("UI bundle contains a duplicate path")
            seen.add(entry.filename)
            total += entry.file_size
            if entry.file_size > MAX_FILE or total > MAX_BUNDLE: raise ValueError("UI bundle exceeds extraction limits")
        version = str(release.get("target_commitish", "unknown"))
        print(f"release={release.get('tag_name')} version={version} files={len(entries)} sha256={expected}")
        current = root / "current"
        if (current / "bundle.sha256").exists() and (current / "bundle.sha256").read_text().strip() == expected:
            print("already current"); return "current"
        print(f"would install checksum-verified UI bundle into {root}")
        if not apply: return "dry-run"
        root.mkdir(parents=True, exist_ok=True)
        stage = pathlib.Path(tempfile.mkdtemp(prefix=".staging-", dir=root))
        try:
            for entry in entries:
                destination = stage.joinpath(*pathlib.PurePosixPath(entry.filename).parts)
                destination.parent.mkdir(parents=True, exist_ok=True)
                destination.write_bytes(bundle.read(entry))
            (stage / "bundle.sha256").write_text(expected + "\n", encoding="ascii")
            (stage / "bundle.version").write_text(version + "\n", encoding="utf-8")
            previous = root / "previous"
            if previous.exists(): shutil.rmtree(previous)
            if current.exists(): os.replace(current, previous)
            os.replace(stage, current)
        finally:
            if stage.exists(): shutil.rmtree(stage)
        print("installed; previous bundle retained for rollback")
        return "installed"

def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo", default=os.environ.get("RAWARR_UI_GITHUB_REPO", "willjasen/rawrarr"))
    parser.add_argument("--token-file", type=pathlib.Path, default=pathlib.Path(os.environ.get("RAWARR_UI_GITHUB_TOKEN_FILE", "/run/secrets/rawrarr_ui_github_token")))
    parser.add_argument("--root", type=pathlib.Path, default=pathlib.Path(os.environ.get("RAWARR_UI_ROOT", "/ui")))
    parser.add_argument("--apply", action="store_true", help="install the checksum-verified bundle; default only reports planned action")
    parser.add_argument("--watch", action="store_true", help="check periodically; intended for the application container")
    parser.add_argument("--interval", type=int, default=DEFAULT_POLL_INTERVAL_SECONDS, help="seconds between watch checks (minimum 300)")
    args = parser.parse_args()
    while True:
        try: update(args.repo, args.token_file, args.root, args.apply)
        except (OSError, ValueError, KeyError, urllib.error.URLError, json.JSONDecodeError, zipfile.BadZipFile, UnicodeDecodeError) as exc:
            print(f"UI update failed: {exc}", file=sys.stderr, flush=True)
            if not args.watch: return 1
        if not args.watch: return 0
        import time
        time.sleep(poll_interval_seconds(args.interval))

if __name__ == "__main__": raise SystemExit(main())
