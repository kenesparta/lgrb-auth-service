ARG UBUNTU_RELEASE=24.04
ARG CHISEL_VERSION=v1.1.0

# Start with image that has the Rust toolchain installed
FROM rust:1.86-alpine AS chef
USER root
# Add cargo-chef to cache dependencies
RUN apk add --no-cache musl-dev & cargo install cargo-chef
WORKDIR /app

FROM chef AS planner
COPY . .
# Capture info needed to build dependencies
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --release --recipe-path recipe.json
# Build application
COPY . .
RUN cargo build --release --bin auth-service

# Build Ubuntu chiseled
FROM ubuntu:$UBUNTU_RELEASE AS chiseled_builder
ARG UBUNTU_RELEASE
ARG CHISEL_VERSION
ADD https://github.com/canonical/chisel/releases/download/$CHISEL_VERSION/chisel_${CHISEL_VERSION}_linux_amd64.tar.gz chisel.tar.gz
RUN tar -xvf chisel.tar.gz -C /usr/bin/
RUN apt-get update \
    && DEBIAN_FRONTEND=noninteractive apt-get install -y ca-certificates
WORKDIR /rootfs
RUN chisel cut --release ubuntu-$UBUNTU_RELEASE --root /rootfs \
    base-files_base \
    base-files_release-info \
    ca-certificates_data \
    libgcc-s1_libs \
    libc6_libs

FROM scratch
WORKDIR /app
COPY --from=chiseled_builder /rootfs /
COPY --from=builder /app/target/release/auth-service /app
COPY --from=builder /app/assets /app/assets
ENTRYPOINT ["/app/auth-service"]
