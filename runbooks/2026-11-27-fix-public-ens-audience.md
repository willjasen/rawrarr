# Restore public ENS login configuration

Issue: https://github.com/willjasen/rawrarr/issues/479

## Objective and initial state
Restore the public coordinator ENS endpoint after the [sign-in response fix](2026-11-27-fix-signin-json-response.md) exposed its 404. The live public container `rawrarr-prod-roarlay` ran published main revision ec88b6463ccaeda55982ae8fc788cac7e6c2e39f but lacked RAWARR_BILLING_AUDIENCE. Source explicitly requires `public` for wallet login. Repository public Compose already supplies this setting; the legacy host Compose did not.

## Actions
Connected to do3 using verified OpenSSH with a PTY inside tmux `ai` after the Tailscale wrapper failed to allocate a terminal. Confirmed default Docker context and existing public Compose labels. Atomically acquired `.container-rebuild-rawrarr-prod-roarlay.lock` in the host Rawrarr configuration directory. Backed up `/opt/rawrarr-roarlay-prod/docker-compose.roarlay-prod.yml` as `.yml.before-ens-audience-20261127`, then added `RAWARR_BILLING_AUDIENCE: public` to its roarlay environment. Reused every existing Compose file/project and recreated only roarlay with `up -d --no-deps --pull never`. No source/image changes or migrations were needed; image remained the published GitHub main revision.

## Verification and remaining work
Container environment now contains the public audience. External POST `/api/auth/eth/login/start` with the public ENS name returns HTTP 400 with a structured unlinked-wallet message, replacing the empty 404. Wallet signature and authenticated login are unverified because the wallet is not linked. Existing users must sign in using a passkey and link the wallet in Settings; wallet-first signup remains issue #445.

## Recovery
Under the same deployment lock, restore the host Compose backup and recreate only public roarlay using its existing Compose file list. This restores the former audience behavior; no data rollback is needed.
