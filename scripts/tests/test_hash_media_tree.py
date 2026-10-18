#!/usr/bin/env python3
"""Check stable media-tree hashing and symlink refusal."""

import importlib.util
from pathlib import Path
import tempfile
import unittest

SCRIPT = Path(__file__).resolve().parents[1] / "hash_media_tree.py"
spec = importlib.util.spec_from_file_location("hash_media_tree", SCRIPT)
media = importlib.util.module_from_spec(spec)
spec.loader.exec_module(media)


class HashMediaTreeTests(unittest.TestCase):
    def test_content_and_path_change_fingerprint(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            child = root / "album"
            child.mkdir()
            song = child / "track.flac"
            song.write_bytes(b"first")
            original = media.fingerprint(root)
            self.assertEqual((original["files"], original["bytes"]), (1, 5))
            song.write_bytes(b"other")
            self.assertNotEqual(original["sha256"], media.fingerprint(root)["sha256"])
            song.rename(child / "renamed.flac")
            self.assertNotEqual(original["sha256"], media.fingerprint(root)["sha256"])
            (root / "link").symlink_to(child)
            with self.assertRaises(ValueError):
                media.fingerprint(root)


if __name__ == "__main__":
    unittest.main()
