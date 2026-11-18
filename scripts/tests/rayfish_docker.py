"""Disposable separate-namespace Rayfish peers, optionally beside real test fossils."""

import hashlib
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import time
import uuid

import rayfish_transport as transport

ROOT = Path(__file__).resolve().parents[2]
PROJECT = "rawrarr-test-environment"
RAYFISH_STACK_LABEL = "com.docker.compose.project=rawrarr-test-rayfish"


def docker(*args, timeout=60):
    result = subprocess.run(["docker", "--context", "orbstack", *args],
                            capture_output=True, text=True, timeout=timeout)
    if result.returncode:
        raise RuntimeError(f"Docker {args[0]} failed: {result.stderr[:1000]}")
    return result.stdout.strip()


def retry(operation, timeout=90):
    deadline = time.monotonic() + timeout
    last = None
    while time.monotonic() < deadline:
        try:
            return operation()
        except (RuntimeError, subprocess.TimeoutExpired) as error:
            last = error
            time.sleep(1)
    raise RuntimeError(f"Rayfish deadline exceeded: {last}")


class Node:
    def __init__(self, prefix, name, network, existing=None, relay_url=None):
        self.name = prefix + "-" + name
        self.volume = self.name + "-state"
        self.label = name
        self.config = {"version": 1, "name": name, "namespace": existing or self.name,
                       "role": "metadata", "network": "rawrarr-pilot", "lan_discovery": True}
        self.created = False
        self.volume_created = False
        self.network = network
        self.existing = existing
        self.relay_url = relay_url

    def create(self):
        docker("volume", "create", "--label", RAYFISH_STACK_LABEL, self.volume)
        self.volume_created = True
        args = ["create", "--name", self.name, "--pull", "never", "--read-only",
                "--cap-drop", "ALL", "--security-opt", "no-new-privileges:true",
                "--tmpfs", "/tmp:rw,nosuid,mode=1777,size=16m",
                "-v", self.volume + ":/state", "-e", "RAWARR_RAYFISH_TEST_INNER=1",
                "-e", "RAWARR_RAYFISH_WORKER=" + self.label,
                "-e", "RAWARR_RAYFISH_DISCOVERY_URL=http://discovery:18090/pkarr",
                "-e", "RAYFISH_ROOTLESS_DIR=/state/" + self.label,
                "--label", RAYFISH_STACK_LABEL,
                "--label", f"com.docker.compose.service={self.label}"]
        if self.existing:
            args += ["--network", "container:" + self.existing, "-e", "RAWARR_RAYFISH_EXTERNAL_APP=1"]
        else:
            args += ["--network", self.network]
        if self.relay_url:
            args += ["-e", "RAWARR_RAYFISH_TEST_RELAY_URL=" + self.relay_url,
                     "-e", "RAWARR_RAYFISH_TEST_RELAY_PROBE=1"]
        args += ["--entrypoint", "python", transport.IMAGE, "/state/test.py"]
        docker(*args)
        self.created = True
        docker("cp", str(ROOT / "scripts/tests/test_rayfish_transport.py"), self.name + ":/state/test.py")
        docker("cp", str(ROOT / "scripts/rayfish_transport.py"), self.name + ":/state/rayfish_transport.py")
        self.update()

    def update(self):
        transport.validate(self.config)
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / (self.label + ".json")
            path.write_text(json.dumps(self.config))
            docker("cp", str(path), self.name + ":/state/" + path.name)

    def start(self):
        docker("start", self.name)
        retry(self.status, timeout=30)

    def stop(self):
        docker("stop", "-t", "20", self.name, timeout=30)

    def cli(self, *args):
        try:
            return docker("exec", self.name, "ray", "--rootless", *args)
        except RuntimeError as error:
            detail = str(error)
            if args[0] == "join":
                detail = detail.replace(args[1], "<synthetic-invite>")
            raise RuntimeError(detail) from None

    def status(self):
        status = json.loads(self.cli("status", "--json"))
        transport.require(status["rootless"] and status["active"], "Rootless test transport is inactive")
        return status

    def request(self, port, path="/api/fixture", authorized=True, token=None):
        token = token or ("rawrarr-test-session" if authorized else None)
        program = (
            "import hashlib,http.client,json\n"
            f"c=http.client.HTTPConnection('127.0.0.1',{port},timeout=5)\n"
            f"c.request('GET',{path!r},headers="
            + repr({"Authorization": "Bearer synthetic-node-token"} if authorized else {}) + ")\n"
            "r=c.getresponse(); b=r.read(); print(json.dumps([r.status,hashlib.sha256(b).hexdigest(),len(b)])); c.close()\n"
        )
        if token is not None:
            # The real-service tier uses a disposable, node-scoped session.
            program = (
                "import hashlib,http.client,json\n"
                f"c=http.client.HTTPConnection('127.0.0.1',{port},timeout=5)\n"
                f"c.request('GET',{path!r},headers={{'Authorization': {'Bearer ' + token!r}}})\n"
                "r=c.getresponse(); b=r.read(); print(json.dumps([r.status,hashlib.sha256(b).hexdigest(),len(b)])); c.close()\n"
            )
        return json.loads(docker("exec", self.name, "python", "-c", program, timeout=10))

    def cleanup(self):
        if self.created:
            docker("rm", "-f", self.name)
        if self.volume_created:
            docker("volume", "rm", self.volume)


