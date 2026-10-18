#!/usr/bin/env python3
"""Start isolated Syncthing web/storage containers in OrbStack and verify sync."""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
import shutil
import socket
import subprocess
import sys
import tempfile
import time
import urllib.parse
import urllib.request
import xml.etree.ElementTree as ET


CONTEXT = "orbstack"
IMAGE = "syncthing/syncthing:2.1.5"
PROJECT = "rawrarr-testing"
BASE36 = "0123456789abcdefghijklmnopqrstuvwxyz"


def timestamp_code() -> str:
    value = time.time_ns() // 1_000_000
    encoded = ""
    while value:
        value, digit = divmod(value, len(BASE36))
        encoded = BASE36[digit] + encoded
    return encoded or BASE36[0]


def free_port() -> int:
    with socket.socket() as listener:
        listener.bind(("127.0.0.1", 0))
        return listener.getsockname()[1]


def docker(*args: str, check: bool = True) -> str:
    result = subprocess.run(["docker", "--context", CONTEXT, *args], check=check,
                            capture_output=True, text=True)
    return result.stdout.strip()


def api(base: str, key: str, path: str, body: object | None = None) -> object:
    data = None if body is None else json.dumps(body).encode()
    request = urllib.request.Request(base + path, data=data,
        method="GET" if body is None else "POST",
        headers={"X-API-Key": key, "Accept": "application/json", "Content-Type": "application/json"})
    with urllib.request.urlopen(request, timeout=5) as response:
        raw = response.read()
        return json.loads(raw) if raw else None


def add_device(base: str, key: str, device_id: str, name: str, address: str) -> None:
    device = api(base, key, "/rest/config/defaults/device")
    device.update({"deviceID": device_id, "name": name, "addresses": [address]})
    api(base, key, "/rest/config/devices", device)


