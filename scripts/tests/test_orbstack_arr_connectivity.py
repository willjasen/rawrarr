#!/usr/bin/env python3
"""Verify the isolated OrbStack fossil can reach its Radarr and Sonarr APIs."""

from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
import time
import urllib.request


PROJECT = "rawrarr-orbstack-test"
COMPOSE = "docker-compose.orbstack-test.yml"
CHECKS = (
    ("radarr", "7878", "/api/v3/system/status", "Radarr"),
    ("sonarr", "8989", "/api/v3/system/status", "Sonarr"),
)


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


def api_key(service: str) -> str:
    path = "/config/config.xml"
    contents = docker("exec", f"{PROJECT}-{service}-1", "cat", path)
    import xml.etree.ElementTree as ET

    key = ET.fromstring(contents).findtext("ApiKey")
    if not key:
        raise RuntimeError(f"{service} has no generated API key")
    return key


def check_from_fossil(service: str, port: str, path: str, expected_name: str) -> None:
    # Send the generated key over stdin so it never appears in argv or output.
    payload = json.dumps({"url": f"http://{service}:{port}{path}", "key": api_key(service)})
    code = (
        "import json,sys,urllib.request; d=json.load(sys.stdin); "
        "r=urllib.request.Request(d['url'],headers={'X-Api-Key':d['key'],'Accept':'application/json'}); "
        "v=json.load(urllib.request.urlopen(r,timeout=15)); "
        "assert v.get('appName','').lower()==d['expected'].lower(), 'unexpected Arr service identity'; "
        "print('PASS fossil reached '+v['appName']+' API')"
    )
    payload = json.dumps({**json.loads(payload), "expected": expected_name})
    result = subprocess.run(
        ["docker", "--context", "orbstack", "exec", "-i", f"{PROJECT}-metadata-a-1", "python3", "-c", code],
        input=payload,
        capture_output=True,
        text=True,
        check=False,
    )
    if result.returncode:
        raise RuntimeError(f"{service} connection failed: {result.stderr.strip()}")
    print(result.stdout.strip())


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--start", action="store_true", help="start the isolated project services before checking")
    args = parser.parse_args()
    if docker("context", "show") != "orbstack":
        raise RuntimeError("Docker context must be orbstack")
    compose = ["compose", "-p", PROJECT, "-f", COMPOSE]
    docker(*compose, "config", "--quiet")
    if args.start:
        # Start the coordinator and services first so the node receives the
        # fingerprint of this isolated instance's actual signing root.
        docker(*compose, "up", "-d", "radarr", "sonarr", "postgres", "headscale", "tailscale-coordinator", "roarlay")
        wait_healthy("roarlay")
        with urllib.request.urlopen("http://127.0.0.1:17397/api/nodes/signing-root", timeout=10) as response:
            fingerprint = json.load(response).get("fingerprint", "")
        if len(fingerprint) != 64:
            raise RuntimeError("coordinator did not provide a valid instance signing-root fingerprint")
        environment = os.environ.copy()
        environment["RAWARR_INSTANCE_ROOT_FINGERPRINT"] = fingerprint
        docker(*compose, "up", "-d", "tailscale-a", "metadata-a", env=environment)
    for service, _port, _path, _name in CHECKS:
        wait_healthy(service)
    wait_healthy("metadata-a")
    for service, port, path, name in CHECKS:
        check_from_fossil(service, port, path, name)
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (OSError, RuntimeError, subprocess.SubprocessError, TimeoutError) as error:
        print(f"FAIL OrbStack Arr connectivity: {error}", file=sys.stderr)
        raise SystemExit(1)
