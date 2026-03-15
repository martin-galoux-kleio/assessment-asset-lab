# Build stage
FROM rust:1.91-bookworm AS builder
WORKDIR /app
COPY Cargo.toml Cargo.lock* ./
COPY src ./src
RUN cargo build --release

# Run stage — no ffmpeg needed, MediaConvert handles transcoding
FROM debian:bookworm-slim
RUN apt-get update \
    && apt-get install -y ca-certificates \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /app/target/release/assetlab-worker /app/assetlab-worker
ENV RUST_LOG=info
CMD ["/app/assetlab-worker"]
