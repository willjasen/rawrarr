#!/usr/bin/env python3
"""Test closed-mesh members without the key holder or, optionally, discovery."""

import argparse
import base64
import hashlib
import json
from pathlib import Path
import sys
import time
import uuid

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import rayfish_transport as transport
from rayfish_docker import Node, docker, retry


def network_record_hash(discovery, key):
    transport.identity(key)
    encoded = base64.b32encode(bytes.fromhex(key)).decode().lower().rstrip("=").translate(
        str.maketrans("abcdefghijklmnopqrstuvwxyz234567", "ybndrfg8ejkmcpqxot1uwisza345h769"))
    filename = hashlib.sha256(("/pkarr/" + encoded).encode()).hexdigest()
    return docker("exec", discovery.name, "python", "-c",
                  "import hashlib,pathlib; "
                  f"p=pathlib.Path('/state/discovery-packets') / {filename!r}; "
                  "print(hashlib.sha256(p.read_bytes()).hexdigest())")


def run(discovery_outage=False, revocation_outage=False):
    transport.require(not revocation_outage or discovery_outage,
                      "Revocation qualification requires the discovery-outage test")
    prefix = "rawrarr-test-rayfish-owner-" + uuid.uuid4().hex[:10]
    nodes = []
    network_created = False
    discovery_paused = False
    try:
        docker("network", "create", prefix)
        network_created = True
        discovery = Node(prefix, "discovery", prefix)
        nodes.append(discovery)
        discovery.create()
        docker("network", "disconnect", prefix, discovery.name)
        docker("network", "connect", "--alias", "discovery", prefix, discovery.name)
        docker("start", discovery.name)
        peers = []
        for label in ("mesh-owner", "fossil-a", "fossil-b"):
            peer = Node(prefix, label, prefix)
            nodes.append(peer)
            peers.append(peer)
            peer.create()
            peer.start()
        owner, first, second = peers
        owner.cli("create", "--closed", "--name", "rawrarr-pilot", "--hostname", "mesh-owner")
        owner.cli("files", "auto-accept", "rawrarr-pilot", "off")
        for peer in (first, second):
            invite = json.loads(owner.cli("invite", "rawrarr-pilot", "--json"))["code"]
            retry(lambda: peer.cli("join", invite, "--name", "rawrarr-pilot",
                                   "--hostname", peer.label, "--no-auto-accept-files"))
        statuses = [first.status(), second.status()]
        for peer, remote in zip((first, second), reversed(statuses)):
            peer.stop()
            mesh, = [item for item in remote["networks"] if item["name"] == "rawrarr-pilot"]
            peer.config.update(expose_app=True, allow_in=[remote["endpoint"]],
                               forwards=[{"node_id": "peer", "peer_id": remote["endpoint"],
                                          "peer_ipv6": mesh["my_ipv6"], "local_port": 17297}])
            peer.update()
            peer.start()
        expected_hash = hashlib.sha256(b"rawrarr-synthetic-ciphertext\n" * 32768).hexdigest()

        def verify():
            for peer in (first, second):
                transport.require(peer.request(17297) == [200, expected_hash, 950272],
                                  "Member response failed without the network key holder")
                transport.require(peer.request(17297, authorized=False)[0] == 401,
                                  "Member bypassed application authorization")
        retry(verify)
        owner.stop()
        retry(verify)
        first.stop()
        first.start()
        retry(verify)
        transport.require([first.status()["endpoint"], second.status()["endpoint"]] ==
                          [item["endpoint"] for item in statuses], "Member restart changed identities")
        print("PASS: two admitted members exchange byte-exact app traffic with the network owner offline", flush=True)
        print("PASS: member cold restart recovers without the network owner; app 401s remain enforced", flush=True)
        if discovery_outage:
            docker("pause", discovery.name)
            discovery_paused = True
            discovery_state, = json.loads(docker("inspect", discovery.name))
            transport.require(discovery_state["State"]["Paused"], "Discovery fixture did not pause")
            for peer in (first, second):
                peer.stop()
            for peer in (first, second):
                peer.start()
                docker("exec", peer.name, "python", "-c",
                       "import socket; socket.gethostbyname('discovery')")
            retry(verify)
            transport.require([first.status()["endpoint"], second.status()["endpoint"]] ==
                              [item["endpoint"] for item in statuses],
                              "Member cold restarts changed identities")
            print("PASS: both members cold-restart and exchange authenticated app traffic with discovery "
                  "and the network owner offline", flush=True)
            docker("unpause", discovery.name)
            discovery_paused = False
            retry(verify)
            print("PASS: both members remain reachable after discovery returns", flush=True)
        if revocation_outage:
            for peer in (first, second):
                peer.stop()
            owner.start()
            owner_network, = [item for item in owner.status()["networks"]
                              if item["name"] == "rawrarr-pilot"]
            transport.require(statuses[1]["endpoint"] in
                              [peer["endpoint_id"] for peer in owner_network["peers"]],
                              "Owner did not list the member before revocation")
            before = network_record_hash(discovery, owner_network["network_key"])
            owner.cli("kick", "rawrarr-pilot", "fossil-b", "--yes")
            owner_network, = [item for item in owner.status()["networks"]
                              if item["name"] == "rawrarr-pilot"]
            transport.require(statuses[1]["endpoint"] not in
                              [peer["endpoint_id"] for peer in owner_network["peers"]],
                              "Owner still lists the kicked member")

            def wait_for_record():
                if network_record_hash(discovery, owner_network["network_key"]) == before:
                    raise RuntimeError("The network discovery record has not changed")
            retry(wait_for_record, timeout=30)
            owner.stop()
            docker("pause", discovery.name)
            discovery_paused = True
            discovery_state, = json.loads(docker("inspect", discovery.name))
            transport.require(discovery_state["State"]["Paused"], "Discovery fixture did not pause")
            for peer in (first, second):
                peer.start()
            reached_while_stale = None
            try:
                retry(verify, timeout=60)
            except RuntimeError as error:
                network, = [item for item in first.status()["networks"] if item["name"] == "rawrarr-pilot"]
                if (statuses[1]["endpoint"] not in [peer["endpoint_id"] for peer in network["peers"]]
                        and first.request(8080)[0] == 200 and second.request(8080)[0] == 200):
                    reached_while_stale = False
                    print("PASS: remaining member removed the kicked peer and the transfer failed", flush=True)
                else:
                    print(f"INCONCLUSIVE: stale-roster transfer did not complete: {error}", flush=True)
            else:
                reached_while_stale = True
                print("BLOCKED QUALIFICATION: kicked member still exchanges authenticated app responses "
                      "with cached rosters and discovery offline", flush=True)
            docker("unpause", discovery.name)
            discovery_paused = False
            owner.start()
            recovery_started = time.monotonic()

            def verify_revocation():
                network, = [item for item in first.status()["networks"] if item["name"] == "rawrarr-pilot"]
                if statuses[1]["endpoint"] in [peer["endpoint_id"] for peer in network["peers"]]:
                    raise RuntimeError("Member has not removed kicked peer from its roster")
                transport.require(first.request(8080)[0] == 200 and second.request(8080)[0] == 200,
                                  "Local application fixture is not healthy")
                try:
                    second.request(17297)
                except RuntimeError:
                    return
                raise RuntimeError("Kicked peer can still reach the application over Rayfish")
            retry(verify_revocation, timeout=180)
            print("PASS: after discovery and the key holder return, the remaining member removes the kicked "
                  f"peer and its Rayfish access fails ({time.monotonic() - recovery_started:.1f}s)", flush=True)
            transport.require(reached_while_stale is False,
                              "Qualification failed or inconclusive: fresh revocation was not enforced during discovery outage")
    finally:
        errors = []
        if discovery_paused:
            try:
                docker("unpause", discovery.name)
            except RuntimeError as error:
                errors.append(error)
        for node in reversed(nodes):
            try:
                node.cleanup()
            except RuntimeError as error:
                errors.append(error)
        if network_created:
            try:
                docker("network", "rm", prefix)
            except RuntimeError as error:
                errors.append(error)
        if errors:
            raise RuntimeError("Owner-offline fixtures could not all be cleaned") from errors[0]


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--discovery-outage", action="store_true",
                        help="Also cold-restart both members while the private discovery server is paused")
    parser.add_argument("--revocation-outage", action="store_true",
                        help="Also qualify a published kick while both members and discovery are offline")
    args = parser.parse_args()
    run(discovery_outage=args.discovery_outage, revocation_outage=args.revocation_outage)
