# syntax=docker/dockerfile:1
FROM alpine:latest AS builder

RUN apk add --no-cache \
    rust cargo \
    aarch64-linux-gnu-gcc \
    musl-dev \
    cmake make

WORKDIR /build
COPY runtime/ .

ENV CARGO_TARGET_AARCH64_UNKNOWN_LINUX_MUSL_LINKER=aarch64-linux-gnu-gcc
RUN cargo build --target aarch64-unknown-linux-musl --release

# Rootfs builder
FROM archlinux/archlinux:latest AS rootfs
COPY --from=builder /build/target/aarch64-unknown-linux-musl/release/astrashelld /usr/bin/
COPY build/scripts/gen-rootfs.sh /tmp/
RUN /tmp/gen-rootfs.sh /tmp/rootfs

# Final image
FROM scratch
COPY --from=rootfs /tmp/astrashell-arch-rootfs.tar.gz /rootfs.tar.gz
COPY --from=rootfs /usr/bin/astrashelld /astrashelld
