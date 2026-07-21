FROM rust:1.96 AS builder

ARG TARGETARCH=arm64

RUN apt-get update && apt-get install -y \
    musl-tools \
    perl \
    make \
    gcc \
    && rm -rf /var/lib/apt/lists/*

RUN case "$TARGETARCH" in \
    amd64) TARGET_TRIPLE=x86_64-unknown-linux-musl ;; \
    arm64) TARGET_TRIPLE=aarch64-unknown-linux-musl ;; \
    *) echo "unsupported TARGETARCH=$TARGETARCH" >&2; exit 1 ;; \
    esac && \
    rustup target add "$TARGET_TRIPLE"

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY crates ./crates
COPY src ./src
COPY assets ./assets

RUN case "$TARGETARCH" in \
    amd64) TARGET_TRIPLE=x86_64-unknown-linux-musl ;; \
    arm64) TARGET_TRIPLE=aarch64-unknown-linux-musl ;; \
    esac && \
    cargo build --release --target "$TARGET_TRIPLE" && \
    cp target/$TARGET_TRIPLE/release/http-server /http-server


FROM scratch

COPY --from=builder /http-server /http-server
COPY --from=builder /app/assets /assets

# COPY cert.pem /cert.pem
# COPY key.pem /key.pem

EXPOSE 8443

ENTRYPOINT ["/http-server"]
