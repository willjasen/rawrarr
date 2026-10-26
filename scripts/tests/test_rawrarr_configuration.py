#!/usr/bin/env python3
"""Configuration API integration body run inside the approved metadata test node."""

from __future__ import annotations

import argparse
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
import json
import base64
import os
from pathlib import Path
import secrets
import ssl
import subprocess
import sys
import threading
import urllib.error
import urllib.request


TEST_API_KEY = 'rawrarr-disposable-configuration-key'
BASE_URL = 'http://127.0.0.1:8080'
SESSION_COOKIE = ''
RADARR_REQUESTS = 0


class RadarrMockHandler(BaseHTTPRequestHandler):
    def do_GET(self):
        global RADARR_REQUESTS
        RADARR_REQUESTS += 1
        if self.path != '/api/v3/system/status' or self.headers.get('X-Api-Key') != TEST_API_KEY:
            self.send_error(401)
            return
        body = json.dumps({'appName': 'Radarr', 'version': '5.0.0', 'urlBase': ''}).encode()
        self.send_response(200)
        self.send_header('Content-Type', 'application/json')
        self.send_header('Content-Length', str(len(body)))
        self.end_headers()
        self.wfile.write(body)

    def log_message(self, *_args):
        return


def request(base_url, path, method='GET', payload=None):
    data = None if payload is None else json.dumps(payload).encode()
    req = urllib.request.Request(base_url + path, data=data, method=method,
        headers={'Accept': 'application/json', 'Content-Type': 'application/json', 'Cookie': SESSION_COOKIE})
    try:
        with urllib.request.urlopen(req, timeout=10) as response:
            return response.status, response.read()
    except urllib.error.HTTPError as error:
        return error.code, error.read()


def expect_json(path, method='GET', payload=None, status=200):
    actual, body = request(BASE_URL, path, method, payload)
    assert actual == status, f'{path} returned HTTP {actual}, expected {status}'
    return json.loads(body) if body else None


def expect_bytes(path, status=200):
    actual, body = request(BASE_URL, path)
    assert actual == status, f'{path} returned HTTP {actual}, expected {status}'
    return body


def encrypt_identity_for_node(identity):
    recipient = expect_bytes('/api/metadata/export/node-recipient').decode().strip()
    encrypted = subprocess.run(['age', '--recipient', recipient], input=identity.encode(),
        check=True, capture_output=True).stdout
    decoded = subprocess.run(['age', '--decrypt', '--identity', '/data/node-age-identity'],
        input=encrypted, check=True, capture_output=True).stdout
    assert decoded == identity.encode(), 'node identity envelope did not round trip locally'
    return base64.b64encode(encrypted).decode('ascii')


def unlock_identity(identity):
    status, body = request(BASE_URL, '/api/settings/age', 'POST', {
        'identityEnvelope': encrypt_identity_for_node(identity),
    })
    if status != 200:
        try:
            reason = json.loads(body).get('error', '')
        except (json.JSONDecodeError, AttributeError):
            reason = ''
        raise AssertionError(f'/api/settings/age returned HTTP {status}: {reason}')
    return json.loads(body)


