FROM rust:1.96 AS builder
ARG TARGETARCH=amd64

RUN case "$TARGETARCH" in \
    amd64) TARGET_TRIPLE=x86_64-unknown-linux-musl ;; \
    arm64) TARGET_TRIPLE=aarch64-unknown-linux-musl ;; \
    *) echo "unsupported TARGETARCH=$TARGETARCH" >&2; exit 1 ;; \
    esac && \
    rustup target add "$TARGET_TRIPLE" && \
    mkdir -p /app

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY src ./src

RUN case "$TARGETARCH" in \
    amd64) TARGET_TRIPLE=x86_64-unknown-linux-musl ;; \
    arm64) TARGET_TRIPLE=aarch64-unknown-linux-musl ;; \
    esac && \
    cargo build --release --target "$TARGET_TRIPLE" && \
    cp /app/target/$TARGET_TRIPLE/release/web-server /web-server

FROM scratch
COPY --from=builder /web-server /web-server

ENTRYPOINT ["/web-server"]
