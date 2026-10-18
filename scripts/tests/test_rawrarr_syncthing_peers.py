#!/usr/bin/env python3
"""Run two isolated Syncthing nodes and verify a web-to-storage folder sync."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path
import secrets
import shutil
import socket
import subprocess
import sys
import tempfile
import time
import urllib.error
import urllib.parse
import urllib.request
import xml.etree.ElementTree as ET


def free_port() -> int:
    with socket.socket() as listener:
        listener.bind(("127.0.0.1", 0))
        return listener.getsockname()[1]


def api(base_url: str, key: str, path: str, body: object | None = None) -> object:
    payload = None if body is None else json.dumps(body).encode()
    request = urllib.request.Request(
        base_url + path,
        data=payload,
        method="GET" if body is None else "POST",
        headers={"X-API-Key": key, "Accept": "application/json", "Content-Type": "application/json"},
    )
    with urllib.request.urlopen(request, timeout=5) as response:
        raw = response.read()
        return json.loads(raw) if raw else None


def set_xml(parent: ET.Element, name: str, value: str) -> None:
    element = parent.find(name)
    if element is None:
        element = ET.SubElement(parent, name)
    element.text = value


def prepare_node(binary: Path, home: Path, folder: Path, gui_port: int, sync_port: int) -> tuple[subprocess.Popen, str, str]:
    folder.mkdir(parents=True, exist_ok=True)
    subprocess.run(
        [str(binary), "generate", "--home", str(home), "--no-port-probing"],
        check=True,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
    )
    config_path = home / "config.xml"
    tree = ET.parse(config_path)
    root = tree.getroot()
    gui = root.find("gui")
    options = root.find("options")
    if gui is None or options is None:
        raise RuntimeError("Syncthing generated an unexpected configuration")
    api_key = secrets.token_hex(24)
    set_xml(gui, "address", f"127.0.0.1:{gui_port}")
    set_xml(gui, "apikey", api_key)
    set_xml(options, "listenAddress", f"tcp://127.0.0.1:{sync_port}")
    set_xml(options, "globalAnnounceEnabled", "false")
    set_xml(options, "localAnnounceEnabled", "false")
    set_xml(options, "relaysEnabled", "false")
    set_xml(options, "natEnabled", "false")
    set_xml(options, "crashReportingEnabled", "false")
    set_xml(options, "autoUpgradeIntervalH", "0")
    for old_folder in list(root.findall("folder")):
        root.remove(old_folder)
    tree.write(config_path, encoding="utf-8", xml_declaration=True)
    os.chmod(config_path, 0o600)
    process = subprocess.Popen(
        [str(binary), "serve", "--home", str(home), "--no-browser", "--no-port-probing", "--no-upgrade"],
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
    )
    return process, f"http://127.0.0.1:{gui_port}", api_key


def wait_ready(processes: list[subprocess.Popen], nodes: list[tuple[str, str]]) -> None:
    deadline = time.monotonic() + 30
    for base_url, key in nodes:
        while time.monotonic() < deadline:
            if any(process.poll() is not None for process in processes):
                raise RuntimeError("A Syncthing node exited during startup")
            try:
                api(base_url, key, "/rest/system/ping")
                break
            except (OSError, urllib.error.URLError):
                time.sleep(0.2)
        else:
            raise RuntimeError("Syncthing REST API did not become ready")


def add_device(base_url: str, key: str, device_id: str, name: str, address: str) -> None:
    device = api(base_url, key, "/rest/config/defaults/device")
    device.update({"deviceID": device_id, "name": name, "addresses": [address]})
    api(base_url, key, "/rest/config/devices", device)


def add_folder(base_url: str, key: str, folder_id: str, label: str, path: Path, peer_id: str, folder_type: str) -> None:
    folder = api(base_url, key, "/rest/config/defaults/folder")
    folder.update(
        {
            "id": folder_id,
            "label": label,
            "path": str(path),
            "type": folder_type,
            "devices": [{"deviceID": peer_id, "introducedBy": ""}],
            "paused": False,
        }
    )
    api(base_url, key, "/rest/config/folders", folder)


def wait_synced(processes: list[subprocess.Popen], nodes: list[tuple[str, str]], folder_id: str, deadline_seconds: int) -> None:
    deadline = time.monotonic() + deadline_seconds
    while time.monotonic() < deadline:
        if any(process.poll() is not None for process in processes):
            raise RuntimeError("A Syncthing node exited during synchronization")
        statuses = []
        try:
            for base_url, key in nodes:
                query = urllib.parse.urlencode({"folder": folder_id})
                statuses.append(api(base_url, key, f"/rest/db/status?{query}"))
            if len(statuses) == 2 and all(
                status.get("state") == "idle"
                and status.get("needFiles", 1) == 0
                and status.get("globalFiles", 0) >= 1
                and status.get("localFiles", 0) >= 1
                for status in statuses
            ):
                return
        except (OSError, urllib.error.URLError, KeyError, TypeError):
            pass
        time.sleep(0.5)
    raise RuntimeError("The folder did not reach synced/idle state before timeout")


def stop(processes: list[subprocess.Popen]) -> None:
    for process in processes:
        if process.poll() is None:
            process.terminate()
    for process in processes:
        if process.poll() is None:
            try:
                process.wait(timeout=8)
            except subprocess.TimeoutExpired:
                process.kill()
                process.wait(timeout=5)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=Path, default=shutil.which("syncthing"), help="Syncthing v2.1+ executable")
    parser.add_argument("--timeout", type=int, default=90, help="seconds to wait for the first folder sync")
    args = parser.parse_args()
    if not args.binary:
        parser.error("Syncthing executable not found; use --binary PATH")
    binary = args.binary.resolve()
    if not binary.is_file():
        parser.error(f"Syncthing executable not found: {binary}")
    version = subprocess.run([str(binary), "--version"], check=True, capture_output=True, text=True).stdout.strip()
    if not version.startswith("syncthing v2.1"):
        parser.error(f"expected Syncthing v2.1+, found: {version.splitlines()[0]}")

    processes: list[subprocess.Popen] = []
    with tempfile.TemporaryDirectory(prefix="rawrarr-syncthing-pair-") as directory:
        root = Path(directory)
        web_home, storage_home = root / "web-node", root / "storage-node"
        web_folder, storage_folder = root / "web-shared", root / "storage-shared"
        nested = web_folder / "metadata" / "movies"
        nested.mkdir(parents=True)
        payload = b"rawrarr syncthing peer sync integration fixture\n"
        (nested / "sample-metadata.json").write_bytes(payload)
        ports = [free_port() for _ in range(4)]
        try:
            web_process, web_url, web_key = prepare_node(binary, web_home, web_folder, ports[0], ports[1])
            processes.append(web_process)
            storage_process, storage_url, storage_key = prepare_node(binary, storage_home, storage_folder, ports[2], ports[3])
            processes.append(storage_process)
            nodes = [(web_url, web_key), (storage_url, storage_key)]
            wait_ready(processes, nodes)
            web_status = api(web_url, web_key, "/rest/system/status")
            storage_status = api(storage_url, storage_key, "/rest/system/status")
            web_id = web_status.get("myID")
            storage_id = storage_status.get("myID")
            if not web_id or not storage_id or web_id == storage_id:
                raise RuntimeError("Syncthing nodes did not report distinct device IDs")

            add_device(web_url, web_key, storage_id, "storage-node", f"tcp://127.0.0.1:{ports[3]}")
            add_device(storage_url, storage_key, web_id, "web-node", f"tcp://127.0.0.1:{ports[1]}")
            folder_id = "rawrarr-peer-test"
            add_folder(web_url, web_key, folder_id, "Rawrarr Shared Test", web_folder, storage_id, "sendonly")
            add_folder(storage_url, storage_key, folder_id, "Rawrarr Shared Test", storage_folder, web_id, "receiveonly")
            wait_synced(processes, nodes, folder_id, args.timeout)

            copied = storage_folder / "metadata" / "movies" / "sample-metadata.json"
            if not copied.is_file() or hashlib.sha256(copied.read_bytes()).digest() != hashlib.sha256(payload).digest():
                raise RuntimeError("Syncthing reported idle but the expected file bytes were not copied")
            print(f"PASS {version}: independent web-node and storage-node identities connected")
            print("PASS folder was configured automatically on both peers; web node is send-only and storage node receive-only")
            print("PASS nested metadata/movies directory and sample-metadata.json synchronized with matching SHA-256")
        finally:
            stop(processes)
    print("PASS temporary peer configs, databases, test folder, and processes were removed")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (AssertionError, OSError, RuntimeError, subprocess.CalledProcessError, urllib.error.URLError) as error:
        print(f"FAIL Syncthing peer sync integration: {type(error).__name__}: {error}", file=sys.stderr)
        raise SystemExit(1)