def exercise_recovery(first, second, discovery):
    expected = [second.request(8080), first.request(8080)]

    def verify():
        for node, baseline in zip((first, second), expected):
            actual = node.request(17297)
            transport.require(actual == baseline and actual[0] == 200,
                              "Recovery changed peer payload or failed authentication")

    docker("stop", "-t", "10", discovery.name)
    verify()
    first.stop()
    second.stop()
    first.start()
    second.start()
    cold_recovered = True
    try:
        retry(verify, timeout=30)
    except RuntimeError:
        cold_recovered = False
        transport.require(first.status()["active"] and second.status()["active"],
                          "Cold restart lost local transport liveness")
        print("BLOCKED QUALIFICATION: cold restart during discovery outage does not reconnect within 30 seconds", flush=True)
    docker("start", discovery.name)
    first.stop()
    second.stop()
    first.start()
    second.start()
    retry(verify)
    print("PASS: established transfers survive discovery loss; dual cold restart recovers after discovery restoration", flush=True)
    docker("pause", first.name)
    try:
        try:
            second.request(17297)
        except RuntimeError:
            pass
        else:
            raise RuntimeError("Suspended peer unexpectedly served an application request")
    finally:
        docker("unpause", first.name)
    retry(verify)
    print("PASS: peer process suspension fails explicitly and resumes byte-exact transfers", flush=True)
    return cold_recovered


