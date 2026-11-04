import importlib.util
import hashlib
import os
import subprocess
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
    @unittest.skipUnless(os.environ.get("RAWARR_TEST_BACKUP_POSTGRES_CONTAINER"),
                         "requires an explicitly named disposable PostgreSQL container")
    def test_real_postgres_resolves_an_offline_fossil_pin(self):
        container = os.environ["RAWARR_TEST_BACKUP_POSTGRES_CONTAINER"]
        if not container.startswith("rawrarr-test-regression-owner-postgres-"):
            self.fail("refusing a container outside the owner regression-test namespace")
        import uuid
        schema = "backup_owner_" + uuid.uuid4().hex

        def query(sql):
            return subprocess.run(
                ["docker", "exec", container, "psql", "-X", "-qAt",
                 "--set", "ON_ERROR_STOP=1", "-U", "postgres", "-d", "rawrarr_owner_test",
                 "-c", sql], check=True, capture_output=True, text=True,
            )

        def isolated_run(args, **kwargs):
            self.assertEqual(args[:4], ["docker", "exec", backup.CONTAINERS["postgres"], "psql"])
            return query(f"SET search_path TO {schema}; " + args[-1])

        query(f"CREATE SCHEMA {schema}; CREATE TABLE {schema}.rawrarr_users(id UUID PRIMARY KEY);")
        try:
            migration = SCRIPT.parent.parent / "rust-arr/migrations/0012_environment_owner.sql"
            query(f"SET search_path TO {schema}; " + migration.read_text())
            with patch.object(backup, "run", side_effect=isolated_run):
                with self.assertRaisesRegex(RuntimeError, "provision one valid"):
                    backup.load_recipient()
                recipient = "age1pq1" + "a" * 32
                owner = str(uuid.uuid4())
                query(f"SET search_path TO {schema}; INSERT INTO rawrarr_users VALUES('{owner}'); "
                      "INSERT INTO rawrarr_environment_owner(singleton,owner_user_id,age_recipient,source_node_id) "
                      f"VALUES(TRUE,'{owner}','{recipient}','offline-and-removed-fossil');")
                self.assertEqual(backup.load_recipient(),
                                 (recipient, hashlib.sha256(recipient.encode()).hexdigest()))
        finally:
            query(f"DROP SCHEMA {schema} CASCADE;")

    def test_loads_only_the_pinned_internal_database_public_recipient(self):
        recipient = "age1pq1" + "a" * 32
        with patch.object(backup, "run", return_value=CompletedProcess([], 0, recipient + "\n", "")) as mocked:
            value, digest = backup.load_recipient()
            self.assertEqual(value, recipient)
            self.assertEqual(digest, hashlib.sha256(recipient.encode()).hexdigest())
        args = mocked.call_args.args[0]
        self.assertEqual(args[:4], ["docker", "exec", backup.CONTAINERS["postgres"], "psql"])
        self.assertIn("BEGIN READ ONLY;", args[-1])
        self.assertIn("rawrarr_environment_owner", args[-1])
        self.assertNotIn("signing", args[-1])

    def test_rejects_missing_private_group_marker_and_ambiguous_values(self):
        recipient = "age1pq1" + "a" * 32
        for value in ["", "AGE-SECRET-KEY-PQ-1private", "/shared/group-age-recipient",
                      recipient + "\n" + recipient, "age1" + "a" * 40]:
            with self.subTest(value=value), patch.object(
                backup, "run", return_value=CompletedProcess([], 0, value, "")
            ):
                with self.assertRaisesRegex(RuntimeError, "provision one valid"):
                    backup.load_recipient()

    def test_database_failure_has_no_recipient_fallback(self):
        from subprocess import CalledProcessError
        with patch.object(backup, "run", side_effect=CalledProcessError(1, ["psql"])):
            with self.assertRaises(CalledProcessError):
                backup.load_recipient()

    def test_both_backup_paths_stop_before_collection_without_a_pin(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            root.chmod(0o700)
            source = root / "internal-coordinator-test"
            source.mkdir(mode=0o700)
            with patch.object(backup, "BACKUP_ROOT", root), \
                 patch.object(backup.os, "geteuid", return_value=0), \
                 patch.object(backup, "require_tmpfs_staging"), \
                 patch.object(backup, "load_recipient", side_effect=RuntimeError("missing pin")) as resolver, \
                 patch.object(backup, "dump_database") as dump, \
                 patch.object(backup, "encrypt_directory") as encrypt:
                with self.assertRaisesRegex(RuntimeError, "missing pin"):
                    backup.create_backup(root)
                with self.assertRaisesRegex(RuntimeError, "missing pin"):
                    backup.encrypt_existing_backup(source, root)
                self.assertEqual(resolver.call_count, 2)
                dump.assert_not_called()
                encrypt.assert_not_called()
                self.assertEqual(list(root.iterdir()), [source])

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
