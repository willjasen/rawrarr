#!/usr/bin/env python3
"""Focused checks for safe album filter updates."""

import importlib.util
from pathlib import Path
import unittest


SCRIPT = Path(__file__).resolve().parents[1] / "add_music_sync_albums.py"
SPEC = importlib.util.spec_from_file_location("add_music_sync_albums", SCRIPT)
MODULE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(MODULE)


class AlbumPatternsTests(unittest.TestCase):
    def test_exact_album_does_not_open_an_entire_artist(self):
        updated, additions = MODULE.updated_lines("!/[Aa]*\n!/[Aa]*/**\n*\n", ["DU Records/Noise Tent 2K Spring Sampler"])
        self.assertEqual(additions, ["!/DU Records/Noise Tent 2K Spring Sampler"])
        self.assertEqual(updated[-1], "*")
        self.assertNotIn("!/DU Records", updated)

    def test_repeated_album_is_idempotent(self):
        first, _ = MODULE.updated_lines("*\n", ["Artist/Album"])
        second, additions = MODULE.updated_lines("\n".join(first) + "\n", ["Artist/Album"])
        self.assertEqual(first, second)
        self.assertEqual(additions, [])

    def test_rejects_globs_and_traversal(self):
        for album in ("../Album", "Artist/*", "Artist/Album/Disc", "Artist/.."):
            with self.subTest(album=album), self.assertRaises(ValueError):
                MODULE.album_patterns(album)


if __name__ == "__main__":
    unittest.main()
