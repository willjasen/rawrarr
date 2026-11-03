import hashlib
import importlib.util
import io
import json
import pathlib
import tempfile
import unittest
import zipfile
from unittest.mock import patch

ROOT = pathlib.Path(__file__).resolve().parents[2]


def load(name, path):
    spec = importlib.util.spec_from_file_location(name, ROOT / path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


package = load("package_ui_bundle", "scripts/package_ui_bundle.py")
updater = load("update_ui_bundle", "rust-arr/update_ui_bundle.py")


class UiBundleTests(unittest.TestCase):
    def test_coordinator_updater_poll_interval_defaults_to_two_minutes(self):
        self.assertEqual(updater.DEFAULT_POLL_INTERVAL_SECONDS, 120)
        self.assertEqual(updater.MIN_POLL_INTERVAL_SECONDS, 120)
        self.assertEqual(updater.poll_interval_seconds(1), 120)
        self.assertEqual(updater.poll_interval_seconds(120), 120)
        self.assertEqual(updater.poll_interval_seconds(900), 900)

    @staticmethod
    def release(assets=None):
        return {
            "tag_name": "ui-test",
            "target_commitish": "main",
            "assets": assets or [
                {"name": "ui-bundle.zip", "id": 1},
                {"name": "ui-bundle.zip.sha256", "id": 2},
            ],
        }

    def test_package_allowlist_and_checksum(self):
        with tempfile.TemporaryDirectory() as folder:
            output = pathlib.Path(folder) / "bundle.zip"
            package.package(output)
            with zipfile.ZipFile(output) as archive:
                self.assertEqual(set(archive.namelist()), set(package.FILES))
            digest = hashlib.sha256(output.read_bytes()).hexdigest()
            self.assertIn(digest, output.with_suffix(".zip.sha256").read_text())

    def test_unsigned_release_dry_run_and_checksum_verified_install(self):
        buffer = io.BytesIO()
        with zipfile.ZipFile(buffer, "w") as archive:
            archive.writestr("ui.html", "<h1>new</h1>")
        bundle = buffer.getvalue()
        digest = hashlib.sha256(bundle).hexdigest().encode()
        release = self.release()
        with tempfile.TemporaryDirectory() as folder:
            token = pathlib.Path(folder) / "token"
            token.write_text("test-token")
            root = pathlib.Path(folder) / "ui"
            responses = [json.dumps(release).encode(), bundle, digest]
            with patch.object(updater, "fetch", side_effect=responses):
                self.assertEqual(updater.update("owner/repo", token, root, False), "dry-run")
            with patch.object(updater, "fetch", side_effect=responses):
                self.assertEqual(updater.update("owner/repo", token, root, True), "installed")
            current = root / "current"
            self.assertEqual((current / "ui.html").read_text(), "<h1>new</h1>")
            self.assertEqual((current / "bundle.sha256").read_text().strip(), digest.decode())
            self.assertEqual((current / "bundle.version").read_text().strip(), "main")
            self.assertFalse((current / "bundle.signature-status").exists())

    def test_github_asset_download_and_repeat_check(self):
        buffer = io.BytesIO()
        with zipfile.ZipFile(buffer, "w") as archive:
            archive.writestr("coordinator.html", "<h1>GitHub release UI</h1>")
        bundle = buffer.getvalue()
        digest = hashlib.sha256(bundle).hexdigest()
        release = {
            "tag_name": "ui-abc123",
            "target_commitish": "abc123",
            "assets": [
                {"name": "ui-bundle.zip", "id": 41},
                {"name": "ui-bundle.zip.sha256", "id": 42},
                # Publisher signatures may remain attached to releases, but
                # temporary signature-disabled clients do not fetch them.
                {"name": "ui-bundle.zip.sig", "id": 43},
            ],
        }
        with tempfile.TemporaryDirectory() as folder:
            token_file = pathlib.Path(folder) / "github-token"
            token_file.write_text("protected-test-token\n")
            root = pathlib.Path(folder) / "ui"
            requests = []

            def github_fetch(url, token, *, accept="application/vnd.github+json"):
                requests.append((url, token, accept))
                if url == "https://api.github.com/repos/owner/repo/releases/latest":
                    return json.dumps(release).encode()
                if url.endswith("/assets/41"):
                    return bundle
                if url.endswith("/assets/42"):
                    return f"{digest}  ui-bundle.zip\n".encode()
                self.fail(f"unexpected GitHub API request: {url}")

            with patch.object(updater, "fetch", side_effect=github_fetch):
                self.assertEqual(updater.update("owner/repo", token_file, root, True), "installed")
            self.assertEqual(
                [request[0] for request in requests],
                [
                    "https://api.github.com/repos/owner/repo/releases/latest",
                    "https://api.github.com/repos/owner/repo/releases/assets/41",
                    "https://api.github.com/repos/owner/repo/releases/assets/42",
                ],
            )
            self.assertTrue(all(request[1] == "protected-test-token" for request in requests))
            self.assertEqual(requests[0][2], "application/vnd.github+json")
            self.assertTrue(all(request[2] == "application/octet-stream" for request in requests[1:]))
            self.assertEqual((root / "current" / "coordinator.html").read_text(), "<h1>GitHub release UI</h1>")
            self.assertEqual((root / "current" / "bundle.version").read_text().strip(), "abc123")
            self.assertEqual((root / "current" / "bundle.sha256").read_text().strip(), digest)

            requests.clear()
            with patch.object(updater, "fetch", side_effect=github_fetch):
                self.assertEqual(updater.update("owner/repo", token_file, root, True), "current")
            self.assertEqual(len(requests), 3)

    def test_rejects_bad_checksum_and_traversal(self):
        buffer = io.BytesIO()
        with zipfile.ZipFile(buffer, "w") as archive:
            archive.writestr("../escape", "bad")
        bundle = buffer.getvalue()
        release = self.release()
        with tempfile.TemporaryDirectory() as folder:
            token = pathlib.Path(folder) / "token"
            token.write_text("test-token")
            root = pathlib.Path(folder) / "ui"
            current = root / "current"
            current.mkdir(parents=True)
            (current / "ui.html").write_text("active bundle")
            with (
                patch.object(
                    updater,
                    "fetch",
                    side_effect=[json.dumps(release).encode(), bundle, b"0" * 64],
                ),
            ):
                with self.assertRaisesRegex(ValueError, "checksum mismatch"):
                    updater.update("owner/repo", token, root, True)
            self.assertEqual((current / "ui.html").read_text(), "active bundle")
            digest = hashlib.sha256(bundle).hexdigest().encode()
            with (
                patch.object(
                    updater,
                    "fetch",
                    side_effect=[json.dumps(release).encode(), bundle, digest],
                ),
            ):
                with self.assertRaisesRegex(ValueError, "unsafe path"):
                    updater.update("owner/repo", token, root, True)
            self.assertEqual((current / "ui.html").read_text(), "active bundle")
            self.assertFalse((pathlib.Path(folder) / "escape").exists())

    def test_missing_bundle_or_checksum_preserves_active_ui(self):
        release = self.release(assets=[{"name": "ui-bundle.zip", "id": 1}])
        with tempfile.TemporaryDirectory() as folder:
            token = pathlib.Path(folder) / "token"
            token.write_text("test-token")
            root = pathlib.Path(folder) / "ui"
            current = root / "current"
            current.mkdir(parents=True)
            (current / "ui.html").write_text("trusted")
            with patch.object(updater, "fetch", return_value=json.dumps(release).encode()):
                with self.assertRaisesRegex(ValueError, "missing its bundle or checksum"):
                    updater.update("owner/repo", token, root, True)
            self.assertEqual((current / "ui.html").read_text(), "trusted")


if __name__ == "__main__":
    unittest.main()
