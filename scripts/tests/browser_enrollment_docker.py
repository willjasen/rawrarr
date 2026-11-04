"""Offline, disposable production-service tier for browser enrollment and export."""

from pathlib import Path
import secrets
import subprocess
import tempfile
import time


def run(context: str, *, inject_failure: bool = False) -> int:
    from run_rawrarr_tests_in_docker import IMAGE, PROJECT, copy_project_into_container, docker, timestamp_code

    name = f"{PROJECT}-tests-browser-export-{timestamp_code()}-{secrets.token_hex(3)}"
    postgres = name + "-postgres"
    password = secrets.token_hex(32)
    created = []

    def checked(*args):
        try:
            return docker(context, *args)
        except subprocess.CalledProcessError as error:
            # Arguments can include generated credentials. Never echo them.
            raise RuntimeError(f"browser export Docker {args[0]} failed (exit {error.returncode})") from None

    def execute(*args, **kwargs):
        return subprocess.run(["docker", "--context", context, "exec", *args], **kwargs)

    try:
        checked("image", "inspect", IMAGE)
        checked("image", "inspect", "postgres:17-alpine")
        checked(
            "create", "--entrypoint", "/bin/sh", "--name", name,
            "--label", f"com.docker.compose.project={PROJECT}",
            "--label", "com.docker.compose.service=tests-browser-export",
            "--network", "bridge", "--env", "HOME=/tmp/rawrarr-test-home",
            "--env", "RAWARR_BROWSER_EXPORT_TEST=disposable",
            "--env", f"RAWARR_PG_PASSWORD={password}",
            IMAGE, "-c", "sleep infinity",
        )
        created.append(name)
        with tempfile.TemporaryDirectory(prefix="rawrarr-browser-source-") as source:
            copy_project_into_container(context, name, Path(source))
        checked("start", name)
        print(f"BROWSER EXPORT TEST CONTAINER: {name}", flush=True)
        execute(name, "sh", "/workspace/scripts/tests/prepare-test-clone.sh", check=True)
        checked("network", "disconnect", "bridge", name)
        checked(
            "create", "--name", postgres,
            "--label", f"com.docker.compose.project={PROJECT}",
            "--label", "com.docker.compose.service=tests-browser-export-postgres",
            "--network", f"container:{name}",
            "--tmpfs", "/var/lib/postgresql/data:rw,noexec,nosuid",
            "--env", "POSTGRES_USER=rawrarr",
            "--env", "POSTGRES_DB=rawrarr_browser_export_test",
            "--env", f"POSTGRES_PASSWORD={password}",
            "postgres:17-alpine", "-c", "listen_addresses=127.0.0.1",
        )
        created.append(postgres)
        checked("start", postgres)
        for _ in range(120):
            if execute(postgres, "pg_isready", "-U", "rawrarr", "-d", "rawrarr_browser_export_test",
                       stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, check=False).returncode == 0:
                break
            time.sleep(0.25)
        else:
            raise RuntimeError("disposable PostgreSQL readiness timed out")
        checked("exec", name, "sh", "-ec",
                "test -e /workspace/rust-arr/web-export/node_modules || "
                "ln -s /opt/rawrarr-web-export/node_modules /workspace/rust-arr/web-export/node_modules")
        execute(name, "cargo", "build", "--locked", "--offline",
                "--manifest-path", "/workspace/rust-arr/Cargo.toml", "--bin", "rust-arr", check=True)
        if not inject_failure:
            execute(name, "cargo", "test", "--locked", "--offline",
                    "--manifest-path", "/workspace/rust-arr/Cargo.toml", "metadata_export_tests", check=True)
            execute(name, "node", "--test",
                    "/workspace/scripts/tests/test_rawrarr_coordinator_workspace.mjs",
                    "/workspace/scripts/tests/test_rawrarr_mini_export.mjs", check=True)
        helper = "/workspace/scripts/tests/helpers/browser_enrollment_export.mjs"
        result = execute(name, "node", helper, *(["--inject-failure"] if inject_failure else []), check=False)
        if result.returncode:
            return result.returncode
        # Dump remains only in memory and is passed over stdin, never to logs or host files.
        dump = execute(postgres, "pg_dump", "-U", "rawrarr", "-d", "rawrarr_browser_export_test",
                       "--data-only", capture_output=True, check=True)
        execute("-i", name, "node", helper, "--audit-database", input=dump.stdout, check=True)
        return 0
    finally:
        failures = []
        for container in reversed(created):
            result = subprocess.run(
                ["docker", "--context", context, "rm", "--force", "--volumes", container],
                capture_output=True, check=False,
            )
            if result.returncode:
                failures.append(container)
        if failures:
            raise RuntimeError("could not remove owned browser export containers: " + ", ".join(failures))
        if created:
            print("BROWSER EXPORT CLEANUP: removed owned containers and ephemeral data; no host ports, bind mounts, or persistent volumes")
