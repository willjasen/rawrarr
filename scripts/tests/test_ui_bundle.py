import hashlib
import importlib.util
import io
import json
import pathlib
import subprocess
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
    def test_ui_release_poll_interval_defaults_to_one_minute_and_has_a_floor(self):
        self.assertEqual(updater.DEFAULT_POLL_INTERVAL_SECONDS, 60)
        self.assertEqual(updater.poll_interval_seconds(updater.DEFAULT_POLL_INTERVAL_SECONDS), 60)
        self.assertEqual(updater.poll_interval_seconds(1), 60)
        self.assertEqual(updater.poll_interval_seconds(120), 120)

    def setUp(self):
        self.keys = tempfile.TemporaryDirectory()
        self.key_dir = pathlib.Path(self.keys.name)
        self.private_key = self.key_dir / "private.pem"
        self.public_key = self.key_dir / "public.pem"
        subprocess.run(["openssl", "genpkey", "-algorithm", "ED25519", "-out", str(self.private_key)], check=True)
        subprocess.run(["openssl", "pkey", "-in", str(self.private_key), "-pubout", "-out", str(self.public_key)], check=True)
        self.previous_public_key = updater.PINNED_PUBLIC_KEY_FILE
        updater.PINNED_PUBLIC_KEY_FILE = self.public_key

    def tearDown(self):
        updater.PINNED_PUBLIC_KEY_FILE = self.previous_public_key
        self.keys.cleanup()

    def sign(self, bundle):
        source, signature = self.key_dir / "bundle.zip", self.key_dir / "bundle.sig"
        source.write_bytes(bundle)
        subprocess.run(["openssl", "pkeyutl", "-sign", "-rawin", "-inkey", str(self.private_key), "-in", str(source), "-out", str(signature)], check=True)
        return signature.read_bytes()

    @staticmethod
    def release(assets=None):
        return {"tag_name": "ui-test", "target_commitish": "main", "assets": assets or [
            {"name": "ui-bundle.zip", "id": 1}, {"name": "ui-bundle.zip.sha256", "id": 2}, {"name": "ui-bundle.zip.sig", "id": 3}]}

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
        signature = self.sign(bundle)
        release = self.release()
        with tempfile.TemporaryDirectory() as folder:
            token = pathlib.Path(folder) / "token"
            token.write_text("test-token")
            root = pathlib.Path(folder) / "ui"
            for apply, expected in ((False, "dry-run"), (True, "installed")):
                with patch.object(updater, "fetch", side_effect=[json.dumps(release).encode(), bundle, digest, signature]):
                    self.assertEqual(updater.update("owner/repo", token, root, apply), expected)
            current = root / "current"
            self.assertEqual((current / "ui.html").read_text(), "<h1>new</h1>")
            self.assertEqual((current / "bundle.sha256").read_text().strip(), digest.decode())
            self.assertEqual((current / "bundle.signature-status").read_text().strip(), "verified")

    def test_github_asset_download_and_repeat_check(self):
        buffer = io.BytesIO()
        with zipfile.ZipFile(buffer, "w") as archive:
            archive.writestr("ui.html", "signed bundle")
        bundle = buffer.getvalue()
        digest = hashlib.sha256(bundle).hexdigest()
        signature = self.sign(bundle)
        release = self.release([{"name": "ui-bundle.zip", "id": 41}, {"name": "ui-bundle.zip.sha256", "id": 42}, {"name": "ui-bundle.zip.sig", "id": 43}])
        with tempfile.TemporaryDirectory() as folder:
            token_file, root = pathlib.Path(folder) / "token", pathlib.Path(folder) / "ui"
            token_file.write_text("protected-test-token\n")
            requests = []
            def github_fetch(url, token, *, accept="application/vnd.github+json"):
                requests.append((url, token, accept))
                if url.endswith("/releases/latest"): return json.dumps(release).encode()
                if url.endswith("/assets/41"): return bundle
                if url.endswith("/assets/42"): return f"{digest}  ui-bundle.zip\n".encode()
                if url.endswith("/assets/43"): return signature
                self.fail(f"unexpected request: {url}")
            with patch.object(updater, "fetch", side_effect=github_fetch):
                self.assertEqual(updater.update("owner/repo", token_file, root, True), "installed")
            self.assertEqual(len(requests), 4)
            self.assertTrue(all(request[1] == "protected-test-token" for request in requests))
            with patch.object(updater, "fetch", side_effect=github_fetch):
                self.assertEqual(updater.update("owner/repo", token_file, root, True), "current")

    def test_rejects_bad_checksum_and_traversal(self):
        buffer = io.BytesIO()
        with zipfile.ZipFile(buffer, "w") as archive: archive.writestr("../escape", "bad")
        bundle = buffer.getvalue()
        release = self.release()
        with tempfile.TemporaryDirectory() as folder:
            token = pathlib.Path(folder) / "token"
            token.write_text("test-token")
            signature = self.sign(bundle)
            with patch.object(updater, "fetch", side_effect=[json.dumps(release).encode(), bundle, b"0" * 64, signature]):
                with self.assertRaisesRegex(ValueError, "checksum mismatch"):
                    updater.update("owner/repo", token, pathlib.Path(folder) / "ui", True)
            digest = hashlib.sha256(bundle).hexdigest().encode()
            with patch.object(updater, "fetch", side_effect=[json.dumps(release).encode(), bundle, digest, signature]):
                with self.assertRaisesRegex(ValueError, "unsafe path"):
                    updater.update("owner/repo", token, pathlib.Path(folder) / "ui", True)
            self.assertFalse((pathlib.Path(folder) / "escape").exists())

    def test_rejects_invalid_or_missing_signature_without_replacing_current(self):
        bundle = b"attacker bundle"
        release = self.release()
        with tempfile.TemporaryDirectory() as folder:
            token, root = pathlib.Path(folder) / "token", pathlib.Path(folder) / "ui"
            token.write_text("test")
            current = root / "current"
            current.mkdir(parents=True)
            (current / "ui.html").write_text("trusted")
            digest = hashlib.sha256(bundle).hexdigest().encode()
            with patch.object(updater, "fetch", side_effect=[json.dumps(release).encode(), bundle, digest, b"invalid"]):
                with self.assertRaisesRegex(ValueError, "signature verification failed"):
                    updater.update("owner/repo", token, root, True)
            self.assertEqual((current / "ui.html").read_text(), "trusted")
            missing = self.release(release["assets"][:-1])
            with patch.object(updater, "fetch", side_effect=[json.dumps(missing).encode()]):
                with self.assertRaisesRegex(ValueError, "missing its bundle, checksum, or signature"):
                    updater.update("owner/repo", token, root, True)
            self.assertEqual((current / "ui.html").read_text(), "trusted")

if __name__ == "__main__": unittest.main()
