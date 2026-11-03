"""Guard the Compose inventory and isolated environment entry point."""

import importlib.util
from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]


class ComposeLayoutTests(unittest.TestCase):
    def test_retired_host_specific_storage_templates_are_not_inputs(self):
        for name in (
            "docker-compose.macbook-storage.yml",
            "docker-compose.tailscale-mini.yml",
            "docker-compose.mini-relays.yml",
            "storage-proxy.Caddyfile",
        ):
            with self.subTest(name=name):
                self.assertFalse((ROOT / name).exists())
        self.assertTrue((ROOT / "docker-compose.storage.yml").is_file())
        self.assertTrue((ROOT / "docker-compose.metadata.yml").is_file())

    def test_retired_staged_headscale_clients_are_not_installation_inputs(self):
        for name in ("docker-compose.headscale-macbook.yml", "docker-compose.headscale-mini.yml"):
            with self.subTest(name=name):
                self.assertFalse((ROOT / name).exists())
        self.assertTrue((ROOT / "deploy" / "testing" / "headscale" / "config.yaml").is_file())
        self.assertTrue((ROOT / "docker-compose.storage.yml").is_file())
        self.assertFalse((ROOT / "docker-compose.tailscale-macbook.yml").exists())

    def test_retired_mac_coordinator_definitions_are_not_installation_inputs(self):
        for name in ("docker-compose.internal-mini.yml", "docker-compose.tailscale-serve.yml", "docker-compose.tailscale-macbook.yml"):
            with self.subTest(name=name):
                self.assertFalse((ROOT / name).exists())
        for name in (
            "docker-compose.yml",
            "docker-compose.coordinator.yml",
            "docker-compose.metadata.yml",
            "docker-compose.storage.yml",
            "deploy/do3/docker-compose.internal-coordinator.yml",
        ):
            with self.subTest(name=name):
                self.assertTrue((ROOT / name).is_file())

    def test_every_root_compose_file_has_an_inventory_entry(self):
        inventory = (ROOT / "docs" / "compose-layout.md").read_text(encoding="utf-8")
        for path in ROOT.glob("docker-compose*.yml"):
            with self.subTest(path=path.name):
                self.assertIn(f"| `{path.name}` |", inventory)

    def test_connectivity_runner_uses_absolute_relocated_compose_path(self):
        path = ROOT / "scripts" / "tests" / "test_orbstack_arr_connectivity.py"
        spec = importlib.util.spec_from_file_location("orbstack_arr_connectivity", path)
        module = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(module)
        self.assertEqual(module.PROJECT, "rawrarr-test-environment")
        self.assertEqual(Path(module.COMPOSE), ROOT / "deploy" / "testing" / "compose.yml")
        self.assertTrue(Path(module.COMPOSE).is_file())

    def test_static_test_fixtures_are_colocated(self):
        stack = ROOT / "deploy" / "testing"
        compose = (stack / "compose.yml").read_text(encoding="utf-8")
        self.assertIn("name: rawrarr-test-environment\n", compose)
        self.assertIn("context: ../../rust-arr\n", compose)
        for path in ("arr.Caddyfile", "headscale"):
            with self.subTest(path=path):
                self.assertIn(f"./{path}:", compose)
                self.assertTrue((stack / path).exists())
        self.assertTrue((stack / "headscale" / "config.yaml").is_file())
        self.assertFalse((ROOT / "docker-compose.orbstack-test.yml").exists())


if __name__ == "__main__":
    unittest.main()
