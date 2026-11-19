#!/usr/bin/env python3
"""Generate and run an opt-in, application-only rootless Rayfish sidecar."""

from __future__ import annotations

import argparse
import fcntl
import ipaddress
import json
import os
from pathlib import Path
import re
import stat
import subprocess
import sys
import tempfile
import tomllib
from urllib.parse import urlsplit


VERSION = 1
IMAGE = "rawrarr-rayfish:0.5.16"
APP_PORT = 8080
SYNCTHING_PORT = 22000
CONFIG_PEER_PORT = 8443
SERVICE_PORTS = {"app": APP_PORT, "syncthing": SYNCTHING_PORT, "config-peer": CONFIG_PEER_PORT}
INBOUND_SERVICE_KEYS = {"allow_in": APP_PORT, "allow_syncthing_in": SYNCTHING_PORT,
                        "allow_config_peer_in": CONFIG_PEER_PORT}
NAME = re.compile(r"[a-z][a-z0-9-]{0,62}\Z")
IDENTITY = re.compile(r"[0-9a-f]{64}\Z")
INFRASTRUCTURE = {"discovery": "discovery_dns", "relay": "relay"}
DNS_LABEL = re.compile(r"[a-z0-9](?:[a-z0-9-]{0,61}[a-z0-9])?\Z")
SAFE_SETTINGS = {
    "mdns_enabled": False,
    "dns_mode": "off",
    "ssh_enabled": False,
    "v4_bridge": False,
    "auto_update": False,
    "inbound": "none",
}

class ConfigurationError(ValueError):
    pass


def require(condition, message):
    if not condition:
        raise ConfigurationError(message)


def unique_object(pairs):
    result = {}
    for key, value in pairs:
        require(key not in result, "Duplicate JSON key")
        result[key] = value
    return result


def read_json(path):
    require(path.is_file() and not path.is_symlink(), "Configuration must be a regular file")
    require(path.stat().st_size <= 65536, "Configuration exceeds 64 KiB")
    return json.loads(path.read_text(), object_pairs_hook=unique_object)


def fields(value, required, optional=()):
    require(isinstance(value, dict), "Expected a JSON object")
    require(set(required) <= value.keys(), "Missing required configuration fields")
    require(value.keys() <= set(required) | set(optional), "Unknown configuration fields")


def identity(value):
    require(isinstance(value, str) and IDENTITY.fullmatch(value), "Expected a full lowercase Rayfish endpoint ID")
    return value


def infrastructure_settings(config):
    if "infrastructure" not in config:
        return {}
    value = config["infrastructure"]
    fields(value, INFRASTRUCTURE)
    tables = {}
    for key, table in INFRASTRUCTURE.items():
        url = value[key]
        require(isinstance(url, str) and len(url) <= 512 and url.isascii()
                and re.fullmatch(r"[a-z0-9:/.-]+", url), f"Invalid {key} endpoint URL")
        endpoint = urlsplit(url)
        hostname = endpoint.hostname or ""
        port = endpoint.port
        require(len(hostname) <= 253 and all(DNS_LABEL.fullmatch(label) for label in hostname.split("."))
                and endpoint.netloc == hostname + (f":{port}" if port else ""),
                f"Invalid {key} endpoint authority")
        require(endpoint.scheme == "https" or (
            endpoint.scheme == "http" and (hostname == "127.0.0.1" or
                                           (NAME.fullmatch(hostname) and DNS_LABEL.fullmatch(hostname)))
        ), f"{key} requires HTTPS outside a local container namespace")
        require(endpoint.path == "/pkarr" if key == "discovery" else endpoint.path in ("", "/"),
                f"Invalid {key} endpoint path")
        tables[table] = {"servers": [url], "replace": True}
    return tables


