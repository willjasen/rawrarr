#!/usr/bin/env python3
"""Restore Rawrarr's encrypted Radarr recovery archive to a user-deployed Radarr."""

from __future__ import annotations

import argparse
import getpass
import http.client
import ipaddress
import json
import os
from pathlib import Path, PurePosixPath
import secrets
import shutil
import subprocess
import sys
import tempfile
from urllib.parse import urlsplit
import zipfile
import zlib


MAX_ARCHIVE_BYTES = 512 * 1024 * 1024
MAX_UNCOMPRESSED_BYTES = 20 * 1024 * 1024 * 1024
MAX_ARCHIVE_ENTRIES = 10_000
CHUNK_BYTES = 1024 * 1024


class RecoveryError(Exception):
    pass


def validate_radarr_url(value: str):
    try:
        parsed = urlsplit(value.strip())
    except ValueError as error:
        raise RecoveryError("invalid Radarr URL") from error
    hostname = parsed.hostname
    if (
        parsed.scheme not in {"https", "http"}
        or not hostname
        or parsed.username is not None
        or parsed.password is not None
        or parsed.query
        or parsed.fragment
    ):
        raise RecoveryError("invalid Radarr URL")
    if parsed.scheme == "http":
        try:
            local = hostname.lower() == "localhost" or ipaddress.ip_address(hostname).is_loopback
        except ValueError:
            local = False
        if not local:
            raise RecoveryError("Radarr must use HTTPS except on loopback")
    try:
        port = parsed.port
    except ValueError as error:
        raise RecoveryError("invalid Radarr URL") from error
    if port is not None and not 1 <= port <= 65535:
        raise RecoveryError("invalid Radarr URL")
    return parsed


def validate_archive(path: Path) -> None:
    if not path.is_file() or path.stat().st_size > MAX_ARCHIVE_BYTES:
        raise RecoveryError("recovery archive is missing or exceeds the 512 MiB limit")
    try:
        with zipfile.ZipFile(path) as archive:
            entries = archive.infolist()
            if not entries or len(entries) > MAX_ARCHIVE_ENTRIES:
                raise RecoveryError("recovery archive has an invalid number of entries")
            total_size = 0
            for entry in entries:
                name = entry.filename
                normalized = PurePosixPath(name)
                mode = entry.external_attr >> 16
                if (
                    not name
                    or "\\" in name
                    or "\x00" in name
                    or ":" in name
                    or normalized.is_absolute()
                    or any(part in {"", ".", ".."} for part in normalized.parts)
                    or (mode & 0o170000) == 0o120000
                ):
                    raise RecoveryError("recovery archive contains an unsafe path")
                total_size += entry.file_size
                if total_size > MAX_UNCOMPRESSED_BYTES:
                    raise RecoveryError("recovery archive exceeds the uncompressed size limit")
            if archive.testzip() is not None:
                raise RecoveryError("recovery archive failed its integrity check")
    except (OSError, zipfile.BadZipFile, RuntimeError, NotImplementedError, zlib.error) as error:
        raise RecoveryError("recovery archive is invalid") from error


def decrypt_backup(backup: Path, identity_file: Path, destination: Path) -> None:
    if not backup.is_file() or backup.stat().st_size > MAX_ARCHIVE_BYTES:
        raise RecoveryError("encrypted recovery archive is missing or exceeds the 512 MiB limit")
    if not identity_file.is_file():
        raise RecoveryError("age identity file is missing")
    if shutil.which("age") is None:
        raise RecoveryError("age is required to decrypt the recovery archive")

    try:
        with destination.open("xb") as output:
            os.chmod(destination, 0o600)
            process = subprocess.Popen(
                ["age", "--decrypt", "--identity", str(identity_file), str(backup)],
                stdin=subprocess.DEVNULL,
                stdout=subprocess.PIPE,
                stderr=subprocess.DEVNULL,
            )
            assert process.stdout is not None
            total = 0
            try:
                while chunk := process.stdout.read(CHUNK_BYTES):
                    total += len(chunk)
                    if total > MAX_ARCHIVE_BYTES:
                        process.terminate()
                        process.wait()
                        raise RecoveryError("decrypted recovery archive exceeds the 512 MiB limit")
                    output.write(chunk)
            finally:
                process.stdout.close()
            if process.wait() != 0:
                raise RecoveryError("could not decrypt the recovery archive")
    except OSError as error:
        destination.unlink(missing_ok=True)
        raise RecoveryError("could not read the encrypted recovery archive") from error
    except RecoveryError:
        destination.unlink(missing_ok=True)
        raise

    try:
        validate_archive(destination)
    except RecoveryError:
        destination.unlink(missing_ok=True)
        raise


