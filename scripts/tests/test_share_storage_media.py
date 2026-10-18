#!/usr/bin/env python3
"""Checks for explicit storage-media Syncthing pairing."""

import importlib.util
from pathlib import Path
from types import SimpleNamespace
import unittest
from unittest.mock import patch

SCRIPT = Path(__file__).resolve().parents[1] / "share_storage_media.py"
spec = importlib.util.spec_from_file_location("share_storage_media", SCRIPT)
media = importlib.util.module_from_spec(spec)
spec.loader.exec_module(media)


class MediaShareTests(unittest.TestCase):
    def setUp(self):
        self.args = SimpleNamespace(local_id="SOURCE", peer_id="DESTINATION",
            folder_id="rawrarr-media-test", path="/media/ebooks", peer_name="MacBook",
            peer_address="dynamic", label="Media test", role="source")

    def test_rejects_unsafe_paths_and_remote_plain_http(self):
        for path in ("/media", "/shared/ebooks", "/media/../shared", "/media/ebooks/"):
            with self.assertRaises(ValueError):
                media.media_path(path)
        self.assertEqual(media.media_path("/media/ebooks"), "/media/ebooks")
        with self.assertRaises(ValueError):
            media.validate_url("http://example.test:8384")
        self.assertEqual(media.validate_url("http://127.0.0.1:8385"), "http://127.0.0.1:8385/")

    def test_rejects_existing_or_overlapping_folder(self):
        def fake_api(_base, _key, path, _body=None):
            if path == "rest/system/status":
                return {"myID": "SOURCE"}
            if path == "rest/config/folders":
                return [{"id": "another", "path": "/media"}]
            return []
        with patch.object(media, "api", side_effect=fake_api):
            with self.assertRaisesRegex(ValueError, "already managed"):
                media.inspect("http://127.0.0.1:8385/", "key", self.args)

    def test_reuses_peer_and_sets_direction_without_auto_accept(self):
        calls = []
        def fake_api(_base, _key, path, body=None):
            calls.append((path, body))
            if path == "rest/system/status":
                return {"myID": "SOURCE"}
            if path == "rest/config/folders":
                return [{"id": "shared", "path": "/shared"}]
            if path == "rest/config/devices":
                return [{"deviceID": "DESTINATION", "paused": False}]
            if path == "rest/config/defaults/folder":
                return {}
            if path == "rest/config/defaults/device":
                return {}
        with patch.object(media, "api", side_effect=fake_api):
            self.assertFalse(media.inspect("base", "key", self.args))
            media.configure("base", "key", self.args, False)
        self.assertFalse(any(path == "rest/config/devices" and body for path, body in calls))
        folder = next(body for path, body in calls if path == "rest/config/folders" and body)
        self.assertEqual(folder["type"], "sendonly")
        self.assertEqual(folder["devices"], [{"deviceID": "DESTINATION"}])
        self.args.role = "destination"
        calls.clear()
        with patch.object(media, "api", side_effect=fake_api):
            media.configure("base", "key", self.args, True)
        peer = next(body for path, body in calls if path == "rest/config/devices" and body)
        folder = next(body for path, body in calls if path == "rest/config/folders" and body)
        self.assertFalse(peer["autoAcceptFolders"])
        self.assertEqual(folder["type"], "receiveonly")


if __name__ == "__main__":
    unittest.main()
