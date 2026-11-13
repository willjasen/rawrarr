"""Check private-relay proof uses only disposable namespace packet filters."""

import json
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

    def test_restart_filters_reuse_addresses_without_offline_dns(self):
        peer = Mock(name="peer")
        peer.name = "rawrarr-test-rayfish-relay-example-fossil"
        destinations = [("172.30.0.2", 3340), ("172.30.0.3", 18090)]
        with patch.object(fixture, "docker") as docker:
            self.assertEqual(fixture.restrict_peer_egress(peer, destinations), destinations)
        self.assertTrue(all(call.args[0] == "run" for call in docker.call_args_list))
        for address, port in destinations:
            self.assertTrue(any(address in call.args and str(port) in call.args
                                for call in docker.call_args_list))

    def test_replacement_flushes_only_owned_namespace_output_chains(self):
        peer = Mock()
        peer.name = "rawrarr-test-rayfish-relay-example-fossil"
        with patch.object(fixture, "docker") as docker:
            fixture.restrict_peer_egress(peer, [("172.30.0.2", 3340)], replace=True)
        flushes = [call.args for call in docker.call_args_list if "-F" in call.args]
        self.assertEqual(len(flushes), 2)
        for command in flushes:
            self.assertEqual(command[command.index("--network") + 1], "container:" + peer.name)
            self.assertEqual(command[-2:], ("-F", "OUTPUT"))
        self.assertTrue(any("iptables" in command for command in flushes))
        self.assertTrue(any("ip6tables" in command for command in flushes))

    def test_direct_probe_uses_current_address_and_live_listener(self):
        first, second = Mock(name="first"), Mock(name="second")
        first.name, second.name = "peer-a", "peer-b"
        second.request.return_value = [200, "hash", 0]
        target = [{"NetworkSettings": {"Networks": {"test": {"IPAddress": "172.30.0.9"}}}}]
        with patch.object(fixture, "docker", side_effect=[json.dumps(target), "111"]) as docker:
            fixture.verify_direct_isolation(first, second, "test")
        second.request.assert_called_once_with(18092, "/health")
        self.assertIn("172.30.0.9", docker.call_args.args[-1])
        with patch.object(fixture, "docker", side_effect=[json.dumps(target), "0"]):
            with self.assertRaises(ValueError):
                fixture.verify_direct_isolation(first, second, "test")


if __name__ == "__main__":
    unittest.main()
