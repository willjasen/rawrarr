#!/usr/bin/env python3
"""Verify private relay-only traffic between peers on isolated Docker bridges."""

import argparse
import hashlib
import json
from pathlib import Path
import re
import sys
import tempfile
import time
import uuid

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import rayfish_transport as transport
from rayfish_docker import Node, docker, retry

IMAGE = "rawrarr-test-rayfish-relay:1.3.0"


def restrict_peer_egress(peer, destinations=None, replace=False):
    # A test helper owns packet filters only in this disposable namespace.
    # The Rayfish process itself remains UID 10001 with all capabilities dropped.
    helper = ["run", "--rm", "--pull", "never", "--network", "container:" + peer.name,
              "--cap-drop", "ALL", "--cap-add", "NET_ADMIN", "--user", "0:0",
              "--read-only", "--entrypoint", "iptables", IMAGE]
    if destinations is None:
        destinations = [
            (docker("exec", peer.name, "python", "-c",
                    f"import socket;print(socket.gethostbyname({hostname!r}))"), port)
            for hostname, port in (("relay", 3340), ("discovery", 18090))
        ]
    if replace:
        docker(*helper, "-F", "OUTPUT")
        ipv6_helper = helper.copy()
        ipv6_helper[-2] = "ip6tables"
        docker(*ipv6_helper, "-F", "OUTPUT")
    docker(*helper, "-A", "OUTPUT", "-d", "127.0.0.0/8", "-j", "ACCEPT")
    for address, port in destinations:
        docker(*helper, "-A", "OUTPUT", "-d", address, "-p", "tcp", "--dport", str(port), "-j", "ACCEPT")
    docker(*helper, "-A", "OUTPUT", "-j", "REJECT")
    helper[-2] = "ip6tables"
    docker(*helper, "-A", "OUTPUT", "-d", "::1/128", "-j", "ACCEPT")
    docker(*helper, "-A", "OUTPUT", "-j", "REJECT")
    return destinations


def relay_bytes(relay):
    metrics = docker("exec", relay, "python", "-c",
                     "import urllib.request;print(urllib.request.urlopen('http://127.0.0.1:9090/metrics',timeout=5).read().decode())")
    values = re.findall(r"^\w*bytes_sent(?:_total)? ([0-9.e+]+)$", metrics, re.MULTILINE)
    transport.require(bool(values), "Relay fixture has no expected forwarded-byte counter")
    return sum(float(value) for value in values)


def verify_direct_isolation(first, second, network):
    target, = json.loads(docker("inspect", second.name))
    address = target["NetworkSettings"]["Networks"][network]["IPAddress"]
    transport.require(bool(address), "Direct-path probe has no current peer address")
    probe = ("import socket\ns=socket.socket();s.settimeout(3)\n"
             f"print(s.connect_ex(({address!r},18092)))\ns.close()\n")
    transport.require(second.request(18092, "/health")[0] == 200, "Isolation probe listener is not live")
    transport.require(int(docker("exec", first.name, "python", "-c", probe)) != 0,
                      "Peers permit direct connectivity; relay-only proof is invalid")


