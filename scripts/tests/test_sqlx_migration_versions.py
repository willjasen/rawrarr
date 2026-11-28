#!/usr/bin/env python3
"""Ensure embedded SQLx migrations have distinct, ordered version numbers."""

from pathlib import Path
import re
import unittest


ROOT = Path(__file__).resolve().parents[2]
MIGRATIONS = ROOT / "rust-arr" / "migrations"


class SqlxMigrationVersionTests(unittest.TestCase):
    def test_migration_versions_are_unique_and_increasing(self):
        versions = []
        for path in sorted(MIGRATIONS.glob("*.sql")):
            match = re.fullmatch(r"(\d+)_.*\.sql", path.name)
            self.assertIsNotNone(match, f"unexpected SQLx migration filename: {path.name}")
            versions.append((int(match.group(1)), path.name))

        self.assertTrue(versions, "expected at least one SQLx migration")
        ordered = sorted(versions)
        self.assertEqual(
            len({version for version, _name in ordered}), len(ordered),
            f"SQLx migration versions collide: {ordered}",
        )
        self.assertEqual(versions, ordered, "migration files should be in version order")

    def test_coordinator_can_start_with_newer_additive_migrations_recorded(self):
        coordinator = (ROOT / "rust-arr" / "src" / "coordinator.rs").read_text(encoding="utf-8")
        self.assertRegex(coordinator, r"migrator\.ignore_missing\s*=\s*true")


if __name__ == "__main__":
    unittest.main()
