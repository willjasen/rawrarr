#!/bin/sh
# Refresh the writable linked-clone layer after current source/fixtures are copied.
set -eu

if [ ! -d /workspace/rust-arr ] || [ ! -d /workspace/scripts/tests ]; then
    echo "ERROR: current Rust source or scripts/tests fixtures are missing from the test clone." >&2
    exit 1
fi

missing_packages=""
for pair in "curl:curl" "gnupg:gpg" "chromium:chromium" "cjxl:libjxl-tools" "rsvg-convert:librsvg2-bin"; do
    command_name=${pair%%:*}
    package_name=${pair#*:}
    if ! command -v "$command_name" >/dev/null 2>&1; then
        missing_packages="$missing_packages $package_name"
    fi
done
if [ ! -x /usr/bin/chromium ]; then
    missing_packages="$missing_packages chromium"
fi
if [ -n "$missing_packages" ]; then
    apt-get update
    # De-duplicate package names while preserving a shell-safe argument list.
    apt-get install -y --no-install-recommends $(printf '%s\n' $missing_packages | sort -u)
    rm -rf /var/lib/apt/lists/*
fi

node_major=26
current_node_major=$(node --version 2>/dev/null | sed -E 's/^v([0-9]+).*/\1/' || true)
if [ "$current_node_major" != "$node_major" ]; then
    install -d -m 0755 /usr/share/keyrings
    curl -fsSL https://deb.nodesource.com/gpgkey/nodesource-repo.gpg.key | gpg --dearmor --batch --yes -o /usr/share/keyrings/nodesource.gpg
    printf 'deb [signed-by=/usr/share/keyrings/nodesource.gpg] https://deb.nodesource.com/node_%s.x nodistro main\n' "$node_major" > /etc/apt/sources.list.d/nodesource.list
    apt-get update
    apt-get install -y --no-install-recommends nodejs
    rm -rf /var/lib/apt/lists/*
fi
if [ "$(node --version | sed -E 's/^v([0-9]+).*/\1/')" != "$node_major" ]; then
    echo "ERROR: expected Node.js major $node_major; found $(node --version)." >&2
    exit 1
fi
if ! command -v npm >/dev/null 2>&1; then
    echo "ERROR: npm is missing after installing Node.js $node_major." >&2
    exit 1
fi

case "$(dpkg --print-architecture)" in
    amd64)
        age_arch=amd64
        age_sha256=cbe24006683f8eb669266162894b9a522a1af52f2665fbc63a4bb032ed26ac10
        ;;
    arm64)
        age_arch=arm64
        age_sha256=6b8dc4333c53a5a57c9e5834e3a48f92605d7154014cd07269ff3327db5d37f4
        ;;
    *)
        echo "ERROR: unsupported test clone architecture: $(dpkg --print-architecture)" >&2
        exit 1
        ;;
esac

if ! age --version >/dev/null 2>&1 || ! age-keygen -h >/dev/null 2>&1; then
    archive="age-v1.3.2-linux-${age_arch}.tar.gz"
    curl -fsSL "https://github.com/FiloSottile/age/releases/download/v1.3.2/${archive}" -o /tmp/age.tar.gz
    echo "$age_sha256  /tmp/age.tar.gz" | sha256sum -c -
    tar -xzf /tmp/age.tar.gz -C /tmp
    install -m 0755 /tmp/age/age /usr/local/bin/age
    install -m 0755 /tmp/age/age-keygen /usr/local/bin/age-keygen
    rm -rf /tmp/age /tmp/age.tar.gz
fi

for fixture in \
    test_rawrarr_movie_snapshot.rs \
    test_rawrarr_sonarr.rs \
    test_rawrarr_node_enrollment.rs \
    test_rawrarr_node_auth.rs \
    test_rawrarr_node_roles.rs \
    test_rawrarr_quality_profiles.rs \
    test_rawrarr_metadata_cache.rs \
    test_rawrarr_filename_secret.rs \
    test_rawrarr_hmac_metadata_filenames.rs \
    test_rawrarr_arr_settings.rs \
    test_rawrarr_lidarr.rs \
    test_rawrarr_sonarr_metadata.rs \
    test_rawrarr_config_schema.rs \
    test_rawrarr_storage_node.rs; do
    if [ ! -f "/workspace/scripts/tests/$fixture" ]; then
        echo "ERROR: required Rust test fixture was not copied into the test clone: $fixture" >&2
        exit 1
    fi
done

cargo fetch --locked --manifest-path /workspace/rust-arr/Cargo.toml
printf 'TEST CLONE READY: arch=%s rust=%s node=%s age=%s\n' \
    "$(dpkg --print-architecture)" "$(rustc --version)" "$(node --version)" "$(age --version)"
