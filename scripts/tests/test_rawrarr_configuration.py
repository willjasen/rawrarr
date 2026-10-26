#!/usr/bin/env python3
"""Exercise Rawrarr age-encrypted configuration using an isolated local server."""

from __future__ import annotations

import argparse
import base64
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
import hashlib
import json
import os
from pathlib import Path
import secrets
import shutil
import socket
import stat
import subprocess
import sys
import tempfile
import time
import urllib.error
import urllib.request
from urllib.parse import urlsplit
import uuid


ROOT = Path(__file__).resolve().parents[2]
DEFAULT_BINARY = ROOT / "rust-arr" / "target" / "release" / "rust-arr"
TEST_API_KEY = "rawrarr-configuration-test-key-do-not-log"
SESSION_COOKIE: str | None = None


class RadarrMockHandler(BaseHTTPRequestHandler):
    def do_GET(self) -> None:
        if self.path != "/api/v3/system/status" or self.headers.get("X-Api-Key") != TEST_API_KEY:
            self.send_error(401)
            return
        body = json.dumps({"appName": "Radarr", "version": "5.0.0", "urlBase": ""}).encode()
        self.send_response(200)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)

    def log_message(self, _format: str, *_args: object) -> None:
        return


def start_radarr_mock() -> tuple[ThreadingHTTPServer, str]:
    server = ThreadingHTTPServer(("127.0.0.1", 0), RadarrMockHandler)
    import threading

    threading.Thread(target=server.serve_forever, daemon=True).start()
    return server, f"http://127.0.0.1:{server.server_port}"


def request_json(base_url: str, path: str, method: str = "GET", body: dict | None = None) -> dict:
    payload = None if body is None else json.dumps(body).encode()
    request = urllib.request.Request(
        base_url + path,
        data=payload,
        method=method,
        headers={"Accept": "application/json", "Content-Type": "application/json"},
    )
    if SESSION_COOKIE:
        request.add_header("Cookie", SESSION_COOKIE)
    with urllib.request.urlopen(request, timeout=3) as response:
        return json.loads(response.read())


def request_bytes(base_url: str, path: str) -> tuple[bytes, str]:
    with urllib.request.urlopen(base_url + path, timeout=3) as response:
        return response.read(), response.headers.get_content_type()


def age_encrypt_to_recipient(recipient: str, plaintext: bytes) -> str:
    result = subprocess.run(
        ["age", "--recipient", recipient], input=plaintext, capture_output=True, check=True
    )
    return base64.b64encode(result.stdout).decode("ascii")


def start_server(
    binary: Path, data_dir: Path, shared_dir: Path, media_dir: Path, port: int, database_url: str,
    plaintext_config: bool = False,
) -> subprocess.Popen:
    env = os.environ.copy()
    env["RAWARR_DATA_DIR"] = str(data_dir)
    env["RAWARR_SHARED_DATA_DIR"] = str(shared_dir)
    env["RAWARR_LIBRARY_PATH"] = str(media_dir)
    env["RAWARR_LIBRARY_NAME"] = "Fixture library"
    env["RAWARR_DEVICE_ID"] = "storage-fixture"
    env["RAWARR_DATABASE_URL"] = database_url
    env["RUST_ARR_LISTEN_ADDR"] = f"127.0.0.1:{port}"
    env["RAWARR_WEBAUTHN_RP_ID"] = "localhost"
    env["RAWARR_WEBAUTHN_ORIGIN"] = f"http://localhost:{port}"
    env["RAWARR_LOCAL_PLAINTEXT_CONFIG"] = "true" if plaintext_config else "false"
    env.pop("RAWARR_AGE_IDENTITY_FILE", None)
    return subprocess.Popen(
        [str(binary)], cwd=ROOT, env=env, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL
    )


