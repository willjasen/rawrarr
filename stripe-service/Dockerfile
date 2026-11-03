FROM rust:1.85-bookworm AS build
WORKDIR /build
COPY stripe-service/Cargo.toml stripe-service/Cargo.lock ./
COPY stripe-service/src ./src
COPY stripe-service/migrations ./migrations
RUN find ./migrations -type f -name '._*' -delete
RUN cargo build --locked --release

FROM debian:bookworm-slim
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --system --uid 10001 --no-create-home rawrarr
COPY --from=build /build/target/release/rawrarr-stripe-service /usr/local/bin/rawrarr-stripe-service
USER 10001:10001
EXPOSE 8090
ENTRYPOINT ["/usr/local/bin/rawrarr-stripe-service"]
