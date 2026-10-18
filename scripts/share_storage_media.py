#!/usr/bin/env python3
"""Pair one approved storage node's media folder with one Syncthing peer."""

import argparse
import json
from pathlib import Path, PurePosixPath
import re
import shutil
import subprocess
import sys
import xml.etree.ElementTree as ET
from urllib.parse import urlencode, urlsplit
from urllib.request import Request, urlopen


def api(base, key, path, body=None):
    data = None if body is None else json.dumps(body).encode()
    request = Request(base + path, data=data, headers={
        "X-API-Key": key, "Accept": "application/json", "Content-Type": "application/json",
    }, method="GET" if data is None else "POST")
    with urlopen(request, timeout=10) as response:
        raw = response.read()
    return json.loads(raw) if raw else None


def validate_url(value):
    parsed = urlsplit(value)
    if (parsed.scheme not in ("http", "https") or not parsed.hostname or
            parsed.username or parsed.password or parsed.query or parsed.fragment or
            parsed.path not in ("", "/")):
        raise ValueError("API URL must be a bare HTTP(S) origin without credentials")
    if parsed.scheme == "http" and parsed.hostname not in ("localhost", "127.0.0.1", "::1"):
        raise ValueError("plain HTTP API access is allowed only through local loopback")
    return value.rstrip("/") + "/"


MUSIC_A_IGNORE = "!/[Aa]*\n!/[Aa]*/**\n*\n"


def valid_music_filter(value):
    lines = value.splitlines()
    if lines[:2] != ["!/[Aa]*", "!/[Aa]*/**"] or not lines or lines[-1] != "*":
        return False
    return all(re.fullmatch(r"!/(?!\.)[^/*?\[\]!\\]+/(?!\.)[^/*?\[\]!\\]+", line)
               for line in lines[2:-1])


def media_path(value, include_a_folders=False):
    if value == "/media" and include_a_folders:
        return value
    path = PurePosixPath(value)
    if (not value.startswith("/media/") or path.as_posix() != value or
            len(path.parts) < 3 or any(part in (".", "..") for part in value.split("/"))):
        raise ValueError("folder path must be a canonical subfolder beneath /media")
    return value


def check_a_filter(container, role):
    if not container:
        raise ValueError("the filtered /media share requires --docker-container for a mount preflight")
    docker = shutil.which("docker") or "/usr/local/bin/docker"
    actual = subprocess.check_output([
        docker, "--context", "orbstack", "exec", container, "cat", "/media/.stignore",
    ], stderr=subprocess.DEVNULL).decode()
    if not valid_music_filter(actual):
        raise ValueError("/media/.stignore must contain the documented A-folder filter and only exact album additions")
    if role == "source":
        subprocess.check_call([
            docker, "--context", "orbstack", "exec", container,
            "test", "-d", "/media/.stfolder",
        ], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)


def overlaps(left, right):
    return left == right or left.startswith(right.rstrip("/") + "/") or right.startswith(left.rstrip("/") + "/")


def inspect(base, key, args):
    local_id = api(base, key, "rest/system/status")["myID"]
    if local_id != args.local_id or local_id == args.peer_id:
        raise ValueError("Syncthing device IDs do not match the expected pair")
    folders = api(base, key, "rest/config/folders")
    devices = api(base, key, "rest/config/devices")
    if not isinstance(folders, list) or not isinstance(devices, list):
        raise ValueError("Syncthing returned unexpected configuration")
    for folder in folders:
        path = folder.get("path", "")
        if folder.get("id") == args.folder_id or (path.startswith("/") and overlaps(path, args.path)):
            raise ValueError("folder ID or media path is already managed by Syncthing")
    peer = next((device for device in devices if device.get("deviceID") == args.peer_id), None)
    if peer and peer.get("paused"):
        raise ValueError("the selected peer is paused")
    if peer and peer.get("autoAcceptFolders"):
        raise ValueError("the selected peer auto-accepts folders; disable that before pairing media")
    return peer is None


