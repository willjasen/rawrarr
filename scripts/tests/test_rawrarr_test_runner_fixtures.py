import importlib.util
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch


RUNNER_PATH = Path(__file__).with_name("run_rawrarr_tests_in_docker.py")
SPEC = importlib.util.spec_from_file_location("rawrarr_test_runner", RUNNER_PATH)
RUNNER = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(RUNNER)


class RootFixtureCopyTests(unittest.TestCase):
    def test_netlify_build_script_is_copied_into_test_workspace(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            (root / "netlify-ignore-build.sh").write_text("#!/bin/sh\n", encoding="utf-8")
            with patch.object(RUNNER, "ROOT", root), patch.object(RUNNER.subprocess, "run") as run:
                RUNNER.copy_root_fixtures("orbstack", "rawrarr-test-regression-tests-example")

        self.assertIn("netlify-ignore-build.sh", RUNNER.ROOT_FIXTURES)
        run.assert_called_once_with(
            [
                "docker", "--context", "orbstack", "cp",
                str(root / "netlify-ignore-build.sh"),
                "rawrarr-test-regression-tests-example:/workspace/",
            ],
            check=True,
        )


if __name__ == "__main__":
    unittest.main()
