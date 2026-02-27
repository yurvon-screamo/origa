# syntax=docker/dockerfile:1.4

ARG RUST_VERSION=1.88
ARG TRUNK_VERSION=0.21.14

FROM ubuntu:22.04 AS builder

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
COPY origa/Cargo.toml ./origa/
COPY tokenizer/Cargo.toml ./tokenizer/
COPY origa_ui/Cargo.toml ./origa_ui/
COPY tauri/Cargo.toml ./tauri/

RUN mkdir -p origa/src && echo "pub fn dummy() {}" > origa/src/lib.rs
RUN mkdir -p tokenizer/src && echo "pub fn dummy() {}" > tokenizer/src/lib.rs
RUN mkdir -p origa_ui/src && echo "pub fn dummy() {}" > origa_ui/src/lib.rs
RUN mkdir -p tauri/src && echo "pub fn dummy() {}" > tauri/src/lib.rs

RUN --mount=type=cache,target=/root/.cargo/registry \
    --mount=type=cache,target=/app/target \
    cargo build --release -p origa_ui && \
    rm -rf origa/src tokenizer/src origa_ui/src

COPY origa ./origa
COPY tokenizer ./tokenizer
COPY origa_ui ./origa_ui

RUN --mount=type=cache,target=/root/.cargo/registry \
    --mount=type=cache,target=/app/target \
    --mount=type=cache,target=/root/.cache/trunk \
    cd origa_ui && trunk build --release --dist dist

RUN cp -r origa_ui/dist /dist

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    nginx \
    tzdata \
    curl \
    && rm -rf /var/lib/apt/lists/*

COPY origa_ui/nginx.conf /etc/nginx/nginx.conf

COPY --from=builder /dist /usr/share/nginx/html

RUN chown -R www-data:www-data /usr/share/nginx/html && \
    chmod -R 755 /usr/share/nginx/html

EXPOSE 80

USER www-data

HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost/ || exit 1

CMD ["nginx", "-g", "daemon off;"]
