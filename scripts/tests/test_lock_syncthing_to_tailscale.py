#!/usr/bin/env python3
"""Checks that tailnet lockdown scopes exactly one approved peer."""

from pathlib import Path
import sys
import unittest
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import lock_syncthing_to_tailscale as tailnet


class TailnetLockTests(unittest.TestCase):
    def test_inspect_requires_exact_pair(self):
        def fake_api(_base, _key, path):
            if path == "rest/system/status":
                return {"myID": "LOCAL"}
            if path == "rest/config/devices":
                return [{"deviceID": "LOCAL"}, {"deviceID": "PEER", "addresses": ["dynamic"]}]
            return {"relaysEnabled": True}
        with patch.object(tailnet, "api", side_effect=fake_api):
            options, peer = tailnet.inspect("base", "key", "LOCAL", "PEER")
            self.assertTrue(options["relaysEnabled"])
            self.assertEqual(peer["addresses"], ["dynamic"])
            with self.assertRaises(ValueError):
                tailnet.inspect("base", "key", "WRONG", "PEER")

    def test_inspect_refuses_unrelated_peer(self):
        def fake_api(_base, _key, path):
            if path == "rest/system/status":
                return {"myID": "LOCAL"}
            if path == "rest/config/devices":
                return [{"deviceID": "LOCAL"}, {"deviceID": "PEER"}, {"deviceID": "OTHER"}]
            return {}
        with patch.object(tailnet, "api", side_effect=fake_api):
            with self.assertRaisesRegex(ValueError, "exactly"):
                tailnet.inspect("base", "key", "LOCAL", "PEER")


if __name__ == "__main__":
    unittest.main()
