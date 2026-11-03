"""Lifecycle checks for the opt-in disposable browser service tier."""

import subprocess
import sys
import unittest
from unittest.mock import patch

import browser_enrollment_docker as harness
import run_rawrarr_tests_in_docker as runner


class BrowserEnrollmentDockerTests(unittest.TestCase):
    def execute(self, *, failure=None, inject_failure=False, cleanup_failure=False):
        self.calls = []
        self.copied = []

        def docker(context, *args):
            self.calls.append(args)
            if failure == "prepare-create" and args[0] == "create" and "--tmpfs" in args:
                raise subprocess.CalledProcessError(1, ["docker", "private-credential"])
            return ""

        def run(command, **kwargs):
            self.calls.append(tuple(command[3:]))
            code = 0
            if "exec" in command:
                if failure == "prepare" and "sh" in command and command[-1].endswith("prepare-test-clone.sh"):
                    raise subprocess.CalledProcessError(1, command)
                if command[-1] == "--inject-failure":
                    code = 1
                if failure == "browser" and command[-1].endswith("browser_enrollment_export.mjs"):
                    code = 1
            if "rm" in command and cleanup_failure:
                code = 1
            return subprocess.CompletedProcess(command, code, stdout=b"fixture dump", stderr=b"")

        with patch.object(runner, "docker", side_effect=docker), \
             patch.object(runner, "copy_project_into_container", side_effect=lambda *args: self.copied.append(args[:2])), \
             patch.object(runner, "timestamp_code", return_value="fixture"), \
             patch.object(harness.subprocess, "run", side_effect=run):
            return harness.run("orbstack", inject_failure=inject_failure)

    def test_success_uses_offline_snapshot_and_cleans_owned_containers(self):
        self.assertEqual(self.execute(), 0)
        self.assertEqual(len(self.copied), 1)
        creates = [call for call in self.calls if call[0] == "create"]
        self.assertEqual(len(creates), 2)
        for call in creates:
            self.assertNotIn("-p", call)
            self.assertNotIn("-v", call)
            self.assertNotIn("--mount", call)
        self.assertIn("--tmpfs", creates[1])
        disconnect = next(i for i, call in enumerate(self.calls) if call[:2] == ("network", "disconnect"))
        postgres_create = self.calls.index(creates[1])
        self.assertLess(disconnect, postgres_create)
        removals = [call for call in self.calls if call[0] == "rm"]
        self.assertEqual(len(removals), 2)
        names = [call[call.index("--name") + 1] for call in creates]
        self.assertEqual([call[-1] for call in removals], list(reversed(names)))
        self.assertTrue(any("--audit-database" in call for call in self.calls))

    def test_browser_failure_and_injected_failure_clean_both_containers(self):
        for kwargs in [{"failure": "browser"}, {"inject_failure": True}]:
            with self.subTest(kwargs=kwargs):
                self.assertEqual(self.execute(**kwargs), 1)
                self.assertEqual(len([call for call in self.calls if call[0] == "rm"]), 2)
                self.assertFalse(any("--audit-database" in call for call in self.calls))

    def test_preparation_failure_cleans_clone_before_postgres_exists(self):
        with self.assertRaises(subprocess.CalledProcessError):
            self.execute(failure="prepare")
        self.assertEqual(len([call for call in self.calls if call[0] == "rm"]), 1)

    def test_postgres_creation_failure_cleans_clone_without_logging_credentials(self):
        with self.assertRaisesRegex(RuntimeError, r"browser export Docker create failed \(exit 1\)") as caught:
            self.execute(failure="prepare-create")
        self.assertNotIn("private-credential", str(caught.exception))
        self.assertEqual(len([call for call in self.calls if call[0] == "rm"]), 1)

    def test_cleanup_failure_is_explicit_and_still_attempts_both_removals(self):
        with self.assertRaisesRegex(RuntimeError, "could not remove owned"):
            self.execute(cleanup_failure=True)
        self.assertEqual(len([call for call in self.calls if call[0] == "rm"]), 2)

    def test_runner_dispatches_without_running_other_tiers_or_old_container_cleanup(self):
        with patch.object(sys, "argv", ["runner", "--browser-enrollment-export", "--browser-enrollment-inject-failure"]), \
             patch.object(runner, "docker", return_value="orbstack"), \
             patch.object(runner, "cleanup_if_due") as cleanup, \
             patch.object(harness, "run", return_value=0) as run:
            self.assertEqual(runner.main(), 0)
            run.assert_called_once_with("orbstack", inject_failure=True)
            cleanup.assert_not_called()

    def test_runner_requires_explicit_tier_for_failure_injection(self):
        with patch.object(sys, "argv", ["runner", "--browser-enrollment-inject-failure"]), \
             patch("sys.stderr"), self.assertRaises(SystemExit) as caught:
            runner.main()
        self.assertEqual(caught.exception.code, 2)


if __name__ == "__main__":
    unittest.main()
