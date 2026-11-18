#!/usr/bin/env python3
"""Verify real signed nodes with Rayfish and HTTPS, without any Tailscale services."""

import argparse
import base64
import hashlib
import json
from pathlib import Path
import secrets
import signal
import subprocess
import tempfile
import uuid

from test_orbstack_arr_connectivity import APP_FLOW, container_api_key, docker, run_openssl
from test_rayfish_transport import transport
from rayfish_docker import Node, RAYFISH_STACK_LABEL, retry
from rayfish_relay_docker import IMAGE as RELAY_IMAGE
from rayfish_syncthing_fixture import FORWARD_PORT, start as start_syncthing, verify_transfer

REPAIR_PORT = 17443


CIPHERTEXT_DIGEST = """
import hashlib, ssl, sys, urllib.request
request = urllib.request.Request(sys.argv[1], headers={"Cookie": "rawrarr_session=" + sys.stdin.read()})
with urllib.request.urlopen(request, timeout=12, context=ssl.create_default_context(cafile="/run/coordinator-ca")) as response:
    ciphertext = response.read(16385)
if len(ciphertext) > 16384 or not ciphertext.startswith(b"age-encryption.org/v1\\n"):
    raise SystemExit("Expected a bounded binary age config export")
print(hashlib.sha256(ciphertext).hexdigest())
"""


