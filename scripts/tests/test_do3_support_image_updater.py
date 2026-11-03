#!/usr/bin/env python3
"""Regression checks for the do3 support-image updater."""

import importlib.util
import json
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest.mock import patch


ROOT = Path(__file__).resolve().parents[2]
MODULE_PATH = ROOT / "deploy" / "do3" / "update_support_images.py"
SPEC = importlib.util.spec_from_file_location("rawrarr_do3_support_image_updater", MODULE_PATH)
UPDATER = importlib.util.module_from_spec(SPEC)
assert SPEC.loader is not None
SPEC.loader.exec_module(UPDATER)


class SupportImageUpdaterTests(unittest.TestCase):
    def test_targets_are_limited_to_repo_declared_support_images(self):
        projects = {target[0] for target in UPDATER.TARGETS}
        self.assertEqual(projects, set(UPDATER.PROJECTS))
        self.assertTrue(all(not target[3].startswith("rawrarr:") for target in UPDATER.TARGETS))
        internal_containers = {
            container for project, container, _service, _image in UPDATER.TARGETS
            if project == "rawrarr-internal-do3"
        }
        self.assertEqual(internal_containers, {
            "rawrarr-int-postgres",
            "rawrarr-int-tailscale",
            "rawrarr-int-caddy",
            "rawrarr-int-umami-postgres",
        })
        self.assertEqual(
            UPDATER.PROJECTS["rawrarr-internal-do3"]["health_container"],
            "rawrarr-int-roarlay",
        )
        self.assertEqual(
            UPDATER.PROJECTS["rawrarr-internal-do3"]["tailscale_container"],
            "rawrarr-int-tailscale",
        )
        production_containers = {
            container for project, container, _service, _image in UPDATER.TARGETS
            if project == "rawrarr-roarlay-prod"
        }
        self.assertEqual(production_containers, {
            "rawrarr-prod-postgres",
            "rawrarr-prod-tailscale",
            "rawrarr-prod-caddy",
            "rawrarr-prod-cloudflared",
            "rawrarr-prod-crowdsec",
            "rawrarr-prod-headscale",
            "rawrarr-prod-umami",
            "rawrarr-prod-umami-postgres",
        })
        self.assertEqual(
            UPDATER.PROJECTS["rawrarr-roarlay-prod"]["health_container"],
            "rawrarr-prod-roarlay",
        )
        self.assertEqual(
            UPDATER.PROJECTS["rawrarr-roarlay-prod"]["tailscale_container"],
            "rawrarr-prod-tailscale",
        )
        for project, settings in UPDATER.PROJECTS.items():
            override = ROOT / "deploy" / "do3" / settings["compose"]
            self.assertTrue(override.is_file())
            for target_project, _container, _service, image in UPDATER.TARGETS:
                if target_project == project:
                    self.assertIn(f"image: {image}", override.read_text(encoding="utf-8"))
                    self.assertIn("pull_policy: always", override.read_text(encoding="utf-8"))

    def test_compose_context_preserves_active_files_and_checks_project_service(self):
        with tempfile.TemporaryDirectory() as temporary:
            project_dir = Path(temporary)
            base = project_dir / "compose.yml"
            overlay = project_dir / "support.yml"
            base.touch()
            overlay.touch()
            labels = {
                "com.docker.compose.project": "rawrarr-internal-do3",
                "com.docker.compose.project.working_dir": str(project_dir),
                "com.docker.compose.service": "postgres",
                "com.docker.compose.project.config_files": str(base),
            }
            data = {"Config": {"Labels": labels}}
            with patch.object(UPDATER, "docker_container", return_value=data), \
                    patch.dict(UPDATER.PROJECTS["rawrarr-internal-do3"], {"directory": project_dir}):
                actual_dir, files = UPDATER.compose_context(
                    "rawrarr-internal-do3", "container", "postgres", overlay
                )
            self.assertEqual(actual_dir, project_dir)
            self.assertEqual(files, [base, overlay])
            labels["com.docker.compose.service"] = "roarlay"
            with patch.object(UPDATER, "docker_container", return_value=data), \
                    patch.dict(UPDATER.PROJECTS["rawrarr-internal-do3"], {"directory": project_dir}):
                with self.assertRaisesRegex(RuntimeError, "expected Compose service"):
                    UPDATER.compose_context("rawrarr-internal-do3", "container", "postgres", overlay)

    def test_compose_command_scopes_to_service_and_never_pulls_during_recreation(self):
        args = UPDATER.compose_args(
            "rawrarr-internal-do3",
            Path("/opt/rawrarr-internal-do3"),
            [Path("base.yml"), Path("support.yml")],
            ["up", "-d", "--no-deps", "--pull", "never", "postgres"],
        )
        self.assertEqual(args[-6:], ["up", "-d", "--no-deps", "--pull", "never", "postgres"])
        self.assertEqual(args.count("-f"), 2)

    def test_compose_command_includes_protected_environment_file(self):
        env_file = Path("/opt/rawrarr-internal-do3/secrets/analytics.env")
        args = UPDATER.compose_args(
            "rawrarr-internal-do3",
            Path("/opt/rawrarr-internal-do3"),
            [Path("base.yml")],
            ["config", "--quiet"],
            env_file=env_file,
        )
        env_file_index = args.index("--env-file")
        self.assertEqual(args[env_file_index:env_file_index + 2], ["--env-file", str(env_file)])
        self.assertEqual(args[-2:], ["config", "--quiet"])

    def test_internal_compose_environment_file_is_used_when_present(self):
        with tempfile.TemporaryDirectory() as temporary:
            project_dir = Path(temporary)
            env_file = project_dir / "secrets" / "analytics.env"
            env_file.parent.mkdir()
            env_file.write_text("DATABASE_PASSWORD=not-a-real-secret\n", encoding="utf-8")
            with patch.dict(
                UPDATER.PROJECTS["rawrarr-internal-do3"],
                {"environment_file": Path("secrets/analytics.env")},
            ):
                self.assertEqual(
                    UPDATER.compose_environment_file("rawrarr-internal-do3", project_dir),
                    env_file,
                )

    def test_internal_compose_environment_file_rejects_paths_outside_project(self):
        with tempfile.TemporaryDirectory() as temporary, tempfile.TemporaryDirectory() as outside:
            project_dir = Path(temporary)
            env_file = project_dir / "secrets" / "analytics.env"
            env_file.parent.mkdir()
            outside_file = Path(outside) / "analytics.env"
            outside_file.write_text("DATABASE_PASSWORD=not-a-real-secret\n", encoding="utf-8")
            env_file.symlink_to(outside_file)
            with patch.dict(
                UPDATER.PROJECTS["rawrarr-internal-do3"],
                {"environment_file": Path("secrets/analytics.env")},
            ):
                with self.assertRaisesRegex(RuntimeError, "Invalid protected Compose environment file"):
                    UPDATER.compose_environment_file("rawrarr-internal-do3", project_dir)

    def test_internal_compose_environment_uses_the_running_app_image(self):
        app = {"Config": {"Image": "rawrarr:published-revision-amd64"}}
        with patch.object(UPDATER, "docker_container", return_value=app), \
                patch.dict(UPDATER.os.environ, {"RAWARR_INTERNAL_IMAGE": "rawrarr:stale"}), \
                patch.dict(
                    UPDATER.PROJECTS["rawrarr-internal-do3"],
                    {"directory": Path("/opt/rawrarr-internal-do3")},
                ):
            environment = UPDATER.compose_environment(
                "rawrarr-internal-do3", Path("/opt/rawrarr-internal-do3"), []
            )
        self.assertEqual(environment["RAWARR_INTERNAL_IMAGE"], "rawrarr:published-revision-amd64")

    def test_internal_compose_environment_fails_without_a_rawrarr_image(self):
        app = {"Config": {"Image": "postgres:17-alpine"}}
        with patch.object(UPDATER, "docker_container", return_value=app):
            with self.assertRaisesRegex(RuntimeError, "active internal coordinator image"):
                UPDATER.compose_environment(
                    "rawrarr-internal-do3", Path("/opt/rawrarr-internal-do3"), []
                )

    def test_public_crowdsec_compose_values_come_from_active_project_files(self):
        revision = "a" * 40
        with tempfile.TemporaryDirectory() as temporary:
            project_dir = Path(temporary)
            source_root = project_dir / "builds" / "published" / "source"
            crowdsec_compose = source_root / "deploy" / "do3" / "crowdsec" / "compose.yml"
            crowdsec_compose.parent.mkdir(parents=True)
            crowdsec_compose.touch()
            rawrarr_compose = project_dir / f"docker-compose.rawrarr-main-{revision}.yml"
            rawrarr_compose.touch()
            secret_file = project_dir / "secrets" / "caddy-crowdsec-bouncer"
            secret_file.parent.mkdir()
            secret_file.touch()
            with patch.dict(
                UPDATER.PROJECTS["rawrarr-roarlay-prod"],
                {"directory": project_dir},
            ), patch.dict(UPDATER.os.environ, {}, clear=True):
                environment = UPDATER.compose_environment(
                    "rawrarr-roarlay-prod", project_dir, [rawrarr_compose, crowdsec_compose]
                )
        self.assertEqual(environment["CROWDSEC_SOURCE_ROOT"], str(source_root))
        self.assertEqual(environment["RAWRARR_SOURCE_SHA"], revision)
        self.assertEqual(environment["CROWDSEC_BOUNCER_SECRET_FILE"], str(secret_file))

    def test_override_is_private_and_never_pulls_during_up(self):
        with tempfile.TemporaryDirectory() as temporary:
            path = Path(temporary) / "support.override.yml"
            UPDATER.write_override(path, "postgres", "postgres:17-alpine")
            self.assertEqual(
                path.read_text(encoding="utf-8"),
                "services:\n  postgres:\n    image: postgres:17-alpine\n    pull_policy: never\n",
            )
            self.assertEqual(path.stat().st_mode & 0o777, 0o600)

    def test_deployment_lock_is_released_after_guarded_work(self):
        with tempfile.TemporaryDirectory() as temporary:
            marker = Path(temporary) / ".container-rebuild-postgres.lock"
            with UPDATER.deployment_lock(marker, "rawrarr-int-postgres"):
                self.assertTrue(marker.exists())
                self.assertIn("container=rawrarr-int-postgres", marker.read_text())
            self.assertFalse(marker.exists())

    def test_internal_route_verifies_local_health_and_tailscale_serve(self):
        serve = {
            "Web": {
                "rawrarr.risk-mermaid.ts.net:443": {
                    "Handlers": {"/": {"Proxy": "http://127.0.0.1:8080"}}
                }
            }
        }
        with patch.object(
            UPDATER,
            "command",
            side_effect=[
                subprocess.CompletedProcess([], 0, "", ""),
                subprocess.CompletedProcess([], 0, json.dumps(serve), ""),
                subprocess.CompletedProcess([], 0, "", ""),
            ],
        ) as command:
            UPDATER.verify_route("rawrarr-internal-do3")
        self.assertEqual(command.call_args_list[0].args[0][:4], [
            "docker", "exec", "rawrarr-int-roarlay", "curl",
        ])
        self.assertEqual(command.call_args_list[2].args[0][-1], UPDATER.PROJECTS["rawrarr-internal-do3"]["health_url"])

    def test_public_route_requires_local_health_not_cloudflare_edge(self):
        with patch.object(
            UPDATER,
            "command",
            return_value=subprocess.CompletedProcess([], 0, "", ""),
        ) as command:
            UPDATER.verify_route("rawrarr-roarlay-prod")
        self.assertEqual(command.call_count, 1)
        self.assertIn(
            "rawrarr-prod-roarlay",
            command.call_args.args[0],
        )
        self.assertNotEqual(command.call_args.args[0][0], "curl")

    def test_public_edge_403_is_a_warning_after_local_health(self):
        with patch.object(
            UPDATER,
            "command",
            return_value=subprocess.CompletedProcess([], 0, "403", ""),
        ), patch("builtins.print") as print_warning:
            UPDATER.verify_public_edge()
        print_warning.assert_called_once()
        self.assertIn("403", print_warning.call_args.args[0])

    def test_command_can_capture_expected_nonzero_health_probe(self):
        completed = subprocess.CompletedProcess(["curl"], 0, "403", "")
        with patch.object(UPDATER.subprocess, "run", return_value=completed) as run:
            result = UPDATER.command(["curl"], check=False)
        self.assertIs(result, completed)
        self.assertFalse(run.call_args.kwargs["check"])


if __name__ == "__main__":
    unittest.main()
