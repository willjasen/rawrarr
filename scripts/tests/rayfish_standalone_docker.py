#!/usr/bin/env python3
"""Verify real signed nodes with Rayfish and HTTPS, without any Tailscale services."""

import argparse
import base64
import hashlib
import json
from pathlib import Path
import secrets
import signal
import tempfile
import uuid

from test_orbstack_arr_connectivity import docker, run_openssl
from test_rayfish_transport import transport
from rayfish_docker import Node, RAYFISH_STACK_LABEL, retry


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
                          "subjectAltName=DNS:coordinator.test\n")
    run_openssl("req", "-new", "-newkey", "rsa:2048", "-nodes", "-subj", "/CN=coordinator.test",
                "-keyout", str(key), "-out", str(csr))
    run_openssl("x509", "-req", "-in", str(csr), "-days", "1", "-CA", str(ca), "-CAkey", str(ca_key),
                "-CAcreateserial", "-extfile", str(extensions), "-out", str(certificate))
    ca_key.chmod(0o600)
    key.chmod(0o600)
    owner = uuid.uuid4()

    def create(label, image, *options, command=(), alias=None):
        name = f"{prefix}-{label}"
        args = ["create", "--name", name, "--pull", "never", "--network", prefix,
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
        caddyfile = directory / "Caddyfile"
        caddyfile.write_text(
            "{\n auto_https disable_redirects\n servers {\n protocols h1 h2\n }\n}\n"
            "https://coordinator.test {\n tls /run/cert.pem /run/key.pem\n reverse_proxy roarlay:8080\n}\n"
        )
        https = create("https", "caddy:2-alpine", "--mount", bind(caddyfile, "/etc/caddy/Caddyfile"),
                       "--mount", bind(certificate, "/run/cert.pem"), "--mount", bind(key, "/run/key.pem"),
                       alias="coordinator.test")
        docker("start", https)
        retry(lambda: docker("exec", coordinator, "curl", "--max-time", "2", "-fsS", "--cacert",
                             "/run/coordinator-ca", origin + "/health"), timeout=30)
        discovery = Node(prefix, "discovery", prefix)
        peers.append(discovery)
        discovery.create()
        docker("network", "disconnect", prefix, discovery.name)
        docker("network", "connect", "--alias", "discovery", prefix, discovery.name)
        docker("start", discovery.name)
        route_peer = Node(prefix, "coordinator-peer", prefix, coordinator)
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
                "RAWARR_COORDINATOR_URL": origin, "RAWARR_NODE_TOKEN_FILE": "/run/node-token",
                "RAWARR_INSTANCE_ROOT_FINGERPRINT": records["fingerprint"], "SSL_CERT_FILE": "/run/coordinator-ca",
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
            peer = Node(prefix, f"peer-{label}", prefix, node_name)
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

        token = secrets.token_hex(32)
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
            sql(session_statement(hashlib.sha256(node_token.encode()).hexdigest(), node["nodeId"]))
            baseline = request(fossil, "http://127.0.0.1:8080" + path, node_token)
            if baseline[0] != 200 or not baseline[1].startswith("age1"):
                raise RuntimeError(f"Fossil baseline did not return its recipient: HTTP {baseline[0]}")
            route = origin + f"/nodes/{node['nodeId']}" + path
            retry(lambda: check_route(route, baseline))
            if request(coordinator, route)[0] != 401:
                raise RuntimeError("Anonymous HTTPS request was not denied")
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
        print("PASS: two signed fossils register over verified HTTPS and relay distinct authenticated responses over Rayfish")
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
