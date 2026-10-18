#!/usr/bin/env python3
"""Exercise Rawrarr passkey onboarding against a disposable PostgreSQL database."""

from __future__ import annotations

import argparse
import json
import os
from pathlib import Path
import re
import shutil
import socket
import subprocess
import sys
import tempfile
import time
import urllib.error
import urllib.request
from urllib.parse import urlsplit


ROOT = Path(__file__).resolve().parents[2]
DEFAULT_BINARY = ROOT / "rust-arr" / "target" / "release" / "rust-arr"


class NoRedirect(urllib.request.HTTPRedirectHandler):
    def redirect_request(self, request, response, code, message, headers, new_url):
        return None


def request(base_url: str, path: str, method: str = "GET", body: dict | None = None):
    payload = None if body is None else json.dumps(body).encode()
    req = urllib.request.Request(
        base_url + path,
        data=payload,
        method=method,
        headers={"Accept": "application/json", "Content-Type": "application/json"},
    )
    opener = urllib.request.build_opener(NoRedirect)
    try:
        response = opener.open(req, timeout=3)
    except urllib.error.HTTPError as error:
        return error.code, error.headers, error.read()
    with response:
        return response.status, response.headers, response.read()


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=Path, default=DEFAULT_BINARY)
    parser.add_argument(
        "--database-url",
        default=os.environ.get("RAWARR_TEST_DATABASE_URL"),
        help="URL for an empty, disposable PostgreSQL database whose name ends in _test",
    )
    args = parser.parse_args()
    compose = (ROOT / "docker-compose.yml").read_text()
    assert "RAWARR_WEBAUTHN_ORIGIN: ${RAWARR_WEBAUTHN_ORIGIN:-http://localhost:${RAWARR_HOST_PORT:-8080}}" in compose
    binary = args.binary.resolve()
    if not binary.is_file():
        parser.error(f"Rawrarr binary not found: {binary}; build with cargo build --release --locked")
    if not shutil.which("psql"):
        parser.error("required executable 'psql' is not on PATH")
    if not args.database_url:
        parser.error("provide --database-url or RAWARR_TEST_DATABASE_URL for a disposable PostgreSQL database")
    database_name = urlsplit(args.database_url).path.rsplit("/", 1)[-1].split("?", 1)[0]
    if not database_name.endswith("_test"):
        parser.error("refusing to run: database name must end in _test")

    with tempfile.TemporaryDirectory(prefix="rawrarr-passkey-test-") as temporary:
        with socket.socket() as listener:
            listener.bind(("127.0.0.1", 0))
            port = listener.getsockname()[1]
        base_url = f"http://localhost:{port}"
        env = os.environ.copy()
        env.update(
            RAWARR_DATABASE_URL=args.database_url,
            RAWARR_DATA_DIR=str(Path(temporary) / "data"),
            RAWARR_WEBAUTHN_RP_ID="localhost",
            RAWARR_WEBAUTHN_ORIGIN=base_url,
            RUST_ARR_LISTEN_ADDR=f"127.0.0.1:{port}",
        )
        env.pop("RAWARR_AGE_IDENTITY_FILE", None)
        process = subprocess.Popen(
            [str(binary)], cwd=ROOT, env=env, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL
        )
        challenge_id = None
        try:
            deadline = time.monotonic() + 15
            while time.monotonic() < deadline:
                if process.poll() is not None:
                    raise RuntimeError("Rawrarr exited before becoming ready")
                try:
                    status, _, _ = request(base_url, "/health")
                    if status == 200:
                        break
                except OSError:
                    time.sleep(0.15)
            else:
                raise RuntimeError("Rawrarr did not become ready")

            status, _, page = request(base_url, "/auth")
            assert status == 200 and b"Sign in with passkey" in page
            assert b'id="username"' not in page and b'id="new-username"' not in page
            assert b'id="display-name"' not in page
            assert b"status.userExists ? 'login-panel' : 'register-panel'" in page
            status, _, payload = request(base_url, "/api/auth/status")
            assert status == 200 and json.loads(payload) == {"userExists": False}
            status, headers, _ = request(base_url, "/")
            assert 300 <= status < 400 and headers.get("Location") == "/auth"
            status, _, _ = request(base_url, "/api/radarr/status")
            assert status == 401

            status, _, payload = request(
                base_url,
                "/api/auth/register/start",
                "POST",
                {},
            )
            assert status == 200
            challenge = json.loads(payload)
            challenge_id = challenge["challengeId"]
            assert re.fullmatch(r"[0-9a-f-]{36}", challenge_id)
            assert challenge["options"]["publicKey"]["rp"]["id"] == "localhost"
            assert re.fullmatch(
                r"[a-z]+-[a-z]+",
                challenge["options"]["publicKey"]["user"]["name"],
            )

            status, _, _ = request(
                base_url,
                "/api/auth/register/finish",
                "POST",
                {"challengeId": challenge_id, "credential": {}},
            )
            assert status in (401, 422)
            status, _, payload = request(base_url, "/api/auth/status")
            assert status == 200 and json.loads(payload) == {"userExists": False}
            print("PASS minimal sign-in page, first-user registration challenge, and protected route")
            print("PASS malformed registration is rejected before creating a user or session")
        finally:
            if process.poll() is None:
                process.terminate()
                try:
                    process.wait(timeout=5)
                except subprocess.TimeoutExpired:
                    process.kill()
                    process.wait(timeout=5)
            if challenge_id:
                # Challenge IDs are UUID-validated before interpolation; cleanup is confined to the _test database.
                subprocess.run(
                    ["psql", "-v", "ON_ERROR_STOP=1", "-c",
                     f"DELETE FROM rawrarr_auth_challenges WHERE id = '{challenge_id}'::uuid"],
                    env={**os.environ, "PGDATABASE": args.database_url},
                    stdout=subprocess.DEVNULL,
                    stderr=subprocess.DEVNULL,
                    check=True,
                )
    print("PASS temporary app data removed")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (AssertionError, OSError, RuntimeError, subprocess.CalledProcessError, urllib.error.URLError) as error:
        print(f"FAIL Rawrarr passkey integration: {type(error).__name__}", file=sys.stderr)
        raise SystemExit(1)
