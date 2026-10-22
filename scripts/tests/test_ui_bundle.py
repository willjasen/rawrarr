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