def _connection(parsed):
    port = parsed.port or (443 if parsed.scheme == "https" else 80)
    if parsed.scheme == "https":
        return http.client.HTTPSConnection(parsed.hostname, port, timeout=15)
    return http.client.HTTPConnection(parsed.hostname, port, timeout=15)


def _api_path(parsed, endpoint: str) -> str:
    return f"{parsed.path.rstrip('/')}/api/v3/{endpoint}"


def check_radarr(parsed, api_key: str) -> None:
    connection = _connection(parsed)
    try:
        connection.request(
            "GET",
            _api_path(parsed, "system/status"),
            headers={"Accept": "application/json", "X-Api-Key": api_key},
        )
        response = connection.getresponse()
        body = response.read(64 * 1024 + 1)
        if response.status != 200 or len(body) > 64 * 1024:
            raise RecoveryError("could not verify the target Radarr")
        try:
            status = json.loads(body)
        except (json.JSONDecodeError, UnicodeDecodeError) as error:
            raise RecoveryError("target Radarr returned an invalid status") from error
        app_name = status.get("appName") if isinstance(status, dict) else None
        if not isinstance(app_name, str) or app_name.lower() != "radarr":
            raise RecoveryError("the target URL did not identify itself as Radarr")
    except (OSError, http.client.HTTPException) as error:
        raise RecoveryError("could not connect to the target Radarr") from error
    finally:
        connection.close()


def restore_to_radarr(parsed, archive: Path, api_key: str) -> None:
    boundary = f"rawrarr-{secrets.token_hex(24)}"
    prefix = (
        f"--{boundary}\r\n"
        'Content-Disposition: form-data; name="restore"; filename="radarr-recovery.zip"\r\n'
        "Content-Type: application/zip\r\n\r\n"
    ).encode()
    suffix = f"\r\n--{boundary}--\r\n".encode()
    content_length = len(prefix) + archive.stat().st_size + len(suffix)
    connection = _connection(parsed)
    try:
        connection.putrequest("POST", _api_path(parsed, "system/backup/restore/upload"))
        connection.putheader("X-Api-Key", api_key)
        connection.putheader("Content-Type", f"multipart/form-data; boundary={boundary}")
        connection.putheader("Content-Length", str(content_length))
        connection.endheaders()
        connection.send(prefix)
        with archive.open("rb") as source:
            while chunk := source.read(CHUNK_BYTES):
                connection.send(chunk)
        connection.send(suffix)
        response = connection.getresponse()
        response.read(4096)
        if response.status != 200:
            raise RecoveryError(f"Radarr rejected the restore (HTTP {response.status})")
    except (OSError, http.client.HTTPException) as error:
        raise RecoveryError("could not send the recovery archive to Radarr") from error
    finally:
        connection.close()


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--backup", required=True, type=Path, help="path to metadata/radarr/recovery.zip.aged")
    parser.add_argument("--identity-file", required=True, type=Path, help="local age identity file; never place it in shared data")
    parser.add_argument("--url", required=True, help="URL of the newly deployed Radarr")
    args = parser.parse_args()

    try:
        parsed = validate_radarr_url(args.url)
        print("This will replace the target Radarr database and settings. It will not copy or modify media files.")
        if input("Type RESTORE to continue: ") != "RESTORE":
            raise RecoveryError("restore cancelled")
        api_key = getpass.getpass("Target Radarr API key: ").strip()
        if not api_key or any(not character.isprintable() for character in api_key):
            raise RecoveryError("target Radarr API key is required")
        check_radarr(parsed, api_key)
        with tempfile.TemporaryDirectory(prefix="rawrarr-radarr-recovery-") as directory:
            decrypted = Path(directory) / "recovery.zip"
            decrypt_backup(args.backup, args.identity_file, decrypted)
            restore_to_radarr(parsed, decrypted, api_key)
        print("Radarr accepted the restore and should restart. Recheck its root-folder paths after it returns.")
        return 0
    except (RecoveryError, OSError) as error:
        print(f"Radarr recovery failed: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
