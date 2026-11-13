#!/usr/bin/env python3
"""Prove two members remain usable without the closed mesh's key holder online."""

import hashlib
import json
from pathlib import Path
import sys
import uuid

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import rayfish_transport as transport
from rayfish_docker import Node, docker, retry


def run():
    prefix = "rawrarr-test-rayfish-owner-" + uuid.uuid4().hex[:10]
    nodes = []
    network_created = False
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
    finally:
        errors = []
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
    run()
