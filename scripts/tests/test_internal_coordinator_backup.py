import importlib.util
import hashlib
import tempfile
import unittest
from pathlib import Path
from subprocess import CompletedProcess
from unittest.mock import patch


SCRIPT = Path(__file__).parents[1] / "backup_internal_coordinator_do3.py"
SPEC = importlib.util.spec_from_file_location("internal_coordinator_backup", SCRIPT)
backup = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(backup)


class InternalCoordinatorBackupTests(unittest.TestCase):
    def test_loads_only_a_restricted_post_quantum_recipient(self):
        with tempfile.TemporaryDirectory() as temp:
            recipient_file = Path(temp) / "recipient"
            recipient = "age1pq1" + "a" * 32
            recipient_file.write_text(recipient + "\n")
            recipient_file.chmod(0o600)
            value, digest = backup.load_recipient(recipient_file)
            self.assertEqual(value, recipient)
            self.assertEqual(digest, hashlib.sha256(recipient.encode()).hexdigest())
            recipient_file.write_text("AGE-SECRET-KEY-PQ-1private\n")
            with self.assertRaisesRegex(RuntimeError, "post-quantum age recipient"):
                backup.load_recipient(recipient_file)

    def test_encrypts_a_tar_archive_to_age_file_without_leaving_plaintext_tar(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            source = root / "source"
            source.mkdir()
            (source / "payload").write_text("sensitive")
            backup_root = root / "backups"
            backup_root.mkdir(mode=0o700)
            staging = root / "tmpfs"
            staging.mkdir()

            def fake_run(args, *, stdout=None, stdin=None, check=True):
                self.assertIn("age", args)
                self.assertEqual(args[-1], "age1pq1" + "a" * 32)
                self.assertTrue(stdin.read().startswith(b"\x1f\x8b"))
                stdout.write(backup.AGE_HEADER + b"test-ciphertext")
                return CompletedProcess(args, 0, None, b"")

            with patch.object(backup, "STAGING_ROOT", staging), patch.object(backup, "run", side_effect=fake_run):
                output = backup.encrypt_directory(source, backup_root, "backup-id", "age1pq1" + "a" * 32)
            self.assertEqual(output.name, "backup-id.tar.gz.age")
            self.assertEqual(output.read_bytes(), backup.AGE_HEADER + b"test-ciphertext")
            self.assertEqual(output.stat().st_mode & 0o777, 0o600)
            self.assertEqual(list(staging.iterdir()), [])

    def test_recipient_preflight_runs_through_the_coordinator_age_binary(self):
        with tempfile.TemporaryDirectory() as temp:
            def fake_run(args, *, stdout=None, stdin=None, check=True):
                if "--version" in args:
                    return CompletedProcess(args, 0, "v1.3.2\n", "")
                self.assertEqual(stdin.read(), b"Rawrarr backup recipient preflight\n")
                stdout.write(backup.AGE_HEADER + b"preflight")
                return CompletedProcess(args, 0, None, b"")

            with patch.object(backup, "STAGING_ROOT", Path(temp)), patch.object(backup, "run", side_effect=fake_run):
                version = backup.validate_age_recipient("age1pq1" + "a" * 32)
            self.assertEqual(version, "v1.3.2")

    def test_container_metadata_requests_only_nonsecret_fields(self):
        with patch.object(
            backup,
            "run",
            return_value=CompletedProcess([], 0, "/container|sha256:id|rawrarr:tag|abc123|running|healthy\n", ""),
        ) as mocked:
            metadata = backup.container_metadata("service")
        self.assertEqual(metadata["revision"], "abc123")
        self.assertEqual(metadata["health"], "healthy")
        self.assertNotIn(".Config.Env", mocked.call_args.args[0][3])

    def test_copies_secret_files_with_restrictive_permissions(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp) / "source"
            root.mkdir()
            secrets = root / "secrets"
            secrets.mkdir()
            (secrets / "postgres-password").write_text("private-value")
            destination = Path(temp) / "backup"
            destination.mkdir()
            external = Path(temp) / "ui-token"
            external.write_text("ui-token-value")
            with patch.object(backup, "ROOT", root), patch.object(backup, "EXTERNAL_SECRET_FILES", {"ui-github-token": external}):
                names = backup.copy_protected_secrets(destination)
            self.assertEqual(names, ["postgres-password", "ui-github-token"])
            self.assertEqual((destination / "secrets/postgres-password").read_text(), "private-value")
            self.assertEqual((destination / "secrets/postgres-password").stat().st_mode & 0o777, 0o600)
            self.assertEqual((destination / "secrets/ui-github-token").read_text(), "ui-token-value")
            self.assertEqual((destination / "secrets/ui-github-token").stat().st_mode & 0o777, 0o600)

    def test_refuses_symlinks_in_secret_directory(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp) / "source"
            (root / "secrets").mkdir(parents=True)
            (root / "secrets/credential").symlink_to("/etc/passwd")
            destination = Path(temp) / "backup"
            destination.mkdir()
            external = Path(temp) / "ui-token"
            external.write_text("ui-token-value")
            with patch.object(backup, "ROOT", root), patch.object(backup, "EXTERNAL_SECRET_FILES", {"ui-github-token": external}):
                with self.assertRaisesRegex(RuntimeError, "non-regular secret entry"):
                    backup.copy_protected_secrets(destination)

    def test_archives_volume_and_rejects_unsafe_tar_paths(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp) / "volume"
            root.mkdir()
            (root / "state").write_text("state")
            destination = Path(temp) / "backup"
            destination.mkdir()
            with patch.object(backup, "volume_mountpoint", return_value=root):
                archive_path = destination / "state.tar.gz"
                self.assertGreater(backup.archive_volume(destination, "state.tar.gz", "test-volume"), 0)
                backup.validate_archive(archive_path)
            import io
            unsafe = destination / "unsafe.tar.gz"
            with backup.tarfile.open(unsafe, "w:gz") as archive:
                info = backup.tarfile.TarInfo("../outside")
                info.size = 0
                archive.addfile(info, io.BytesIO(b""))
            with self.assertRaisesRegex(RuntimeError, "unsafe archive path"):
                backup.validate_archive(unsafe)

    def test_checksums_cover_nested_backup_artifacts(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            nested = root / "secrets"
            nested.mkdir()
            (nested / "key").write_bytes(b"payload")
            backup.write_checksums(root)
            line = (root / "SHA256SUMS").read_text().strip()
            expected = backup.sha256(nested / "key")
            self.assertEqual(line, f"{expected}  secrets/key")

    def test_existing_backup_must_match_its_checksum_inventory(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            (root / "manifest.json").write_text("{}")
            backup.write_checksums(root)
            backup.verify_plain_backup(root)
            (root / "unlisted-secret").write_text("unexpected")
            with self.assertRaisesRegex(RuntimeError, "file set does not match"):
                backup.verify_plain_backup(root)


if __name__ == "__main__":
    unittest.main()