def add_folder(base: str, key: str, folder_id: str, path: str, peer_id: str, role: str) -> None:
    folder = api(base, key, "/rest/config/defaults/folder")
    folder.update({"id": folder_id, "label": "Rawrarr shared metadata", "path": path,
                   "type": role, "paused": False,
                   "devices": [{"deviceID": peer_id, "introducedBy": ""}]})
    api(base, key, "/rest/config/folders", folder)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--keep", action="store_true", help="leave the verified containers, network, and data running")
    args = parser.parse_args()
    if not shutil.which("docker"):
        raise RuntimeError("Docker CLI is required")
    context = docker("context", "show")
    if context != CONTEXT:
        raise RuntimeError(f"expected OrbStack Docker context '{CONTEXT}', got '{context}'")
    docker("info", "--format", "{{.ServerVersion}}")
    docker("pull", IMAGE)

    suffix = timestamp_code()
    network = f"{PROJECT}-net-{suffix}"
    names = [f"{PROJECT}-web-{suffix}", f"{PROJECT}-storage-{suffix}"]
    ports = [free_port(), free_port()]
    root = Path(tempfile.mkdtemp(prefix="rawrarr-orbstack-sync-"))
    passed = False
    try:
        docker("network", "create", "--label", f"com.docker.compose.project={PROJECT}", network)
        homes = [root / "web-home", root / "storage-home"]
        shares = [root / "web-share", root / "storage-share"]
        for home, share in zip(homes, shares):
            home.mkdir()
            share.mkdir()
        nested = shares[0] / "metadata" / "movies"
        nested.mkdir(parents=True)
        payload = b"rawrarr Syncthing OrbStack integration fixture\n"
        (nested / "sample-metadata.json").write_bytes(payload)
        identity_result = subprocess.run(
            ["age-keygen", "-pq"], check=True, capture_output=True, text=True
        )
        recipient = next(
            line.removeprefix("# public key: ").strip()
            for line in identity_result.stdout.splitlines()
            if line.startswith("# public key: age1pq1")
        )
        config_plaintext = b'{"radarrUrl":"http://radarr.invalid","radarrApiKey":"fixture-only"}'
        subprocess.run(
            ["age", "--recipient", recipient, "--output", str(shares[0] / "config.aged"), "-"],
            input=config_plaintext,
            check=True,
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
        )
        encrypted_config = (shares[0] / "config.aged").read_bytes()
        if not encrypted_config.startswith(b"age-encryption.org/v1\n") or config_plaintext in encrypted_config:
            raise RuntimeError("Rawrarr config fixture was not age-encrypted before sync")
        services = [f"web-{suffix}", f"storage-{suffix}"]
        for name, service, home, share, port in zip(names, services, homes, shares, ports):
            docker("run", "-d", "--name", name,
                   "--label", f"com.docker.compose.project={PROJECT}",
                   "--label", f"com.docker.compose.service={service}",
                   "--network", network,
                   "-p", f"127.0.0.1:{port}:8384",
                   "-v", f"{home}:/var/syncthing",
                   "-v", f"{share}:/shared",
                   IMAGE, "serve", "--no-browser", "--no-restart", "--no-upgrade",
                   "--gui-address", "0.0.0.0:8384")

        for name, service in zip(names, services):
            labels = json.loads(docker("inspect", "--format", "{{json .Config.Labels}}", name))
            if labels.get("com.docker.compose.project") != PROJECT or labels.get("com.docker.compose.service") != service:
                raise RuntimeError(f"{name} is not labeled as {PROJECT}/{service}")
        network_labels = json.loads(docker("network", "inspect", "--format", "{{json .Labels}}", network))
        if network_labels.get("com.docker.compose.project") != PROJECT:
            raise RuntimeError(f"{network} is not labeled as part of {PROJECT}")

        nodes: list[tuple[str, str]] = []
        deadline = time.monotonic() + 45
        for port, home in zip(ports, homes):
            config = home / "config" / "config.xml"
            while time.monotonic() < deadline and not config.is_file():
                time.sleep(0.25)
            if not config.is_file():
                raise RuntimeError("Syncthing container did not create config.xml")
            tree = ET.parse(config)
            gui = tree.getroot().find("gui")
            if gui is None or not gui.findtext("apikey"):
                raise RuntimeError("Syncthing container did not create its REST API key")
            api_key = gui.findtext("apikey", "").strip()
            nodes.append((f"http://127.0.0.1:{port}", api_key))

        for base, key in nodes:
            while time.monotonic() < deadline:
                try:
                    api(base, key, "/rest/system/ping")
                    break
                except (OSError, urllib.error.URLError):
                    time.sleep(0.25)
            else:
                raise RuntimeError("Syncthing container REST API did not become reachable")

        web_id = api(nodes[0][0], nodes[0][1], "/rest/system/status").get("myID")
        storage_id = api(nodes[1][0], nodes[1][1], "/rest/system/status").get("myID")
        if not web_id or not storage_id or web_id == storage_id:
            raise RuntimeError("container peers did not report distinct device identities")
        add_device(nodes[0][0], nodes[0][1], storage_id, "rawrarr-storage", f"tcp://{names[1]}:22000")
        add_device(nodes[1][0], nodes[1][1], web_id, "rawrarr-web", f"tcp://{names[0]}:22000")
        folder_id = "rawrarr-orbstack-test"
        add_folder(nodes[0][0], nodes[0][1], folder_id, "/shared", storage_id, "sendonly")
        add_folder(nodes[1][0], nodes[1][1], folder_id, "/shared", web_id, "receiveonly")

        deadline = time.monotonic() + 90
        while time.monotonic() < deadline:
            query = urllib.parse.urlencode({"folder": folder_id})
            try:
                statuses = [api(base, key, f"/rest/db/status?{query}") for base, key in nodes]
                if all(item.get("state") == "idle" and item.get("needFiles", 1) == 0
                       and item.get("globalFiles", 0) >= 2 and item.get("localFiles", 0) >= 2
                       for item in statuses):
                    break
            except (OSError, KeyError, TypeError):
                pass
            time.sleep(0.5)
        else:
            logs = "\n".join(docker("logs", name, check=False)[-1500:] for name in names)
            raise RuntimeError(f"container peers did not finish syncing; logs:\n{logs}")

        copied = shares[1] / "metadata" / "movies" / "sample-metadata.json"
        if not copied.is_file() or hashlib.sha256(copied.read_bytes()).digest() != hashlib.sha256(payload).digest():
            raise RuntimeError("Syncthing reported idle but the container did not receive matching file bytes")
        copied_config = shares[1] / "config.aged"
        if not copied_config.is_file() or copied_config.read_bytes() != encrypted_config:
            raise RuntimeError("Syncthing did not replicate the encrypted config.aged artifact exactly")
        if config_plaintext in copied_config.read_bytes():
            raise RuntimeError("Syncthing storage peer received plaintext config contents")
        print(f"PASS OrbStack Docker context {context}; project {PROJECT}; {IMAGE}")
        print("PASS separate web/storage containers connected and configured send-only/receive-only sharing")
        print("PASS nested metadata/movies folder and sample-metadata.json synced with matching SHA-256")
        print("PASS age-encrypted config.aged synced byte-for-byte; plaintext absent from shared folder")
        passed = True
    finally:
        if not (args.keep and passed):
            for name in names:
                docker("rm", "-f", name, check=False)
            docker("network", "rm", network, check=False)
            shutil.rmtree(root, ignore_errors=True)
    if args.keep:
        print(f"KEPT containers: {', '.join(names)}")
        print(f"KEPT Docker network: {network}")
        print(f"KEPT test data: {root}")
        print(f"Syncthing GUIs: http://127.0.0.1:{ports[0]} and http://127.0.0.1:{ports[1]}")
    else:
        print("PASS temporary containers, network, configuration, and shared folders removed")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (OSError, RuntimeError, subprocess.CalledProcessError, urllib.error.URLError) as error:
        print(f"FAIL OrbStack Syncthing peer integration: {type(error).__name__}: {error}", file=sys.stderr)
        raise SystemExit(1)
