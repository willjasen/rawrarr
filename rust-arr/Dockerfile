# syntax=docker/dockerfile:1

FROM golang:1.25-bookworm AS age-builder
RUN go install filippo.io/age/cmd/...@v1.3.2

FROM rust:1.85-bookworm AS builder
WORKDIR /src
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY migrations ./migrations
RUN --mount=type=cache,target=/usr/local/cargo/registry,sharing=locked \
    --mount=type=cache,target=/usr/local/cargo/git/db,sharing=locked \
    --mount=type=cache,target=/src/target,sharing=locked \
    cargo build --release --locked \
    && cp /src/target/release/rust-arr /rust-arr

FROM debian:bookworm-slim
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates curl libssl3 \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --system --uid 10001 --home-dir /nonexistent --shell /usr/sbin/nologin rustarr \
    && mkdir -p /data \
    && ln -s /shared/metadata /metadata \
    && chown 10001:10001 /data
COPY --from=builder /rust-arr /usr/local/bin/rust-arr
COPY --from=age-builder /go/bin/age /go/bin/age-keygen /usr/local/bin/
USER 10001:10001
EXPOSE 8080
ENV RUST_ARR_LISTEN_ADDR=0.0.0.0:8080
ENV RAWARR_DATA_DIR=/data
ENV RAWARR_SHARED_DATA_DIR=/shared
ENV RAWARR_METADATA_DIR=/metadata
VOLUME ["/data", "/shared"]
ENTRYPOINT ["/usr/local/bin/rust-arr"]
HEALTHCHECK --interval=10s --timeout=3s --start-period=15s --retries=5 \
    CMD ["curl", "--fail", "--silent", "--show-error", "--output", "/dev/null", "http://127.0.0.1:8080/health"]
