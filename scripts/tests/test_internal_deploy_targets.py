import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from classify_internal_deploy_targets import classify_paths


class InternalDeployTargetTests(unittest.TestCase):
    def test_coordinator_only_changes_select_coordinator(self):
        result = classify_paths(["rust-arr/src/coordinator.rs", "docker-compose.internal-mini.yml"])
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


if __name__ == "__main__":
    unittest.main()