def verify_configuration(session, mode):
    global SESSION_COOKIE, RADARR_REQUESTS
    SESSION_COOKIE = 'rawrarr_session=' + session
    data_dir = Path(os.environ['RAWARR_DATA_DIR'])
    shared_dir = Path(os.environ['RAWARR_SHARED_DATA_DIR'])
    if mode == 'locked':
        view = expect_json('/api/configuration')
        assert view['encryption']['status'] == 'locked', 'encrypted config did not remain locked after restart'
        assert view['encryption']['ciphertextExists'] and view['encryption']['ageFormatRecognized']
        assert view['configuration'] is None
        assert TEST_API_KEY not in json.dumps(view)
        assert not (data_dir / 'radarr-config.json').exists(), 'production startup should remove the plaintext mirror'
        print('PASS production-mode restart removes the local plaintext Radarr mirror and leaves encrypted config locked')
        return

    if mode == 'storage':
        media_dir = Path(os.environ['RAWARR_LIBRARY_PATH'])
        identity = payload_identity
        unlock_identity(identity)
        scan = expect_json('/api/library/scan', 'POST', {})
        assert len(scan['items']) == 2 and scan['sidecarsCreated'] == 2 and scan['sidecarErrors'] == 0, 'storage scan did not create exactly two sidecars'
        sidecars = list((shared_dir / 'metadata' / 'library' / 'text').glob('*.json.aged'))
        assert len(sidecars) == 2 and all(path.read_bytes().startswith(b'age-encryption.org/v1\n') for path in sidecars), 'shared storage sidecars are missing or unencrypted'
        group_identity = (data_dir / 'group-age-identity').read_text()
        wrapped_group_identity = subprocess.run(
            ['age', '--decrypt', '--identity', '-', str(shared_dir / 'group-age-user-envelope.aged')],
            input=identity.encode(), check=True, capture_output=True).stdout
        assert wrapped_group_identity == group_identity.encode(), 'storage node did not recover the user-wrapped group identity'
        decrypted = subprocess.run(['age', '--decrypt', '--identity', '-', str(sidecars[0])],
            input=group_identity.encode(), check=True, capture_output=True).stdout
        user_decryption = subprocess.run(['age', '--decrypt', '--identity', '-', str(sidecars[0])],
            input=identity.encode(), capture_output=True)
        assert user_decryption.returncode != 0, 'user identity decrypted group metadata directly'
        record = json.loads(decrypted)
        assert record['deviceId'] == os.environ['RAWARR_DEVICE_ID'], 'sidecar device ID did not match storage node'
        assert record['libraryName'] == 'Fixture library', 'sidecar library name was not retained'
        assert record['file']['path'].endswith('.mkv'), 'sidecar does not describe a media path'
        assert not list(media_dir.rglob('*.json')), 'scanner wrote JSON plaintext into the media tree'
        print('PASS storage node scans its media into decryptable age-encrypted sidecars in shared data')
        return

    initial = expect_json('/api/configuration')
    assert initial['encryption']['status'] == 'not_configured' and initial['configuration'] is None, 'fresh metadata config should be unconfigured'
    generated = expect_json('/api/settings/age/generate', 'POST', {})
    identity = generated['identity']
    assert identity.startswith('#') and 'AGE-SECRET-KEY-PQ-1' in identity
    derived = subprocess.run(['age-keygen', '-y'], input=identity.encode(), check=True, capture_output=True).stdout.decode().strip()
    assert derived == generated['recipient'], 'generated user identity does not match its public recipient'
    unlock = unlock_identity(identity)
    assert unlock['unlocked'] is True, 'metadata age identity did not unlock'
    recipient = (shared_dir / 'group-age-recipient').read_text().strip()
    assert recipient.startswith('age1pq1') and recipient != generated['recipient']
    assert (shared_dir / 'age-recipient').read_text().strip() == recipient
    assert identity.encode() not in (shared_dir / 'age-recipient').read_bytes()
    group_identity_path = data_dir / 'group-age-identity'
    group_identity = group_identity_path.read_text()
    assert data_dir.stat().st_mode & 0o777 == 0o700 and group_identity_path.stat().st_mode & 0o777 == 0o600
    wrapped_group_identity = subprocess.run(
        ['age', '--decrypt', '--identity', '-', str(shared_dir / 'group-age-user-envelope.aged')],
        input=identity.encode(), check=True, capture_output=True).stdout
    assert wrapped_group_identity == group_identity.encode(), 'user-wrapped group identity did not decrypt for the owner'

    expect_json('/api/settings/radarr', 'PUT', {'url': 'http://127.0.0.1:1', 'apiKey': TEST_API_KEY}, 400)
    expect_json('/api/settings/radarr', 'PUT', {'url': 'https://127.0.0.1:1', 'apiKey': TEST_API_KEY}, 502)
    mock = ThreadingHTTPServer(('127.0.0.1', 0), RadarrMockHandler)
    context = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)
    context.load_cert_chain('/test-ca/radarr.crt', '/test-ca/radarr.key')
    mock.socket = context.wrap_socket(mock.socket, server_side=True)
    thread = threading.Thread(target=mock.serve_forever, daemon=True)
    thread.start()
    radarr_url = f'https://127.0.0.1:{mock.server_port}'
    try:
        probe = subprocess.run(['curl', '--fail', '--silent', '--show-error', '--cacert', '/test-ca/ca.crt',
            '-H', 'X-Api-Key: ' + TEST_API_KEY, radarr_url + '/api/v3/system/status'], capture_output=True)
        assert probe.returncode == 0, 'temporary HTTPS Radarr mock failed its in-container TLS/API preflight'
        RADARR_REQUESTS = 0
        actual, body = request(BASE_URL, '/api/settings/radarr', 'PUT', {'url': radarr_url, 'apiKey': TEST_API_KEY})
        assert actual == 200, f'Radarr settings save returned HTTP {actual}; mock received request={RADARR_REQUESTS > 0}'
    finally:
        mock.shutdown()
        mock.server_close()
        thread.join(timeout=2)

    verified = expect_json('/api/configuration')
    assert verified['encryption']['status'] == 'verified', 'saved Radarr settings were not verified from encrypted config'
    assert verified['configuration']['source'] == 'age_encrypted_config'
    assert verified['configuration']['radarr']['apiKey'] == '[REDACTED]', 'Radarr API key was not redacted'
    assert TEST_API_KEY not in json.dumps(verified)
    config_path = shared_dir / 'config.aged'
    assert config_path.is_file() and config_path.read_bytes().startswith(b'age-encryption.org/v1\n')
    assert TEST_API_KEY.encode() not in config_path.read_bytes()
    local_path = data_dir / 'radarr-config.json'
    assert json.loads(local_path.read_text()) == {'radarr_url': radarr_url, 'radarr_api_key': TEST_API_KEY}, 'local Radarr mirror does not match the encrypted settings'
    assert local_path.stat().st_mode & 0o777 == 0o600
    print('PASS Radarr settings are HTTPS-validated, encrypted in shared config, redacted on readback, and mirrored with private file permissions')

    wrong_identity = subprocess.run(['age-keygen', '-pq'], check=True, capture_output=True, text=True).stdout
    expect_json('/api/settings/age', 'POST', {
        'identityEnvelope': encrypt_identity_for_node(wrong_identity),
    }, 400)
    assert expect_json('/api/settings/age')['unlocked'] is True

    poster = b'disposable encrypted poster fixture'
    artwork = shared_dir / 'metadata' / 'movies' / 'artwork'
    artwork.mkdir(parents=True, exist_ok=True)
    poster_path = artwork / '12345.aged'
    subprocess.run(['age', '--recipient', recipient, '--output', str(poster_path), '-'],
        input=b'image/jpeg\n' + poster, check=True, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    assert poster not in poster_path.read_bytes()
    served = expect_bytes('/api/metadata/movies/12345/poster')
    assert served == poster
    print('PASS encrypted poster is decrypted only through the unlocked metadata API')
    print('TEST_IDENTITY_B64:' + base64.b64encode(identity.encode()).decode())


payload_identity = ''

def main():
    global payload_identity
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('mode', choices=('metadata', 'storage', 'locked'),
        help='test stage; the disposable role integration supplies the approved node and session')
    args = parser.parse_args()
    payload = json.loads(sys.stdin.readline())
    session = payload.get('session')
    if not isinstance(session, str) or len(session) != 64 or any(c not in '0123456789abcdef' for c in session):
        raise SystemExit('invalid disposable session input')
    payload_identity = payload.get('identity', '')
    if args.mode == 'storage' and (not isinstance(payload_identity, str) or 'AGE-SECRET-KEY-PQ-1' not in payload_identity):
        raise SystemExit('storage-mode test requires the temporary test age identity')
    verify_configuration(session, args.mode)


if __name__ == '__main__':
    try:
        main()
    except (AssertionError, OSError, RuntimeError, subprocess.CalledProcessError, urllib.error.URLError) as error:
        detail = f': {error}' if isinstance(error, AssertionError) and str(error) else ''
        print('FAIL Rawrarr configuration integration: ' + type(error).__name__ + detail, file=sys.stderr)
        raise SystemExit(1)
