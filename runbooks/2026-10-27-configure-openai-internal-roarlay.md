# Configure OpenAI for internal roarlay

Issue: [#230](https://github.com/willjasen/rawrarr/issues/230)

## Objective

Enable the existing OpenAI-backed assistant on the internal coordinator, using the protected key file on Brandon's Mac mini. Keep the key out of the synced repository and Git.

## Findings

- The coordinator was healthy and the tailnet route sent HTTPS `/` to `http://127.0.0.1:8080`, but its container had no `RAWARR_OPENAI_*` environment settings or OpenAI secret mount.
- The owner created `~/Library/Application Support/rawrarr/openai-api-key` on Brandon's Mac mini. The file was secured to mode `0600`; its contents were never read or logged.
- The optional `docker-compose.openai.yml` overlay already existed in the coordinator's published source directory. The existing ARM64 base image revision was `906c2e580597b25d429ffc6dc92503aa8389a7c9`.

## Change and verification

- Added the OpenAI overlay to the `rawrarr-internal-mini` Compose invocation and recreated only `roarlay`, with `--no-deps --no-build --pull never`.
- Confirmed the container has the expected OpenAI environment variable names and `/run/secrets/rawrarr_openai_api_key` mount without printing the key.
- Verified the container can read the mounted secret, the roarlay health route succeeds, and Tailscale Serve still maps HTTPS `/` to `http://127.0.0.1:8080`.
- Verified `https://rawrarr.risk-mermaid.ts.net/health` succeeds and identifies roarlay.

## Notes

The Compose overlay and host-local key are required for future coordinator recreation. To disable the integration, omit `docker-compose.openai.yml`; do not remove the key unless the owner asks. Do not place the key in `/sync`, the repository, a container environment value, logs, or this runbook.
