# syntax=docker/dockerfile:1

FROM golang:1.25-bookworm AS age-builder
RUN go install filippo.io/age/cmd/...@v1.3.2

FROM rust:1.85-bookworm AS builder
ARG TARGETARCH
ARG RAWARR_BUILD_REVISION=unknown
WORKDIR /src
COPY Cargo.toml Cargo.lock ./
RUN mkdir src migrations \
    && printf '%s\n' 'fn main() {}' > src/main.rs
RUN --mount=type=cache,target=/usr/local/cargo/registry,sharing=locked \
    --mount=type=cache,target=/usr/local/cargo/git/db,sharing=locked \
    case "$TARGETARCH" in \
      amd64) rust_target=x86_64-unknown-linux-gnu ;; \
      arm64) rust_target=aarch64-unknown-linux-gnu ;; \
      *) echo "Unsupported target architecture: $TARGETARCH" >&2; exit 1 ;; \
    esac \
    && rustup target add "$rust_target" \
    && cargo build --release --locked --target "$rust_target"
COPY src ./src
COPY migrations ./migrations
RUN --mount=type=cache,target=/usr/local/cargo/registry,sharing=locked \
    --mount=type=cache,target=/usr/local/cargo/git/db,sharing=locked \
    case "$TARGETARCH" in \
      amd64) rust_target=x86_64-unknown-linux-gnu ;; \
      arm64) rust_target=aarch64-unknown-linux-gnu ;; \
      *) echo "Unsupported target architecture: $TARGETARCH" >&2; exit 1 ;; \
    esac \
    && rustup target add "$rust_target" \
    && cargo build --release --locked --target "$rust_target" \
    && cp "/src/target/$rust_target/release/rust-arr" /rust-arr \
    && printf '%s\n' "$RAWARR_BUILD_REVISION" > /rust-arr-build-revision \
    && printf '%s\n' "$TARGETARCH" > /rust-arr-target-arch

FROM debian:bookworm-slim
ARG TARGETARCH
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates curl ffmpeg libssl3 librsvg2-bin libjxl-tools openssl python3 \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --system --uid 10001 --home-dir /nonexistent --shell /usr/sbin/nologin rustarr \
    && mkdir -p /data /shared/in /shared/out /ui \
    && ln -s /shared/in/metadata /metadata \
    && chown 10001:10001 /data /shared /shared/in /shared/out /ui
COPY --from=builder /rust-arr /usr/local/bin/rust-arr
COPY --from=builder /rust-arr-build-revision /usr/share/rawrarr/build-revision
COPY --from=builder /rust-arr-target-arch /usr/share/rawrarr/target-arch
COPY --from=age-builder /go/bin/age /go/bin/age-keygen /usr/local/bin/
COPY update_ui_bundle.py /usr/local/bin/rawrarr-update-ui
COPY ui-publisher-public-key.pem /etc/rawrarr/ui-publisher-public-key.pem
COPY container-entrypoint.sh /usr/local/bin/rawrarr-entrypoint
RUN chmod 755 /usr/local/bin/rawrarr-entrypoint /usr/local/bin/rawrarr-update-ui
RUN test "$(cat /usr/share/rawrarr/target-arch)" = "$TARGETARCH"
USER 10001:10001
EXPOSE 8080
ENV RUST_ARR_LISTEN_ADDR=0.0.0.0:8080
ENV RAWARR_DATA_DIR=/data
ENV RAWARR_SHARED_DATA_DIR=/shared
ENV RAWARR_METADATA_DIR=/metadata
ENV RAWARR_BUILD_REVISION_FILE=/usr/share/rawrarr/build-revision
ENTRYPOINT ["/usr/local/bin/rawrarr-entrypoint"]
HEALTHCHECK --interval=10s --timeout=3s --start-period=15s --retries=5 \
    CMD ["curl", "--fail", "--silent", "--show-error", "--output", "/dev/null", "http://127.0.0.1:8080/health"]
