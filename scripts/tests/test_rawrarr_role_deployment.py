#!/usr/bin/env python3
"""Disposable Docker integration for coordinator-approved metadata and storage nodes."""

import argparse
import base64
import hashlib
import json
import os
from pathlib import Path
import secrets
import socket
import subprocess
import tempfile
import time
import urllib.error
import urllib.request
import uuid


def docker(*args, input=None, check=True):
    result = subprocess.run(['docker', *args], input=input, capture_output=True, text=True, check=False)
    if check and result.returncode:
        reason = result.stderr.strip().replace('\n', ' ')[:1200]
        raise RuntimeError('Docker operation failed: ' + ' '.join(args[:3]) + (': ' + reason if reason else ''))
    return result


def free_port():
    with socket.socket() as listener:
        listener.bind(('127.0.0.1', 0))
        return listener.getsockname()[1]


def request(base, path, method='GET', payload=None, cookie=None):
    headers = {'Accept': 'application/json', 'Content-Type': 'application/json'}
    if cookie:
        headers['Cookie'] = 'rawrarr_session=' + cookie
    body = None if payload is None else json.dumps(payload).encode()
    req = urllib.request.Request(base + path, data=body, method=method, headers=headers)
    try:
        with urllib.request.urlopen(req, timeout=5) as response:
            return response.status, response.read()
    except urllib.error.HTTPError as error:
        return error.code, error.read()


