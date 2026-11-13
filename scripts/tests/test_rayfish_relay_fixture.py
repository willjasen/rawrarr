"""Check private-relay proof uses only disposable namespace packet filters."""

from pathlib import Path
import sys
import unittest
from unittest.mock import Mock, patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import rayfish_relay_docker as fixture


class RelayFixtureTests(unittest.TestCase):
    def test_filters_are_namespace_scoped_and_block_direct_paths(self):
        peer = Mock()
        peer.name = "rawrarr-test-rayfish-relay-example-fossil"
        with patch.object(fixture, "docker", return_value="172.30.0.2") as docker:
            fixture.restrict_peer_egress(peer)
        filter_commands = [call.args for call in docker.call_args_list if call.args[0] == "run"]
        self.assertEqual(len(filter_commands), 6)
        for command in filter_commands:
            self.assertEqual(command[command.index("--network") + 1], "container:" + peer.name)
            self.assertEqual(command[command.index("--cap-drop") + 1], "ALL")
            self.assertEqual(command[command.index("--cap-add") + 1], "NET_ADMIN")
            self.assertIn("--rm", command)
            self.assertIn("--read-only", command)
            self.assertNotIn("--privileged", command)
            self.assertNotIn("-v", command)
        self.assertEqual(sum(command[-1] == "REJECT" for command in filter_commands), 2)
        self.assertTrue(any("3340" in command for command in filter_commands))
        self.assertTrue(any("18090" in command for command in filter_commands))
        self.assertFalse(any("9090" in command for command in filter_commands))

    def test_byte_counter_is_required_not_silently_zero(self):
        with patch.object(fixture, "docker", return_value="# counter\nrelay_bytes_sent_total 12345\n"):
            self.assertEqual(fixture.relay_bytes("test-relay"), 12345)
        with patch.object(fixture, "docker", return_value="# no bytes\n"), self.assertRaises(ValueError):
            fixture.relay_bytes("test-relay")


if __name__ == "__main__":
    unittest.main()
