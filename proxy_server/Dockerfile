FROM rust:1-slim AS build
WORKDIR /build/proxy_server
COPY proxy_server/Cargo.toml proxy_server/Cargo.lock ./
COPY proxy_server/src ./src
RUN cargo build --release

FROM debian:bookworm-slim
COPY --from=build /build/proxy_server/target/release/proxy_server /usr/local/bin/proxy_server
EXPOSE 15440
ENTRYPOINT ["/usr/local/bin/proxy_server"]