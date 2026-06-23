FROM rust:1.96 AS builder

RUN rustup target add aarch64-unknown-linux-musl

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY src ./src

RUN cargo build --release --target aarch64-unknown-linux-musl

FROM scratch

COPY --from=builder \
    /app/target/aarch64-unknown-linux-musl/release/web-server \
    /web-server

ENTRYPOINT ["/web-server"]
