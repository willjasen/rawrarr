#!/usr/bin/env python3
"""Private PKARR HTTP store. No DHT client, forwarding, or enumeration."""

import base64
import re
import sqlite3
import time
from http.server import BaseHTTPRequestHandler, HTTPServer
from pathlib import Path

from cryptography.exceptions import InvalidSignature
from cryptography.hazmat.primitives.asymmetric.ed25519 import Ed25519PublicKey

ALPHABET = "ybndrfg8ejkmcpqxot1uwisza345h769"
STANDARD = "ABCDEFGHIJKLMNOPQRSTUVWXYZ234567"
RECORD_PATH = re.compile(rf"/pkarr/([{ALPHABET}]{{52}})")
MAX_RECORDS = 512
MAX_DNS_BYTES = 1000
MAX_FUTURE_US = 300_000_000


def decode_key(path):
    match = RECORD_PATH.fullmatch(path)
    if not match:
        return None
    encoded = match[1]
    key = base64.b32decode(encoded.translate(str.maketrans(ALPHABET, STANDARD)) + "====")
    canonical = base64.b32encode(key).decode().rstrip("=").translate(str.maketrans(STANDARD, ALPHABET))
    return key if encoded == canonical else None


def verify_packet(key, packet):
    if not 84 <= len(packet) <= 72 + MAX_DNS_BYTES:
        raise ValueError("Invalid packet size")
    timestamp = int.from_bytes(packet[64:72], "big")
    if not 0 < timestamp <= time.time_ns() // 1000 + MAX_FUTURE_US:
        raise ValueError("Invalid packet timestamp")
    dns = packet[72:]
    Ed25519PublicKey.from_public_bytes(key).verify(
        packet[:64], f"3:seqi{timestamp}e1:v{len(dns)}:".encode() + dns
    )
    return timestamp


class Discovery(HTTPServer):
    def __init__(self, address, database):
        self.store = sqlite3.connect(database)
        try:
            with self.store:
                self.store.execute("""
                    CREATE TABLE IF NOT EXISTS records (
                        key BLOB PRIMARY KEY, timestamp INTEGER NOT NULL, packet BLOB NOT NULL
                    ) WITHOUT ROWID
                """)
            super().__init__(address, Request)
        except (OSError, sqlite3.Error):
            self.store.close()
            raise

    def server_close(self):
        super().server_close()
        self.store.close()

    def get_request(self):
        connection, address = super().get_request()
        connection.settimeout(3)
        return connection, address


class Request(BaseHTTPRequestHandler):
    def respond(self, status, packet=b""):
        self.send_response(status)
        self.send_header("Content-Length", str(len(packet)))
        self.send_header("Cache-Control", "no-store")
        if packet:
            self.send_header("Content-Type", "application/octet-stream")
        self.end_headers()
        if packet:
            self.wfile.write(packet)

    def do_GET(self):
        path = self.path.partition("?")[0]
        if path == "/healthz":
            self.server.store.execute("SELECT key FROM records LIMIT 1").fetchone()
            return self.respond(200)
        key = decode_key(path)
        if key is None:
            return self.respond(404)
        row = self.server.store.execute("SELECT packet FROM records WHERE key = ?", (key,)).fetchone()
        return self.respond(200, row[0]) if row else self.respond(404)

    def do_PUT(self):
        key = decode_key(self.path)
        if key is None:
            return self.respond(404)
        lengths = self.headers.get_all("Content-Length", [])
        if self.headers.get("Transfer-Encoding") or len(lengths) != 1 or not re.fullmatch(r"[0-9]{1,4}", lengths[0]):
            return self.respond(400)
        size = int(lengths[0])
        if not 84 <= size <= 72 + MAX_DNS_BYTES:
            return self.respond(413)
        packet = self.rfile.read(size)
        try:
            if len(packet) != size:
                raise ValueError("Incomplete packet")
            timestamp = verify_packet(key, packet)
        except (ValueError, InvalidSignature):
            return self.respond(400)
        store = self.server.store
        with store:
            store.execute("BEGIN IMMEDIATE")
            current = store.execute("SELECT timestamp, packet FROM records WHERE key = ?", (key,)).fetchone()
            if current and timestamp <= current[0]:
                status = 200 if packet == current[1] else 409
            elif not current and store.execute("SELECT COUNT(*) FROM records").fetchone()[0] >= MAX_RECORDS:
                status = 507
            else:
                store.execute("INSERT OR REPLACE INTO records VALUES (?, ?, ?)", (key, timestamp, packet))
                status = 200
        return self.respond(status)

    def log_request(self, code="-", size="-"):
        if self.command == "PUT" or str(code).startswith("5"):
            print(f"Rayfish discovery: {self.command} HTTP {code}", flush=True)


if __name__ == "__main__":
    with Discovery(("0.0.0.0", 18090), Path("/state/discovery.sqlite3")) as server:
        print("Rayfish discovery: ready on TCP 18090 (private signed records only)", flush=True)
        server.serve_forever()
