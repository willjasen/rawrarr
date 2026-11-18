#!/usr/bin/env python3
"""Qualify two separate closed user meshes with one shared coordinator namespace."""

import json
from pathlib import Path
import sys
import uuid

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "scripts"))
import rayfish_docker as fixture
import rayfish_transport as transport


def named_request(peer, hostname):
    program = (
        "import hashlib,http.client,json,os,re,socket,sys\n"
        "from pathlib import Path\n"
        "state=Path(os.environ['RAYFISH_ROOTLESS_DIR'])\n"
        "log=Path('/state') / (state.name + '.log')\n"
        "matches=re.findall(r'SOCKS5 proxy: socks5h://127\\.0\\.0\\.1:(\\d+)',log.read_text())\n"
        "assert matches, 'missing local Rayfish SOCKS listener'\n"
        "password=(state/'proxy-password').read_bytes().strip()\n"
        "s=socket.create_connection(('127.0.0.1',int(matches[-1])),timeout=10)\n"
        "def read(n):\n"
        " b=b''\n"
        " while len(b)<n:\n"
        "  chunk=s.recv(n-len(b))\n"
        "  if not chunk: raise RuntimeError('SOCKS closed before its reply')\n"
        "  b+=chunk\n"
        " return b\n"
        "s.sendall(b'\\x05\\x01\\x02'); assert read(2)==b'\\x05\\x02'\n"
        "s.sendall(bytes([1,3])+b'ray'+bytes([len(password)])+password); assert read(2)==b'\\x01\\x00'\n"
        f"host={hostname!r}.encode('ascii')\n"
        "s.sendall(b'\\x05\\x01\\x00\\x03'+bytes([len(host)])+host+(8080).to_bytes(2,'big'))\n"
        "head=read(4)\n"
        "assert head[0]==5\n"
        "if head[1]: print(json.dumps([0,'',0])); sys.exit(0)\n"
        "length={1:4,4:16}.get(head[3])\n"
        "if head[3]==3: length=read(1)[0]\n"
        "assert length is not None\n"
        "read(length+2)\n"
        "s.sendall(b'GET /api/fixture HTTP/1.1\\r\\nHost: fixture\\r\\n"
        "Authorization: Bearer rawrarr-test-session\\r\\nConnection: close\\r\\n\\r\\n')\n"
        "r=http.client.HTTPResponse(s); r.begin(); body=r.read()\n"
        "print(json.dumps([r.status,hashlib.sha256(body).hexdigest(),len(body)]))\n"
    )
    return json.loads(fixture.docker("exec", peer.name, "python", "-c", program, timeout=20))