def run(discovery_outage=False):
    info, = json.loads(docker("image", "inspect", IMAGE))
    host = docker("info", "--format", "{{.OSType}}/{{.Architecture}}")
    expected = {"linux/x86_64": "amd64", "linux/aarch64": "arm64", "linux/arm64": "arm64"}.get(host)
    transport.require(info["Architecture"] == expected and info["Os"] == "linux",
                      "Private relay fixture must use the host's native architecture")
    prefix = "rawrarr-test-rayfish-relay-" + uuid.uuid4().hex[:10]
    networks, nodes = [], []
    relay = prefix + "-server"
    volume = relay + "-state"
    relay_created = volume_created = False
    try:
        for suffix in ("a", "b"):
            network = prefix + "-" + suffix
            docker("network", "create", network)
            networks.append(network)
        discovery = Node(prefix, "discovery", networks[0])
        nodes.append(discovery)
        discovery.create()
        docker("network", "disconnect", networks[0], discovery.name)
        for network in networks:
            docker("network", "connect", "--alias", "discovery", network, discovery.name)
        docker("start", discovery.name)
        docker("volume", "create", volume)
        volume_created = True
        docker("create", "--name", relay, "--pull", "never", "--read-only",
               "--cap-drop", "ALL", "--security-opt", "no-new-privileges:true",
               "--network", networks[0], "--network-alias", "relay", "-v", volume + ":/state",
               "--label", "com.docker.compose.project=rawrarr-test-regression",
               IMAGE, "--dev", "--config-path", "/state/relay.toml")
        relay_created = True
        docker("network", "connect", "--alias", "relay", networks[1], relay)
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "relay.toml"
            path.write_text('http_bind_addr = "0.0.0.0:3340"\nenable_metrics = true\n'
                            'metrics_bind_addr = "0.0.0.0:9090"\nenable_quic_addr_discovery = false\n')
            docker("cp", str(path), relay + ":/state/relay.toml")
        docker("start", relay)
        peers = []
        for label, network in zip(("fossil-a", "fossil-b"), networks):
            peer = Node(prefix, label, network, relay_url="http://relay:3340")
            peer.config["lan_discovery"] = False
            nodes.append(peer)
            peers.append(peer)
            peer.create()
            peer.start()
        first, second = peers
        for peer in peers:
            restrict_peer_egress(peer)
        verify_direct_isolation(first, second, networks[1])
        first.cli("create", "--closed", "--name", "rawrarr-pilot", "--hostname", "fossil-a")
        first.cli("files", "auto-accept", "rawrarr-pilot", "off")
        invite = json.loads(first.cli("invite", "rawrarr-pilot", "--json"))["code"]
        retry(lambda: second.cli("join", invite, "--name", "rawrarr-pilot",
                                 "--hostname", "fossil-b", "--no-auto-accept-files"))
        statuses = [peer.status() for peer in peers]
        for peer, remote in zip(peers, reversed(statuses)):
            peer.stop()
            mesh, = [item for item in remote["networks"] if item["name"] == "rawrarr-pilot"]
            peer.config.update(expose_app=True, allow_in=[remote["endpoint"]],
                               forwards=[{"node_id": "peer", "peer_id": remote["endpoint"],
                                          "peer_ipv6": mesh["my_ipv6"], "local_port": 17297}])
            peer.update()
        egress = {}
        for peer in peers:
            peer.start()
            # Docker stop/start replaces the network namespace and its filters.
            egress[peer.name] = restrict_peer_egress(peer)
        verify_direct_isolation(first, second, networks[1])
        payload_hash = hashlib.sha256(b"rawrarr-synthetic-ciphertext\n" * 32768).hexdigest()

        def verify():
            for peer in peers:
                response = peer.request(17297)
                transport.require(response == [200, payload_hash, 950272], "Relay payload changed")
                transport.require(peer.request(17297, authorized=False)[0] == 401, "Relay bypassed app authorization")
        before_bytes = relay_bytes(relay)
        retry(verify)
        transport.require(relay_bytes(relay) - before_bytes >= 2 * 950272,
                          "Relay metrics do not account for both transferred payloads")
        docker("stop", "-t", "10", relay)
        time.sleep(3)
        for peer in peers:
            try:
                response = peer.request(17297)
            except RuntimeError:
                continue
            raise RuntimeError(f"Relay outage returned HTTP {response[0]} ({response[2]} bytes); relay-only proof failed")
        docker("start", relay)
        retry(verify)
        logs = docker("logs", relay)
        for marker in ("rawrarr-synthetic-ciphertext", "rawrarr-test-session", "/api/fixture"):
            transport.require(marker not in logs, "Relay logs exposed a synthetic payload, credential, or HTTP path")
        print("PASS: isolated bridges, LAN discovery disabled, bidirectional byte-exact private-relay transfers", flush=True)
        print("PASS: relay outage fails explicitly; restoration recovers both peers and preserves app 401s", flush=True)
        print("PASS: relay byte counters cover both payloads; tested plaintext markers absent from relay logs", flush=True)
        if discovery_outage:
            docker("stop", "-t", "10", discovery.name)
            for peer in peers:
                peer.stop()
            for peer in peers:
                peer.start()
                # Docker removes discovery's DNS alias while that container is stopped.
                restrict_peer_egress(peer, egress[peer.name])
            verify_direct_isolation(first, second, networks[1])
            before_bytes = relay_bytes(relay)
            cold_recovered = True
            try:
                retry(verify)
            except RuntimeError as error:
                cold_recovered = False
                transport.require(all(peer.status()["active"] for peer in peers),
                                  "Discovery-outage restart lost local liveness")
                print("BLOCKED QUALIFICATION: private relay does not restore dual cold-start traffic "
                      "with discovery offline within 90 seconds", flush=True)
                print(f"Recovery error: {error}", flush=True)
            else:
                transport.require(relay_bytes(relay) - before_bytes >= 2 * 950272,
                                  "Cold restart traffic did not traverse the private relay")
                print("PASS: both peers cold-restart with discovery offline and the private relay available", flush=True)
            docker("start", discovery.name)
            # A restarted Docker service can receive new IPs; update only these test namespaces.
            for peer in peers:
                restrict_peer_egress(peer, replace=True)
            verify_direct_isolation(first, second, networks[1])
            before_bytes = relay_bytes(relay)
            restored_without_restart = True
            try:
                retry(verify)
            except RuntimeError as error:
                restored_without_restart = False
                print(f"BLOCKED RECOVERY: discovery restoration alone did not recover within 90 seconds: {error}",
                      flush=True)
                for peer in peers:
                    peer.stop()
                for peer in peers:
                    peer.start()
                    restrict_peer_egress(peer)
                verify_direct_isolation(first, second, networks[1])
                retry(verify)
                print("PASS: discovery restoration plus another peer restart recovers relay traffic", flush=True)
            else:
                print("PASS: restoring discovery recovers relay traffic without another peer restart", flush=True)
            transport.require(relay_bytes(relay) - before_bytes >= 2 * 950272,
                              "Discovery restoration traffic did not traverse the private relay")
            transport.require([peer.status()["endpoint"] for peer in peers] ==
                              [status["endpoint"] for status in statuses],
                              "Discovery-outage restart changed endpoint identities")
            transport.require(cold_recovered, "Private-relay cold-start/discovery-outage qualification failed")
            transport.require(restored_without_restart, "Private-relay discovery restoration required another peer restart")
    finally:
        errors = []
        if relay_created:
            try:
                docker("rm", "-f", relay)
            except RuntimeError as error:
                errors.append(error)
        for node in reversed(nodes):
            try:
                node.cleanup()
            except RuntimeError as error:
                errors.append(error)
        if volume_created:
            try:
                docker("volume", "rm", volume)
            except RuntimeError as error:
                errors.append(error)
        for network in reversed(networks):
            try:
                docker("network", "rm", network)
            except RuntimeError as error:
                errors.append(error)
        if errors:
            raise RuntimeError("Private relay fixture cleanup failed") from errors[0]


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--discovery-outage", action="store_true",
                        help="Additionally qualify simultaneous cold restart while private discovery is stopped")
    run(discovery_outage=parser.parse_args().discovery_outage)
