#!/usr/bin/env python3
"""Test the secret-safe local Radarr inspector without Docker or Radarr."""

from __future__ import annotations

import importlib.util
import io
import json
from pathlib import Path
import sys
import tarfile
import unittest
from unittest.mock import patch


ROOT = Path(__file__).resolve().parents[2]
sys.dont_write_bytecode = True
SPEC = importlib.util.spec_from_file_location("local_radarr", ROOT / "scripts" / "local_radarr.py")
assert SPEC and SPEC.loader
local_radarr = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(local_radarr)


class LocalRadarrTests(unittest.TestCase):
    def test_reads_config_from_docker_copy_tar_without_printing_it(self) -> None:
        config = {"radarr_url": "https://radarr.example", "radarr_api_key": "test-only-key"}
        payload = json.dumps(config).encode()
        archive_bytes = io.BytesIO()
        with tarfile.open(fileobj=archive_bytes, mode="w") as archive:
            member = tarfile.TarInfo("data/radarr-config.json")
            member.size = len(payload)
            archive.addfile(member, io.BytesIO(payload))

        with patch.object(
            local_radarr.subprocess,
            "run",
            return_value=type("Result", (), {"stdout": archive_bytes.getvalue()})(),
        ) as run:
            result = local_radarr.read_local_config("rawrarr-local-rawrarr-1")

        self.assertEqual(result, config)
        self.assertEqual(run.call_args.args[0][:3], ["docker", "cp", "rawrarr-local-rawrarr-1:/data/radarr-config.json"])

    def test_summarizes_movie_files_without_paths_or_credentials(self) -> None:
        result = local_radarr.summarize_movies(
            [
                {"hasFile": True, "monitored": True, "movieFile": {"size": 1024, "path": "/private/path"}},
                {"hasFile": False, "monitored": False},
            ]
        )

        self.assertEqual(
            result,
            {"movieCount": 2, "fileCount": 1, "missingCount": 1, "monitoredCount": 1, "totalSizeBytes": 1024},
        )
        self.assertNotIn("path", json.dumps(result))
        self.assertNotIn("api_key", json.dumps(result))


if __name__ == "__main__":
    unittest.main()