def run(binary: Path, directory: Path) -> None:
    if not binary.is_file() or docker("context", "show") != "orbstack":
        raise RuntimeError("Provide the native current-worktree binary and use OrbStack")
    prefix = "rawrarr-test-rayfish-standalone-" + uuid.uuid4().hex[:8]
    containers, peers, fossils = [], [], []
    image = "rawrarr:orbstack-test"
    records = json.loads(docker(
        "run", "--rm", "--pull", "never", "--label", RAYFISH_STACK_LABEL,
        "--network", "none", "--workdir", "/opt/rawrarr-web-export",
        "--entrypoint", "node", "rawrarr-testing-base:local", "--input-type=module", "-e",
        (Path(__file__).parent / "helpers/environment_signing_fixture.mjs").read_text(),
    ))
    tokens = [secrets.token_hex(32) for _ in records["nodes"]]
    origin = "https://coordinator.test"
    ca, ca_key = directory / "ca.pem", directory / "ca-key.pem"
    certificate, key, csr = directory / "cert.pem", directory / "key.pem", directory / "server.csr"
    run_openssl("req", "-x509", "-newkey", "rsa:2048", "-nodes", "-days", "1",
                "-subj", "/CN=Disposable Rawrarr Test CA",
                "-addext", "basicConstraints=critical,CA:TRUE", "-keyout", str(ca_key), "-out", str(ca))
    extensions = directory / "server.ext"
    extensions.write_text("basicConstraints=critical,CA:FALSE\nextendedKeyUsage=serverAuth\n"
                          "subjectAltName=DNS:coordinator.test,DNS:radarr.test,DNS:sonarr.test\n")
    run_openssl("req", "-new", "-newkey", "rsa:2048", "-nodes", "-subj", "/CN=coordinator.test",
                "-keyout", str(key), "-out", str(csr))
    run_openssl("x509", "-req", "-in", str(csr), "-days", "1", "-CA", str(ca), "-CAkey", str(ca_key),
                "-CAcreateserial", "-extfile", str(extensions), "-out", str(certificate))
    ca_key.chmod(0o600)
    key.chmod(0o600)
    owner = uuid.uuid4()

    def create(label, image, *options, command=(), alias=None, namespace=None):
        name = f"{prefix}-{label}"
        args = ["create", "--name", name, "--pull", "never",
                "--network", f"container:{namespace}" if namespace else prefix,
                "--label", RAYFISH_STACK_LABEL,
                "--label", f"com.docker.compose.service={label}"]
        if alias:
            args += ["--network-alias", alias]
        docker(*args, *options, image, *command)
        containers.append(name)
        return name

    def bind(path, target):
        return f"type=bind,source={path},target={target},readonly"

    def sql(statement):
        return docker("exec", "-i", database, "psql", "-XAt", "-U", "rawrarr", "-d", "rawrarr",
                      "-v", "ON_ERROR_STOP=1", input=statement)

    def start_ready(name):
        docker("start", name)

        def healthy():
            state = json.loads(docker("inspect", "--format", "{{json .State}}", name))
            if not state["Running"]:
                # Only test-app startup logs; never print database or Rayfish state.
                raise ValueError(f"{name} exited: {docker('logs', '--tail', '8', name, check=False)}")
            docker("exec", name, "curl", "--max-time", "2", "-fsS", "http://127.0.0.1:8080/health")
        retry(healthy, timeout=45)

    docker("network", "create", "--label", RAYFISH_STACK_LABEL, prefix)
    try:
        password = directory / "pg-password"
        password.write_text(secrets.token_hex(32))
        database = create("postgres", "postgres:17-alpine", "--tmpfs", "/var/lib/postgresql/data",
                          "--mount", bind(password, "/run/pg-password"), "-e", "POSTGRES_USER=rawrarr",
                          "-e", "POSTGRES_PASSWORD_FILE=/run/pg-password", alias="postgres")
        docker("start", database)
        retry(lambda: docker("exec", database, "pg_isready", "-h", "127.0.0.1", "-U", "rawrarr"))
        registry = directory / "registry.json"
        registry.write_text(json.dumps([
            dict(id=node["nodeId"], role="metadata", rp_id="coordinator.test", origin=origin,
                 token_sha256=hashlib.sha256(token.encode()).hexdigest())
            for node, token in zip(records["nodes"], tokens)
        ]))
        runtime = ("--mount", bind(binary, "/tmp/test-rawrarr"), "--entrypoint", "/tmp/test-rawrarr")
        targets = {node["nodeId"]: f"http://127.0.0.1:{port}/"
                   for node, port in zip(records["nodes"], (17297, 17298))}
        coordinator = create(
            "roarlay", image, *runtime, "--mount", bind(password, "/run/pg-password"),
            "--mount", bind(registry, "/run/registry"), "--mount", bind(ca, "/run/coordinator-ca"),
            "-e", "RAWARR_ROLE=coordinator",
            "-e", "RAWARR_PG_HOST=postgres", "-e", "RAWARR_PG_PASSWORD_FILE=/run/pg-password",
            "-e", "RAWARR_APPROVED_NODES_FILE=/run/registry", "-e", f"RAWARR_COORDINATOR_WEB_ORIGIN={origin}",
            "-e", "RAWARR_COORDINATOR_WEB_RP_ID=coordinator.test", "-e", "RAWARR_NODE_DIRECT_TARGETS={}",
            "-e", "RAWARR_NODE_RELAY_TARGETS=" + json.dumps(targets), alias="roarlay")
        start_ready(coordinator)
        statement = (
            f"BEGIN;\nINSERT INTO rawrarr_users(id,username,display_name) VALUES ('{owner}','fixture','Fixture');\n"
            "INSERT INTO rawrarr_instance_signing_root(singleton,algorithm,public_key,fingerprint,owner_user_id) "
            f"VALUES (true,'ML-DSA-65',decode('{records['publicKey']}','base64'),'{records['fingerprint']}','{owner}');\n"
        )
        for node in records["nodes"]:
            statement += (
                "UPDATE rawrarr_nodes SET "
                f"user_id='{owner}', signing_public_key=decode('{node['publicKeyHex']}','hex'),"
                f"signing_fingerprint='{node['fingerprint']}',signing_approval_payload='{json.dumps(node['payload'])}'::jsonb,"
                f"signing_approval_signature=decode('{node['signature']}','base64') WHERE id='{node['nodeId']}';\n"
            )
        sql(statement + "COMMIT;")
        docker("stop", "-t", "5", coordinator)
        start_ready(coordinator)
        arr = {"radarr": 7878, "sonarr": 8989}
        arr_containers = {
            service: create(service, f"lscr.io/linuxserver/{service}:latest", alias=service)
            for service in arr
        }
        for name in arr_containers.values():
            docker("start", name)

        caddyfile = directory / "Caddyfile"
        caddyfile.write_text(
            "{\n auto_https disable_redirects\n servers {\n protocols h1 h2\n }\n}\n"
            "https://coordinator.test {\n tls /run/cert.pem /run/key.pem\n reverse_proxy roarlay:8080\n}\n"
            + "".join(f"https://{service}.test {{\n tls /run/cert.pem /run/key.pem\n"
                      f" reverse_proxy {service}:{port}\n}}\n" for service, port in arr.items())
        )
        https = create("https", "caddy:2-alpine", "--mount", bind(caddyfile, "/etc/caddy/Caddyfile"),
                       "--mount", bind(certificate, "/run/cert.pem"), "--mount", bind(key, "/run/key.pem"),
                       "--network-alias", "radarr.test", "--network-alias", "sonarr.test",
                       alias="coordinator.test")
        docker("start", https)
        retry(lambda: docker("exec", coordinator, "curl", "--max-time", "2", "-fsS", "--cacert",
                             "/run/coordinator-ca", origin + "/health"), timeout=30)
        for service in arr:
            retry(lambda: docker("exec", coordinator, "curl", "--max-time", "2", "-fsS", "--cacert",
                                 "/run/coordinator-ca", f"https://{service}.test/ping"), timeout=120)
        arr_keys = {service + "Key": container_api_key(name) for service, name in arr_containers.items()}

        discovery = Node(prefix, "discovery", prefix)
        peers.append(discovery)
        discovery.create()
        docker("network", "disconnect", prefix, discovery.name)
        docker("network", "connect", "--alias", "discovery", prefix, discovery.name)
        docker("start", discovery.name)
        relay_config = directory / "relay.toml"
        relay_config.write_text('http_bind_addr = "0.0.0.0:3340"\nenable_quic_addr_discovery = false\n')
        relay = create("iroh-relay", RELAY_IMAGE, "--read-only", "--cap-drop", "ALL",
                       "--security-opt", "no-new-privileges:true",
                       "--mount", bind(relay_config, "/run/relay.toml"), alias="relay",
                       command=("--dev", "--config-path", "/run/relay.toml"))
        docker("start", relay)
        relay_url = "http://relay:3340"
        route_peer = Node(prefix, "rayfish-coord", prefix, coordinator, relay_url=relay_url)
        route_peer.config["role"] = "coordinator"
        peers.append(route_peer)
        route_peer.create()
        route_peer.start()
        route_peer.cli("create", "--closed", "--name", "rawrarr-pilot", "--hostname", "coordinator-peer")
        route_peer.cli("files", "auto-accept", "rawrarr-pilot", "off")
        endpoint = route_peer.status()["endpoint"]

        for label, node, token, port in zip(("a", "b"), records["nodes"], tokens, (17297, 17298)):
            token_file, seed_file = directory / f"token-{label}", directory / f"seed-{label}"
            token_file.write_text(token)
            seed_file.write_bytes(b"RAWARR-NODE-SIGNING-KEY\x00\x01" + base64.b64decode(node.pop("seed"), validate=True))
            # The disposable app can read its bind-mounted credentials, but Rayfish never mounts them.
            env = {
                "RAWARR_ROLE": "metadata", "RAWARR_NODE_ID": node["nodeId"],
                "RAWARR_DATA_DIR": "/data", "RAWARR_SHARED_DATA_DIR": "/shared",
                "RAWARR_COORDINATOR_URL": origin, "RAWARR_NODE_TOKEN_FILE": "/run/node-token",
                "RAWARR_INSTANCE_ROOT_FINGERPRINT": records["fingerprint"], "SSL_CERT_FILE": "/run/coordinator-ca",
                "RAWARR_ARR_CA_CERT_FILE": "/run/coordinator-ca", "RAWARR_LOCAL_PLAINTEXT_CONFIG": "false",
                "RAWARR_CONFIG_PEER_RAYFISH_HTTP": "true",
                "RAWARR_CONFIG_PEERS": json.dumps({record["nodeId"]: f"http://127.0.0.1:{REPAIR_PORT}"
                                                   for record in records["nodes"] if record != node}),
                "RUST_ARR_LISTEN_ADDR": "127.0.0.1:8080", "RUST_ARR_HEALTH_ADDR": "127.0.0.1:7299",
            }
            node_name = create(
                f"metadata-{label}", image, *runtime, "--mount", bind(token_file, "/run/node-token"),
                "--mount", bind(seed_file, "/data/node-signing-key"), "--mount", bind(ca, "/run/coordinator-ca"),
                "--tmpfs", "/shared:uid=10001,gid=10001,mode=0700",
                *(item for pair in env.items() for item in ("-e", "=".join(pair))),
            )
            start_ready(node_name)
            fossils.append(node_name)
            peer = Node(prefix, f"peer-{label}", prefix, node_name, relay_url=relay_url)
            peers.append(peer)
            peer.create()
            peer.start()
            invite = json.loads(route_peer.cli("invite", "rawrarr-pilot", "--json"))["code"]
            retry(lambda: peer.cli("join", invite, "--name", "rawrarr-pilot", "--hostname", peer.label,
                                   "--no-auto-accept-files"))
            status = peer.status()
            mesh, = status["networks"]
            route_peer.config.setdefault("forwards", []).append({
                "node_id": node["nodeId"], "peer_id": status["endpoint"],
                "peer_ipv6": mesh["my_ipv6"], "local_port": port,
            })
            peer.stop()
            peer.config.update(expose_app=True, allow_in=[endpoint])
            peer.update()
            peer.start()
        route_peer.stop()
        route_peer.update()
        route_peer.start()

        services = json.loads(docker("inspect", coordinator, *fossils))
        if [service["Config"]["Labels"].get("com.docker.compose.service") for service in services] != [
            "roarlay", "metadata-a", "metadata-b",
        ]:
            raise RuntimeError("The test stack must label only the coordinator roarlay")
        if any(service["HostConfig"]["PortBindings"] for service in services):
            raise RuntimeError("The coordinator and subscribers must not publish host ports")

        token = secrets.token_hex(32)
        node_tokens = []
        config_path = "/api/metadata/export/config"
        digest = hashlib.sha256(token.encode()).hexdigest()
        def session_statement(digest, node_id):
            return ("INSERT INTO rawrarr_sessions(token_hash,user_id,expires_at,node_id) "
                    f"VALUES(decode('{digest}','hex'),'{owner}',now()+interval '10 minutes','{node_id}');")

        sql(session_statement(digest, "coordinator"))
        path = "/api/metadata/export/node-recipient"
        routes, baselines = [], []

        def request(container, url, session=None):
            config = f'cookie = "rawrarr_session={session}"\n' if session else ""
            output = docker("exec", "-i", container, "curl", "--max-time", "8", "-sS",
                            "--cacert", "/run/coordinator-ca", "--config", "-",
                            "--write-out", "\\nHTTP_STATUS=%{http_code}", url, input=config)
            body, separator, status = output.rpartition("\nHTTP_STATUS=")
            if not separator and output.startswith("HTTP_STATUS="):
                body, status = "", output.removeprefix("HTTP_STATUS=")
            return int(status), body.strip()

        def check_route(route, expected):
            actual = request(coordinator, route, token)
            if actual != expected:
                raise RuntimeError(f"Unexpected response for {route}: HTTP {actual[0]}, expected {expected[0]}")

        for node, fossil in zip(records["nodes"], fossils):
            node_token = secrets.token_hex(32)
            node_tokens.append(node_token)
            sql(session_statement(hashlib.sha256(node_token.encode()).hexdigest(), node["nodeId"]))
            baseline = request(fossil, "http://127.0.0.1:8080" + path, node_token)
            if baseline[0] != 200 or not baseline[1].startswith("age1"):
                raise RuntimeError(f"Fossil baseline did not return its recipient: HTTP {baseline[0]}")
            route = origin + f"/nodes/{node['nodeId']}" + path
            retry(lambda: check_route(route, baseline))
            if request(coordinator, route)[0] != 401:
                raise RuntimeError("Anonymous HTTPS request was not denied")
            print(docker("exec", "-i", fossil, "python3", "-c", APP_FLOW,
                         input=json.dumps({"session": node_token, **arr_keys})))
            local = docker("exec", "-i", fossil, "python3", "-c", CIPHERTEXT_DIGEST,
                           "http://127.0.0.1:8080" + config_path, input=node_token)
            forwarded = docker("exec", "-i", coordinator, "python3", "-c", CIPHERTEXT_DIGEST,
                               origin + f"/nodes/{node['nodeId']}" + config_path, input=token)
            if local != forwarded:
                raise RuntimeError("Rayfish config export differed from the local ciphertext")
            routes.append(route)
            baselines.append(baseline)
        if baselines[0] == baselines[1]:
            raise RuntimeError("The two selected routes reached the same fossil")
        route_peer.stop()
        for route, fossil in zip(routes, fossils):
            if request(coordinator, route, token)[0] != 502:
                raise RuntimeError("Removing the Rayfish forward did not produce HTTP 502")
            if request(fossil, origin + "/health")[0] != 200:
                raise RuntimeError("Node-to-coordinator HTTPS stopped working without Rayfish")
        route_peer.start()
        for route, baseline in zip(routes, baselines):
            retry(lambda: check_route(route, baseline))

        sender, dialer = peers[-2:]
        sender_status = sender.status()
        sender_mesh, = sender_status["networks"]
        authorized_peer = dialer.status()["endpoint"]
        sender.config["allow_syncthing_in"] = [authorized_peer]
        sender.config["allow_config_peer_in"] = [authorized_peer]
        forward = {
            "node_id": records["nodes"][0]["nodeId"], "peer_id": sender_status["endpoint"],
            "peer_ipv6": sender_mesh["my_ipv6"],
        }
        dialer.config["forwards"] = [
            dict(forward, local_port=FORWARD_PORT, service="syncthing"),
            dict(forward, local_port=REPAIR_PORT, service="config-peer"),
        ]
        def exported_digest(index):
            return docker("exec", "-i", fossils[index], "python3", "-c", CIPHERTEXT_DIGEST,
                          "http://127.0.0.1:8080" + config_path, input=node_tokens[index])

        original_digest = exported_digest(0)
        docker("exec", fossils[1], "mv", "/shared/config.aged", "/shared/config.fixture-backup")
        # Simulate a fresh receiver with neither a local revision journal nor a Valkey mirror.
        docker("exec", fossils[1], "sh", "-c",
               "if test -d /data/config-revisions; then mv /data/config-revisions /data/config-revisions.fixture-backup; fi")
        if request(fossils[1], "http://127.0.0.1:8080" + config_path, node_tokens[1])[0] != 404:
            raise RuntimeError("The receiver still exports config before the Rayfish route is available")
        for peer in (sender, dialer):
            peer.stop()
            peer.update()
            peer.start()
        peer_url = f"http://127.0.0.1:{REPAIR_PORT}"

        def check_config_peer():
            code, status = request(fossils[1], "http://127.0.0.1:8080/api/config-peer/status", node_tokens[1])
            if code != 200:
                raise RuntimeError(f"Config-peer status returned HTTP {code}")
            snapshot = json.loads(status)
            if snapshot.get("reachablePeers") != [records["nodes"][0]["nodeId"]]:
                raise RuntimeError(
                    f"Signed config-peer did not reach the sender (state={snapshot.get('state')}, "
                    f"configured={snapshot.get('configuredPeers')})"
                )
            if exported_digest(1) != original_digest:
                raise RuntimeError("Signed Rayfish repair did not restore the expected ciphertext")

        try:
            def check_unsigned():
                if request(fossils[1], peer_url + "/config-peer/manifest")[0] != 401:
                    raise RuntimeError("Unsigned request to the config-peer manifest was not denied")
                if request(fossils[1], peer_url + "/api/metadata/export/config")[0] != 404:
                    raise RuntimeError("The config-peer listener exposed an unrelated application route")

            retry(check_unsigned, timeout=20)
            retry(check_config_peer, timeout=100)
        except RuntimeError:
            for fossil in fossils:
                logs = subprocess.run(["docker", "--context", "orbstack", "logs", "--tail", "150", fossil],
                                      stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True, check=False)
                print("\n".join(line for line in logs.stdout.splitlines()
                                if "config-peer" in line and "/api/config-peer/status" not in line)[-3000:])
            raise
        print("PASS: signed Rayfish config-peer repair restores exact ciphertext; unsigned requests denied")
        shares = start_syncthing(directory, create, fossils)
        verify_transfer(shares, 0, "from-a.bin")
        verify_transfer(shares, 1, "from-b.bin")
        print("PASS: approved Syncthing peers exchange matching file bytes over pinned Rayfish TCP")
        print("PASS: two signed fossils register over verified HTTPS and relay distinct authenticated responses over Rayfish")
        print("PASS: both fossils use real Radarr and Sonarr and export matching config ciphertext through Rayfish")
        print("PASS: no Tailscale or Headscale in the fixture; Rayfish outage breaks relay only, and restart restores it")
    finally:
        failures = []
        for cleanup in [*(peer.cleanup for peer in reversed(peers)),
                        *(lambda name=name: docker("rm", "-fv", name) for name in reversed(containers)),
                        lambda: docker("network", "rm", prefix)]:
            try:
                cleanup()
            except RuntimeError as error:
                failures.append(str(error))
        if failures:
            raise RuntimeError("Standalone fixture cleanup failed: " + "; ".join(failures))


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=Path, required=True)
    args = parser.parse_args()

    def interrupted(*_):
        raise KeyboardInterrupt

    signal.signal(signal.SIGTERM, interrupted)
    with tempfile.TemporaryDirectory(prefix="rawrarr-rayfish-standalone-") as directory:
        run(args.binary.resolve(), Path(directory))
