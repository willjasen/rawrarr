#!/usr/bin/env python3
"""Exercise a current coordinator binary through Rayfish and a real Tailscale SOCKS proxy."""

import argparse
import hashlib
from http.cookies import SimpleCookie
import json
from pathlib import Path
import secrets
import sys
import uuid

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "scripts"))
import rayfish_docker as fixture
import rayfish_transport as transport


def main(binary):
    transport.require(binary.is_file(), "Provide a built current-worktree coordinator binary")
    transport.require(fixture.docker("context", "show") == "orbstack", "Docker context must be orbstack")
    project = fixture.PROJECT
    names = [project + "-" + name + "-1" for name in
             ("postgres", "tailscale-coordinator", "roarlay", "tailscale-a", "metadata-a")]
    before = {}
    for name in names:
        item, = json.loads(fixture.docker("inspect", name))
        transport.require(item["State"]["Running"] and
                          item["Config"]["Labels"]["com.docker.compose.project"] == project,
                          "Start the isolated signed-fixture environment first")
        before[name] = (item["Id"], item["State"]["StartedAt"])

    prefix = "rawrarr-test-rayfish-coordinator-" + uuid.uuid4().hex[:12]
    network = project + "_default"
    nodes = []
    containers = []
    hashes = []
    user_id = str(uuid.uuid4())
    coordinator_token, node_token, other_token = (secrets.token_hex(32) for _ in range(3))
    owner = fixture.docker("exec", names[0], "psql", "-U", "rawrarr", "-d", "rawrarr",
                           "-Atc", "SELECT owner_user_id FROM rawrarr_instance_signing_root WHERE singleton=TRUE;")
    transport.require(len(owner) == 36 and all(c in "0123456789abcdef-" for c in owner),
                      "The isolated signed fixture needs its owner")

    def sql(statement):
        return fixture.docker("exec", "-i", names[0], "psql", "-U", "rawrarr", "-d", "rawrarr",
                              "-v", "ON_ERROR_STOP=1", "-Atc", statement)

    def request(peer, port, path, cookie=None, include_cookie=False):
        headers = {"Cookie": cookie} if cookie else {}
        program = (
            "import hashlib,http.client,json\n"
            f"c=http.client.HTTPConnection('127.0.0.1',{port},timeout=15)\n"
            f"c.request('GET',{path!r},headers={headers!r})\n"
            "try:\n"
            " r=c.getresponse(); b=r.read()\n"
            f" print(json.dumps([r.status,hashlib.sha256(b).hexdigest(),len(b)]"
            f"{' + [r.getheader(\"Set-Cookie\")]' if include_cookie else ''}))\n"
            "except TimeoutError:\n"
            f" print(json.dumps({[0, '', 0, None] if include_cookie else [0, '', 0]!r}))\n"
        )
        return json.loads(fixture.docker("exec", peer.name, "python", "-c", program, timeout=20))

    def expect_response(operation, expected, message):
        response = operation()
        if response != expected:
            raise RuntimeError(message)
        return response

    def start_coordinator(name, bypass, listen, forward, socks):
        args = ["create", "--name", name, "--pull", "never",
                "--label", "com.docker.compose.project=rawrarr-test-regression",
                "--network", "container:" + names[1], "--volumes-from", names[2] + ":ro"]
        env = {
            "RAWARR_ROLE": "coordinator",
            "RUST_ARR_LISTEN_ADDR": f"127.0.0.1:{listen}",
            "RAWARR_PG_HOST": "postgres",
            "RAWARR_PG_PORT": "5432",
            "RAWARR_PG_USER": "rawrarr",
            "RAWARR_PG_DATABASE": "rawrarr",
            "RAWARR_PG_PASSWORD_FILE": "/run/secrets/postgres_password",
            "RAWARR_APPROVED_NODES_FILE": "/run/secrets/approved_nodes",
            "RAWARR_COORDINATOR_WEB_RP_ID": "localhost",
            "RAWARR_COORDINATOR_WEB_ORIGIN": f"http://localhost:{listen}",
            "RAWARR_NODE_RELAY_TARGETS": json.dumps({"test-metadata-a": f"http://127.0.0.1:{forward}"}),
            "RAWARR_NODE_DIRECT_TARGETS": "{}",
            "RAWARR_NODE_RELAY_SOCKS5_URL": f"socks5h://127.0.0.1:{socks}",
            "RAWARR_NODE_RELAY_BYPASS_LOOPBACK_PROXY": str(bypass).lower(),
        }
        for key, value in env.items():
            args.extend(["-e", key + "=" + value])
        args += ["--entrypoint", "/tmp/test-coordinator", "rawrarr-testing-base:local"]
        fixture.docker(*args)
        containers.append(name)
        fixture.docker("cp", str(binary), name + ":/tmp/test-coordinator")
        fixture.docker("start", name)
        def healthy():
            if request(coordinator, listen, "/health")[0] != 200:
                raise RuntimeError("Test coordinator is not ready")
        fixture.retry(healthy, timeout=60)

    try:
        sessions = [(coordinator_token, owner, "coordinator"),
                    (node_token, owner, "test-metadata-a"),
                    (other_token, user_id, "coordinator")]
        hashes = [hashlib.sha256(token.encode()).hexdigest() for token, _, _ in sessions]
        statement = (f"INSERT INTO rawrarr_users(id,username,display_name) VALUES "
                     f"('{user_id}'::uuid,'rayfish_{user_id.replace('-', '')}','Disposable Rayfish test');\n")
        for digest, (_, user, node_id) in zip(hashes, sessions):
            statement += ("INSERT INTO rawrarr_sessions(token_hash,user_id,expires_at,node_id) "
                          f"VALUES(decode('{digest}','hex'),'{user}'::uuid,now()+interval '10 minutes','{node_id}');\n")
        sql("BEGIN;\n" + statement + "COMMIT;\n")

        discovery = fixture.Node(prefix, "discovery", network)
        nodes.append(discovery)
        discovery.create()
        fixture.docker("network", "disconnect", network, discovery.name)
        fixture.docker("network", "connect", "--alias", "discovery", network, discovery.name)
        fixture.docker("start", discovery.name)
        coordinator = fixture.Node(prefix, "coordinator", network, names[1])
        coordinator.config["role"] = "coordinator"
        fossil = fixture.Node(prefix, "fossil", network, names[3])
        for peer in (coordinator, fossil):
            nodes.append(peer)
            peer.create()
            peer.start()
        coordinator.cli("create", "--closed", "--name", "rawrarr-pilot", "--hostname", "coordinator")
        coordinator.cli("files", "auto-accept", "rawrarr-pilot", "off")
        invite = json.loads(coordinator.cli("invite", "rawrarr-pilot", "--json"))["code"]
        fixture.retry(lambda: fossil.cli("join", invite, "--name", "rawrarr-pilot",
                                         "--hostname", "fossil", "--no-auto-accept-files"))
        coordinator_status, fossil_status = coordinator.status(), fossil.status()
        ports = json.loads(fixture.docker(
            "exec", coordinator.name, "python", "-c",
            "import json,socket\n"
            "s=[]\n"
            "for port in range(17000,18000):\n"
            " x=socket.socket()\n"
            " try: x.bind(('127.0.0.1',port))\n"
            " except OSError: x.close(); continue\n"
            " s.append(x)\n"
            " if len(s)==3: break\n"
            "assert len(s)==3\n"
            "print(json.dumps([x.getsockname()[1] for x in s]))"))
        listen, forward, socks = ports
        mesh, = [entry for entry in fossil_status["networks"] if entry["name"] == "rawrarr-pilot"]
        coordinator.stop()
        fossil.stop()
        coordinator.config["forwards"] = [{"node_id": "test-metadata-a", "peer_id": fossil_status["endpoint"],
                                            "peer_ipv6": mesh["my_ipv6"], "local_port": forward}]
        fossil.config.update(expose_app=True, allow_in=[coordinator_status["endpoint"]])
        coordinator.update()
        fossil.update()
        coordinator.start()
        fossil.start()

        path = "/api/metadata/export/node-recipient"
        baseline = fossil.request(8080, path, token=node_token)
        transport.require(baseline[0] == 200, "Real fossil baseline failed")
        fixture.retry(lambda: expect_response(
            lambda: coordinator.request(forward, path, token=node_token), baseline,
            "Rayfish forward does not reach the fossil"))
        transport.require(coordinator.request(forward, path, authorized=False)[0] == 401,
                          "The fossil accepted an anonymous Rayfish request")

        outsider = fixture.Node(prefix, "outsider", network)
        nodes.append(outsider)
        outsider.create()
        outsider.start()
        invite = json.loads(coordinator.cli("invite", "rawrarr-pilot", "--json"))["code"]
        fixture.retry(lambda: outsider.cli("join", invite, "--name", "rawrarr-pilot",
                                           "--hostname", "outsider", "--no-auto-accept-files"))
        outsider_status = outsider.status()
        outsider.stop()
        outsider.config["forwards"] = [{"node_id": "test-metadata-a", "peer_id": fossil_status["endpoint"],
                                         "peer_ipv6": mesh["my_ipv6"], "local_port": forward}]
        outsider.update()
        outsider.start()
        try:
            outsider.request(forward, path, token=node_token)
        except RuntimeError:
            pass
        else:
            raise RuntimeError("An admitted but unapproved peer reached the real fossil")
        for peer, remote, direction, protocol, expected in (
            (fossil, coordinator_status["endpoint"], "in", "tcp:8080", "allow"),
            (fossil, coordinator_status["endpoint"], "in", "tcp:6379", "deny"),
            (fossil, coordinator_status["endpoint"], "in", "tcp:5432", "deny"),
            (fossil, coordinator_status["endpoint"], "in", "udp:8080", "deny"),
            (fossil, outsider_status["endpoint"], "in", "tcp:8080", "deny"),
            (coordinator, fossil_status["endpoint"], "in", "tcp:8080", "deny"),
            (coordinator, fossil_status["endpoint"], "out", "tcp:8080", "allow"),
            (coordinator, fossil_status["endpoint"], "out", "tcp:6379", "deny"),
            (coordinator, fossil_status["endpoint"], "out", "udp:8080", "deny"),
        ):
            verdict = json.loads(peer.cli("firewall", "test", remote, protocol, "--direction", direction,
                                         "--network", "rawrarr-pilot", "--json"))
            transport.require(verdict["action"] == expected,
                              f"Unexpected {direction} {protocol} Rayfish ACL verdict: {verdict['action']}")

        socks_name = prefix + "-tailscale-socks"
        fixture.docker("create", "--name", socks_name, "--pull", "never", "--cap-drop", "ALL",
                       "--security-opt", "no-new-privileges:true",
                       "--label", "com.docker.compose.project=rawrarr-test-regression",
                       "--network", "container:" + names[1], "--entrypoint", "/usr/local/bin/tailscaled",
                       "tailscale/tailscale:stable", "--tun=userspace-networking",
                       "--state=mem:", "--socket=/tmp/isolated-test.sock",
                       f"--socks5-server=127.0.0.1:{socks}", "--no-logs-no-support")
        containers.append(socks_name)
        fixture.docker("start", socks_name)
        handshake = ("import socket; s=socket.create_connection(('127.0.0.1'," + str(socks) + "),2); "
                     "s.sendall(b'\\x05\\x01\\x00'); assert s.recv(2)==b'\\x05\\x00'")
        fixture.retry(lambda: fixture.docker("exec", coordinator.name, "python", "-c", handshake), timeout=30)

        cookie = f"rawrarr_session={coordinator_token}; rawrarr_node_session={node_token}"
        route = "/nodes/test-metadata-a" + path
        disabled = prefix + "-disabled"
        start_coordinator(disabled, False, listen, forward, socks)
        active_status = request(coordinator, listen, route, cookie)[0]
        transport.require(active_status in (200, 502), f"Unexpected SOCKS route status: {active_status}")
        fixture.docker("stop", "-t", "10", socks_name)
        disabled_status = request(coordinator, listen, route, cookie)[0]
        transport.require(disabled_status == 502,
                          f"Disabled bypass with SOCKS stopped returned HTTP {disabled_status}, expected 502")
        fixture.docker("stop", "-t", "10", disabled)
        enabled = prefix + "-enabled"
        start_coordinator(enabled, True, listen, forward, socks)
        fixture.retry(lambda: expect_response(
            lambda: request(coordinator, listen, route, cookie), baseline,
            "Authenticated coordinator response differs from the real fossil with SOCKS stopped"))
        fixture.docker("start", socks_name)
        fixture.retry(lambda: fixture.docker("exec", coordinator.name, "python", "-c", handshake), timeout=30)
        fixture.retry(lambda: expect_response(
            lambda: request(coordinator, listen, route, cookie), baseline,
            "Authenticated coordinator response differs from the real fossil with SOCKS running"))
        issued = request(coordinator, listen, route, f"rawrarr_session={coordinator_token}", include_cookie=True)
        parsed = SimpleCookie()
        if issued[3]:
            parsed.load(issued[3])
            if "rawrarr_node_session" in parsed:
                hashes.append(hashlib.sha256(parsed["rawrarr_node_session"].value.encode()).hexdigest())
        transport.require(issued[:3] == baseline and issued[3] and "rawrarr_node_session" in parsed,
                          "Coordinator did not issue a node-scoped session")
        transport.require(sql("SELECT count(*) FROM rawrarr_sessions "
                              f"WHERE token_hash=decode('{hashes[-1]}','hex') AND user_id='{owner}'::uuid "
                              "AND node_id='test-metadata-a';") == "1",
                          "Delegated session has the wrong owner or node")
        transport.require(request(coordinator, listen, route)[0] == 401, "Anonymous coordinator request was accepted")
        transport.require(request(coordinator, listen, route, f"rawrarr_session={other_token}")[0] == 404,
                          "An unrelated user accessed the fossil")
        transport.require(request(coordinator, 8080, "/health")[0] == 200,
                          "Existing coordinator ingress stopped responding")
        fossil.stop()
        outage = request(coordinator, listen, route, cookie)
        transport.require(outage[0] in (0, 502), "Rayfish outage silently used another route")
        print("PASS: real coordinator delegates authenticated requests over Rayfish with Tailscale SOCKS configured")
        print("PASS: bypass off requires SOCKS; bypass on works with it stopped or running")
        print(f"INFO: bypass off with this running Tailscale SOCKS returned HTTP {active_status}")
        print("PASS: pinned peer/port ACL verdicts and actual admitted-but-unapproved peer denial")
        print("PASS: anonymous and other-user requests are denied, outage does not fall back")
        print("INFO: outage returned " + ("HTTP 502" if outage[0] == 502 else "a 15-second client timeout"))
        print("PASS: existing isolated coordinator ingress remains healthy")
    finally:
        errors = []
        for name in reversed(containers):
            try:
                fixture.docker("rm", "-f", name)
            except (RuntimeError, ValueError) as error:
                errors.append(str(error))
        for node in reversed(nodes):
            try:
                node.cleanup()
            except (RuntimeError, ValueError) as error:
                errors.append(str(error))
        if hashes:
            try:
                values = ",".join(f"decode('{digest}','hex')" for digest in hashes)
                sql(f"BEGIN; DELETE FROM rawrarr_sessions WHERE token_hash IN ({values}); "
                    f"DELETE FROM rawrarr_users WHERE id='{user_id}'::uuid; COMMIT;")
            except RuntimeError as error:
                errors.append(str(error))
        for name, expected in before.items():
            try:
                item, = json.loads(fixture.docker("inspect", name))
                transport.require((item["Id"], item["State"]["StartedAt"]) == expected and item["State"]["Running"],
                                  f"Existing test service changed: {name}")
            except RuntimeError as error:
                errors.append(str(error))
        if errors:
            raise RuntimeError("Test cleanup or preservation failed: " + "; ".join(errors))


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=Path, required=True,
                        help="Linux-native coordinator binary built from the current worktree")
    args = parser.parse_args()
    try:
        main(args.binary)
    except (OSError, RuntimeError, ValueError) as error:
        print(f"FAIL coordinator Rayfish integration: {error}", file=sys.stderr)
        sys.exit(1)