def validate(config):
    fields(config, ("version", "name", "namespace", "role", "network"),
           ("expose_app", *INBOUND_SERVICE_KEYS, "forwards", "lan_discovery", "infrastructure"))
    require(type(config["version"]) is int and config["version"] == VERSION, "Unsupported transport configuration version")
    for key in ("name", "namespace", "network"):
        require(isinstance(config[key], str) and NAME.fullmatch(config[key]), f"Invalid {key}")
    require(config["role"] in ("coordinator", "metadata", "storage"), "Invalid Rawrarr role")
    require(type(config.get("lan_discovery", False)) is bool, "lan_discovery must be a boolean")
    infrastructure_settings(config)
    expose = config.get("expose_app", False)
    require(type(expose) is bool, "expose_app must be a boolean")
    require(not expose or config["role"] != "coordinator", "The pilot does not expose coordinator ingress")
    allow = config.get("allow_in", [])
    forwards = config.get("forwards", [])
    require(isinstance(allow, list) and len(allow) <= 16, "Invalid inbound peer list")
    require(isinstance(forwards, list) and len(forwards) <= 16, "Invalid forward list")
    for peer in allow:
        identity(peer)
    require(len(set(allow)) == len(allow), "Duplicate inbound peer")
    require(expose == bool(allow), "Application exposure requires explicit inbound peers, and vice versa")
    for key, role_set in (("allow_syncthing_in", ("metadata", "storage")), ("allow_config_peer_in", ("metadata",))):
        grants = config.get(key, [])
        require(isinstance(grants, list) and len(grants) <= 16, f"Invalid {key} peer list")
        for peer in grants:
            identity(peer)
        require(len(set(grants)) == len(grants), f"Duplicate {key} inbound peer")
        require(not grants or config["role"] in role_set, f"{key} is unavailable for this role")
    ports, nodes = set(), set()
    for forward in forwards:
        fields(forward, ("node_id", "peer_id", "peer_ipv6", "local_port"), ("service",))
        service = forward.get("service", "app")
        require(isinstance(service, str) and service in SERVICE_PORTS, "Unknown forward service")
        require(service != "syncthing" or config["role"] != "coordinator", "Syncthing forwards are subscriber-only")
        require(service != "config-peer" or config["role"] == "metadata", "Config-peer forwards are metadata-only")
        identity(forward["peer_id"])
        require(isinstance(forward["node_id"], str) and re.fullmatch(r"[A-Za-z0-9_-]{1,128}", forward["node_id"]),
                "Invalid Rawrarr node ID")
        require(isinstance(forward["peer_ipv6"], str), "Expected a mesh IPv6 address")
        address = ipaddress.IPv6Address(forward["peer_ipv6"])
        require(address in ipaddress.IPv6Network("200::/7"), "Forward destination is not a Rayfish mesh address")
        port = forward["local_port"]
        require(type(port) is int and 17000 <= port <= 17999, "Pilot loopback ports must be between 17000 and 17999")
        destination = (forward["node_id"], service)
        require(port not in ports and destination not in nodes, "Duplicate forward port or node service")
        ports.add(port)
        nodes.add(destination)
    return config


def compose(config, config_path):
    validate(config)
    volume = config["name"] + "-state"
    return {
        "services": {
            config["name"]: {
                "image": IMAGE,
                "pull_policy": "never",
                "profiles": ["rayfish"],
                "network_mode": "container:" + config["namespace"],
                "user": "10001:10001",
                "read_only": True,
                "cap_drop": ["ALL"],
                "security_opt": ["no-new-privileges:true"],
                "tmpfs": ["/tmp:rw,noexec,nosuid,size=16m"],
                "volumes": [
                    {"type": "volume", "source": volume, "target": "/state"},
                    {"type": "bind", "source": str(config_path.resolve()),
                     "target": "/run/rawrarr/rayfish.json", "read_only": True,
                     "bind": {"create_host_path": False}},
                ],
                "healthcheck": {
                    "test": ["CMD", "python", "/opt/rawrarr/rayfish_transport.py", "health"],
                    "interval": "10s", "timeout": "5s", "retries": 3,
                    "start_period": "30s",
                },
                "restart": "unless-stopped",
            },
        },
        "volumes": {volume: {}},
    }


def relay_targets(config, existing):
    validate(config)
    require(config["role"] == "coordinator", "Relay-target selection is coordinator-only")
    require(isinstance(existing, dict) and all(isinstance(key, str) and isinstance(value, str)
                                             for key, value in existing.items()), "Expected an existing relay-target map")
    result = dict(existing)
    for forward in config.get("forwards", []):
        require(forward["node_id"] in result, "Pilot node must already have a configured relay route")
        result[forward["node_id"]] = f"http://127.0.0.1:{forward['local_port']}/"
    return result


