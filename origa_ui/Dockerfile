# syntax=docker/dockerfile:1.4

ARG RUST_VERSION=1.93
ARG TRUNK_VERSION=0.21.14

FROM ubuntu:24.04 AS builder

ARG RUST_VERSION
ARG TRUNK_VERSION

ENV DEBIAN_FRONTEND=noninteractive

RUN apt-get update && apt-get install -y --no-install-recommends \
    curl \
    ca-certificates \
    protobuf-compiler \
    pkg-config \
    libssl-dev \
    build-essential \
    clang \
    lld \
    && rm -rf /var/lib/apt/lists/*

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain ${RUST_VERSION}
ENV PATH="/root/.cargo/bin:${PATH}"

WORKDIR /app

RUN rustup target add wasm32-unknown-unknown

RUN curl -fsSL https://github.com/trunk-rs/trunk/releases/download/v${TRUNK_VERSION}/trunk-x86_64-unknown-linux-gnu.tar.gz \
    | tar -xzf - -C /usr/local/bin

COPY Cargo.toml Cargo.lock ./
COPY tokenizer/Cargo.toml ./tokenizer/
COPY tauri/Cargo.toml ./tauri/
COPY origa/Cargo.toml ./origa/
COPY origa_ui/Cargo.toml ./origa_ui/

RUN mkdir -p origa/src && echo "pub fn dummy() {}" > origa/src/lib.rs
RUN mkdir -p origa_ui/src && echo "pub fn dummy() {}" > origa_ui/src/lib.rs
RUN mkdir -p tokenizer/src && echo "pub fn dummy() {}" > tokenizer/src/lib.rs
RUN mkdir -p tauri/src && echo "pub fn dummy() {}" > tauri/src/lib.rs

RUN --mount=type=cache,target=/root/.cargo/registry \
    --mount=type=cache,target=/app/target \
    cargo build --release -p origa_ui --target wasm32-unknown-unknown && \
    rm -rf origa origa_ui

COPY origa ./origa
COPY origa_ui ./origa_ui

RUN rm -rf origa_ui/public/dictionaries/unidic

RUN --mount=type=cache,target=/root/.cargo/registry \
    --mount=type=cache,target=/app/target \
    cargo clean -p origa -p origa_ui && \
    cargo build --release -p origa --target wasm32-unknown-unknown && \
    cargo build --release -p origa_ui --target wasm32-unknown-unknown

RUN --mount=type=cache,target=/root/.cargo/registry \
    --mount=type=cache,target=/app/target \
    --mount=type=cache,target=/root/.cache/trunk \
    cd origa_ui && trunk build --release --dist dist

RUN cp -r origa_ui/dist /dist

FROM trailbase/trailbase:0.24.2

USER root
WORKDIR /app

COPY --from=builder /dist /app/public

RUN chown -R trailbase:trailbase /app/public

USER trailbase

EXPOSE 4000

HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl --fail http://localhost:4000/api/healthcheck || exit 1

CMD ["/app/trail", "run", "--address", "0.0.0.0:4000", "--public-dir", "/app/public", "--spa"]
