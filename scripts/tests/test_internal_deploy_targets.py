import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from classify_internal_deploy_targets import classify_paths


class InternalDeployTargetTests(unittest.TestCase):
    def test_coordinator_only_changes_select_coordinator(self):
        result = classify_paths(["rust-arr/src/coordinator.rs", "rust-arr/src/coordinator-passkey.mjs"])
        self.assertEqual(result["targets"], ["coordinator"])

    def test_ui_updater_changes_select_coordinator_only(self):
        result = classify_paths(["rust-arr/update_ui_bundle.py"])
        self.assertEqual(result["targets"], ["coordinator"])

    def test_coordinator_signing_asset_and_database_migrations_are_coordinator_only(self):
        result = classify_paths([
            "rust-arr/src/assets/device-signing-client.js",
            "rust-arr/web-export/device-signing-client.js",
            "rust-arr/migrations/0011_signed_device_rejections.sql",
        ])
        self.assertEqual(result["targets"], ["coordinator"])

    def test_node_only_changes_select_nodes(self):
        result = classify_paths(["rust-arr/src/library.rs", "docker-compose.metadata.yml"])
        self.assertEqual(result["targets"], ["nodes"])

    def test_shared_or_ambiguous_runtime_changes_select_both(self):
        result = classify_paths(["rust-arr/src/main.rs", "rust-arr/src/future_runtime_module.rs"])
        self.assertEqual(result["targets"], ["coordinator", "nodes"])

    def test_docs_tests_and_runbooks_do_not_trigger_container_deploys(self):
        result = classify_paths(["AGENTS.md", "runbooks/new.md", "scripts/tests/test_example.py"])
        self.assertEqual(result["targets"], [])

    def test_mixed_role_changes_select_both_once(self):
        result = classify_paths(["rust-arr/src/coordinator.rs", "rust-arr/src/library.rs"])
        self.assertEqual(result["targets"], ["coordinator", "nodes"])

    def test_test_stack_relocation_does_not_trigger_live_deploys(self):
        result = classify_paths([
            "docker-compose.orbstack-test.yml",
            "arr-orbstack-test.Caddyfile",
            "headscale-orbstack-test/config.yaml",
            "deploy/testing/compose.yml",
            "deploy/testing/arr.Caddyfile",
            "deploy/testing/headscale/config.yaml",
        ])
        self.assertEqual(result["targets"], [])

    def test_retired_mac_coordinator_files_do_not_trigger_live_deploys(self):
        for path in ("docker-compose.internal-mini.yml", "docker-compose.tailscale-serve.yml", "docker-compose.tailscale-macbook.yml"):
            with self.subTest(path=path):
                self.assertEqual(classify_paths([path])["targets"], [])

    def test_retired_staged_clients_do_not_trigger_live_deploys(self):
        for path in ("docker-compose.headscale-macbook.yml", "docker-compose.headscale-mini.yml"):
            with self.subTest(path=path):
                self.assertEqual(classify_paths([path])["targets"], [])
                self.assertEqual(
                    classify_paths([path, "rust-arr/src/library.rs"])["targets"],
                    ["nodes"],
                )


if __name__ == "__main__":
    unittest.main()
