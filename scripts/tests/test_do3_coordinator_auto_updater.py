#!/usr/bin/env python3
"""Regression checks for the do3 coordinator auto-updater."""

import importlib.util
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch


ROOT = Path(__file__).resolve().parents[2]
MODULE_PATH = ROOT / "deploy" / "do3" / "auto_update_coordinators.py"
SPEC = importlib.util.spec_from_file_location("rawrarr_do3_auto_updater", MODULE_PATH)
UPDATER = importlib.util.module_from_spec(SPEC)
assert SPEC.loader is not None
SPEC.loader.exec_module(UPDATER)


class CoordinatorAutoUpdaterTests(unittest.TestCase):
    def test_runtime_paths_select_coordinator_updates(self):
        self.assertTrue(UPDATER.affects_coordinator(["rust-arr/src/coordinator.rs"]))
        self.assertTrue(UPDATER.affects_coordinator(["rust-arr/src/node_role.rs"]))
        self.assertTrue(UPDATER.affects_coordinator(["deploy/do3/docker-compose.internal-coordinator.yml"]))
        self.assertFalse(UPDATER.affects_coordinator(["deploy/do3/auto_update_coordinators.py"]))

    def test_documentation_tests_and_landing_page_changes_are_skipped(self):
        self.assertFalse(UPDATER.affects_coordinator([
            "AGENTS.md", "docs/container-deployment.md", "scripts/tests/test_example.py",
            ".github/workflows/tests.yml", "website/styles.css",
        ]))

    def test_override_pins_local_image_and_records_deployment_time(self):
        with tempfile.TemporaryDirectory() as temporary:
            path = Path(temporary) / "deployment-info.override.yml"
            UPDATER.write_override(path, "rawrarr:abc123-amd64", "2026-11-03T05:00:00Z")
            self.assertEqual(
                path.read_text(encoding="utf-8"),
                'services:\n  roarlay:\n    image: rawrarr:abc123-amd64\n'
                '    pull_policy: never\n    environment:\n'
                '      RAWARR_DEPLOYED_AT: "2026-11-03T05:00:00Z"\n',
            )
            self.assertEqual(path.stat().st_mode & 0o777, 0o600)

    def test_compose_command_recreates_only_selected_service_without_pulling(self):
        args = UPDATER.compose_command(
            "rawrarr-internal-do3", Path("/opt/rawrarr-internal-do3"),
            [Path("base.yml"), Path("override.yml")], "roarlay",
        )
        self.assertEqual(args[-6:], ["up", "-d", "--no-deps", "--pull", "never", "roarlay"])
        self.assertEqual(args.count("-f"), 2)

    def test_deployment_lock_is_released_after_the_guarded_work(self):
        with tempfile.TemporaryDirectory() as temporary:
            marker = Path(temporary) / ".container-rebuild-roarlay.lock"
            with UPDATER.deployment_lock(marker, "a" * 40):
                self.assertTrue(marker.exists())
                self.assertIn("revision=" + "a" * 40, marker.read_text(encoding="utf-8"))
            self.assertFalse(marker.exists())

    def test_compose_files_keep_active_overrides_and_replace_old_timestamp_file(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            project = root / "project"
            project.mkdir()
            base = project / "compose.yml"
            image_override = project / "image.override.yml"
            previous_timestamp = project / "builds" / "old" / "deployment-info.override.yml"
            next_timestamp = project / "builds" / "new" / "deployment-info.override.yml"
            for path in (base, image_override, previous_timestamp):
                path.parent.mkdir(parents=True, exist_ok=True)
                path.touch()
            labels = {
                "com.docker.compose.project": "rawrarr-test",
                "com.docker.compose.project.working_dir": str(project),
                "com.docker.compose.service": "roarlay",
                "com.docker.compose.project.config_files": ",".join(
                    (str(base), str(image_override), str(previous_timestamp))
                ),
            }
            with patch.object(UPDATER, "docker_container", return_value={"Config": {"Labels": labels}}):
                _name, _directory, _service, files = UPDATER.compose_files("service", next_timestamp)
            self.assertEqual(files, [base, image_override, next_timestamp])


if __name__ == "__main__":
    unittest.main()
