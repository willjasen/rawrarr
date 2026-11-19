#!/usr/bin/env python3
"""Exercise the private PKARR store using actual Ed25519 signatures."""

import base64
import http.client
from pathlib import Path
import sys
import tempfile
import threading
import time
import unittest
from unittest.mock import patch

from cryptography.hazmat.primitives.asymmetric.ed25519 import Ed25519PrivateKey

sys.path.insert(0, "/opt/rawrarr")
import discovery


class PrivateDiscoveryTests(unittest.TestCase):
    def test_signed_updates_and_restart(self):
        private, other = Ed25519PrivateKey.generate(), Ed25519PrivateKey.generate()

        def path(key):
            encoded = base64.b32encode(key.public_key().public_bytes_raw()).decode().rstrip("=")
            return "/pkarr/" + encoded.translate(str.maketrans(discovery.STANDARD, discovery.ALPHABET))

        def packet(key, timestamp, dns=b"\0" * 12):
            sig = key.sign(f"3:seqi{timestamp}e1:v{len(dns)}:".encode() + dns)
            return sig + timestamp.to_bytes(8, "big") + dns

        def exercise(database, operations):
            with discovery.Discovery(("127.0.0.1", 0), database) as server:
                # Keep SQLite and the HTTP server on the thread that opened the store.
                def client():
                    try:
                        for method, url, body, expected in operations:
                            with_client = http.client.HTTPConnection(*server.server_address, timeout=5)
                            try:
                                with_client.request(method, url, body)
                                response = with_client.getresponse()
                                self.assertEqual((response.status, response.read()), expected)
                            finally:
                                with_client.close()
                    except (AssertionError, OSError, http.client.HTTPException) as error:
                        errors.append(error)
                    finally:
                        server.shutdown()

                errors = []
                thread = threading.Thread(target=client)
                thread.start()
                server.serve_forever(poll_interval=0.01)
                thread.join()
                if errors:
                    raise errors[0]

        now = time.time_ns() // 1000
        url = path(private)
        first, second = packet(private, now), packet(private, now + 1)
        with tempfile.TemporaryDirectory() as directory:
            database = Path(directory) / "records.sqlite3"
            with patch.object(discovery, "MAX_RECORDS", 1):
                exercise(database, [
                    ("GET", url, None, (404, b"")),
                    ("PUT", url, packet(other, now), (400, b"")),
                    ("PUT", url, packet(private, now + 600_000_000), (400, b"")),
                    ("PUT", url, first, (200, b"")),
                    ("PUT", url, first, (200, b"")),
                    ("PUT", url, packet(private, now, b"\1" * 12), (409, b"")),
                    ("PUT", url, second, (200, b"")),
                    ("PUT", path(other), packet(other, now), (507, b"")),
                    ("GET", "/pkarr", None, (404, b"")),
                ])
                exercise(database, [
                    ("GET", url, None, (200, second)),
                    ("PUT", url, first, (409, b"")),
                    ("GET", url, None, (200, second)),
                ])


if __name__ == "__main__":
    unittest.main()
