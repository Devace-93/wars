# Multi-stage build: compile the server with the stable toolchain, ship a slim image.
FROM rust:1-bookworm AS build
WORKDIR /src
COPY Cargo.toml Cargo.lock rust-toolchain.toml ./
COPY crates ./crates
COPY scripts ./scripts
RUN cargo build --release -p wars-server --locked

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates curl && rm -rf /var/lib/apt/lists/*
COPY --from=build /src/target/release/wars-server /usr/local/bin/wars-server
ENV PORT=8086
EXPOSE 8086
HEALTHCHECK --interval=30s --timeout=5s CMD curl -fsS http://localhost:8086/healthz || exit 1
CMD ["wars-server"]
