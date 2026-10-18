FROM rust:1.85-bookworm

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates curl nodejs \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /workspace

# This image is the reusable dependency base. Project source is copied into each
# uniquely named test container after creation, so source edits never rebuild it.
COPY rust-arr/Cargo.toml rust-arr/Cargo.lock /workspace/rust-arr/
RUN mkdir -p /workspace/rust-arr/src \
    && printf 'fn main() {}\n' > /workspace/rust-arr/src/main.rs \
    && cargo fetch --locked --manifest-path /workspace/rust-arr/Cargo.toml

ENV CARGO_TARGET_DIR=/opt/rawrarr-test-target
RUN cargo test --locked --offline --no-run --manifest-path /workspace/rust-arr/Cargo.toml

RUN curl -fsSL https://github.com/FiloSottile/age/releases/download/v1.3.2/age-v1.3.2-linux-amd64.tar.gz -o /tmp/age.tar.gz \
    && echo 'cbe24006683f8eb669266162894b9a522a1af52f2665fbc63a4bb032ed26ac10  /tmp/age.tar.gz' | sha256sum -c - \
    && tar -xzf /tmp/age.tar.gz -C /tmp \
    && install -m 0755 /tmp/age/age /usr/local/bin/age \
    && install -m 0755 /tmp/age/age-keygen /usr/local/bin/age-keygen \
    && rm -rf /tmp/age /tmp/age.tar.gz

RUN mkdir -p /workspace/scripts/tests