def wait_for_server(process: subprocess.Popen, base_url: str) -> None:
    deadline = time.monotonic() + 10
    while time.monotonic() < deadline:
        if process.poll() is not None:
            raise RuntimeError("Rawrarr exited before becoming ready")
        try:
            with urllib.request.urlopen(base_url + "/health", timeout=0.5) as response:
                if response.status == 200:
                    return
        except (OSError, urllib.error.URLError):
            time.sleep(0.1)
    raise RuntimeError("Rawrarr did not become ready within 10 seconds")


def stop_server(process: subprocess.Popen) -> None:
    if process.poll() is None:
        process.terminate()
        try:
            process.wait(timeout=5)
        except subprocess.TimeoutExpired:
            process.kill()
            process.wait(timeout=5)


def main() -> int:
    global SESSION_COOKIE
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=Path, default=DEFAULT_BINARY, help="Rawrarr executable to exercise")
    parser.add_argument(
        "--database-url",
        default=os.environ.get("RAWARR_TEST_DATABASE_URL"),
        help="URL for an empty disposable PostgreSQL database whose name ends in _test",
    )
    args = parser.parse_args()
    binary = args.binary.resolve()
    if not binary.is_file():
        parser.error(f"Rawrarr binary not found: {binary}; build it with cargo build --release --locked")
    if not args.database_url:
        parser.error("provide --database-url or RAWARR_TEST_DATABASE_URL for a disposable PostgreSQL database")
    database_name = urlsplit(args.database_url).path.rsplit("/", 1)[-1]
    if not database_name.endswith("_test"):
        parser.error("refusing to run: database name must end in _test")
    if not shutil.which("psql"):
        parser.error("required executable 'psql' is not on PATH")
    for tool in ("age", "age-keygen"):
        if not any((Path(directory) / tool).is_file() for directory in os.environ.get("PATH", "").split(os.pathsep)):
            parser.error(f"required executable '{tool}' is not on PATH")

    with tempfile.TemporaryDirectory(prefix="rawrarr-config-test-") as temporary:
        data_dir = Path(temporary) / "data"
        shared_dir = Path(temporary) / "syncthing-shared"
        media_dir = Path(temporary) / "media"
        (media_dir / "Example Film (2024)").mkdir(parents=True)
        (media_dir / "Example Film (2024)" / "Example Film (2024).mkv").write_bytes(b"fixture video")
        (media_dir / "Example Show" / "Season 01").mkdir(parents=True)
        (media_dir / "Example Show" / "Season 01" / "Example.Show.S01E01.mkv").write_bytes(b"fixture episode")
        with socket.socket() as listener:
            listener.bind(("127.0.0.1", 0))
            port = listener.getsockname()[1]
        base_url = f"http://127.0.0.1:{port}"
        process = start_server(
            binary, data_dir, shared_dir, media_dir, port, args.database_url, plaintext_config=True
        )
        radarr_server, radarr_url = start_radarr_mock()
        user_id = str(uuid.uuid4())
        session_token = secrets.token_urlsafe(32)
        session_hash = hashlib.sha256(session_token.encode()).hexdigest()
        seeded_session = False
        try:
            wait_for_server(process, base_url)
            seed_sql = (
                "INSERT INTO rawrarr_users (id, username, display_name) "
                f"VALUES ('{user_id}'::uuid, 'config_test_{user_id[:8]}', 'Config test'); "
                "INSERT INTO rawrarr_sessions (token_hash, user_id, expires_at) "
                f"VALUES (decode('{session_hash}', 'hex'), '{user_id}'::uuid, now() + interval '1 hour');"
            )
            result = subprocess.run(
                ["psql", "-v", "ON_ERROR_STOP=1", "-c", seed_sql],
                env={**os.environ, "PGDATABASE": args.database_url},
                stdout=subprocess.DEVNULL,
                stderr=subprocess.DEVNULL,
            )
            if result.returncode:
                raise RuntimeError("could not seed isolated passkey session in the _test database")
            seeded_session = True
            SESSION_COOKIE = f"rawrarr_session={session_token}"
            initial = request_json(base_url, "/api/configuration")
            assert initial["encryption"]["status"] == "not_configured"
            assert initial["configuration"] is None

            generated = request_json(base_url, "/api/settings/age/generate", "POST")
            identity = generated["identity"]
            assert identity.startswith("#") and "AGE-SECRET-KEY-PQ-1" in identity
            expected_theme_seed = hashlib.sha256(generated["recipient"].encode()).hexdigest()
            assert generated["themeSeed"] == expected_theme_seed
            node_recipient, _ = request_bytes(base_url, "/api/metadata/export/node-recipient")
            identity_envelope = age_encrypt_to_recipient(node_recipient.decode().strip(), identity.encode())
            unlock = request_json(base_url, "/api/settings/age", "POST", {"identityEnvelope": identity_envelope})
            assert unlock["unlocked"] is True
            assert unlock["themeSeed"] == expected_theme_seed
            age_status = request_json(base_url, "/api/settings/age")
            assert age_status["unlocked"] is True
            assert age_status["themeSeed"] == expected_theme_seed
            assert generated["recipient"] not in json.dumps(age_status)
            recipient_path = shared_dir / "age-recipient"
            group_recipient = recipient_path.read_text().strip()
            assert group_recipient.startswith("age1pq1")
            assert group_recipient != generated["recipient"]
            assert (shared_dir / "group-age-recipient").read_text().strip() == group_recipient
            assert (shared_dir / "group-age-user-envelope.aged").read_bytes().startswith(b"age-encryption.org/v1\n")
            assert identity.encode() not in recipient_path.read_bytes()
            group_identity = (data_dir / "group-age-identity").read_bytes()
            user_envelope = subprocess.run(
                ["age", "--decrypt", "--identity", "-", str(shared_dir / "group-age-user-envelope.aged")],
                input=identity.encode(), check=True, capture_output=True,
            ).stdout
            assert user_envelope == group_identity
            scan = request_json(base_url, "/api/library/scan", "POST")
            assert len(scan["items"]) == 2
            assert scan["sidecarsCreated"] == 2 and scan["sidecarErrors"] == 0
            sidecar_paths = list((shared_dir / "metadata" / "library" / "text").glob("*.json.aged"))
            assert len(sidecar_paths) == 2
            assert all(path.read_bytes().startswith(b"age-encryption.org/v1\n") for path in sidecar_paths)
            assert all(b"Example Film" not in path.read_bytes() for path in sidecar_paths)
            decrypted = subprocess.run(
                ["age", "--decrypt", "--identity", "-", str(sidecar_paths[0])],
                input=group_identity,
                check=True,
                capture_output=True,
            )
            user_decryption = subprocess.run(
                ["age", "--decrypt", "--identity", "-", str(sidecar_paths[0])],
                input=identity.encode(), capture_output=True,
            )
            assert user_decryption.returncode != 0, "user identity must not decrypt group metadata directly"
            sidecar = json.loads(decrypted.stdout)
            assert sidecar["schemaVersion"] == 1
            assert sidecar["deviceId"] == "storage-fixture"
            assert sidecar["libraryName"] == "Fixture library"
            assert sidecar["file"]["path"].endswith(".mkv")
            print("PASS storage scan writes decryptable age-encrypted JSON sidecars in the shared folder")
            storage_env = os.environ.copy()
            storage_env.update(
                {
                    "RAWARR_DATA_DIR": str(Path(temporary) / "storage-node-data"),
                    "RAWARR_SHARED_DATA_DIR": str(shared_dir),
                    "RAWARR_LIBRARY_PATH": str(media_dir),
                    "RAWARR_LIBRARY_NAME": "Storage CLI fixture",
                    "RAWARR_DEVICE_ID": "storage-cli-fixture",
                }
            )
            storage_env.pop("RAWARR_AGE_IDENTITY_FILE", None)
            ffprobe_stub = Path(temporary) / "ffprobe-fixture"
            ffprobe_stub.write_text("#!/bin/sh\nprintf '%s\\n' '{\"format\":{\"format_name\":\"fixture\"},\"streams\":[{\"codec_type\":\"video\"}]}'\n")
            ffprobe_stub.chmod(0o755)
            storage_env["RAWARR_FFPROBE"] = str(ffprobe_stub)
            storage_run = subprocess.run(
                [str(binary), "storage-scan-once"],
                env=storage_env,
                check=True,
                capture_output=True,
                text=True,
            )
            assert "2 media files; 2 encrypted sidecars saved" in storage_run.stdout
            storage_sidecars = []
            for path in (shared_dir / "metadata" / "library" / "text").glob("*.json.aged"):
                decrypted_sidecar = subprocess.run(
                    ["age", "--decrypt", "--identity", "-", str(path)],
                    input=identity.encode(),
                    check=True,
                    capture_output=True,
                )
                record = json.loads(decrypted_sidecar.stdout)
                if record["deviceId"] == "storage-cli-fixture":
                    storage_sidecars.append(record)
            assert len(storage_sidecars) == 2
            assert all(record["libraryName"] == "Storage CLI fixture" for record in storage_sidecars)
            assert all(set(record["hashes"]) == {"md5", "sha1", "sha256"} for record in storage_sidecars)
            assert all(record["format"]["format_name"] == "fixture" for record in storage_sidecars)
            assert all(record["streams"][0]["codec_type"] == "video" for record in storage_sidecars)
            assert not list(media_dir.rglob("*.json"))
            repeat_scan = subprocess.run(
                [str(binary), "storage-scan-once"],
                env=storage_env,
                check=True,
                capture_output=True,
                text=True,
            )
            assert "0 encrypted sidecars saved" in repeat_scan.stdout
            assert "2 already complete" in repeat_scan.stdout
            print("PASS storage-only scan runs without web login, PostgreSQL, or a private age identity")
            try:
                request_json(base_url, "/api/settings/radarr", "PUT", {
                    "url": "http://127.0.0.1:1", "apiKey": TEST_API_KEY,
                })
            except urllib.error.HTTPError as error:
                assert error.code == 400, f"HTTP Radarr URL returned {error.code}, expected 400"
            else:
                raise AssertionError("plaintext HTTP Radarr URL was unexpectedly accepted")
            try:
                request_json(base_url, "/api/settings/radarr", "PUT", {
                    "url": "https://127.0.0.1:1", "apiKey": TEST_API_KEY,
                })
            except urllib.error.HTTPError as error:
                assert error.code == 502
            else:
                raise AssertionError("unreachable Radarr was unexpectedly saved")
            request_json(base_url, "/api/settings/radarr", "PUT", {
                "url": radarr_url, "apiKey": TEST_API_KEY,
            })
            radarr_status = request_json(base_url, "/api/radarr/status")
            assert radarr_status["appName"] == "Radarr"
            verified = request_json(base_url, "/api/configuration")
            assert verified["encryption"] == {
                "status": "verified",
                "ciphertextExists": True,
                "ageFormatRecognized": True,
                "identityUnlocked": True,
                "decryptionVerified": True,
            }
            assert verified["configuration"]["source"] == "age_encrypted_config"
            assert verified["configuration"]["radarr"]["apiKey"] == "[REDACTED]"
            assert TEST_API_KEY not in json.dumps(verified)
            config_path = shared_dir / "config.aged"
            assert config_path.is_file()
            assert not (data_dir / "config.aged").exists()
            ciphertext = config_path.read_bytes()
            assert ciphertext.startswith(b"age-encryption.org/v1\n")
            assert TEST_API_KEY.encode() not in ciphertext
            assert stat.S_IMODE(config_path.stat().st_mode) == 0o644
            local_config_path = data_dir / "radarr-config.json"
            local_config = json.loads(local_config_path.read_text())
            assert local_config == {"radarr_url": radarr_url, "radarr_api_key": TEST_API_KEY}
            assert stat.S_IMODE(local_config_path.stat().st_mode) == 0o600
            assert stat.S_IMODE(data_dir.stat().st_mode) == 0o700
            print("PASS local settings mirror the age-encrypted config with private file permissions")

            wrong_identity = subprocess.run(
                ["age-keygen", "-pq"], check=True, capture_output=True, text=True
            ).stdout
            try:
                request_json(
                    base_url,
                    "/api/settings/age",
                    "POST",
                    {"identity": wrong_identity},
                )
            except urllib.error.HTTPError as error:
                assert error.code == 400
            else:
                raise AssertionError("wrong age identity unexpectedly unlocked the config")
            still_unlocked = request_json(base_url, "/api/settings/age")
            assert still_unlocked["unlocked"] is True
            print("PASS wrong age identity is rejected without locking the active vault")

            poster = b"disposable encrypted poster fixture"
            artwork_dir = shared_dir / "metadata" / "movies" / "artwork"
            artwork_dir.mkdir(parents=True)
            artwork_path = artwork_dir / "12345.aged"
            subprocess.run(
                ["age", "--recipient", group_recipient, "--output", str(artwork_path), "-"],
                input=b"image/jpeg\n" + poster,
                check=True,
                stdout=subprocess.DEVNULL,
                stderr=subprocess.DEVNULL,
            )
            sealed_poster = artwork_path.read_bytes()
            assert sealed_poster.startswith(b"age-encryption.org/v1\n")
            assert poster not in sealed_poster
            served_poster, content_type = request_bytes(
                base_url, "/api/metadata/movies/12345/poster"
            )
            assert content_type == "image/jpeg" and served_poster == poster
            print("PASS .aged movie artwork is decrypted only through the unlocked API")

            stop_server(process)
            process = start_server(
                binary, data_dir, shared_dir, media_dir, port, args.database_url,
                plaintext_config=True,
            )
            wait_for_server(process, base_url)
            locked = request_json(base_url, "/api/configuration")
            assert locked["encryption"]["status"] == "locked"
            assert locked["encryption"]["ciphertextExists"] is True
            assert locked["encryption"]["ageFormatRecognized"] is True
            assert locked["encryption"]["decryptionVerified"] is False
            assert locked["configuration"] is None
            assert TEST_API_KEY not in json.dumps(locked)
            assert request_json(base_url, "/api/radarr/status")["appName"] == "Radarr"
            try:
                request_json(base_url, "/api/library/scan", "POST")
            except urllib.error.HTTPError as error:
                assert error.code == 423
            else:
                raise AssertionError("library scan unexpectedly wrote sidecars while age was locked")
            try:
                request_bytes(base_url, "/api/metadata/movies/12345/poster")
            except urllib.error.HTTPError as error:
                assert error.code == 423
            else:
                raise AssertionError("poster endpoint served artwork while the vault was locked")
            print("PASS restart without identity keeps local Radarr connected while encrypted features stay locked")

            stop_server(process)
            process = start_server(binary, data_dir, shared_dir, media_dir, port, args.database_url)
            wait_for_server(process, base_url)
            assert not local_config_path.exists()
            print("PASS production mode removes the local plaintext mirror")
        finally:
            radarr_server.shutdown()
            radarr_server.server_close()
            stop_server(process)
            SESSION_COOKIE = None
            if seeded_session:
                cleanup = subprocess.run(
                    ["psql", "-v", "ON_ERROR_STOP=1", "-c",
                     f"DELETE FROM rawrarr_users WHERE id = '{user_id}'::uuid"],
                    env={**os.environ, "PGDATABASE": args.database_url},
                    stdout=subprocess.DEVNULL,
                    stderr=subprocess.DEVNULL,
                )
                if cleanup.returncode:
                    raise RuntimeError("could not clean the isolated user from the _test database")
    print("PASS temporary test state removed")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (AssertionError, OSError, RuntimeError, urllib.error.URLError) as error:
        # Never print request payloads or server output: tests contain disposable secrets.
        print(f"FAIL Rawrarr configuration integration: {type(error).__name__}", file=sys.stderr)
        raise SystemExit(1)
