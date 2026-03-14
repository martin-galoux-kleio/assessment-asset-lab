# Build stage
# Rust 1.85+ required for deps using edition 2024 (e.g. time-core)
FROM rust:1.85.0-bookworm AS builder
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
