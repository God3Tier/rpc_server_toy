# Adjust the source path below to wherever your actual Cargo project lives
# relative to the repo root (this assumes ./server/).
FROM rust:1-slim AS build
WORKDIR /build
COPY remote_server/Cargo.toml remote_server/Cargo.lock ./
COPY remote_server/src ./src
RUN cargo build --release

FROM debian:bookworm-slim
COPY --from=build /build/target/release/remote_server /usr/local/bin/remote_server
EXPOSE 15440
ENTRYPOINT ["/usr/local/bin/remote_server"]
