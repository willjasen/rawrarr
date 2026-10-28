import importlib.util
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


if __name__ == "__main__":
    unittest.main()
