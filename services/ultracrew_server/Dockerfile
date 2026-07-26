# ─────────────────────────────────────────────────────────────────────────────
# Stage 1: Build
# Uses the official Rust image. Builds only the ultracrew_server binary from
# the Cargo workspace root.
# ─────────────────────────────────────────────────────────────────────────────
FROM rust:1.79-slim AS builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the entire workspace (Cargo.toml + all members)
COPY . .

# Build only the ultracrew_server binary in release mode
RUN cargo build --release --bin ultracrew_server

# ─────────────────────────────────────────────────────────────────────────────
# Stage 2: Runtime
# Minimal Debian slim image — no Rust toolchain, just the binary + data files.
# ─────────────────────────────────────────────────────────────────────────────
FROM debian:bookworm-slim AS runtime

# Install runtime dependencies (OpenSSL, CA certs for HTTPS)
RUN apt-get update && apt-get install -y \
    libssl3 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the compiled binary
COPY --from=builder /app/target/release/ultracrew_server /app/ultracrew_server

# Copy the INRC scenario data files that the server reads at startup
COPY --from=builder /app/adapters/ultracrew/tests/data /app/adapters/ultracrew/tests/data

# Render injects PORT; the server binds to 0.0.0.0:3001 by default.
# If Render assigns a different port, set PORT env var and the server will use it.
EXPOSE 3001

ENV RUST_LOG=info

CMD ["/app/ultracrew_server"]