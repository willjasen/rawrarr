# Fix local Rawrarr passkey origin

## Objective and initial state

The local Rawrarr login returned “Passkey verification failed; try again.” The container was published on host port 8084, but its configured WebAuthn origin was `http://localhost:8080`. WebAuthn verifies the browser origin exactly, including the port.

## Changes

- Make the default `RAWARR_WEBAUTHN_ORIGIN` follow `RAWARR_HOST_PORT`, while preserving an explicit origin override for deployments behind a domain or reverse proxy.
- Keep the existing relying-party ID (`localhost`) and stored passkey data unchanged.

## Verification

The resolved Compose configuration and recreated container both report `http://localhost:8084` with RP ID `localhost`; `/health` returns `ok`. One Rawrarr web container is running on port 8084, so there were no duplicate Rawrarr web servers to stop. The stored passkey and PostgreSQL container were left intact. A successful interactive assertion still needs confirmation by signing in through `http://localhost:8084`.
