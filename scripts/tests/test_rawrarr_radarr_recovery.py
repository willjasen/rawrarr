#!/usr/bin/env python3
"""Test encrypted Radarr recovery validation and upload without Docker or Radarr."""

from __future__ import annotations

import base64
import http.server
import importlib.util
import io
import json
import os
from pathlib import Path
import stat
import sys
import tempfile
import threading
import unittest
import zipfile
from unittest.mock import patch


ROOT = Path(__file__).resolve().parents[2]
sys.dont_write_bytecode = True
SOURCE = (ROOT / "rust-arr" / "src" / "main.rs").read_text()
SECURE_CONFIG = (ROOT / "rust-arr" / "src" / "secure_config.rs").read_text()
SPEC = importlib.util.spec_from_file_location("restore_radarr", ROOT / "scripts" / "restore_radarr.py")
assert SPEC and SPEC.loader
restore_radarr = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(restore_radarr)


def make_zip(entries: dict[str, bytes]) -> bytes:
    stream = io.BytesIO()
    with zipfile.ZipFile(stream, "w", compression=zipfile.ZIP_DEFLATED) as archive:
        for name, contents in entries.items():
            archive.writestr(name, contents)
    return stream.getvalue()


class RadarrRecoveryTests(unittest.TestCase):
    def test_rawrarr_captures_latest_age_encrypted_backup_on_six_hour_cycle(self) -> None:
        self.assertIn('json!({"name": "Backup"})', SOURCE)
        self.assertIn('format!("{}/api/v3/system/backup", self.base_url)', SOURCE)
        self.assertIn(
            'wait_for_radarr_backup(&state, Duration::from_secs(6 * 60 * 60)).await;',
            SOURCE,
        )
        self.assertIn("state.radarr_backup_notify.notify_one()", SOURCE)
        self.assertIn('join("recovery.zip.aged")', SECURE_CONFIG)
        self.assertIn("encrypt_shared_file(&self.recipient, &self.radarr_backup_path(), bytes)", SECURE_CONFIG)

    def test_backup_download_supports_ui_login_without_redirecting_or_putting_secrets_in_urls(self) -> None:
        start = SOURCE.index("async fn backup_download_response")
        end = SOURCE.index("async fn search_movies", start)
        download = SOURCE[start:end]
        self.assertIn(".basic_auth(username.as_str(), Some(password.as_str()))", download)
        self.assertIn('format!("{}/login", self.base_url)', download)
        self.assertIn(".header(COOKIE, cookie)", download)
        self.assertNotIn('.header("X-Api-Key"', download)
        self.assertIn(".redirect(reqwest::redirect::Policy::none())", SOURCE)

    def test_ui_login_is_not_returned_or_written_to_local_plaintext_config(self) -> None:
        settings_info = SOURCE[SOURCE.index("struct ArrSettingsInfo"):SOURCE.index("struct AgeSettingsInfo")]
        self.assertIn("has_ui_auth: bool", settings_info)
        self.assertNotIn("ui_username", settings_info)
        local_config = SECURE_CONFIG[
            SECURE_CONFIG.index("struct LocalRadarrConfig"):
            SECURE_CONFIG.index("let plaintext", SECURE_CONFIG.index("struct LocalRadarrConfig"))
        ]
        self.assertNotIn("ui_username", local_config)
        self.assertNotIn("ui_password", local_config)
        self.assertIn("radarr_ui_username", SECURE_CONFIG)
        self.assertIn("radarr_ui_password", SECURE_CONFIG)

    def test_url_validation_requires_https_except_loopback_and_rejects_url_credentials(self) -> None:
        self.assertEqual(
            restore_radarr.validate_radarr_url("https://radarr.example/radarr").hostname,
            "radarr.example",
        )
        self.assertEqual(
            restore_radarr.validate_radarr_url("http://127.0.0.1:7878").hostname,
            "127.0.0.1",
        )
        for url in (
            "http://radarr.example:7878",
            "https://user:secret@radarr.example",
            "https://radarr.example?token=secret",
            "ftp://radarr.example",
        ):
            with self.subTest(url=url), self.assertRaises(restore_radarr.RecoveryError):
                restore_radarr.validate_radarr_url(url)

    def test_archive_validation_accepts_native_zip_and_rejects_unsafe_entries(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            archive = Path(temporary) / "backup.zip"
            archive.write_bytes(make_zip({"radarr.db": b"fixture"}))
            restore_radarr.validate_archive(archive)

            archive.write_bytes(make_zip({"../escape": b"fixture"}))
            with self.assertRaisesRegex(restore_radarr.RecoveryError, "unsafe path"):
                restore_radarr.validate_archive(archive)

            stream = io.BytesIO()
            with zipfile.ZipFile(stream, "w") as zip_file:
                symlink = zipfile.ZipInfo("linked")
                symlink.create_system = 3
                symlink.external_attr = (stat.S_IFLNK | 0o777) << 16
                zip_file.writestr(symlink, "outside")
            archive.write_bytes(stream.getvalue())
            with self.assertRaisesRegex(restore_radarr.RecoveryError, "unsafe path"):
                restore_radarr.validate_archive(archive)

    def test_decryption_uses_identity_path_and_leaves_private_temporary_archive(self) -> None:
        fixture = make_zip({"radarr.db": b"fixture"})
        with tempfile.TemporaryDirectory() as temporary:
            folder = Path(temporary)
            fake_bin = folder / "bin"
            fake_bin.mkdir()
            encrypted = folder / "recovery.zip.aged"
            encrypted.write_bytes(b"age fixture")
            identity = folder / "age-identity"
            identity.write_text("AGE-SECRET-FIXTURE")
            age = fake_bin / "age"
            age.write_text(
                f"#!{sys.executable}\n"
                "import base64, sys\n"
                f"sys.stdout.buffer.write(base64.b64decode({base64.b64encode(fixture).decode('ascii')!r}))\n"
            )
            age.chmod(0o700)
            destination = folder / "decrypted.zip"
            with patch.dict(os.environ, {"PATH": f"{fake_bin}{os.pathsep}{os.environ['PATH']}"}):
                restore_radarr.decrypt_backup(encrypted, identity, destination)
            self.assertEqual(destination.read_bytes(), fixture)
            self.assertEqual(stat.S_IMODE(destination.stat().st_mode), 0o600)

    def test_upload_targets_the_radarr_restore_api_without_logging_credentials(self) -> None:
        received = {}
        fixture = make_zip({"radarr.db": b"fixture"})

        class Handler(http.server.BaseHTTPRequestHandler):
            def do_GET(self):
                received["status_path"] = self.path
                received["status_key"] = self.headers.get("X-Api-Key")
                payload = json.dumps({"appName": "Radarr"}).encode()
                self.send_response(200)
                self.send_header("Content-Length", str(len(payload)))
                self.end_headers()
                self.wfile.write(payload)

            def do_POST(self):
                received["restore_path"] = self.path
                received["restore_key"] = self.headers.get("X-Api-Key")
                body = self.rfile.read(int(self.headers["Content-Length"]))
                received["body"] = body
                self.send_response(200)
                self.send_header("Content-Length", "0")
                self.end_headers()

            def log_message(self, _format, *_args):
                return

        server = http.server.HTTPServer(("127.0.0.1", 0), Handler)
        thread = threading.Thread(target=server.serve_forever, daemon=True)
        thread.start()
        try:
            with tempfile.TemporaryDirectory() as temporary:
                archive = Path(temporary) / "recovery.zip"
                archive.write_bytes(fixture)
                parsed = restore_radarr.validate_radarr_url(
                    f"http://127.0.0.1:{server.server_port}/radarr"
                )
                restore_radarr.check_radarr(parsed, "fixture-api-key")
                restore_radarr.restore_to_radarr(parsed, archive, "fixture-api-key")
        finally:
            server.shutdown()
            thread.join(timeout=2)
            server.server_close()

        self.assertEqual(received["status_path"], "/radarr/api/v3/system/status")
        self.assertEqual(received["restore_path"], "/radarr/api/v3/system/backup/restore/upload")
        self.assertEqual(received["status_key"], "fixture-api-key")
        self.assertEqual(received["restore_key"], "fixture-api-key")
        self.assertIn(b'name="restore"; filename="radarr-recovery.zip"', received["body"])
        self.assertIn(fixture, received["body"])


if __name__ == "__main__":
    unittest.main()
