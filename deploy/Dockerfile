# Multi-stage build: compile the server with the stable toolchain, ship a slim image.
FROM rust:1-bookworm AS build
WORKDIR /src
COPY Cargo.toml Cargo.lock rust-toolchain.toml ./
COPY crates ./crates
COPY scripts ./scripts
COPY apps/web ./apps/web
RUN cargo build --release -p wars-server --locked
# Browser client: wasm32 build + wasm-bindgen glue, served by the server at /play/.
ARG WASM_BINDGEN_VERSION=0.2.129
RUN rustup target add wasm32-unknown-unknown \
    && curl -fsSL "https://github.com/wasm-bindgen/wasm-bindgen/releases/download/${WASM_BINDGEN_VERSION}/wasm-bindgen-${WASM_BINDGEN_VERSION}-x86_64-unknown-linux-musl.tar.gz" \
       | tar -xz -C /usr/local/bin --strip-components=1 "wasm-bindgen-${WASM_BINDGEN_VERSION}-x86_64-unknown-linux-musl/wasm-bindgen" \
    && cargo build --release --target wasm32-unknown-unknown -p wars-web --locked \
    && wasm-bindgen --target web --no-typescript --out-dir apps/web/pkg target/wasm32-unknown-unknown/release/wars_web.wasm

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates curl && rm -rf /var/lib/apt/lists/*
COPY --from=build /src/target/release/wars-server /usr/local/bin/wars-server
COPY --from=build /src/apps/web /app/web
ENV PORT=8086 WEB_DIR=/app/web
EXPOSE 8086
HEALTHCHECK --interval=30s --timeout=5s CMD curl -fsS http://localhost:8086/healthz || exit 1
CMD ["wars-server"]
