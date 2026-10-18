#!/usr/bin/env python3
"""Read Radarr status and library totals using Rawrarr's node-local config mirror."""

from __future__ import annotations

import argparse
import json
import subprocess
import sys
import tarfile
import urllib.error
import urllib.request
from urllib.parse import urlsplit
from io import BytesIO


PROJECT_LABEL = "com.docker.compose.project=rawrarr-local"
SERVICE_LABEL = "com.docker.compose.service=rawrarr"
CONFIG_PATH = "/data/radarr-config.json"


class NoRedirect(urllib.request.HTTPRedirectHandler):
    def redirect_request(self, request, response, code, message, headers, new_url):
        return None


def find_container() -> str:
    result = subprocess.run(
        ["docker", "ps", "--filter", f"label={PROJECT_LABEL}", "--filter", f"label={SERVICE_LABEL}", "--format", "{{.Names}}"],
        check=True,
        capture_output=True,
        text=True,
    )
    containers = [name for name in result.stdout.splitlines() if name]
    if len(containers) != 1:
        raise RuntimeError("expected one running rawrarr-local web container")
    return containers[0]


def read_local_config(container: str) -> dict:
    result = subprocess.run(
        ["docker", "cp", f"{container}:{CONFIG_PATH}", "-"],
        check=True,
        capture_output=True,
    )
    with tarfile.open(fileobj=BytesIO(result.stdout), mode="r:") as archive:
        members = [member for member in archive.getmembers() if member.isfile()]
        if len(members) != 1 or members[0].size > 64 * 1024:
            raise RuntimeError("local Radarr config mirror is missing or invalid")
        stream = archive.extractfile(members[0])
        if stream is None:
            raise RuntimeError("could not read local Radarr config mirror")
        config = json.load(stream)
    if not isinstance(config, dict) or not isinstance(config.get("radarr_url"), str) or not isinstance(config.get("radarr_api_key"), str):
        raise RuntimeError("local Radarr config mirror is invalid")
    return config


def get_json(base_url: str, path: str, api_key: str):
    parsed = urlsplit(base_url)
    if (
        parsed.scheme not in {"http", "https"}
        or not parsed.hostname
        or parsed.username is not None
        or parsed.password is not None
        or parsed.query
        or parsed.fragment
    ):
        raise ValueError("invalid Radarr URL")
    request = urllib.request.Request(
        base_url.rstrip("/") + path,
        headers={"Accept": "application/json", "X-Api-Key": api_key},
    )
    opener = urllib.request.build_opener(NoRedirect)
    with opener.open(request, timeout=10) as response:
        return json.load(response)


def summarize_movies(movies: list[dict]) -> dict:
    return {
        "movieCount": len(movies),
        "fileCount": sum(bool(movie.get("hasFile")) for movie in movies),
        "missingCount": sum(not movie.get("hasFile", False) for movie in movies),
        "monitoredCount": sum(bool(movie.get("monitored")) for movie in movies),
        "totalSizeBytes": sum(
            int((movie.get("movieFile") or {}).get("size", 0) or 0) for movie in movies
        ),
    }


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--container", help="Rawrarr Docker container name; discovered automatically by default")
    args = parser.parse_args()
    try:
        container = args.container or find_container()
        config = read_local_config(container)
        status = get_json(config["radarr_url"].rstrip("/") + "/api/v3", "/system/status", config["radarr_api_key"])
        movies = get_json(config["radarr_url"].rstrip("/") + "/api/v3", "/movie", config["radarr_api_key"])
        result = {
            "appName": status.get("appName"),
            "version": status.get("version"),
            "urlBase": status.get("urlBase", ""),
            **summarize_movies(movies),
        }
        print(json.dumps(result, indent=2))
    except (OSError, ValueError, KeyError, tarfile.TarError, urllib.error.URLError, subprocess.CalledProcessError, RuntimeError) as error:
        if isinstance(error, urllib.error.HTTPError):
            print(f"Radarr returned HTTP {error.code}", file=sys.stderr)
        else:
            print(f"Could not inspect local Radarr configuration ({type(error).__name__})", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