def firewall(config):
    validate(config)
    text = 'default_inbound = "deny"\ndefault_outbound = "deny"\nreject = true\ndisabled = false\n'
    inbound = [(peer, port) for key, port in INBOUND_SERVICE_KEYS.items() for peer in config.get(key, [])]
    rules = [("in", peer, port) for peer, port in inbound]
    # Rayfish tracks only locally initiated flows. Server replies need outbound
    # TCP permission to the pinned caller's ephemeral port; rootless exposes
    # only configured services and creates only the explicitly configured local forwards.
    rules += [("out", peer, None) for peer, _ in inbound]
    rules += [("out", forward["peer_id"], SERVICE_PORTS[forward.get("service", "app")])
              for forward in config.get("forwards", [])]
    if not rules:
        text += "rules = []\n"
    for direction, peer, port in sorted(set(rules), key=lambda rule: (rule[0], rule[1], rule[2] or 0)):
        text += (
            f'\n[[rules]]\ndirection = "{direction}"\naction = "allow"\nprotocol = "tcp"\n'
            f'network = "{config["network"]}"\npeer = {{ identity = "{peer}" }}\n'
        )
        if port is not None:
            text += f"port = {{ start = {port}, end = {port} }}\n"
    return text


def private_write(path, text):
    if path.exists() or path.is_symlink():
        require(not path.is_symlink() and path.is_file(), "Managed state path is not a regular file")
    fd, temporary = tempfile.mkstemp(prefix=".rayfish-", dir=path.parent)
    try:
        with os.fdopen(fd, "w") as output:
            output.write(text)
            output.flush()
            os.fsync(output.fileno())
        os.replace(temporary, path)
    finally:
        if os.path.exists(temporary):
            os.unlink(temporary)


def prepare_state(config, state):
    validate(config)
    require(not state.is_symlink(), "Transport state must not be a symlink")
    state.mkdir(mode=0o700, parents=True, exist_ok=True)
    info = state.stat()
    require(info.st_uid == os.getuid() and stat.S_IMODE(info.st_mode) == 0o700,
            "Transport state must be owned by the runtime user with mode 0700")
    marker = state / "rawrarr-transport-v1.json"
    expected = {"version": VERSION, "name": config["name"], "role": config["role"],
                "namespace": config["namespace"], "network": config["network"]}
    if marker.exists():
        require(read_json(marker) == expected, "Transport state belongs to another node, role, namespace, or network")
    else:
        require(not any(path.name not in (".rawrarr.lock",) for path in state.iterdir()),
                "Refusing to import existing transport state")
        private_write(marker, json.dumps(expected))
    settings = state / "settings.toml"
    expected_settings = dict(SAFE_SETTINGS, mdns_enabled=config.get("lan_discovery", False))
    expected_tables = infrastructure_settings(config)
    if not settings.exists():
        entries = [f"{key} = {json.dumps(value)}\n" for key, value in expected_settings.items()]
        for table, options in expected_tables.items():
            entries.append(f"\n[{table}]\n")
            entries.extend(f"{key} = {json.dumps(value)}\n" for key, value in options.items())
        private_write(settings, "".join(entries))
    require(not settings.is_symlink() and settings.is_file(), "Settings must be a regular file")
    saved = tomllib.loads(settings.read_text())
    require(all(type(saved.get(key)) is type(value) and saved.get(key) == value
                for key, value in expected_settings.items()), "Unsafe saved Rayfish settings; restore pilot settings before restarting")
    for table in INFRASTRUCTURE.values():
        actual = saved.get(table)
        expected_table = expected_tables.get(table)
        require(actual == expected_table and
                (actual is None or actual.get("replace") is True),
                "Saved discovery or relay settings do not match configuration; use fresh transport state")
    require(not saved.get("file_auto_accept_peers") and not saved.get("controllers")
            and not saved.get("managed_machines") and not saved.get("device_generations"),
            "File auto-accept, device pairing, and remote management are outside the pilot")
    key = state / "secret_key"
    if key.exists() or key.is_symlink():
        require(key.is_file() and not key.is_symlink(), "Endpoint key must be a regular file")
    networks = state / "networks"
    require(not networks.is_symlink(), "Network state must not be a symlink")
    if networks.exists():
        for path in networks.iterdir():
            require(path.is_file() and not path.is_symlink() and path.suffix == ".toml", "Unexpected network state")
            network = tomllib.loads(path.read_text())
            require(network.get("name") == config["network"], "Only the configured pilot network may be joined")
            require(network.get("group_mode") == "restricted", "Pilot membership must use a closed network")
            require(network.get("auto_accept_files") is False, "Disable network file auto-accept before restarting")
            require(not network.get("auto_accept_firewall") and not network.get("allow_trusted") and not network.get("exit_allow")
                    and not network.get("exit_node_use") and not network.get("ssh_allow")
                    and not network.get("managed_ssh_allow"), "Unsafe pilot network settings")
    private_write(state / "firewall.toml", firewall(config))


