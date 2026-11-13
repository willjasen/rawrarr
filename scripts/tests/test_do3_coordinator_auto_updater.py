#!/usr/bin/env python3
"""Regression checks for the do3 coordinator auto-updater."""

import importlib.util
import json
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
    def test_retired_host_specific_storage_paths_do_not_rebuild_coordinators(self):
        for path in (
            "docker-compose.macbook-storage.yml", "docker-compose.tailscale-mini.yml",
            "docker-compose.mini-relays.yml", "storage-proxy.Caddyfile",
        ):
            with self.subTest(path=path):
                self.assertFalse(UPDATER.affects_coordinator([path]))
                self.assertTrue(UPDATER.affects_coordinator([path, "rust-arr/src/coordinator.rs"]))

    def test_internal_container_references_use_the_stable_int_prefix(self):
        self.assertEqual(UPDATER.INTERNAL["container"], "rawrarr-int-roarlay")
        self.assertEqual(UPDATER.INTERNAL_TAILSCALE_CONTAINER, "rawrarr-int-tailscale")

    def test_public_container_reference_uses_the_prod_prefix(self):
        self.assertEqual(UPDATER.PUBLIC["container"], "rawrarr-prod-roarlay")

    def test_published_image_pull_uses_the_exact_sha_and_native_platform(self):
        revision = "a" * 40
        image = f"ghcr.io/{UPDATER.REPOSITORY}:sha-{revision}"
        image_id = "sha256:" + "b" * 64
        with (
            patch.object(UPDATER, "command") as run,
            patch.object(UPDATER, "image_matches", return_value=True) as matches,
        ):
            run.side_effect = [
                UPDATER.subprocess.CompletedProcess([], 0, "", ""),
                UPDATER.subprocess.CompletedProcess([], 0, f'[{{"Id":"{image_id}"}}]', ""),
            ]
            pulled_image, pulled_id = UPDATER.pull_published_image(revision)

        self.assertEqual(pulled_image, image)
        self.assertEqual(pulled_id, image_id)
        run.assert_any_call(["docker", "pull", "--platform", "linux/amd64", image], capture=False)
        run.assert_any_call(["docker", "image", "inspect", image])
        matches.assert_called_once_with(image, revision)

    def test_published_image_with_mismatched_metadata_is_rejected(self):
        revision = "a" * 40
        image = f"ghcr.io/{UPDATER.REPOSITORY}:sha-{revision}"
        with (
            patch.object(UPDATER, "command") as run,
            patch.object(UPDATER, "image_matches", return_value=False),
        ):
            with self.assertRaisesRegex(RuntimeError, "does not match the requested revision/platform"):
                UPDATER.pull_published_image(revision)

        run.assert_called_once_with(["docker", "pull", "--platform", "linux/amd64", image], capture=False)

    def test_latest_published_image_uses_its_source_sha_when_main_has_docs_only(self):
        revision = "a" * 40
        image_id = "sha256:" + "b" * 64
        image = {
            "Id": image_id,
            "Os": "linux",
            "Architecture": "amd64",
            "Config": {"Labels": {"org.opencontainers.image.revision": revision}},
        }
        with (
            patch.object(UPDATER, "command") as run,
            patch.object(UPDATER, "compare_revisions", return_value={
                "status": "ahead", "files": [{"filename": "docs/container-deployment.md"}],
            }),
        ):
            run.side_effect = [
                UPDATER.subprocess.CompletedProcess([], 0, "", ""),
                UPDATER.subprocess.CompletedProcess([], 0, json.dumps([image]), ""),
                UPDATER.subprocess.CompletedProcess([], 1, "", ""),
                UPDATER.subprocess.CompletedProcess([], 0, "", ""),
            ]
            result = UPDATER.pull_latest_published_image("token", "c" * 40)

        tag = f"ghcr.io/{UPDATER.REPOSITORY}:sha-{revision}"
        self.assertEqual(result, (tag, image_id, revision))
        run.assert_any_call(
            ["docker", "pull", "--platform", "linux/amd64", f"ghcr.io/{UPDATER.REPOSITORY}:main"],
            capture=False,
        )
        run.assert_any_call(["docker", "tag", image_id, tag])

    def test_latest_published_image_waits_if_main_has_unpublished_runtime_changes(self):
        revision = "a" * 40
        image = {
            "Id": "sha256:" + "b" * 64,
            "Os": "linux",
            "Architecture": "amd64",
            "Config": {"Labels": {"org.opencontainers.image.revision": revision}},
        }
        with (
            patch.object(UPDATER, "command") as run,
            patch.object(UPDATER, "compare_revisions", return_value={
                "status": "ahead", "files": [{"filename": "rust-arr/src/coordinator.rs"}],
            }),
        ):
            run.side_effect = [
                UPDATER.subprocess.CompletedProcess([], 0, "", ""),
                UPDATER.subprocess.CompletedProcess([], 0, json.dumps([image]), ""),
            ]
            self.assertIsNone(UPDATER.pull_latest_published_image("token", "c" * 40))

        self.assertEqual(run.call_count, 2)

    def test_runtime_paths_select_coordinator_updates(self):
        self.assertTrue(UPDATER.affects_coordinator(["rust-arr/src/coordinator.rs"]))
        self.assertTrue(UPDATER.affects_coordinator(["rust-arr/src/node_role.rs"]))
        self.assertFalse(UPDATER.affects_coordinator([
            "deploy/do3/docker-compose.internal-coordinator.yml",
            "deploy/do3/docker-compose.analytics-dashboard-internal.yml",
            "deploy/do3/docker-compose.production-container-names.yml",
        ]))
        self.assertFalse(UPDATER.affects_coordinator(["deploy/do3/auto_update_coordinators.py"]))

    def test_changed_paths_include_previous_names_for_renamed_files(self):
        with patch.object(UPDATER, "github_json", return_value={
            "files": [{
                "filename": "docs/renamed-coordinator-guide.md",
                "previous_filename": "rust-arr/old-runtime-file.rs",
            }],
        }):
            self.assertEqual(
                UPDATER.changed_paths("token", "a" * 40, "b" * 40),
                ["docs/renamed-coordinator-guide.md", "rust-arr/old-runtime-file.rs"],
            )

    def test_documentation_tests_and_landing_page_changes_are_skipped(self):
        self.assertFalse(UPDATER.affects_coordinator([
            "AGENTS.md", "docs/container-deployment.md", "scripts/tests/test_example.py",
            ".github/workflows/tests.yml", "website/styles.css",
        ]))

    def test_test_stack_relocation_does_not_rebuild_coordinators(self):
        for path in [
            "docker-compose.orbstack-test.yml",
            "deploy/testing/compose.yml",
            "deploy/testing/arr.Caddyfile",
            "deploy/testing/headscale/config.yaml",
        ]:
            with self.subTest(path=path):
                self.assertFalse(UPDATER.affects_coordinator([path]))

    def test_retired_mac_coordinator_files_do_not_rebuild_coordinators(self):
        for path in ("docker-compose.internal-mini.yml", "docker-compose.tailscale-serve.yml", "docker-compose.tailscale-macbook.yml"):
            with self.subTest(path=path):
                self.assertFalse(UPDATER.affects_coordinator([path]))

    def test_retired_staged_clients_do_not_rebuild_coordinators(self):
        for path in ("docker-compose.headscale-macbook.yml", "docker-compose.headscale-mini.yml"):
            with self.subTest(path=path):
                self.assertFalse(UPDATER.affects_coordinator([path]))
                self.assertTrue(UPDATER.affects_coordinator([path, "rust-arr/src/coordinator.rs"]))

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
        with tempfile.TemporaryDirectory() as temporary:
            project_dir = Path(temporary)
            env_file = project_dir / "secrets" / "analytics.env"
            env_file.parent.mkdir()
            env_file.write_text("RAWARR_ANALYTICS_DB_PASSWORD=test-only\n", encoding="utf-8")
            args = UPDATER.compose_command(
                "rawrarr-internal-do3", project_dir,
                [Path("base.yml"), Path("override.yml")], "roarlay",
            )
        self.assertEqual(args[args.index("--env-file") + 1], str(env_file))
        self.assertEqual(args[-6:], ["up", "-d", "--no-deps", "--pull", "never", "roarlay"])
        self.assertEqual(args.count("-f"), 2)

    def test_compose_command_requires_the_project_analytics_env_file(self):
        with tempfile.TemporaryDirectory() as temporary:
            with self.assertRaisesRegex(RuntimeError, "Coordinator Compose env file is missing"):
                UPDATER.compose_command(
                    "rawrarr-internal-do3", Path(temporary),
                    [Path("base.yml"), Path("override.yml")], "roarlay",
                )

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

    def test_failed_run_reports_error_without_raising(self):
        with patch.object(UPDATER, "run_once", side_effect=RuntimeError("internal verification failed")), \
                patch("sys.stderr") as stderr:
            self.assertEqual(UPDATER.main(), 1)
        written = "".join(call.args[0] for call in stderr.write.call_args_list)
        self.assertIn("coordinator update failed: internal verification failed", written)


if __name__ == "__main__":
    unittest.main()
