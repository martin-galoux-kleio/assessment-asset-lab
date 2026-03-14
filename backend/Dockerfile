# Build stage
# Rust 1.91+ required for aws-sdk-* and time crates
FROM rust:1.91-bookworm AS builder
WORKDIR /app

COPY Cargo.toml Cargo.lock* ./
COPY src ./src

RUN cargo build --release

# Run stage
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /app/target/release/assetlab-backend /app/assetlab-backend
EXPOSE 3000
ENV RUST_LOG=info
CMD ["/app/assetlab-backend"]
