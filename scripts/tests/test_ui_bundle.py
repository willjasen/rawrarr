import hashlib
import importlib.util
import io
import json
import pathlib
import tempfile
import urllib.request
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
    def test_package_allowlist_and_checksum(self):
        with tempfile.TemporaryDirectory() as folder:
            output = pathlib.Path(folder) / "bundle.zip"
            package.package(output)
            with zipfile.ZipFile(output) as archive:
                self.assertEqual(set(archive.namelist()), set(package.FILES))
            digest = hashlib.sha256(output.read_bytes()).hexdigest()
            self.assertIn(digest, output.with_suffix(".zip.sha256").read_text())

    def test_dry_run_and_verified_install(self):
        buffer = io.BytesIO()
        with zipfile.ZipFile(buffer, "w") as archive:
            archive.writestr("ui.html", "<h1>new</h1>")
        bundle = buffer.getvalue()
        digest = hashlib.sha256(bundle).hexdigest().encode()
        release = {"tag_name": "ui-test", "target_commitish": "main", "assets": [{"name": "ui-bundle.zip", "id": 1}, {"name": "ui-bundle.zip.sha256", "id": 2}]}
        with tempfile.TemporaryDirectory() as folder:
            token = pathlib.Path(folder) / "token"
            token.write_text("test-token")
            root = pathlib.Path(folder) / "ui"
            with patch.object(updater, "fetch", side_effect=[json.dumps(release).encode(), bundle, digest]):
                self.assertEqual(updater.update("owner/repo", token, root, False), "dry-run")
            with patch.object(updater, "fetch", side_effect=[json.dumps(release).encode(), bundle, digest]):
                self.assertEqual(updater.update("owner/repo", token, root, True), "installed")
            self.assertEqual((root / "current" / "ui.html").read_text(), "<h1>new</h1>")
            self.assertEqual((root / "current" / "bundle.sha256").read_text().strip(), digest.decode())

    def test_github_release_refresh_installs_and_recognizes_current_version(self):
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
        release = {"tag_name": "ui-test", "assets": [{"name": "ui-bundle.zip", "id": 1}, {"name": "ui-bundle.zip.sha256", "id": 2}]}
        with tempfile.TemporaryDirectory() as folder:
            token = pathlib.Path(folder) / "token"
            token.write_text("test-token")
            with patch.object(updater, "fetch", side_effect=[json.dumps(release).encode(), bundle, b"0" * 64]):
                with self.assertRaisesRegex(ValueError, "checksum mismatch"):
                    updater.update("owner/repo", token, pathlib.Path(folder) / "ui", True)
            digest = hashlib.sha256(bundle).hexdigest().encode()
            with patch.object(updater, "fetch", side_effect=[json.dumps(release).encode(), bundle, digest]):
                with self.assertRaisesRegex(ValueError, "unsafe path"):
                    updater.update("owner/repo", token, pathlib.Path(folder) / "ui", True)
            self.assertFalse((pathlib.Path(folder) / "escape").exists())

if __name__ == "__main__":
    unittest.main()