def configure(base, key, args, add_peer):
    if add_peer:
        peer = api(base, key, "rest/config/defaults/device")
        peer.update({"deviceID": args.peer_id, "name": args.peer_name,
                     "addresses": [args.peer_address], "autoAcceptFolders": False})
        api(base, key, "rest/config/devices", peer)
    folder = api(base, key, "rest/config/defaults/folder")
    folder.update({"id": args.folder_id, "label": args.label, "path": args.path,
                   "type": "sendonly" if args.role == "source" else "receiveonly",
                   "devices": [{"deviceID": args.peer_id}], "paused": False})
    api(base, key, "rest/config/folders", folder)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--api-url", default="http://127.0.0.1:8385")
    key_source = parser.add_mutually_exclusive_group(required=True)
    key_source.add_argument("--api-key-file", type=Path)
    key_source.add_argument("--docker-container", help="read API key from this local OrbStack Syncthing container")
    parser.add_argument("--show-id", action="store_true", help="print this Syncthing node's public device ID")
    parser.add_argument("--status", action="store_true", help="show one configured folder's sync state")
    parser.add_argument("--scan", action="store_true", help="request a Syncthing scan for one configured folder")
    parser.add_argument("--role", choices=("source", "destination"))
    parser.add_argument("--local-id", help="expected local Syncthing device ID")
    parser.add_argument("--peer-id", help="approved counterpart's Syncthing device ID")
    parser.add_argument("--peer-name")
    parser.add_argument("--peer-address", help="tcp://host:22000 or dynamic")
    parser.add_argument("--folder-id")
    parser.add_argument("--label")
    parser.add_argument("--path", help="existing folder inside the Syncthing /media bind")
    parser.add_argument("--include-a-folders", action="store_true",
                        help="share one /media root, limited by a preinstalled A/a top-level folder filter")
    parser.add_argument("--confirm-empty-destination", action="store_true",
                        help="required on destination after checking its host folder is empty")
    parser.add_argument("--apply", action="store_true", help="write Syncthing configuration")
    args = parser.parse_args()
    if (args.status or args.scan) and not args.folder_id:
        parser.error("--status and --scan require --folder-id")
    if not (args.show_id or args.status or args.scan):
        required = ("role", "local_id", "peer_id", "peer_name", "peer_address", "folder_id", "label", "path")
        missing = [name.replace("_", "-") for name in required if not getattr(args, name)]
        if missing:
            parser.error("missing pairing options: " + ", ".join("--" + name for name in missing))
    try:
        base = validate_url(args.api_url)
        if args.docker_container:
            docker = shutil.which("docker") or "/usr/local/bin/docker"
            raw = subprocess.check_output([
                docker, "--context", "orbstack", "exec", args.docker_container,
                "cat", "/var/syncthing/config/config.xml",
            ], stderr=subprocess.DEVNULL)
            key = (ET.fromstring(raw).findtext("gui/apikey") or "").strip()
        else:
            key = args.api_key_file.read_text().strip()
        if not key:
            raise ValueError("API key file is empty")
        if args.show_id:
            print(api(base, key, "rest/system/status")["myID"])
            return 0
        if args.scan:
            path = "rest/db/scan?" + urlencode({"folder": args.folder_id})
            if not args.apply:
                print("Dry run: would request a scan of " + args.folder_id)
                return 0
            api(base, key, path, {})
            print("Scan requested for " + args.folder_id)
            return 0
        if args.status:
            folders = api(base, key, "rest/config/folders")
            folder = next((item for item in folders if item.get("id") == args.folder_id), None)
            if folder is None:
                raise ValueError("folder is not configured")
            status = api(base, key, "rest/db/status?" + urlencode({"folder": args.folder_id}))
            print(json.dumps({"id": args.folder_id, "type": folder.get("type"),
                              "state": status.get("state"), "localFiles": status.get("localFiles"),
                              "globalFiles": status.get("globalFiles"), "needFiles": status.get("needFiles"),
                              "needBytes": status.get("needBytes"), "errors": status.get("errors")}))
            return 0
        media_path(args.path, args.include_a_folders)
        if args.include_a_folders:
            if args.path != "/media":
                raise ValueError("--include-a-folders requires --path /media")
            check_a_filter(args.docker_container, args.role)
        if not args.folder_id or len(args.folder_id) > 128 or "/" in args.folder_id:
            raise ValueError("folder ID must be a nonempty Syncthing ID without slashes")
        if not args.label.strip() or len(args.label) > 128 or not args.peer_name.strip():
            raise ValueError("label and peer name are required")
        if args.peer_address != "dynamic" and not args.peer_address.startswith("tcp://"):
            raise ValueError("peer address must be dynamic or tcp://host:port")
        add_peer = inspect(base, key, args)
        print(f"{args.role}: {args.path} -> folder {args.folder_id}, peer {args.peer_id}")
        print("Syncthing will use " + ("send-only" if args.role == "source" else "receive-only") + "; " +
              ("add peer" if add_peer else "reuse existing peer"))
        if not args.apply:
            print("Dry run; use --apply after verifying both paths, available space, and peer IDs.")
            return 0
        if args.role == "destination" and not args.confirm_empty_destination:
            raise ValueError("destination needs --confirm-empty-destination after inspecting the host folder")
        configure(base, key, args, add_peer)
        print("Configuration saved. Check Syncthing folder state and verify file hashes on both hosts.")
        return 0
    except (OSError, ValueError, KeyError, TypeError, ET.ParseError, subprocess.CalledProcessError) as error:
        # Do not include request bodies, API keys, or remote response text in errors.
        print("Media share failed: " + (str(error) if isinstance(error, ValueError) else type(error).__name__), file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
