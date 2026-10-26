#!/usr/bin/env python3
"""Verify fossil saves encrypted Radarr/Sonarr settings and reads both APIs."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path
import secrets
import socket
import subprocess
import sys
import time
import urllib.request
import uuid
import xml.etree.ElementTree as ET


PROJECT = "rawrarr-test-environment"
COMPOSE = "docker-compose.orbstack-test.yml"
TEST_STATE = Path.home() / "Library" / "Application Support" / "rawrarr" / "orbstack-test"
TLS_STATE = TEST_STATE / "arr-tls"
FOSSIL = f"{PROJECT}-metadata-a-1"
POSTGRES = f"{PROJECT}-postgres-1"


def docker(*args: str, check: bool = True, input: str | None = None, env=None) -> str:
    result = subprocess.run(
        ["docker", "--context", "orbstack", *args],
        input=input,
        capture_output=True,
        text=True,
        check=False,
        env=env,
    )
    if check and result.returncode:
        raise RuntimeError(result.stderr.strip() or f"docker {' '.join(args[:2])} failed")
    return result.stdout.strip()


def ensure_test_certificates() -> None:
    if not shutil_which("openssl"):
        raise RuntimeError("openssl is required to create the private OrbStack test CA")
    TLS_STATE.mkdir(parents=True, exist_ok=True, mode=0o700)
    TLS_STATE.chmod(0o700)
    ca_cert = TLS_STATE / "arr-test-ca.crt"
    ca_key = TLS_STATE / "arr-test-ca.key"
    server_cert = TLS_STATE / "arr-test.crt"
    server_key = TLS_STATE / "arr-test.key"
    extension_file = TLS_STATE / "arr-server.ext"
    if all(path.is_file() for path in (ca_cert, ca_key, server_cert, server_key)):
        verify_results = [
            subprocess.run(
                ["openssl", "verify", "-CAfile", str(ca_cert), "-verify_hostname", hostname, str(server_cert)],
                capture_output=True,
                text=True,
                check=False,
            )
            for hostname in ("radarr.test", "sonarr.test")
        ]
        if all(result.returncode == 0 for result in verify_results):
            return
    for path in (ca_cert, ca_key, server_cert, server_key, extension_file, TLS_STATE / "arr-test-ca.srl", TLS_STATE / "arr-server.csr"):
        path.unlink(missing_ok=True)
    extension_file.write_text(
        "basicConstraints=critical,CA:FALSE\n"
        "keyUsage=critical,digitalSignature,keyEncipherment\n"
        "extendedKeyUsage=serverAuth\n"
        "subjectAltName=DNS:radarr.test,DNS:sonarr.test\n",
        encoding="ascii",
    )
    run_openssl(
        "req", "-x509", "-newkey", "rsa:2048", "-nodes", "-sha256", "-days", "3650",
        "-keyout", str(ca_key), "-out", str(ca_cert), "-subj", "/CN=Rawrarr OrbStack Arr Test CA",
        "-addext", "basicConstraints=critical,CA:TRUE", "-addext", "keyUsage=critical,keyCertSign,cRLSign",
    )
    run_openssl(
        "req", "-newkey", "rsa:2048", "-nodes", "-sha256", "-keyout", str(server_key),
        "-out", str(TLS_STATE / "arr-server.csr"), "-subj", "/CN=radarr.test",
    )
    run_openssl(
        "x509", "-req", "-in", str(TLS_STATE / "arr-server.csr"), "-CA", str(ca_cert),
        "-CAkey", str(ca_key), "-CAcreateserial", "-out", str(server_cert), "-days", "825",
        "-sha256", "-extfile", str(extension_file),
    )
    for path in (ca_key, server_key):
        path.chmod(0o600)
    for hostname in ("radarr.test", "sonarr.test"):
        run_openssl("verify", "-CAfile", str(ca_cert), "-verify_hostname", hostname, str(server_cert))


def shutil_which(command: str) -> str | None:
    from shutil import which

    return which(command)


def run_openssl(*args: str) -> None:
    result = subprocess.run(["openssl", *args], capture_output=True, text=True, check=False)
    if result.returncode:
        raise RuntimeError("openssl could not prepare or validate the local test certificate")


def wait_healthy(service: str, timeout: int = 240) -> None:
    deadline = time.monotonic() + timeout
    name = f"{PROJECT}-{service}-1"
    while time.monotonic() < deadline:
        state = docker("inspect", "--format", "{{.State.Health.Status}}", name, check=False)
        if state == "healthy":
            return
        running = docker("inspect", "--format", "{{.State.Running}}", name, check=False)
        if running == "false":
            raise RuntimeError(f"{service} exited before becoming healthy")
        time.sleep(2)
    raise TimeoutError(f"{service} did not become healthy within {timeout}s")


def assert_no_host_port_bindings(services: tuple[str, ...]) -> None:
    for service in services:
        name = f"{PROJECT}-{service}-1"
        raw = docker("inspect", "--format", "{{json .HostConfig.PortBindings}}", name)
        bindings = json.loads(raw or "{}") or {}
        published = {port: entries for port, entries in bindings.items() if entries}
        if published:
            raise RuntimeError(f"subscriber container {service} has host-published ports")
    print("PASS fossil and Tailscale containers have no host-published ports")


def coordinator_url() -> str:
    raw = docker("inspect", "--format", "{{json .NetworkSettings.Ports}}", f"{PROJECT}-tailscale-coordinator-1")
    ports = json.loads(raw or "{}") or {}
    bindings = ports.get("8080/tcp") or []
    if len(bindings) != 1 or not bindings[0].get("HostPort"):
        raise RuntimeError("test coordinator has no host port mapping")
    return f"http://127.0.0.1:{bindings[0]['HostPort']}"


def available_loopback_port() -> int:
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as sock:
        sock.bind(("127.0.0.1", 0))
        return int(sock.getsockname()[1])


def generate_test_signing_root() -> tuple[str, str]:
    source = """