def soak(first, second, hours, report, cold_recovered):
    expected = [second.request(8080), first.request(8080)]
    endpoints = [first.status()["endpoint"], second.status()["endpoint"]]
    start = time.monotonic()
    samples = 0
    last_sample_wall = time.time()
    result = {"status": "running", "requested_hours": hours, "samples": 0,
              "elapsed_seconds": 0, "bytes": 0, "coverage": "direct LAN synthetic peers; no NAT or relay",
              "cold_restart_during_discovery_outage": "passed" if cold_recovered else "blocked",
              "containers": [first.name, second.name], "largest_wall_gap_seconds": 0,
              "largest_monotonic_gap_seconds": 0}
    last_sample_monotonic = start
    try:
        while True:
            now_wall = time.time()
            now_monotonic = time.monotonic()
            result["largest_wall_gap_seconds"] = max(result["largest_wall_gap_seconds"], now_wall - last_sample_wall)
            result["largest_monotonic_gap_seconds"] = max(
                result["largest_monotonic_gap_seconds"], now_monotonic - last_sample_monotonic)
            transport.require(now_wall - last_sample_wall <= 90,
                              "Soak sampling gap exceeded 90 seconds; suspend or scheduling outage invalidated continuity")
            transport.require(now_monotonic - last_sample_monotonic <= 90,
                              "Soak monotonic sampling gap exceeded 90 seconds")
            for node, baseline in zip((first, second), expected):
                actual = node.request(17297)
                transport.require(actual == baseline and actual[0] == 200,
                                  "Soak response changed or peer stopped responding")
                result["bytes"] += actual[2]
                transport.require(node.request(17297, authorized=False)[0] == 401,
                                  "Soak bypassed application authorization")
            transport.require([first.status()["endpoint"], second.status()["endpoint"]] == endpoints,
                              "Soak endpoint identities changed")
            samples += 1
            last_sample_wall = now_wall
            last_sample_monotonic = now_monotonic
            elapsed = time.monotonic() - start
            result.update(samples=samples, elapsed_seconds=elapsed)
            transport.private_write(report, json.dumps(result, indent=2) + "\n")
            if elapsed >= hours * 3600:
                break
            time.sleep(min(30, hours * 3600 - elapsed))
        result["status"] = "passed"
        print(f"PASS: {hours:g}-hour direct-LAN soak, {samples} bidirectional samples", flush=True)
    except BaseException:
        result["status"] = "failed_or_interrupted"
        raise
    finally:
        result["elapsed_seconds"] = time.monotonic() - start
        transport.private_write(report, json.dumps(result, indent=2) + "\n")


def soak_status(report):
    result = transport.read_json(report)
    transport.require(isinstance(result, dict), "Invalid soak report")
    transport.require(result.get("status") in ("running", "passed", "failed_or_interrupted"),
                      "Invalid soak report status")
    age = time.time() - report.stat().st_mtime
    transport.require(age >= 0, "Soak report timestamp is in the future; check the system clock")
    result["report_age_seconds"] = age
    if result["status"] == "running" and age > 90:
        result["status"] = "stale"
    return result


