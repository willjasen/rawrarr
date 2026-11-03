#!/usr/bin/env bash
set -euo pipefail

if [[ "$(id -u)" -ne 0 ]]; then
  echo "Run this installer as root on do3." >&2
  exit 1
fi

script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
install -d -o root -g root -m 0700 /opt/rawrarr-coordinator-auto-updater
install -o root -g root -m 0750 "$script_dir/auto_update_coordinators.py" \
  /opt/rawrarr-coordinator-auto-updater/auto_update_coordinators.py
install -o root -g root -m 0644 "$script_dir/rawrarr-coordinator-auto-update.service" \
  /etc/systemd/system/rawrarr-coordinator-auto-update.service
install -o root -g root -m 0644 "$script_dir/rawrarr-coordinator-auto-update.timer" \
  /etc/systemd/system/rawrarr-coordinator-auto-update.timer
systemctl daemon-reload
systemctl enable --now rawrarr-coordinator-auto-update.timer
