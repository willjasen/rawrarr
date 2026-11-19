"""Validate the persistent, unpublished Rayfish coordinator access path."""

import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest


ROOT = Path(__file__).resolve().parents[2]
COMPOSE = shutil.which("docker")


@unittest.skipUnless(COMPOSE, "Docker Compose is required for Compose model validation")
class RayfishCoordinatorPathTests(unittest.TestCase):
    def compose_model(self, project, files, env=None, profiles=()):
        command = [COMPOSE, "compose", "-p", project]
        for profile in profiles:
            command.extend(("--profile", profile))
        for path in files:
            command.extend(("-f", str(path)))
        command.extend(("config", "--format", "json"))
        environment = os.environ.copy()
        environment.update(env or {})
        result = subprocess.run(command, cwd=ROOT, env=environment, text=True,
                                capture_output=True, check=True)
        return json.loads(result.stdout)

    def test_coordinator_overlay_is_internal_and_keeps_existing_namespace(self):
        model = self.compose_model(
            "rawrarr-coordinator-path-test",
            [ROOT / "deploy/do3/docker-compose.internal-coordinator.yml",
             ROOT / "deploy/do3/docker-compose.internal-rayfish-coordinator.yml"],
            {"RAWARR_INTERNAL_IMAGE": "rawrarr:test"},
        )
        self.assertEqual(model["services"]["roarlay"]["network_mode"], "service:tailscale")
        self.assertEqual(model["services"]["tailscale"]["networks"]["rayfish-coordinator"]["ipv4_address"],
                         "172.31.99.3")
        network = model["networks"]["rayfish-coordinator"]
        self.assertTrue(network["external"])
        self.assertEqual(network["name"], "rawrarr-internal-rayfish-coordinator")

    def test_private_proxy_has_no_published_ports_and_uses_only_required_networks(self):
        model = self.compose_model(
            "rawrarr-rayfish-coordinator-path-test",
            [ROOT / "deploy/rayfish/docker-compose.coordinator-access.yml"],
        )
        proxy = model["services"]["coordinator-proxy"]
        self.assertNotIn("ports", proxy)
        self.assertTrue(proxy["read_only"])
        self.assertEqual(proxy["cap_drop"], ["ALL"])
        self.assertEqual(proxy["networks"]["coordinator-access"]["ipv4_address"], "172.31.99.2")
        self.assertEqual(set(proxy["networks"]), {
            "coordinator-access", "relay-backend", "discovery-backend",
        })
        network = model["networks"]["coordinator-access"]
        self.assertTrue(network["internal"])
        self.assertEqual(network["ipam"]["config"][0]["subnet"], "172.31.99.0/29")

    def test_canonical_endpoint_override_is_sidecar_only(self):
        with tempfile.TemporaryDirectory() as temporary:
            config = Path(temporary) / "rayfish.json"
            config.write_text(json.dumps({
                "version": 1,
                "name": "rayfish-roarlay",
                "namespace": "rawrarr-int-tailscale",
                "role": "coordinator",
                "network": "rawrarr-pilot",
                "infrastructure": {
                    "discovery": "https://do3.risk-mermaid.ts.net:18444/pkarr",
                    "relay": "https://do3.risk-mermaid.ts.net:18443",
                },
            }), encoding="utf-8")
            generated = subprocess.run(
                [sys.executable, str(ROOT / "scripts/rayfish_transport.py"),
                 "compose", "--config", str(config)],
                cwd=ROOT, text=True, capture_output=True, check=True,
            )
            generated_path = Path(temporary) / "rayfish-compose.json"
            generated_path.write_text(generated.stdout, encoding="utf-8")
            model = self.compose_model(
                "rawrarr-rayfish-sidecar-path-test",
                [generated_path, ROOT / "deploy/rayfish/compose.roarlay-coordinator-hosts.yml"],
                profiles=("rayfish",),
            )
        sidecar = model["services"]["rayfish-roarlay"]
        self.assertEqual(sidecar["network_mode"], "container:rawrarr-int-tailscale")
        self.assertEqual(sidecar["extra_hosts"], ["do3.risk-mermaid.ts.net=172.31.99.2"])
        self.assertNotIn("extra_hosts", model["services"].get("roarlay", {}))

    def test_proxy_denies_other_sources_and_preserves_canonical_tls_hosts(self):
        caddyfile = (ROOT / "deploy/rayfish/coordinator-access.Caddyfile").read_text(encoding="utf-8")
        self.assertEqual(caddyfile.count("https://do3.risk-mermaid.ts.net:"), 2)
        self.assertEqual(caddyfile.count("@coordinator remote_ip 172.31.99.3"), 2)
        self.assertEqual(caddyfile.count("respond 403"), 2)
        self.assertIn("reverse_proxy relay:3340", caddyfile)
        self.assertIn("reverse_proxy discovery:18090", caddyfile)


if __name__ == "__main__":
    unittest.main()