def run(config, state):
    require(os.getuid() != 0, "Run the pilot as an unprivileged user")
    require(not state.is_symlink(), "Transport state must not be a symlink")
    state.mkdir(mode=0o700, parents=True, exist_ok=True)
    lock_path = state / ".rawrarr.lock"
    fd = os.open(lock_path, os.O_CREAT | os.O_RDWR | os.O_NOFOLLOW, 0o600)
    try:
        fcntl.flock(fd, fcntl.LOCK_EX | fcntl.LOCK_NB)
        prepare_state(config, state)
        verify_forward_addresses(config)
        arguments = ["ray", "rootless", "--socks5", "127.0.0.1:0"]
        for key, port in INBOUND_SERVICE_KEYS.items():
            if config.get(key):
                arguments += ["--expose", f"{port}=127.0.0.1:{port}"]
        for forward in config.get("forwards", []):
            address = ipaddress.IPv6Address(forward["peer_ipv6"])
            port = SERVICE_PORTS[forward.get("service", "app")]
            arguments += ["--forward", f"127.0.0.1:{forward['local_port']}=[{address}]:{port}"]
        os.environ["RAYFISH_ROOTLESS_DIR"] = str(state)
        os.environ.pop("RAYFISH_CONFIG_DIR", None)
        # Retain the node-state lock across exec for the lifetime of the daemon.
        os.set_inheritable(fd, True)
        os.execvp("ray", arguments)
    finally:
        os.close(fd)


def verify_forward_addresses(config):
    for forward in config.get("forwards", []):
        # Pinned upstream derive_ipv6 hashes the lowercase endpoint hex text.
        result = subprocess.run(["b3sum", "--no-names"], input=forward["peer_id"],
                                capture_output=True, text=True, timeout=5, check=True)
        digest = result.stdout.strip()
        require(IDENTITY.fullmatch(digest), "Invalid BLAKE3 address derivation result")
        expected = ipaddress.IPv6Address(bytes.fromhex("02" + digest[:30]))
        require(ipaddress.IPv6Address(forward["peer_ipv6"]) == expected,
                "Forward mesh address does not match its pinned endpoint identity")


def health():
    result = subprocess.run(["ray", "--rootless", "status", "--json"], capture_output=True,
                            text=True, timeout=4, check=True)
    status = json.loads(result.stdout)
    require(status.get("rootless") is True and status.get("active") is True, "Rootless transport is not active")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    subparsers = parser.add_subparsers(dest="action", required=True)
    for action in ("compose", "relay-targets", "run"):
        command = subparsers.add_parser(action)
        command.add_argument("--config", required=True, type=Path)
        if action == "relay-targets":
            command.add_argument("--existing", required=True, type=Path)
        if action == "run":
            command.add_argument("--state", type=Path, default=Path(os.environ.get("RAYFISH_ROOTLESS_DIR", "/state")))
    subparsers.add_parser("health")
    args = parser.parse_args()
    try:
        if args.action == "health":
            health()
        else:
            config = validate(read_json(args.config))
            if args.action == "run":
                run(config, args.state)
            elif args.action == "compose":
                print(json.dumps(compose(config, args.config), indent=2))
            else:
                print(json.dumps(relay_targets(config, read_json(args.existing)), indent=2))
        return 0
    except ConfigurationError as error:
        print(f"Rayfish transport failed: {error}", file=sys.stderr)
        return 1
    except (ValueError, OSError, subprocess.SubprocessError) as error:
        # Configuration and subprocess output can contain identity or invitation material.
        print(f"Rayfish transport failed ({type(error).__name__}); check configuration, private state, and daemon status.", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
