FROM golang:1.25-bookworm AS age-builder
RUN go install filippo.io/age/cmd/...@v1.3.2

FROM rust:1.85-bookworm AS builder
WORKDIR /src
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY migrations ./migrations
RUN cargo build --release --locked

FROM debian:bookworm-slim
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates libssl3 \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --system --uid 10001 --home-dir /nonexistent --shell /usr/sbin/nologin rustarr \
    && mkdir -p /data \
    && chown 10001:10001 /data
COPY --from=builder /src/target/release/rust-arr /usr/local/bin/rust-arr
COPY --from=age-builder /go/bin/age /go/bin/age-keygen /usr/local/bin/
USER 10001:10001
EXPOSE 8080
ENV RUST_ARR_LISTEN_ADDR=0.0.0.0:8080
ENV RAWARR_DATA_DIR=/data
VOLUME ["/data"]
ENTRYPOINT ["/usr/local/bin/rust-arr"]
