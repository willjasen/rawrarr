#!/usr/bin/env python3
"""Policy tests and opt-in disposable, separate-namespace Rayfish integration."""

from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
import json
import os
from pathlib import Path
import signal
import subprocess
import sys
import tempfile
import threading
import tomllib
import unittest
from unittest.mock import patch

INNER = os.environ.get("RAWARR_RAYFISH_TEST_INNER") == "1"
ROOT = Path("/state") if INNER else Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT if INNER else ROOT / "scripts"))
import rayfish_transport as transport


def config(role="coordinator"):
    return {"version": 1, "name": "rayfish-pilot", "namespace": "existing-app",
            "role": role, "network": "rawrarr-pilot"}


def forward(peer="a" * 64, address="200::1", node="fossil-test", port=17297):
    return {"node_id": node, "peer_id": peer, "peer_ipv6": address, "local_port": port}


class PolicyTests(unittest.TestCase):
    def test_default_has_no_exposure_or_forwards(self):
        value = tomllib.loads(transport.firewall(config()))
        self.assertEqual(value["rules"], [])
        self.assertEqual(value["default_inbound"], "deny")
        self.assertEqual(value["default_outbound"], "deny")
        self.assertFalse(value["disabled"])

    def test_inbound_is_peer_network_and_app_pinned_with_tcp_replies(self):
        value = dict(config("metadata"), expose_app=True, allow_in=["a" * 64])
        incoming, reply = tomllib.loads(transport.firewall(value))["rules"]
        self.assertEqual(incoming["direction"], "in")
        self.assertEqual(incoming["peer"], {"identity": "a" * 64})
        self.assertEqual(incoming["port"], {"start": 8080, "end": 8080})
        self.assertEqual(incoming["network"], "rawrarr-pilot")
        self.assertEqual(reply["direction"], "out")
        self.assertEqual(reply["peer"], incoming["peer"])
        self.assertEqual(reply["protocol"], "tcp")
        self.assertNotIn("port", reply)
        outgoing, = tomllib.loads(transport.firewall(dict(config(), forwards=[forward()])))["rules"]
        self.assertEqual(outgoing["port"], {"start": 8080, "end": 8080})

    def test_invalid_configuration_is_rejected(self):
        invalid = [
            {"version": True}, {"version": 2}, {"role": "service"},
            {"namespace": "container:postgres"}, {"network": "../shared"},
            {"expose_app": "false"}, {"expose_app": True}, {"allow_in": ["a" * 64]},
            {"token": "not-accepted"}, {"forwards": [forward(peer="short")]},
            {"forwards": [forward(address="127.0.0.1")]},
            {"forwards": [forward(address="fd00::1")]},
            {"forwards": [forward(port=5432)]}, {"forwards": [forward(port=True)]},
            {"forwards": [forward(), forward()]}, {"lan_discovery": "true"},
        ]
        for change in invalid:
            with self.subTest(fields=list(change)), self.assertRaises(ValueError):
                transport.validate(dict(config(), **change))
        for role in ("metadata", "storage"):
            transport.validate(dict(config(role), forwards=[forward()]))
            with self.assertRaises(ValueError):
                transport.relay_targets(dict(config(role), forwards=[forward()]), {"fossil-test": "http://existing/"})

    def test_compose_is_opt_in_and_does_not_mutate_existing_services(self):
        result = transport.compose(config(), Path("/run/pilot.json"))
        self.assertEqual(set(result["services"]), {"rayfish-pilot"})
        service = result["services"]["rayfish-pilot"]
        self.assertEqual(service["profiles"], ["rayfish"])
        self.assertEqual(service["network_mode"], "container:existing-app")
        self.assertEqual(service["user"], "10001:10001")
        self.assertTrue(service["read_only"])
        self.assertEqual(service["cap_drop"], ["ALL"])
        for key in ("ports", "devices", "privileged", "networks", "depends_on", "secrets"):
            self.assertNotIn(key, service)
        self.assertEqual([mount["target"] for mount in service["volumes"]],
                         ["/state", "/run/rawrarr/rayfish.json"])
        self.assertEqual(service["pull_policy"], "never")
        self.assertEqual(transport.compose(dict(config(), namespace="host"), Path("/config"))["services"]
                         ["rayfish-pilot"]["network_mode"], "container:host")

    def test_compose_normalizes_without_a_daemon(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "pilot.json"
            path.write_text(json.dumps(transport.compose(config(), Path(directory) / "settings.json")))
            result = subprocess.run(["docker", "compose", "--profile", "rayfish", "-p", "rawrarr-test-rayfish",
                                     "-f", str(path), "config", "--format", "json"],
                                    capture_output=True, text=True, check=True)
            service = json.loads(result.stdout)["services"]["rayfish-pilot"]
            self.assertEqual(service["network_mode"], "container:existing-app")
            self.assertNotIn("ports", service)

    def test_route_selection_preserves_the_tailscale_map(self):
        original = {"fossil-test": "https://first.risk-mermaid.ts.net/", "other": "https://second.risk-mermaid.ts.net/"}
        result = transport.relay_targets(dict(config(), forwards=[forward()]), original)
        self.assertEqual(result["fossil-test"], "http://127.0.0.1:17297/")
        self.assertEqual(result["other"], original["other"])
        self.assertEqual(original["fossil-test"], "https://first.risk-mermaid.ts.net/")
        with self.assertRaises(ValueError):
            transport.relay_targets(dict(config(), forwards=[forward()]), {})

    def test_state_preserves_identity_but_replaces_policy(self):
        with tempfile.TemporaryDirectory() as directory:
            state = Path(directory) / "state"
            transport.prepare_state(config(), state)
            key = state / "secret_key"
            key.write_bytes(b"synthetic-key")
            (state / "firewall.toml").write_text('disabled = true\nrules = []\n')
            transport.prepare_state(config(), state)
            self.assertEqual(key.read_bytes(), b"synthetic-key")
            self.assertFalse(tomllib.loads((state / "firewall.toml").read_text())["disabled"])
            self.assertEqual((state / "firewall.toml").stat().st_mode & 0o777, 0o600)
            with self.assertRaises(ValueError):
                transport.prepare_state(dict(config(), name="another-node"), state)

    def test_imported_state_unsafe_settings_and_symlinks_fail_closed(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            old = root / "old"
            old.mkdir(mode=0o700)
            (old / "secret_key").write_text("do-not-import")
            with self.assertRaises(ValueError):
                transport.prepare_state(config(), old)
            state = root / "new"
            transport.prepare_state(config(), state)
            settings = state / "settings.toml"
            settings.write_text(settings.read_text().replace("auto_update = false", "auto_update = true"))
            with self.assertRaises(ValueError):
                transport.prepare_state(config(), state)
            settings.unlink()
            settings.symlink_to(old / "secret_key")
            with self.assertRaises(ValueError):
                transport.prepare_state(config(), state)
            linked = root / "linked"
            linked.symlink_to(old)
            with self.assertRaises(ValueError):
                transport.prepare_state(config(), linked)

    def test_network_must_be_closed_without_file_auto_accept(self):
        with tempfile.TemporaryDirectory() as directory:
            state = Path(directory) / "state"
            transport.prepare_state(config(), state)
            networks = state / "networks"
            networks.mkdir()
            path = networks / "pilot.toml"
            for extra in ('group_mode = "open"\nauto_accept_files = false',
                          'group_mode = "restricted"\nauto_accept_files = true',
                          'group_mode = "restricted"\nauto_accept_files = false\nauto_accept_firewall = true'):
                path.write_text('name = "rawrarr-pilot"\n' + extra)
                with self.assertRaises(ValueError):
                    transport.prepare_state(config(), state)
            path.write_text('name = "rawrarr-pilot"\ngroup_mode = "restricted"\nauto_accept_files = false')
            transport.prepare_state(config(), state)

    def test_health_is_local_liveness_not_peer_readiness(self):
        result = subprocess.CompletedProcess([], 0, '{"rootless":true,"active":true,"networks":[]}')
        with patch.object(transport.subprocess, "run", return_value=result):
            transport.health()
        result.stdout = '{"rootless":true,"active":false}'
        with patch.object(transport.subprocess, "run", return_value=result), self.assertRaises(ValueError):
            transport.health()

    def test_forward_address_must_derive_from_its_endpoint(self):
        value = dict(config(), forwards=[forward(address="200::")])
        result = subprocess.CompletedProcess([], 0, "0" * 64 + "\n")
        with patch.object(transport.subprocess, "run", return_value=result) as command:
            transport.verify_forward_addresses(value)
            self.assertEqual(command.call_args.kwargs["input"], "a" * 64)
            value["forwards"][0]["peer_ipv6"] = "201::"
            with self.assertRaises(ValueError):
                transport.verify_forward_addresses(value)

    def test_duplicate_json_keys_and_secret_fields_are_rejected(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "config.json"
            path.write_text('{"version":1,"version":2}')
            with self.assertRaises(ValueError):
                transport.read_json(path)
        for key in ("invite", "secret_key", "age_identity", "coordinator_token"):
            with self.assertRaises(ValueError):
                transport.validate(dict(config(), **{key: "private-marker"}))


class Fixture(BaseHTTPRequestHandler):
    payload = b"rawrarr-synthetic-ciphertext\n" * 32768

    def do_GET(self):
        if self.path == "/health":
            body, code = b"synthetic application alive", 200
        elif self.headers.get("Authorization") != "Bearer rawrarr-test-session":
            body, code = b"unauthorized", 401
        else:
            body, code = self.payload, 200
        self.send_response(code)
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)

    def log_message(self, *args):
        pass


class Discovery(BaseHTTPRequestHandler):
    """Test-only signed-packet store; Rayfish verifies the signatures."""
    packets = {}

    def do_PUT(self):
        length = int(self.headers["Content-Length"])
        if not 0 < length <= 4096:
            self.send_error(400, "Invalid discovery packet")
            return
        self.packets[self.path] = self.rfile.read(length)
        self.send_response(200)
        self.send_header("Content-Length", "0")
        self.end_headers()

    def do_GET(self):
        packet = self.packets.get(self.path, b"")
        self.send_response(200 if packet else 404)
        self.send_header("Content-Length", str(len(packet)))
        self.end_headers()
        self.wfile.write(packet)

    def log_message(self, *args):
        pass


def worker():
    name = os.environ["RAWARR_RAYFISH_WORKER"]
    if name == "discovery":
        ThreadingHTTPServer(("0.0.0.0", 18090), Discovery).serve_forever()
        return
    servers = []
    if os.environ.get("RAWARR_RAYFISH_EXTERNAL_APP") != "1":
        for port in (8080, 7299):
            server = ThreadingHTTPServer(("127.0.0.1", port), Fixture)
            threading.Thread(target=server.serve_forever, daemon=True).start()
            servers.append(server)
    state = Path("/state") / name
    path = Path("/state") / (name + ".json")
    value = transport.validate(transport.read_json(path))
    transport.prepare_state(value, state)
    settings = state / "settings.toml"
    if "[discovery_dns]" not in settings.read_text():
        with settings.open("a") as output:
            output.write('\n[discovery_dns]\nservers = ["http://discovery:18090/pkarr"]\nreplace = true\n'
                         '\n[relay]\nservers = ["http://127.0.0.1:18091"]\nreplace = true\n')
    process = None
    signal.signal(signal.SIGTERM, lambda *_: process.send_signal(signal.SIGINT) if process else None)
    try:
        with (Path("/state") / (name + ".log")).open("a") as log:
            process = subprocess.Popen(["python", str(ROOT / "rayfish_transport.py"), "run",
                                        "--config", str(path), "--state", str(state)],
                                       env=dict(os.environ, RAYFISH_ROOTLESS_DIR=str(state)),
                                       stdout=log, stderr=log)
            if process.wait() not in (0, -signal.SIGINT):
                raise RuntimeError("Rayfish worker exited unexpectedly; inspect its private test log")
    finally:
        if process and process.poll() is None:
            process.send_signal(signal.SIGINT)
            process.wait(timeout=15)
        for server in servers:
            server.shutdown()
            server.server_close()


if __name__ == "__main__":
    if INNER:
        worker()
    elif sys.argv[1:] in (["--integration"], ["--environment"]):
        import rayfish_docker
        rayfish_docker.run(environment=sys.argv[1] == "--environment")
    else:
        unittest.main()
