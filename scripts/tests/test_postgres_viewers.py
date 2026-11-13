import json
import os
from pathlib import Path
import subprocess
import time
import unittest
import uuid


ROOT = Path(__file__).resolve().parents[2]
COMPOSE = ROOT / "deploy/do3/docker-compose.postgres-viewers.yml"
MIGRATION = ROOT / "deploy/do3/postgres-viewer-v1.sql"


def command(*args, input=None):
    return subprocess.run(args, input=input, text=True, capture_output=True, check=True)


class ViewerComposeTests(unittest.TestCase):
    def test_isolated_loopback_viewers_without_secrets_or_app_services(self):
        config = json.loads(command(
            "docker", "compose", "-p", "rawrarr-postgres-viewers",
            "-f", str(COMPOSE), "config", "--format", "json",
        ).stdout)
        self.assertEqual(set(config["services"]), {"public-viewer", "internal-viewer"})
        for service, container, network, port in (
            ("public-viewer", "rawrarr-prod-postgres", "public-database", "7300"),
            ("internal-viewer", "rawrarr-int-postgres", "internal-database", "7302"),
        ):
            viewer = config["services"][service]
            self.assertEqual(viewer["container_name"], container + "-viewer")
            self.assertEqual(viewer["image"], "adminer:5.4.1-standalone")
            self.assertEqual(viewer["environment"], {"ADMINER_DEFAULT_SERVER": container})
            self.assertEqual(set(viewer["networks"]), {network})
            self.assertEqual(viewer["ports"][0]["host_ip"], "127.0.0.1")
            self.assertEqual(str(viewer["ports"][0]["published"]), port)
            self.assertEqual(viewer["ports"][0]["target"], 8080)
            self.assertEqual(len(viewer["ports"]), 1)
            for key in ("volumes", "secrets", "depends_on", "privileged", "network_mode"):
                self.assertNotIn(key, viewer)
        self.assertEqual(config["networks"]["public-database"]["name"], "rawrarr-roarlay-prod-control")
        self.assertEqual(config["networks"]["internal-database"]["name"], "rawrarr-internal-do3-control")
        self.assertTrue(all(network["external"] for network in config["networks"].values()))


@unittest.skipUnless(os.environ.get("RAWARR_TEST_POSTGRES_VIEWER") == "1",
                     "Set RAWARR_TEST_POSTGRES_VIEWER=1 for disposable PostgreSQL integration")
class ViewerDatabaseTests(unittest.TestCase):
    def setUp(self):
        self.container = "rawrarr-test-postgres-viewer-" + uuid.uuid4().hex[:12]
        command("docker", "image", "inspect", "postgres:17-alpine")
        command(
            "docker", "run", "-d", "--pull", "never", "--name", self.container,
            "--network", "none", "--tmpfs", "/var/lib/postgresql/data",
            "-e", "POSTGRES_USER=rawrarr", "-e", "POSTGRES_DB=rawrarr",
            "-e", "POSTGRES_HOST_AUTH_METHOD=trust", "postgres:17-alpine",
        )
        self.addCleanup(command, "docker", "rm", "-f", self.container)
        for _ in range(60):
            ready = subprocess.run(
                ["docker", "exec", self.container, "pg_isready", "-U", "rawrarr", "-d", "rawrarr"],
                capture_output=True, text=True,
            )
            if ready.returncode == 0:
                break
            time.sleep(1)
        else:
            self.fail("Disposable PostgreSQL did not become ready")
        self.sql("CREATE TABLE before_migration (id integer); INSERT INTO before_migration VALUES (1);")

    def sql(self, sql, user="rawrarr", success=True):
        result = subprocess.run(
            ["docker", "exec", "-i", self.container, "psql", "-X",
             "-v", "ON_ERROR_STOP=1", "-U", user, "-d", "rawrarr", "-At"],
            input=sql, capture_output=True, text=True,
        )
        if success:
            self.assertEqual(result.returncode, 0, result.stderr)
        else:
            self.assertNotEqual(result.returncode, 0, result.stdout)
        return result

    def test_repeatable_grants_reads_and_real_write_denial(self):
        for _ in range(2):
            self.sql(MIGRATION.read_text())
        self.sql("CREATE TABLE after_migration (id integer); INSERT INTO after_migration VALUES (2);")
        for table, value in (("before_migration", "1"), ("after_migration", "2")):
            self.assertEqual(self.sql(f"SELECT * FROM {table};", "rawrarr_viewer").stdout.strip(), value)
        self.assertEqual(
            self.sql("SHOW default_transaction_read_only;", "rawrarr_viewer").stdout.strip(), "on",
        )
        for sql in (
            "INSERT INTO before_migration VALUES (3)",
            "UPDATE before_migration SET id = 3",
            "DELETE FROM before_migration",
            "TRUNCATE before_migration",
            "ALTER TABLE before_migration ADD COLUMN other integer",
            "DROP TABLE before_migration",
            "CREATE TABLE forbidden (id integer)",
            "CREATE ROLE forbidden",
            "SET ROLE rawrarr",
        ):
            # Disable the session default: ACLs, not a UI/session flag, must deny writes.
            result = self.sql("SET default_transaction_read_only = off;\n" + sql + ";",
                              "rawrarr_viewer", success=False)
            self.assertRegex(result.stderr, r"permission denied|must be owner")
        self.assertEqual(self.sql("SELECT * FROM before_migration;").stdout.strip(), "1")

    def test_unmanaged_or_privileged_existing_role_is_rejected(self):
        self.sql("CREATE ROLE rawrarr_viewer LOGIN;")
        self.assertIn("unexpected privileges", self.sql(MIGRATION.read_text(), success=False).stderr)
        self.sql("DROP ROLE rawrarr_viewer;")
        self.sql(MIGRATION.read_text())
        self.sql("GRANT rawrarr TO rawrarr_viewer;")
        self.assertIn("unexpected privileges", self.sql(MIGRATION.read_text(), success=False).stderr)

    def test_security_definer_escape_is_rejected(self):
        self.sql("""
            CREATE FUNCTION viewer_escape() RETURNS void LANGUAGE sql SECURITY DEFINER
            AS 'INSERT INTO before_migration VALUES (9)';
        """)
        self.assertIn("SECURITY DEFINER", self.sql(MIGRATION.read_text(), success=False).stderr)
        self.assertEqual(self.sql("SELECT count(*) FROM pg_roles WHERE rolname='rawrarr_viewer';").stdout.strip(), "0")


if __name__ == "__main__":
    unittest.main()
