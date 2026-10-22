#!/bin/sh
set -eu
if [ "${RAWARR_ROLE:-}" = "coordinator" ] && [ "${RAWARR_UI_UPDATES_ENABLED:-true}" = "true" ] && [ -r "${RAWARR_UI_GITHUB_TOKEN_FILE:-/run/secrets/rawrarr_ui_github_token}" ]; then
  python3 /usr/local/bin/rawrarr-update-ui --apply --watch &
fi
exec /usr/local/bin/rust-arr "$@"
