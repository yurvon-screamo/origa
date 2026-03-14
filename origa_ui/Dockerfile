# syntax=docker/dockerfile:1.4

ARG RUST_VERSION=1.93
ARG TRUNK_VERSION=0.21.14

FROM ubuntu:24.04 AS builder

SHELL ["/bin/bash", "-o", "pipefail", "-c"]

ARG RUST_VERSION
ARG TRUNK_VERSION

ARG ORIGA_VERSION=dev
ARG ORIGA_COMMIT=unknown
ARG ORIGA_BUILD_DATE=unknown

ENV ORIGA_VERSION=${ORIGA_VERSION}
ENV ORIGA_COMMIT=${ORIGA_COMMIT}
ENV ORIGA_BUILD_DATE=${ORIGA_BUILD_DATE}

ENV DEBIAN_FRONTEND=noninteractive

RUN apt-get update && apt-get install -y --no-install-recommends \
    binaryen \
    build-essential \
    ca-certificates \
    clang \
    curl \
    libssl-dev \
    lld \
    pkg-config \
    protobuf-compiler \
    && rm -rf /var/lib/apt/lists/* \
    && curl -fsSL https://github.com/mozilla/sccache/releases/download/v0.14.0/sccache-v0.14.0-x86_64-unknown-linux-musl.tar.gz | tar xz \
    && mv sccache-v0.14.0-x86_64-unknown-linux-musl/sccache /usr/local/bin/ \
    && chmod +x /usr/local/bin/sccache

ENV RUSTC_WRAPPER=sccache
ENV SCCACHE_DIR=/sccache

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain ${RUST_VERSION}
ENV PATH="/root/.cargo/bin:${PATH}"

WORKDIR /app

RUN rustup target add wasm32-unknown-unknown && \
    curl -fsSL https://github.com/trunk-rs/trunk/releases/download/v${TRUNK_VERSION}/trunk-x86_64-unknown-linux-gnu.tar.gz \
    | tar -xzf - -C /usr/local/bin

COPY Cargo.toml Cargo.lock ./
COPY tokenizer/Cargo.toml ./tokenizer/
COPY tauri/Cargo.toml ./tauri/
COPY origa/Cargo.toml ./origa/
COPY origa_ui/Cargo.toml ./origa_ui/

RUN mkdir -p origa/src && echo "pub fn dummy() {}" > origa/src/lib.rs && \
    mkdir -p origa_ui/src && echo "pub fn dummy() {}" > origa_ui/src/lib.rs && \
    mkdir -p tokenizer/src && echo "pub fn dummy() {}" > tokenizer/src/lib.rs && \
    mkdir -p tauri/src && echo "pub fn dummy() {}" > tauri/src/lib.rs

RUN --mount=type=cache,target=/sccache,sharing=locked \
    --mount=type=cache,target=/root/.cargo/registry,sharing=locked \
    cargo build --release -p origa_ui --target wasm32-unknown-unknown && \
    rm -rf origa origa_ui

COPY origa ./origa
COPY origa_ui ./origa_ui

RUN --mount=type=cache,target=/root/.cargo/registry \
    --mount=type=cache,target=/app/target \
    rm -rf origa_ui/public/dictionaries/unidic && \
    rm -rf /app/target/wasm32-unknown-unknown/release/.fingerprint/origa* && \
    rm -rf /app/target/wasm32-unknown-unknown/release/deps/origa* && \
    rm -rf /app/target/wasm32-unknown-unknown/release/deps/liboriga* && \
    rm -rf /app/target/wasm32-unknown-unknown/release/build/origa* && \
    cargo build --release -p origa --target wasm32-unknown-unknown && \
    cargo build --release -p origa_ui --target wasm32-unknown-unknown

WORKDIR /app/origa_ui

RUN --mount=type=cache,target=/root/.cargo/registry \
    --mount=type=cache,target=/app/target \
    --mount=type=cache,target=/root/.cache/trunk \
    trunk build --release --dist dist && \
    cp -r /app/origa_ui/dist /dist

FROM caddy:2.9-alpine AS caddy

FROM trailbase/trailbase:0.24.3

USER root
WORKDIR /app

# hadolint ignore=DL3018
RUN apk add --no-cache curl

COPY --from=caddy /usr/bin/caddy /usr/bin/caddy

COPY --from=builder /dist /app/public
COPY origa_ui/Caddyfile /app/Caddyfile
COPY origa_ui/entrypoint.sh /app/entrypoint.sh

RUN chmod +x /app/entrypoint.sh && \
    chown -R trailbase:trailbase /app

USER trailbase

EXPOSE 4000

HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD ["curl", "--fail", "http://localhost:4000/api/healthcheck"]

CMD ["/app/entrypoint.sh"]
