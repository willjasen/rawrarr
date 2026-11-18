"""Check fresh isolated volume selection and public-only certificate seeding."""

import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

import test_orbstack_arr_connectivity as environment


class SignedFixtureTests(unittest.TestCase):
    def test_fresh_override_preserves_all_previous_volume_names(self):
        original = ["compose", "-p", environment.PROJECT, "-f", environment.COMPOSE]
        definition = {"volumes": {"database": {"name": "old-database"}, "node": {"name": "old-node"}}}
        fixture = {"publicKey": "public-only", "fingerprint": "a" * 64, "nodes": []}
        with tempfile.TemporaryDirectory() as directory:
            with patch.object(environment, "TEST_STATE", Path(directory)), \
                 patch.object(environment, "docker", side_effect=[json.dumps(definition), json.dumps(fixture)]) as docker:
                compose, actual = environment.fresh_fixture(original)
                self.assertEqual(actual, fixture)
                self.assertEqual(compose[:-2], original)
                path = Path(compose[-1])
                self.assertEqual(path.parent.stat().st_mode & 0o777, 0o700)
                overrides = json.loads(path.read_text())["volumes"]
                self.assertEqual(set(overrides), set(definition["volumes"]))
                self.assertTrue(all(value["name"].startswith("signed-fixture-v1-") for value in overrides.values()))
                self.assertNotIn("services", json.loads(path.read_text()))
                self.assertIn("--network", docker.call_args.args)
                self.assertIn("none", docker.call_args.args)
                self.assertFalse(any("rm" == arg for arg in docker.call_args.args))

    def test_database_receives_public_certificate_not_seed(self):
        fixture = {"publicKey": "cHVibGlj", "fingerprint": "a" * 64, "nodes": [{
            "nodeId": "test-metadata-a", "seed": "private-fixture-marker",
            "publicKeyHex": "ab", "fingerprint": "b" * 64,
            "payload": {"domain": "rawrarr-device-approval-v1"}, "signature": "c2ln",
        }]}
        with patch.object(environment, "docker") as docker:
            self.assertEqual(environment.ensure_test_signing_root(fixture), "a" * 64)
            sql = docker.call_args.kwargs["input"]
            self.assertTrue(sql.startswith("BEGIN;"))
            self.assertTrue(sql.endswith("COMMIT;\n"))
            self.assertIn("signing_approval_signature", sql)
            self.assertNotIn("private-fixture-marker", sql)
            self.assertIn("ON_ERROR_STOP=1", docker.call_args.args)


if __name__ == "__main__":
    unittest.main()
