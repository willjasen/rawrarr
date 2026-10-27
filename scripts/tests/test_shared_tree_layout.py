#!/usr/bin/env python3
"""CLI safety and copy behavior for the /shared/in and /shared/out tools."""

from pathlib import Path
import subprocess
import sys
import tempfile
import unittest


ROOT = Path(__file__).resolve().parents[2]
MIGRATE = ROOT / "scripts" / "migrate_shared_metadata.py"
PUBLISH = ROOT / "scripts" / "publish_shared_output.py"


def run_script(script: Path, *args):
    return subprocess.run(
        [sys.executable, str(script), *map(str, args)],
        capture_output=True,
        text=True,
        check=False,
    )


class SharedTreeLayoutTests(unittest.TestCase):
    def test_migration_dry_run_preserves_source_and_apply_copies_missing_files(self):
        with tempfile.TemporaryDirectory() as temporary:
            shared = Path(temporary) / "shared"
            source = shared / "metadata" / "movies" / "text"
            source.mkdir(parents=True)
            legacy_file = source / "record.aged"
            legacy_file.write_bytes(b"age ciphertext")

            dry_run = run_script(MIGRATE, "--shared-dir", shared)
            self.assertEqual(dry_run.returncode, 0, dry_run.stderr)
            self.assertFalse((shared / "in" / "metadata").exists())

            applied = run_script(MIGRATE, "--shared-dir", shared, "--apply")
            self.assertEqual(applied.returncode, 0, applied.stderr)
            target = shared / "in" / "metadata" / "movies" / "text" / "record.aged"
            self.assertEqual(target.read_bytes(), b"age ciphertext")
            self.assertEqual(legacy_file.read_bytes(), b"age ciphertext")

            repeated = run_script(MIGRATE, "--shared-dir", shared, "--apply")
            self.assertEqual(repeated.returncode, 0, repeated.stderr)

    def test_migration_conflict_aborts_without_overwriting_or_copying_other_files(self):
        with tempfile.TemporaryDirectory() as temporary:
            shared = Path(temporary) / "shared"
            legacy = shared / "metadata" / "movies" / "text"
            target = shared / "in" / "metadata" / "movies" / "text"
            legacy.mkdir(parents=True)
            target.mkdir(parents=True)
            (legacy / "conflict.aged").write_bytes(b"legacy")
            (legacy / "new.aged").write_bytes(b"new")
            (target / "conflict.aged").write_bytes(b"target")

            result = run_script(MIGRATE, "--shared-dir", shared, "--apply")
            self.assertEqual(result.returncode, 1)
            self.assertEqual((target / "conflict.aged").read_bytes(), b"target")
            self.assertFalse((target / "new.aged").exists())

    def test_publish_dry_run_then_apply_copies_selected_tree(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            shared = root / "shared"
            shared.mkdir()
            source = root / "media" / "Selected Collection"
            (source / "Disc 1").mkdir(parents=True)
            (source / "Disc 1" / "track.bin").write_bytes(b"large file fixture")
            args = ("--source", source, "--shared-dir", shared, "--name", "selected-collection")

            dry_run = run_script(PUBLISH, *args)
            self.assertEqual(dry_run.returncode, 0, dry_run.stderr)
            self.assertFalse((shared / "out").exists())

            applied = run_script(PUBLISH, *args, "--apply")
            self.assertEqual(applied.returncode, 0, applied.stderr)
            destination = shared / "out" / "selected-collection" / "Disc 1" / "track.bin"
            self.assertEqual(destination.read_bytes(), b"large file fixture")

    def test_publish_conflict_aborts_before_copying_remaining_files(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            shared = root / "shared"
            source = root / "media"
            source.mkdir()
            (source / "conflict.bin").write_bytes(b"source")
            (source / "new.bin").write_bytes(b"new")
            destination = shared / "out" / "collection"
            destination.mkdir(parents=True)
            (destination / "conflict.bin").write_bytes(b"existing")

            result = run_script(
                PUBLISH,
                "--source", source,
                "--shared-dir", shared,
                "--name", "collection",
                "--apply",
            )
            self.assertEqual(result.returncode, 1)
            self.assertEqual((destination / "conflict.bin").read_bytes(), b"existing")
            self.assertFalse((destination / "new.bin").exists())

    def test_publish_rejects_source_inside_shared_tree_and_symlinks(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            shared = root / "shared"
            inside = shared / "out" / "already"
            inside.mkdir(parents=True)
            result = run_script(PUBLISH, "--source", inside, "--shared-dir", shared, "--name", "nested")
            self.assertEqual(result.returncode, 1)

            source = root / "media"
            source.mkdir()
            (source / "link").symlink_to(inside, target_is_directory=True)
            result = run_script(PUBLISH, "--source", source, "--shared-dir", shared, "--name", "linked")
            self.assertEqual(result.returncode, 1)


if __name__ == "__main__":
    unittest.main()