import { randomBytes } from 'node:crypto';
import { ml_dsa65 } from '@noble/post-quantum/ml-dsa.js';
import { sha256 } from '@noble/hashes/sha2.js';
const seed = new Uint8Array(randomBytes(ml_dsa65.lengths.seed));
let secretKey, publicKey;
try {
  ({ secretKey, publicKey } = ml_dsa65.keygen(seed));
  const domain = new TextEncoder().encode('rawrarr-instance-root-v1\\0');
  const input = new Uint8Array(domain.length + publicKey.length);
  input.set(domain);
  input.set(publicKey, domain.length);
  console.log(JSON.stringify({ publicKey: Buffer.from(publicKey).toString('base64'), fingerprint: Buffer.from(sha256(input)).toString('hex') }));
  input.fill(0);
} finally {
  seed.fill(0);
  secretKey?.fill(0);
  publicKey?.fill(0);
}
"""
    result = subprocess.run(
        ["node", "--input-type=module", "-e", source],
        cwd=Path(__file__).resolve().parents[2] / "rust-arr" / "web-export",
        capture_output=True,
        text=True,
        check=False,
    )
    if result.returncode:
        raise RuntimeError("Node.js and the bundled ML-DSA modules are required to prepare the isolated test root")
    payload = json.loads(result.stdout)
    if len(payload.get("fingerprint", "")) != 64 or not payload.get("publicKey"):
        raise RuntimeError("could not prepare an isolated test signing root")
    return payload["publicKey"], payload["fingerprint"]


def prepare_headscale_auth_keys() -> tuple[Path, Path]:
    users_raw = docker("exec", f"{PROJECT}-headscale-1", "headscale", "users", "list", "--output", "json")
    users = json.loads(users_raw or "[]") or []
    user = next((item for item in users if item.get("name") == PROJECT), None)
    if user is None:
        docker("exec", f"{PROJECT}-headscale-1", "headscale", "users", "create", PROJECT)
        users_raw = docker("exec", f"{PROJECT}-headscale-1", "headscale", "users", "list", "--output", "json")
        users = json.loads(users_raw or "[]") or []
        user = next((item for item in users if item.get("name") == PROJECT), None)
    if user is None or not user.get("id"):
        raise RuntimeError("could not prepare the isolated Headscale test user")

    key_dir = TEST_STATE / "generated-headscale-auth"
    key_dir.mkdir(parents=True, exist_ok=True, mode=0o700)
    key_dir.chmod(0o700)
    paths = []
    for suffix in ("a", "b"):
        raw = docker(
            "exec", f"{PROJECT}-headscale-1", "headscale", "preauthkeys", "create",
            "--user", str(user["id"]), "--reusable", "--expiration", "1h", "--output", "json",
        )
        payload = json.loads(raw)
        key = payload.get("key")
        if not isinstance(key, str) or not key:
            raise RuntimeError("Headscale did not return a disposable test auth key")
        path = key_dir / f"{suffix}-{uuid.uuid4().hex}.key"
        descriptor = os.open(path, os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600)
        with os.fdopen(descriptor, "w", encoding="utf-8") as output:
            output.write(key)
        paths.append(path)
    return paths[0], paths[1]


def ensure_test_signing_root() -> str:
    try:
        with urllib.request.urlopen(coordinator_url() + "/api/nodes/signing-root", timeout=10) as response:
            root = json.load(response)
        if len(root.get("fingerprint", "")) != 64:
            raise RuntimeError("coordinator returned an invalid instance signing-root fingerprint")
        return root["fingerprint"]
    except urllib.error.HTTPError as error:
        if error.code != 404:
            raise

    public_key, fingerprint = generate_test_signing_root()
    user_id = str(uuid.uuid4())
    username = f"orb_test_{user_id.replace('-', '')}"
    sql = (
        "INSERT INTO rawrarr_users (id,username,display_name) "
        f"VALUES ('{user_id}'::uuid,'{username}','Disposable environment test');\n"
        "INSERT INTO rawrarr_instance_signing_root(singleton,algorithm,public_key,fingerprint,owner_user_id) "
        f"VALUES (TRUE,'ML-DSA-65',decode('{public_key}','base64'),'{fingerprint}','{user_id}'::uuid) "
        "ON CONFLICT(singleton) DO NOTHING;\n"
    )
    docker("exec", "-i", POSTGRES, "psql", "-U", "rawrarr", "-d", "rawrarr", "-v", "ON_ERROR_STOP=1", input=sql)
    with urllib.request.urlopen(coordinator_url() + "/api/nodes/signing-root", timeout=10) as response:
        root = json.load(response)
    if root.get("fingerprint") != fingerprint:
        raise RuntimeError("isolated test signing root did not match its public key")
    print("PASS coordinator initialized an isolated test-only public signing root")
    return fingerprint


def api_key(service: str) -> str:
    contents = docker("exec", f"{PROJECT}-{service}-1", "cat", "/config/config.xml")
    key = ET.fromstring(contents).findtext("ApiKey")
    if not key:
        raise RuntimeError(f"{service} has no generated API key")
    return key


def seed_session() -> tuple[str, str, str]:
    user_id = str(uuid.uuid4())
    session = secrets.token_hex(32)
    session_digest = hashlib.sha256(session.encode("ascii")).hexdigest()
    username = f"arr_connectivity_test_{user_id.replace('-', '')}"
    sql = (
        "INSERT INTO rawrarr_users (id,username,display_name) "
        f"VALUES ('{user_id}'::uuid,'{username}','Disposable Arr connectivity test');\n"
        "INSERT INTO rawrarr_sessions (token_hash,user_id,expires_at,node_id) "
        f"VALUES (decode('{session_digest}','hex'),'{user_id}'::uuid,now()+interval '10 minutes','test-metadata-a');\n"
    )
    docker("exec", "-i", POSTGRES, "psql", "-U", "rawrarr", "-d", "rawrarr", "-v", "ON_ERROR_STOP=1", input=sql)
    return user_id, session_digest, session


def remove_session(user_id: str, session_digest: str) -> None:
    sql = (
        "DELETE FROM rawrarr_sessions "
        f"WHERE token_hash=decode('{session_digest}','hex') AND user_id='{user_id}'::uuid;\n"
        f"DELETE FROM rawrarr_users WHERE id='{user_id}'::uuid;\n"
    )
    docker("exec", "-i", POSTGRES, "psql", "-U", "rawrarr", "-d", "rawrarr", "-v", "ON_ERROR_STOP=1", input=sql)


APP_FLOW = r'''
import base64, json, os, subprocess, sys, urllib.error, urllib.request
payload=json.load(sys.stdin)
cookie='rawrarr_session='+payload['session']
def call(path, method='GET', body=None):
    data=None if body is None else json.dumps(body).encode()
    request=urllib.request.Request('http://127.0.0.1:8080'+path,data=data,method=method,headers={'Accept':'application/json','Content-Type':'application/json','Cookie':cookie})
    try:
        with urllib.request.urlopen(request,timeout=35) as response:
            raw=response.read()
            return response.status,(json.loads(raw) if raw else None)
    except urllib.error.HTTPError as error:
        raise RuntimeError(path+' returned HTTP '+str(error.code))
def require(condition, message):
    if not condition: raise RuntimeError(message)
def text(path):
    request=urllib.request.Request('http://127.0.0.1:8080'+path,headers={'Accept':'text/plain','Cookie':cookie})
    with urllib.request.urlopen(request,timeout=35) as response: return response.read().decode().strip()
identity_path='/data/age-identity'
if os.path.isfile(identity_path):
    identity=open(identity_path,encoding='utf-8').read().strip()
    remember=False
else:
    code,generated=call('/api/settings/age/generate','POST',{})
    require(code==200 and 'AGE-SECRET-KEY-PQ-1' in generated.get('identity',''),'age identity generation failed (HTTP '+str(code)+')')
    identity=generated['identity']
    remember=True
node_recipient=text('/api/metadata/export/node-recipient')
wrapped=subprocess.run(['age','--recipient',node_recipient],input=identity.encode(),capture_output=True,check=True).stdout
identity_envelope=base64.b64encode(wrapped).decode('ascii')
code,unlocked=call('/api/settings/age','POST',{'identityEnvelope':identity_envelope,'rememberOnDevice':remember})
require(code==200 and unlocked.get('unlocked') is True,'fossil age vault unlock failed')
for service,url,key in [('radarr','https://radarr.test',payload['radarrKey']),('sonarr','https://sonarr.test',payload['sonarrKey'])]:
    code,saved=call('/api/settings/'+service,'PUT',{'url':url,'apiKey':key})
    require(code==200 and saved.get('configured') is True and saved.get('persisted') is True,service+' encrypted settings save failed')
    code,readback=call('/api/settings/'+service)
    require(code==200 and readback.get('configured') is True and readback.get('persisted') is True and readback.get('url')==url,service+' settings readback failed')
    require(key not in json.dumps(readback),service+' API key leaked in settings response')
for service,path,library_path in [('Radarr','/api/radarr/status','/api/radarr/movies'),('Sonarr','/api/sonarr/status','/api/sonarr/series')]:
    code,status=call(path)
    require(code==200 and status.get('appName','').lower()==service.lower(),service+' status did not identify the connected service')
    code,library=call(library_path)
    require(code==200 and isinstance(library,list),service+' library API did not return a list')
    print('PASS fossil saved encrypted '+service+' settings; status and library APIs succeeded')
require(os.path.isfile('/shared/config.aged'),'encrypted shared settings file was not persisted')
'''


def configure_and_check(session: str, radarr_key: str, sonarr_key: str) -> None:
    payload = json.dumps({"session": session, "radarrKey": radarr_key, "sonarrKey": sonarr_key})
    result = subprocess.run(
        ["docker", "--context", "orbstack", "exec", "-i", FOSSIL, "python3", "-c", APP_FLOW],
        input=payload,
        capture_output=True,
        text=True,
        check=False,
    )
    if result.returncode:
        raise RuntimeError("fossil Arr settings integration failed: " + result.stderr.strip())
    print(result.stdout.strip())


def verify_tls_from_fossil() -> None:
    for host in ("radarr.test", "sonarr.test"):
        docker(
            "exec", FOSSIL, "curl", "--fail", "--silent", "--show-error", "--cacert",
            "/run/secrets/arr_test_ca", f"https://{host}/ping",
        )


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--start", action="store_true", help="build and start the isolated project services before checking")
    parser.add_argument("--ports-only", action="store_true", help="inspect subscriber container port bindings in the isolated environment")
    args = parser.parse_args()
    if docker("context", "show") != "orbstack":
        raise RuntimeError("Docker context must be orbstack")
    if args.ports_only:
        assert_no_host_port_bindings(("tailscale-a", "metadata-a", "tailscale-b", "metadata-b"))
        return 0
    if args.start:
        existing = docker(
            "inspect", "--format", "{{json .NetworkSettings.Ports}}",
            f"{PROJECT}-tailscale-coordinator-1", check=False,
        )
        existing_ports = json.loads(existing or "{}") or {}
        existing_bindings = existing_ports.get("8080/tcp") or []
        port = int(existing_bindings[0]["HostPort"]) if existing_bindings and existing_bindings[0].get("HostPort") else available_loopback_port()
        os.environ["RAWARR_TEST_COORDINATOR_HOST_PORT"] = str(port)
    ensure_test_certificates()
    compose = ["compose", "-p", PROJECT, "-f", COMPOSE]
    docker(*compose, "config", "--quiet")
    if args.start:
        docker(*compose, "build", "roarlay")
        docker(*compose, "up", "-d", "radarr", "sonarr", "arr-proxy", "postgres", "headscale", "tailscale-coordinator", "roarlay")
        wait_healthy("roarlay")
        wait_healthy("headscale")
        tailscale_a_key, tailscale_b_key = prepare_headscale_auth_keys()
        fingerprint = ensure_test_signing_root()
        environment = os.environ.copy()
        environment["RAWARR_INSTANCE_ROOT_FINGERPRINT"] = fingerprint
        environment["RAWARR_TEST_TAILSCALE_A_KEY_FILE"] = str(tailscale_a_key)
        environment["RAWARR_TEST_TAILSCALE_B_KEY_FILE"] = str(tailscale_b_key)
        docker(*compose, "up", "-d", "tailscale-a", "metadata-a", "tailscale-b", "metadata-b", env=environment)
    for service in ("radarr", "sonarr"):
        wait_healthy(service)
    assert_no_host_port_bindings(("tailscale-a", "metadata-a", "tailscale-b", "metadata-b"))
    wait_healthy("metadata-a")
    wait_healthy("metadata-b")
    verify_tls_from_fossil()
    user_id, session_digest, session = seed_session()
    try:
        configure_and_check(session, api_key("radarr"), api_key("sonarr"))
    finally:
        remove_session(user_id, session_digest)
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (OSError, RuntimeError, subprocess.SubprocessError, TimeoutError) as error:
        print(f"FAIL OrbStack Arr integration: {error}", file=sys.stderr)
        raise SystemExit(1)