def run(environment, soak_hours=0, report=None):
    transport.require(not soak_hours or not environment, "Soak must use disposable synthetic nodes, not persistent test sessions")
    transport.require(0 <= soak_hours <= 72, "Soak duration must be between 0 and 72 hours")
    transport.require(not soak_hours or report is not None, "Soak needs a persistent report file")
    info, = json.loads(docker("image", "inspect", transport.IMAGE))
    host = docker("info", "--format", "{{.OSType}}/{{.Architecture}}")
    arch = {"linux/x86_64": "amd64", "linux/arm64": "arm64", "linux/aarch64": "arm64"}.get(host)
    transport.require(info["Os"] == "linux" and info["Architecture"] == arch, "A native Rayfish image is required")
    prefix = "rawrarr-test-rayfish-" + uuid.uuid4().hex[:12]
    nodes, discovery = [], None
    network_created = False
    before = {}
    session_hashes = []
    tokens = []
    try:
        if environment:
            for name in ("tailscale-a", "tailscale-b", "metadata-a", "metadata-b"):
                full = PROJECT + "-" + name + "-1"
                item, = json.loads(docker("inspect", full))
                transport.require(item["State"]["Running"] and
                                  item["Config"]["Labels"]["com.docker.compose.project"] == PROJECT,
                                  "Start the isolated coordinator/two-fossil environment first")
                before[full] = (item["Id"], item["State"]["StartedAt"])
            network = PROJECT + "_default"
            owner = docker("exec", PROJECT + "-postgres-1", "psql", "-U", "rawrarr", "-d", "rawrarr",
                           "-Atc", "SELECT owner_user_id FROM rawrarr_instance_signing_root WHERE singleton=TRUE;")
            transport.require(len(owner) == 36 and all(char in "0123456789abcdef-" for char in owner),
                              "Isolated environment needs its signed fixture owner")
            for label in ("a", "b"):
                token = uuid.uuid4().hex + uuid.uuid4().hex
                digest = hashlib.sha256(token.encode()).hexdigest()
                docker("exec", PROJECT + "-postgres-1", "psql", "-U", "rawrarr", "-d", "rawrarr",
                       "-v", "ON_ERROR_STOP=1", "-c",
                       "INSERT INTO rawrarr_sessions(token_hash,user_id,expires_at,node_id) "
                       f"VALUES(decode('{digest}','hex'),'{owner}'::uuid,now()+interval '10 minutes','test-metadata-{label}');")
                session_hashes.append(digest)
                tokens.append(token)
        else:
            network = prefix
            docker("network", "create", "--label", RAYFISH_STACK_LABEL, network)
            network_created = True
        discovery = Node(prefix, "discovery", network)
        # The local signed-record store is a test fixture, not a deployed coordinator/relay.
        discovery.create()
        docker("network", "disconnect", network, discovery.name)
        docker("network", "connect", "--alias", "discovery", network, discovery.name)
        docker("start", discovery.name)
        for label, target in (("fossil-a", "tailscale-a"), ("fossil-b", "tailscale-b")):
            node = Node(prefix, label, network, PROJECT + "-" + target + "-1" if environment else None)
            nodes.append(node)
            node.create()
            node.start()
        first, second = nodes
        first.cli("create", "--closed", "--name", "rawrarr-pilot", "--hostname", "fossil-a")
        first.cli("files", "auto-accept", "rawrarr-pilot", "off")
        invite = json.loads(first.cli("invite", "rawrarr-pilot", "--json"))["code"]
        retry(lambda: second.cli("join", invite, "--name", "rawrarr-pilot", "--hostname", "fossil-b",
                                 "--no-auto-accept-files"))
        statuses = [node.status() for node in nodes]
        for node, other in zip(nodes, reversed(statuses)):
            node.stop()
            mesh, = [entry for entry in other["networks"] if entry["name"] == "rawrarr-pilot"]
            node.config.update(expose_app=True, allow_in=[other["endpoint"]],
                               forwards=[{"node_id": "peer-fossil", "peer_id": other["endpoint"],
                                          "peer_ipv6": mesh["my_ipv6"], "local_port": 17297}])
            node.update()
        for node in nodes:
            node.start()
        for node, remote in ((first, second), (second, first)):
            path = "/api/metadata/export/node-recipient" if environment else "/api/fixture"
            token = tokens[nodes.index(remote)] if environment else None
            baseline = remote.request(8080, path, token=token)
            actual = retry(lambda: node.request(17297, path, token=token))
            transport.require(actual == baseline and actual[0] == 200,
                              f"Peer response mismatch: local HTTP {baseline[0]} ({baseline[2]} bytes), "
                              f"Rayfish HTTP {actual[0]} ({actual[2]} bytes)")
            transport.require(node.request(17297, path, authorized=False)[0] == 401, "App authentication was bypassed")
            if not environment:
                transport.require(actual[1] == hashlib.sha256(b"rawrarr-synthetic-ciphertext\n" * 32768).hexdigest(),
                                  "Synthetic ciphertext changed in transit")
                transport.require(node.request(17297, authorized=False)[0] == 401, "App authentication was bypassed")
                transport.require(node.request(7299, "/health")[0] == 200, "Existing ingress fixture failed")
        for protocol in ("tcp:8081", "udp:8080"):
            verdict = json.loads(first.cli("firewall", "test", statuses[1]["endpoint"], protocol, "--json"))
            transport.require(verdict["action"] == "deny", "Non-application traffic was permitted")
        cold_recovered = None
        if not environment:
            cold_recovered = exercise_recovery(first, second, discovery)
        if soak_hours:
            soak(first, second, soak_hours, report, cold_recovered)
        outsider = Node(prefix, "outsider", network)
        nodes.append(outsider)
        outsider.create()
        outsider.start()
        invite = json.loads(first.cli("invite", "rawrarr-pilot", "--json"))["code"]
        retry(lambda: outsider.cli("join", invite, "--name", "rawrarr-pilot",
                                   "--hostname", "outsider", "--no-auto-accept-files"))
        outsider.stop()
        mesh, = [entry for entry in statuses[0]["networks"] if entry["name"] == "rawrarr-pilot"]
        outsider.config["forwards"] = [{"node_id": "denied-fossil", "peer_id": statuses[0]["endpoint"],
                                       "peer_ipv6": mesh["my_ipv6"], "local_port": 17297}]
        outsider.update()
        outsider.start()
        try:
            outsider.request(17297, "/health")
        except RuntimeError:
            pass
        else:
            raise RuntimeError("Admitted but unapproved transport peer reached the app")
        print("PASS: admitted unapproved peer denied; non-app TCP and UDP firewall verdicts deny", flush=True)
        print("PASS: separate fossil namespaces exchange application responses bidirectionally over real Rayfish", flush=True)
        first.stop()
        if environment:
            transport.require(second.request(8080, "/health")[0] == 200, "Local fossil stopped with Rayfish")
            for label, other in (("a", "b"), ("b", "a")):
                target = PROJECT + "-tailscale-" + other + "-1"
                status = json.loads(docker("exec", target, "tailscale", "status", "--json"))
                address = status["Self"]["TailscaleIPs"][0]
                docker("exec", PROJECT + "-tailscale-" + label + "-1",
                       "tailscale", "ping", "--timeout=5s", "--c=1", address)
            print("PASS: bidirectional Tailscale pings and fossil health survive a Rayfish outage", flush=True)
        try:
            second.request(17297, "/health" if environment else "/api/fixture")
        except RuntimeError:
            pass
        else:
            raise RuntimeError("Transport outage silently used another route")
        first.start()
        retry(lambda: second.request(17297, "/health" if environment else "/api/fixture"))
        transport.require(first.status()["endpoint"] == statuses[0]["endpoint"], "Restart changed endpoint identity")
        first.cli("kick", "rawrarr-pilot", "fossil-b", "--yes")
        time.sleep(3)
        try:
            second.request(17297, "/health" if environment else "/api/fixture")
        except RuntimeError:
            pass
        else:
            raise RuntimeError("Revoked peer reached the app")
        print("PASS: transport outage fails closed, restart preserves identity, and revoked peer is denied", flush=True)
        transport.require(cold_recovered is not False,
                          "Qualification failed: simultaneous cold restart during discovery outage did not recover")
    except Exception:
        for node in nodes:
            if node.created:
                logs = docker("logs", node.name)
                # These are disposable synthetic identities, not live node logs.
                print("\n".join(line for line in logs.splitlines() if " WARN " in line or " ERROR " in line)[-3000:],
                      file=sys.stderr)
        raise
    finally:
        errors = []
        for digest in session_hashes:
            try:
                docker("exec", PROJECT + "-postgres-1", "psql", "-U", "rawrarr", "-d", "rawrarr",
                       "-v", "ON_ERROR_STOP=1", "-c",
                       f"DELETE FROM rawrarr_sessions WHERE token_hash=decode('{digest}','hex');")
            except RuntimeError as error:
                errors.append(error)
        for node in reversed(nodes + ([discovery] if discovery else [])):
            try:
                node.cleanup()
            except RuntimeError as error:
                errors.append(error)
        if network_created:
            try:
                docker("network", "rm", network)
            except RuntimeError as error:
                errors.append(error)
        for name, expected in before.items():
            item, = json.loads(docker("inspect", name))
            transport.require((item["Id"], item["State"]["StartedAt"]) == expected and item["State"]["Running"],
                              "Existing app or Tailscale service was restarted")
        if errors:
            raise RuntimeError("Disposable Rayfish resources could not all be removed")
