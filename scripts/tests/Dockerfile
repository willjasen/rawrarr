FROM rust:1.85-bookworm

ARG NODE_MAJOR=26

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates curl gnupg chromium libjxl-tools librsvg2-bin \
    && curl -fsSL https://deb.nodesource.com/gpgkey/nodesource-repo.gpg.key | gpg --dearmor -o /usr/share/keyrings/nodesource.gpg \
    && echo "deb [signed-by=/usr/share/keyrings/nodesource.gpg] https://deb.nodesource.com/node_${NODE_MAJOR}.x nodistro main" > /etc/apt/sources.list.d/nodesource.list \
    && apt-get update \
    && apt-get install -y --no-install-recommends nodejs \
    && test "$(node --version | cut -d. -f1)" = "v${NODE_MAJOR}" \
    && rm -rf /var/lib/apt/lists/*

RUN npm install --global --prefix /usr/local playwright-core@1.55.0

WORKDIR /workspace

COPY rust-arr/web-export/package.json rust-arr/web-export/package-lock.json /opt/rawrarr-web-export/
RUN npm ci --prefix /opt/rawrarr-web-export

# This image is the reusable dependency base. Project source is copied into each
# uniquely named test container after creation, so source edits never rebuild it.
COPY rust-arr/Cargo.toml rust-arr/Cargo.lock /workspace/rust-arr/
RUN mkdir -p /workspace/rust-arr/src \
    && printf 'fn main() {}\n' > /workspace/rust-arr/src/main.rs \
    && cargo fetch --locked --manifest-path /workspace/rust-arr/Cargo.toml

ENV CARGO_TARGET_DIR=/opt/rawrarr-test-target
RUN cargo test --locked --offline --no-run --manifest-path /workspace/rust-arr/Cargo.toml

ARG TARGETARCH
RUN case "$TARGETARCH" in \
        amd64) age_arch=amd64; age_sha256=cbe24006683f8eb669266162894b9a522a1af52f2665fbc63a4bb032ed26ac10 ;; \
        arm64) age_arch=arm64; age_sha256=6b8dc4333c53a5a57c9e5834e3a48f92605d7154014cd07269ff3327db5d37f4 ;; \
        *) echo "Unsupported test image architecture: $TARGETARCH" >&2; exit 1 ;; \
    esac \
    && curl -fsSL "https://github.com/FiloSottile/age/releases/download/v1.3.2/age-v1.3.2-linux-${age_arch}.tar.gz" -o /tmp/age.tar.gz \
    && echo "$age_sha256  /tmp/age.tar.gz" | sha256sum -c - \
    && test "$(dpkg --print-architecture)" = "$TARGETARCH" \
    && tar -xzf /tmp/age.tar.gz -C /tmp \
    && install -m 0755 /tmp/age/age /usr/local/bin/age \
    && install -m 0755 /tmp/age/age-keygen /usr/local/bin/age-keygen \
    && rm -rf /tmp/age /tmp/age.tar.gz

RUN mkdir -p /workspace/scripts/tests