def node_grant_request(node, payload, cookie):
    script = '''import base64,json,sys,urllib.error,urllib.request
fixture=json.loads(sys.stdin.read())
body=json.dumps(fixture["payload"],separators=(",",":")).encode()
req=urllib.request.Request("http://127.0.0.1:8080/api/auth/app-key-grants",data=body,method="POST",headers={"Content-Type":"application/json","Cookie":"rawrarr_session="+fixture["session"]})
try:
 response=urllib.request.urlopen(req,timeout=20); status=response.status; data=response.read()
except urllib.error.HTTPError as error:
 status=error.code; data=error.read()
print(status); print(base64.b64encode(data).decode())
'''
    result = docker('exec', '-i', node, 'python3', '-c', script,
        input=json.dumps({'payload': payload, 'session': cookie}))
    status_text, body_text = result.stdout.splitlines()[:2]
    return int(status_text), base64.b64decode(body_text)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--apply', action='store_true')
    parser.add_argument('--age-unlock-only', action='store_true',
        help='limit metadata configuration checks to age-vault unlock errors')
    parser.add_argument('--app-key-grants-only', action='store_true',
        help='run only the authenticated app-key grant delivery and revocation integration')
    parser.add_argument('--image', default='rawrarr:roles-test')
    parser.add_argument('--browser-image', default='rawrarr-testing-base:local')
    args = parser.parse_args()
    if args.age_unlock_only and args.app_key_grants_only:
        parser.error('--age-unlock-only and --app-key-grants-only cannot be combined')
    if not args.apply:
        print('Dry run: create a disposable coordinator and PostgreSQL DB, bootstrap the owner in Chromium, enroll metadata and storage nodes with signed approval, test role/session/outage boundaries, and remove only uniquely named fixtures. Pass --apply.')
        return 0

    prefix = 'rawrarr-test-regression-roles-' + uuid.uuid4().hex[:10]
    network = prefix
    containers = []
    helper = None
    with tempfile.TemporaryDirectory(prefix=prefix) as temporary:
        root = Path(temporary)
        (root / 'registry.json').write_text('[]')
        (root / 'registry.json').chmod(0o444)
        for directory in ('coordinator-data', 'metadata-data', 'storage-data', 'shared', 'media'):
            path = root / directory
            path.mkdir(mode=0o777)
            path.chmod(0o777)
            docker('run', '--rm', '-v', f'{path}:/fixture', 'alpine:3.21',
                'chown', '-R', '10001:10001', '/fixture')
        (root / 'media' / 'Example Film (2024)').mkdir()
        (root / 'media' / 'Example Film (2024)' / 'Example Film (2024).mkv').write_bytes(b'disposable film fixture')
        (root / 'media' / 'Example Show' / 'Season 01').mkdir(parents=True)
        (root / 'media' / 'Example Show' / 'Season 01' / 'Example.Show.S01E01.mkv').write_bytes(b'disposable episode fixture')
        cert_dir = root / 'test-ca'
        cert_dir.mkdir(mode=0o755)
        (cert_dir / 'server.ext').write_text(
            'basicConstraints=critical,CA:FALSE\nkeyUsage=critical,digitalSignature,keyEncipherment\n'
            'extendedKeyUsage=serverAuth\nsubjectAltName=DNS:localhost,IP:127.0.0.1\n')
        cert_commands = [
            ['req', '-x509', '-newkey', 'rsa:2048', '-nodes', '-sha256', '-days', '2',
             '-keyout', str(cert_dir / 'ca.key'), '-out', str(cert_dir / 'ca.crt'), '-subj', '/CN=Rawrarr Disposable Test CA',
             '-addext', 'basicConstraints=critical,CA:TRUE', '-addext', 'keyUsage=critical,keyCertSign,cRLSign'],
            ['req', '-newkey', 'rsa:2048', '-nodes', '-sha256', '-keyout', str(cert_dir / 'radarr.key'),
             '-out', str(cert_dir / 'radarr.csr'), '-subj', '/CN=localhost'],
            ['x509', '-req', '-in', str(cert_dir / 'radarr.csr'), '-CA', str(cert_dir / 'ca.crt'),
             '-CAkey', str(cert_dir / 'ca.key'), '-CAcreateserial', '-out', str(cert_dir / 'radarr.crt'),
             '-days', '2', '-sha256', '-extfile', str(cert_dir / 'server.ext')],
        ]
        if any(subprocess.run(['openssl', *command], capture_output=True, check=False).returncode for command in cert_commands):
            raise RuntimeError('could not create the temporary Radarr test CA and server certificate')
        (cert_dir / 'ca.key').chmod(0o600)
        (cert_dir / 'radarr.key').chmod(0o644)

        def run(name, image, *options):
            full = prefix + '-' + name
            containers.append(full)
            docker('run', '-d', '--name', full, '--label', 'com.docker.compose.project=rawrarr-test-regression', *options, image)
            return full

        def sidecar(node_id):
            full = prefix + '-' + node_id + '-network'
            containers.append(full)
            docker('run', '-d', '--name', full, '--label', 'com.docker.compose.project=rawrarr-test-regression',
                '--network', network, '--network-alias', node_id, 'alpine:3.21', 'sh', '-c',
                'while true; do sleep 3600; done')
            return full

        def wait_node(namespace):
            for _ in range(160):
                state = docker('inspect', '--format', '{{.State.Running}}', namespace, check=False)
                if state.returncode or state.stdout.strip() != 'true':
                    raise AssertionError('subscriber exited before becoming ready: ' + namespace)
                probe = docker('exec', namespace, 'curl', '-fsS', 'http://127.0.0.1:8080/health', check=False)
                if probe.returncode == 0:
                    return
                time.sleep(0.25)
            raise AssertionError('subscriber readiness timed out: ' + namespace)

        def start_node(role, node_id, namespace=None, plaintext_config=False):
            namespace = namespace or sidecar(node_id)
            data_dir = root / ('metadata-data' if role == 'metadata' else 'storage-data')
            shared = root / 'shared'
            media_mount = ('-e', 'RAWARR_LIBRARY_PATH=/media', '-e', 'RAWARR_LIBRARY_NAME=Fixture library',
                '-v', f'{root}/media:/media:ro') if role == 'storage' else ()
            metadata_mount = ('-e', 'RAWARR_LOCAL_PLAINTEXT_CONFIG=' + ('true' if plaintext_config else 'false'),
                '-e', 'RAWARR_ARR_CA_CERT_FILE=/test-ca/ca.crt',
                '-v', f'{root}/test-ca:/test-ca:ro') if role == 'metadata' else ()
            config_test_mount = ('-v', f'{Path(__file__).parent}/test_rawrarr_configuration.py:/tests/test_rawrarr_configuration.py:ro',)
            node = run(node_id, args.image, '--network', 'container:' + namespace,
                '-e', 'RAWARR_ROLE=' + role, '-e', 'RAWARR_NODE_ID=' + node_id,
                '-e', 'RAWARR_DEVICE_ID=' + node_id, '-e', 'RAWARR_DEVICE_NAME=' + node_id,
                '-e', 'RAWARR_PG_HOST=postgres', '-e', 'RAWARR_PG_DATABASE=rawrarr_test', '-e', 'RAWARR_PG_PASSWORD=disposable-fixture-password',
                '-e', 'RAWARR_COORDINATOR_URL=http://coordinator:8080', '-e', 'RAWARR_COORDINATOR_LOCAL_HTTP=true',
                '-e', 'RAWARR_NODE_TOKEN_FILE=/data/node-token', '-e', 'RAWARR_NODE_ENROLLMENT_ENABLED=true',
                '-e', 'RAWARR_INSTANCE_ROOT_FINGERPRINT=' + root_fingerprint,
                '-e', 'RAWARR_WEBAUTHN_RP_ID=localhost', '-e', 'RAWARR_WEBAUTHN_ORIGIN=http://localhost:8081',
                '-e', 'RUST_ARR_LISTEN_ADDR=0.0.0.0:8080',
                '-v', f'{data_dir}:/data', '-v', f'{shared}:/shared', *media_mount, *metadata_mount, *config_test_mount)
            return namespace, node

        try:
            docker('network', 'create', network)
            postgres = run('postgres', 'postgres:17-alpine', '--network', network, '--network-alias', 'postgres',
                '-e', 'POSTGRES_PASSWORD=disposable-fixture-password', '-e', 'POSTGRES_USER=rawrarr', '-e', 'POSTGRES_DB=rawrarr_test')
            for _ in range(100):
                if docker('exec', postgres, 'pg_isready', '-U', 'rawrarr', check=False).returncode == 0:
                    break
                time.sleep(0.25)
            else:
                raise RuntimeError('disposable PostgreSQL did not become ready')

            coordinator_port = free_port()
            coordinator = run('coordinator', args.image, '--network', network, '--network-alias', 'coordinator',
                '-p', f'127.0.0.1:{coordinator_port}:8080', '--entrypoint', '/usr/local/bin/rust-arr',
                '-e', 'RAWARR_ROLE=coordinator', '-e', 'RAWARR_PG_HOST=postgres', '-e', 'RAWARR_PG_DATABASE=rawrarr_test',
                '-e', 'RAWARR_PG_PASSWORD=disposable-fixture-password',
                '-e', 'RAWARR_APPROVED_NODES_FILE=/registry.json', '-e', 'RAWARR_COORDINATOR_WEB_RP_ID=localhost',
                '-e', 'RAWARR_COORDINATOR_WEB_ORIGIN=http://localhost:8081',
                '-e', 'RAWARR_NODE_RELAY_TARGETS=' + json.dumps({'metadata-test': f'http://{prefix}-metadata-test-network:8080', 'storage-test': f'http://{prefix}-storage-test-network:8080'}),
                '-e', 'RAWARR_NODE_DIRECT_TARGETS={}', '-e', 'RUST_ARR_LISTEN_ADDR=0.0.0.0:8080',
                '-v', f'{root}/registry.json:/registry.json:ro')
            for _ in range(100):
                if docker('exec', coordinator, 'curl', '-fsS', 'http://127.0.0.1:8080/health', check=False).returncode == 0:
                    break
                time.sleep(0.25)
            else:
                logs = docker('logs', coordinator, check=False)
                detail = (logs.stdout + logs.stderr).splitlines()[-24:]
                raise RuntimeError('disposable coordinator did not become ready: ' + ' | '.join(detail)[-2400:])

            metadata_namespace = sidecar('metadata-test')
            caddy = prefix + '-coordinator-web-proxy'
            containers.append(caddy)
            docker('run', '-d', '--name', caddy, '--label', 'com.docker.compose.project=rawrarr-test-regression',
                '--network', 'container:' + metadata_namespace, 'caddy:2-alpine', 'caddy', 'reverse-proxy',
                '--from', ':8081', '--to', 'coordinator:8080')
            browser_helper = (Path(__file__).parent / 'helpers/role_passkey_browser.mjs').resolve()
            helper = subprocess.Popen(
                ['docker', 'run', '-i', '--rm', '--network', 'container:' + metadata_namespace,
                 '--entrypoint', 'node', '-v', f'{browser_helper}:/test.mjs:ro', args.browser_image, '/test.mjs'],
                stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True, bufsize=1,
            )
            line = helper.stdout.readline().strip()
            if not line.startswith('READY ') or len(line.split()) != 2:
                detail = helper.stderr.read()[-2000:] if helper.poll() is not None else 'browser bootstrap emitted no readiness marker'
                raise RuntimeError('owner browser bootstrap failed: ' + detail)
            root_fingerprint = line.split()[1]
            if len(root_fingerprint) != 64:
                raise RuntimeError('owner browser returned an invalid public root fingerprint')

            metadata_namespace, metadata = start_node('metadata', 'metadata-test', metadata_namespace, plaintext_config=True)
            helper.stdin.write('approve metadata-test\n')
            helper.stdin.flush()
            approval_result = helper.stdout.readline()
            if not approval_result.startswith('APPROVED metadata-test'):
                helper_error = helper.stderr.read()[-2000:] if helper.poll() is not None else approval_result.strip()
                raise RuntimeError('owner did not approve metadata enrollment: ' + helper_error)
            wait_node(metadata)

            storage_namespace, storage = start_node('storage', 'storage-test')
            helper.stdin.write('approve storage-test\n')
            helper.stdin.flush()
            approval_result = helper.stdout.readline()
            if not approval_result.startswith('APPROVED storage-test'):
                helper_error = helper.stderr.read()[-2000:] if helper.poll() is not None else approval_result.strip()
                raise RuntimeError('owner did not approve storage enrollment: ' + helper_error)
            wait_node(storage)
            print('PASS real Chromium passkey owner setup, encrypted root creation, and signed metadata/storage approval')

            rows = docker('exec', postgres, 'psql', '-U', 'rawrarr', '-d', 'rawrarr_test', '-Atc', "SELECT role FROM rawrarr_nodes ORDER BY role").stdout.splitlines()
            assert rows == ['metadata', 'storage'], 'approved role rows were not persisted in PostgreSQL'
            passkeys = docker('exec', postgres, 'psql', '-U', 'rawrarr', '-d', 'rawrarr_test', '-Atc', 'SELECT count(*) FROM rawrarr_passkeys').stdout.strip()
            assert passkeys == '1', 'initial coordinator passkey was not persisted'

            user_id = docker('exec', postgres, 'psql', '-U', 'rawrarr', '-d', 'rawrarr_test', '-Atc', 'SELECT id FROM rawrarr_users LIMIT 1').stdout.strip()
            session = secrets.token_hex(32)
            digest = hashlib.sha256(session.encode()).hexdigest()
            sql = f"INSERT INTO rawrarr_sessions(token_hash,user_id,expires_at,node_id) VALUES(decode('{digest}','hex'),'{user_id}'::uuid,now()+interval '5 minutes','metadata-test')"
            docker('exec', '-i', postgres, 'psql', '-U', 'rawrarr', '-d', 'rawrarr_test', '-v', 'ON_ERROR_STOP=1', input=sql)
            metadata_status = docker('exec', metadata, 'python3', '-c',
                "import urllib.request; r=urllib.request.Request('http://127.0.0.1:8080/api/devices',headers={'Cookie':'rawrarr_session=" + session + "'}); print(urllib.request.urlopen(r).status)").stdout.strip()
            storage_status = docker('exec', storage, 'python3', '-c',
                "import urllib.request,urllib.error; r=urllib.request.Request('http://127.0.0.1:8080/api/devices',headers={'Cookie':'rawrarr_session=" + session + "'});\ntry: print(urllib.request.urlopen(r).status)\nexcept urllib.error.HTTPError as e: print(e.code)").stdout.strip()
            assert metadata_status == '200' and storage_status == '401', 'session was not scoped to its metadata node'
            print('PASS PostgreSQL-backed sessions are scoped to the approved node')

            storage_session = secrets.token_hex(32)
            storage_digest = hashlib.sha256(storage_session.encode()).hexdigest()
            storage_sql = f"INSERT INTO rawrarr_sessions(token_hash,user_id,expires_at,node_id) VALUES(decode('{storage_digest}','hex'),'{user_id}'::uuid,now()+interval '5 minutes','storage-test')"
            docker('exec', '-i', postgres, 'psql', '-U', 'rawrarr', '-d', 'rawrarr_test', '-v', 'ON_ERROR_STOP=1', input=storage_sql)

            app_types = ('radarr', 'sonarr', 'lidarr', 'prowlarr', 'duplicati')
            grants = []
            for app_type in app_types:
                helper.stdin.write(f'grant metadata-test {session} {app_type}\n')
                helper.stdin.flush()
                grant_line = helper.stdout.readline().strip().split()
                if len(grant_line) != 8 or grant_line[0] != 'GRANT':
                    detail = helper.stderr.read()[-1600:] if helper.poll() is not None else ''
                    raise RuntimeError('owner grant fixture failed: ' + ' '.join(grant_line[:2]) + ('; ' + detail if detail else ''))
                _, actual_type, app_instance_id, device_recipient, app_identity_b64, grant_bundle_b64, first_status, retry_status = grant_line
                assert actual_type == app_type, f'grant helper changed app type {app_type!r} to {actual_type!r}'
                assert (first_status, retry_status) == ('201', '204'), f'{app_type} grant submission or identical retry was not accepted idempotently'
                app_identity = base64.b64decode(app_identity_b64)
                grant_bundle = base64.b64decode(grant_bundle_b64)
                parsed_grant = json.loads(grant_bundle)
                assert parsed_grant['signedPayload']['appType'] == app_type
                assert parsed_grant['signedPayload']['deviceId'] == 'metadata-test'
                assert parsed_grant['signedPayload']['deviceRecipient'] == device_recipient
                assert app_identity not in grant_bundle, f'{app_type} request bundle exposed the plaintext app identity'
                stored_bundle_hex = docker('exec', postgres, 'psql', '-U', 'rawrarr', '-d', 'rawrarr_test', '-Atc',
                    "SELECT encode(bundle,'hex') FROM rawrarr_app_key_grants WHERE app_instance_id='" + app_instance_id + "' AND device_id='metadata-test'").stdout.strip()
                stored_bundle = bytes.fromhex(stored_bundle_hex)
                assert stored_bundle == grant_bundle, f'{app_type} grant was not preserved byte-for-byte'
                assert app_identity not in stored_bundle, f'coordinator database contains the plaintext {app_type} identity'
                encrypted_identity = base64.b64decode(parsed_grant['encryptedIdentity'])
                assert app_identity not in encrypted_identity, f'{app_type} device envelope contains plaintext identity'
                grants.append({
                    'type': app_type,
                    'app_instance_id': app_instance_id,
                    'device_recipient': device_recipient,
                    'app_identity': app_identity,
                    'bundle': grant_bundle,
                })
            assert {grant['type'] for grant in grants} == set(app_types)
            app_instance_id = grants[0]['app_instance_id']
            device_recipient = grants[0]['device_recipient']
            grant_bundle = grants[0]['bundle']
            print('PASS app-type-neutral signed grants for Radarr, Sonarr, Lidarr, Prowlarr, and Duplicati; retries are idempotent and every stored bundle is ciphertext-only and byte-exact')

            anonymous_status, _ = request(f'http://127.0.0.1:{coordinator_port}', '/api/nodes/app-key-grants', 'POST',
                {'appInstanceId': app_instance_id, 'recipient': device_recipient})
            assert anonymous_status == 401, 'anonymous grant retrieval was not denied'
            wrong_recipient_status, wrong_recipient_body = node_grant_request(metadata, {
                'appInstanceId': app_instance_id, 'recipient': 'age1pq1' + ('0' * 48),
            }, session)
            wrong_instance_status, wrong_instance_body = node_grant_request(metadata, {
                'appInstanceId': '00000000-0000-0000-0000-000000000000', 'recipient': device_recipient,
            }, session)
            assert (wrong_recipient_status, wrong_instance_status) == (404, 404), (
                'grant routing denial or delivery status was incorrect: '
                f'{wrong_recipient_status}/{wrong_instance_status}; '
                f'bodies={wrong_recipient_body[:120]!r}/{wrong_instance_body[:120]!r}')
            for grant in grants:
                valid_status, retrieved_bundle = node_grant_request(metadata, {
                    'appInstanceId': grant['app_instance_id'], 'recipient': grant['device_recipient'],
                }, session)
                assert valid_status == 200 and retrieved_bundle == grant['bundle'], (
                    f"{grant['type']} fossil retrieval did not return the original grant bytes")
            print('PASS signed retrieval delivers each app type byte-exactly; recipient, cross-instance, and anonymous requests are denied')

            helper.stdin.write('wrong-target storage-test ' + session + '\n')
            helper.stdin.flush()
            wrong_target = helper.stdout.readline().strip().split()
            assert wrong_target == ['WRONG_TARGET', 'storage-test', '403', '403'], 'owner submission accepted a non-fossil target'
            print('PASS owner cannot submit app-key grants to a storage-role node')

            storage_grant_status, _ = node_grant_request(storage, {
                'appInstanceId': app_instance_id, 'recipient': device_recipient,
            }, storage_session)
            assert storage_grant_status == 403, 'storage-role node retrieved a metadata grant'
            print('PASS storage-role request cannot retrieve a fossil app-key grant')

            if args.app_key_grants_only:
                helper.stdin.write('revoke metadata-test\n')
                helper.stdin.flush()
                revoked = helper.stdout.readline().strip().split()
                assert revoked == ['REVOKED', 'metadata-test', '204'], 'approved fossil revocation failed'
                remaining_grants = docker('exec', postgres, 'psql', '-U', 'rawrarr', '-d', 'rawrarr_test', '-Atc',
                    "SELECT count(*) FROM rawrarr_app_key_grants WHERE device_id='metadata-test'").stdout.strip()
                revoked_statuses = [node_grant_request(metadata, {
                    'appInstanceId': grant['app_instance_id'], 'recipient': grant['device_recipient'],
                }, session)[0] for grant in grants]
                assert remaining_grants == '0' and revoked_statuses == [401] * len(grants), (
                    'revoked fossil retained or retrieved one or more app-type grants')
                print('PASS fossil revocation removes every app-type grant and blocks subsequent retrieval')
                return 0

            config_mode = 'metadata-age' if args.age_unlock_only else 'metadata'
            config_full = docker('exec', '-i', metadata, 'python3', '/tests/test_rawrarr_configuration.py', config_mode,
                input=json.dumps({'session': session}))
            if config_full.returncode:
                raise RuntimeError('configuration API integration failed: ' + config_full.stderr.strip())
            output_lines = config_full.stdout.splitlines()
            identity_line = next((line for line in output_lines if line.startswith('TEST_IDENTITY_B64:')), None)
            if identity_line is None:
                raise RuntimeError('configuration integration did not return its temporary age identity')
            test_identity = base64.b64decode(identity_line.removeprefix('TEST_IDENTITY_B64:')).decode()
            print('\n'.join(line for line in output_lines if not line.startswith('TEST_IDENTITY_B64:')))
            if args.age_unlock_only:
                print('PASS age unlock API distinguishes malformed identities, group-envelope failures, and encrypted-config failures')
                return 0

            config_storage = docker('exec', '-i', storage, 'python3', '/tests/test_rawrarr_configuration.py', 'storage',
                input=json.dumps({'session': storage_session, 'identity': test_identity}))
            if config_storage.returncode:
                raise RuntimeError('storage configuration integration failed: ' + config_storage.stderr.strip())
            print(config_storage.stdout.strip())

            docker('stop', metadata)
            docker('rm', metadata)
            metadata_namespace, metadata = start_node('metadata', 'metadata-test', metadata_namespace, plaintext_config=False)
            wait_node(metadata)
            config_locked = docker('exec', '-i', metadata, 'python3', '/tests/test_rawrarr_configuration.py', 'locked',
                input=json.dumps({'session': session}))
            if config_locked.returncode:
                raise RuntimeError('configuration restart integration failed: ' + config_locked.stderr.strip())
            print(config_locked.stdout.strip())

            wrong_namespace = sidecar('wrong-role')
            bad = run('wrong-role', args.image, '--network', 'container:' + wrong_namespace,
                '-e', 'RAWARR_ROLE=storage', '-e', 'RAWARR_NODE_ID=metadata-test', '-e', 'RAWARR_DEVICE_ID=wrong-role',
                '-e', 'RAWARR_PG_HOST=postgres', '-e', 'RAWARR_PG_PASSWORD=disposable-fixture-password',
                '-e', 'RAWARR_COORDINATOR_URL=http://coordinator:8080', '-e', 'RAWARR_COORDINATOR_LOCAL_HTTP=true',
                '-e', 'RAWARR_NODE_TOKEN_FILE=/token', '-e', 'RAWARR_NODE_ENROLLMENT_ENABLED=true',
                '-e', 'RAWARR_INSTANCE_ROOT_FINGERPRINT=' + root_fingerprint,
                '-e', 'RAWARR_WEBAUTHN_RP_ID=localhost', '-e', 'RAWARR_WEBAUTHN_ORIGIN=http://localhost:8081',
                '-v', f'{root}/metadata-data/node-token:/token:ro', '-v', f'{root}/coordinator-data:/data')
            for _ in range(100):
                state = json.loads(docker('inspect', bad).stdout)[0]['State']
                if state['Status'] == 'exited':
                    break
                time.sleep(0.2)
            assert state['Status'] == 'exited' and state['ExitCode'] != 0, 'a metadata credential was accepted for storage role'
            print('PASS role mismatch fails closed')

            helper.stdin.write('revoke metadata-test\n')
            helper.stdin.flush()
            revoked = helper.stdout.readline().strip().split()
            assert revoked == ['REVOKED', 'metadata-test', '204'], 'approved fossil revocation failed'
            remaining_grants = docker('exec', postgres, 'psql', '-U', 'rawrarr', '-d', 'rawrarr_test', '-Atc',
                "SELECT count(*) FROM rawrarr_app_key_grants WHERE device_id='metadata-test'").stdout.strip()
            revoked_status, _ = node_grant_request(metadata, {
                'appInstanceId': app_instance_id, 'recipient': device_recipient,
            }, session)
            assert remaining_grants == '0' and revoked_status == 401, 'revoked fossil retained or retrieved its grant'
            print('PASS fossil revocation removes stored grants and blocks further retrieval')

            docker('stop', coordinator)
            outage = docker('exec', metadata, 'python3', '-c',
                "import urllib.request,urllib.error;\ntry: print(urllib.request.urlopen('http://127.0.0.1:8080/api/nodes/pending').status)\nexcept urllib.error.HTTPError as e: print(e.code)\nexcept Exception: print('unavailable')").stdout.strip()
            assert outage in ('401', '503', 'unavailable'), 'metadata node did not fail closed during coordinator outage'
            print('PASS coordinator outage fails closed')
        finally:
            if helper and helper.poll() is None:
                try:
                    helper.stdin.write('quit\n')
                    helper.stdin.flush()
                    helper.wait(timeout=10)
                except Exception:
                    helper.terminate()
                    try:
                        helper.wait(timeout=5)
                    except subprocess.TimeoutExpired:
                        helper.kill()
                        helper.wait()
            for name in reversed(containers):
                subprocess.run(['docker', 'rm', '-f', name], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
            subprocess.run(['docker', 'network', 'rm', network], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    print('PASS disposable containers, network, PostgreSQL, DB rows, and temporary files cleaned up')
    return 0


if __name__ == '__main__':
    try:
        raise SystemExit(main())
    except (AssertionError, OSError, RuntimeError, subprocess.SubprocessError) as error:
        print('FAIL Rawrarr role integration: ' + str(error), file=__import__('sys').stderr)
        raise SystemExit(1)
