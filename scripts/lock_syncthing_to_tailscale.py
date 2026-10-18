#!/usr/bin/env python3
"""Restrict a two-device Syncthing pair to static Tailscale TCP addresses."""

import argparse
import ipaddress
import json
import shutil
import subprocess
import sys
import xml.etree.ElementTree as ET
from urllib.request import Request, urlopen

from share_storage_media import api, validate_url


FLAGS = ("globalAnnounceEnabled", "localAnnounceEnabled", "relaysEnabled", "natEnabled")


def key_from_container(name):
    docker = shutil.which("docker") or "/usr/local/bin/docker"
    raw = subprocess.check_output([docker, "--context", "orbstack", "exec", name,
                                   "cat", "/var/syncthing/config/config.xml"], stderr=subprocess.DEVNULL)
    return (ET.fromstring(raw).findtext("gui/apikey") or "").strip()


def patch(base, key, path, value):
    request = Request(base + path, data=json.dumps(value).encode(), method="PATCH",
                      headers={"X-API-Key": key, "Content-Type": "application/json"})
    with urlopen(request, timeout=10) as response:
        response.read()


def inspect(base, key, local_id, peer_id):
    actual = api(base, key, "rest/system/status")["myID"]
    if actual != local_id or actual == peer_id:
        raise ValueError("local Syncthing device ID does not match the expected pair")
    devices = api(base, key, "rest/config/devices")
    configured = {item.get("deviceID") for item in devices}
    if configured != {local_id, peer_id}:
        raise ValueError("expected exactly the local device and approved peer")
    options = api(base, key, "rest/config/options")
    peer = next(item for item in devices if item.get("deviceID") == peer_id)
    return options, peer


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--docker-container", required=True)
    parser.add_argument("--api-url", default="http://127.0.0.1:8385")
    parser.add_argument("--local-id", required=True)
    parser.add_argument("--peer-id", required=True)
    parser.add_argument("--peer-tailnet-ip", required=True)
    parser.add_argument("--apply", action="store_true")
    args = parser.parse_args()
    try:
        ip = ipaddress.ip_address(args.peer_tailnet_ip)
        if ip.version != 4 or ip not in ipaddress.ip_network("100.64.0.0/10"):
            raise ValueError("peer address must be a Tailscale IPv4 address")
        base = validate_url(args.api_url)
        key = key_from_container(args.docker_container)
        if not key:
            raise ValueError("local Syncthing API key is unavailable")
        options, peer = inspect(base, key, args.local_id, args.peer_id)
        address = f"tcp://{ip}:22000"
        connection = api(base, key, "rest/system/connections").get("connections", {}).get(args.peer_id, {})
        restart = api(base, key, "rest/config/restart-required")
        print(json.dumps({"peerAddress": address, "currentAddresses": peer.get("addresses"),
                          "listenAddresses": options.get("listenAddresses"),
                          "connection": {name: options.get(name) for name in FLAGS},
                          "peerConnection": {name: connection.get(name) for name in ("connected", "type", "address")},
                          "restartRequired": restart.get("requiresRestart")}))
        if not args.apply:
            print("Dry run: set a static tailnet peer and disable discovery, relays, and NAT traversal.")
            return 0
        patch(base, key, "rest/config/options",
              {**{name: False for name in FLAGS}, "listenAddresses": ["tcp://0.0.0.0:22000"]})
        patch(base, key, "rest/config/devices/" + args.peer_id,
              {"addresses": [address], "autoAcceptFolders": False})
        print("Saved. Verify Compose ALL_PROXY_NO_FALLBACK=1, Tailscale Serve TCP, and a tcp connection.")
        return 0
    except (OSError, ValueError, KeyError, TypeError, ET.ParseError, subprocess.CalledProcessError) as error:
        print("Tailnet lock failed: " + (str(error) if isinstance(error, ValueError) else type(error).__name__), file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