def main():
    transport.require(fixture.docker("context", "show") == "orbstack", "Docker context must be orbstack")
    prefix = "rawrarr-test-rayfish-users-" + uuid.uuid4().hex[:12]
    network = prefix
    nodes = []
    network_created = False
    try:
        fixture.docker("network", "create", "--label", fixture.RAYFISH_STACK_LABEL, network)
        network_created = True
        discovery = fixture.Node(prefix, "discovery", network)
        nodes.append(discovery)
        discovery.create()
        fixture.docker("network", "disconnect", network, discovery.name)
        fixture.docker("network", "connect", "--alias", "discovery", network, discovery.name)
        fixture.docker("start", discovery.name)

        coordinator_a = fixture.Node(prefix, "coordinator-a", network)
        coordinator_b = fixture.Node(prefix, "coordinator-b", network, coordinator_a.name)
        fossil_a = fixture.Node(prefix, "fossil-a", network)
        fossil_b = fixture.Node(prefix, "fossil-b", network)
        peers = (coordinator_a, coordinator_b, fossil_a, fossil_b)
        for peer, user in zip(peers, ("a", "b", "a", "b")):
            nodes.append(peer)
            peer.config["network"] = "user-" + user + "-rawrarr"
            if peer in (coordinator_a, coordinator_b):
                peer.config["role"] = "coordinator"
            peer.create()
            peer.start()
        for user, coordinator, fossil in (("a", coordinator_a, fossil_a), ("b", coordinator_b, fossil_b)):
            alias = "user-" + user + "-rawrarr"
            coordinator.cli("create", "--closed", "--name", alias, "--hostname", "coordinator-" + user)
            coordinator.cli("files", "auto-accept", alias, "off")
            invite = json.loads(coordinator.cli("invite", alias, "--json"))["code"]
            fixture.retry(lambda: fossil.cli("join", invite, "--name", alias, "--hostname", "fossil-" + user,
                                             "--no-auto-accept-files"))

        status = {peer.name: peer.status() for peer in peers}
        for peer in peers:
            transport.require([entry["name"] for entry in status[peer.name]["networks"]] == [peer.config["network"]],
                              "A user peer joined an unexpected network")
        for peer in peers:
            peer.stop()
        for coordinator, fossil, port in ((coordinator_a, fossil_a, 17297), (coordinator_b, fossil_b, 17298)):
            coord_status, fossil_status = status[coordinator.name], status[fossil.name]
            mesh, = fossil_status["networks"]
            coordinator.config["forwards"] = [{
                "node_id": fossil.label, "peer_id": fossil_status["endpoint"],
                "peer_ipv6": mesh["my_ipv6"], "local_port": port,
            }]
            fossil.config.update(expose_app=True, allow_in=[coord_status["endpoint"]])
        # Deliberately misroute user A to B. Membership and B's ACL must both
        # prevent access, even if a local route is configured to B's address.
        mesh_b, = status[fossil_b.name]["networks"]
        coordinator_a.config["forwards"].append({
            "node_id": "wrong-user", "peer_id": status[fossil_b.name]["endpoint"],
            "peer_ipv6": mesh_b["my_ipv6"], "local_port": 17299,
        })
        for peer in peers:
            peer.update()
            peer.start()

        for coordinator, fossil, port in ((coordinator_a, fossil_a, 17297), (coordinator_b, fossil_b, 17298)):
            baseline = fossil.request(8080)
            def same_response():
                if coordinator.request(port) != baseline or baseline[0] != 200:
                    raise RuntimeError("The user's own mesh did not reach its fossil")
            fixture.retry(same_response)
            transport.require(coordinator.request(port, authorized=False)[0] == 401,
                              "A mesh bypassed application authentication")
            hostname = f"{fossil.label}.{coordinator.config['network']}.ray"
            def same_named_response():
                if named_request(coordinator, hostname) != baseline:
                    raise RuntimeError("The user's .ray name did not reach its fossil")
            fixture.retry(same_named_response)
        transport.require(named_request(coordinator_a, "fossil-b.user-b-rawrarr.ray")[0] == 0,
                          "A user-A proxy resolved and reached user B's private .ray name")
        try:
            coordinator_a.request(17299)
        except RuntimeError:
            pass
        else:
            raise RuntimeError("A misrouted user-A peer reached user B's fossil")
        transport.require(coordinator_a.request(17297)[0] == 200 and coordinator_b.request(17298)[0] == 200,
                          "A user's own network stopped working after the cross-user attempt")
        print("PASS: two closed user meshes coexist in one coordinator network namespace")
        print("PASS: each user reaches only its own pinned peer; a cross-user misroute is denied")
        print("PASS: authenticated rootless proxies resolve each user's .ray name but not the other user's name")
        print("PASS: application authentication remains independent of network membership")
    finally:
        errors = []
        for node in reversed(nodes):
            try:
                node.cleanup()
            except RuntimeError as error:
                errors.append(str(error))
        if network_created:
            try:
                fixture.docker("network", "rm", network)
            except RuntimeError as error:
                errors.append(str(error))
        if errors:
            raise RuntimeError("Test cleanup failed: " + "; ".join(errors))


if __name__ == "__main__":
    try:
        main()
    except (OSError, RuntimeError, ValueError) as error:
        print(f"FAIL Rayfish user isolation: {error}", file=sys.stderr)
        sys.exit(1)
