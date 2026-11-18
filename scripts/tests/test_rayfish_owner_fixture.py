"""Verify the offline-revocation fixture inspects the intended network record."""

import hashlib
from pathlib import Path
import sys
import unittest
from unittest.mock import Mock, patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import rayfish_owner_offline_docker as fixture


class OwnerFixtureTests(unittest.TestCase):
    def test_network_record_hash_reads_the_zbase32_network_key(self):
        discovery = Mock()
        discovery.name = "discovery"
        path_hash = hashlib.sha256(("/pkarr/" + "y" * 52).encode()).hexdigest()
        with patch.object(fixture, "docker", return_value="packet-hash") as docker:
            self.assertEqual(fixture.network_record_hash(discovery, "00" * 32), "packet-hash")
        command = docker.call_args.args
        self.assertEqual(command[:4], ("exec", "discovery", "python", "-c"))
        self.assertIn(path_hash, command[4])

    def test_invalid_key_does_not_read_discovery(self):
        with patch.object(fixture, "docker") as docker, self.assertRaises(ValueError):
            fixture.network_record_hash(Mock(), "not-a-network-key")
        docker.assert_not_called()


if __name__ == "__main__":
    unittest.main()
