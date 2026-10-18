FROM rust:1.85-bookworm AS builder
WORKDIR /src
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release --locked

FROM debian:bookworm-slim
RUN useradd --system --uid 10001 --home-dir /nonexistent --shell /usr/sbin/nologin rustarr
COPY --from=builder /src/target/release/rust-arr /usr/local/bin/rust-arr
USER 10001:10001
EXPOSE 8080
ENV RUST_ARR_LISTEN_ADDR=0.0.0.0:8080
ENTRYPOINT ["/usr/local/bin/rust-arr"]
