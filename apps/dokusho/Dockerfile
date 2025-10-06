# Build stage
FROM rust:1.89-slim AS builder

# Install dependencies for building
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Create app directory
WORKDIR /app

# Copy workspace files
COPY Cargo.toml Cargo.lock ./
COPY crates/ ./crates/
COPY apps/ ./apps/

# Build the API binary with SQLx offline mode
ENV SQLX_OFFLINE=true
RUN cargo build --release --bin dokusho

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -m -u 1001 appuser

# Copy the binary from builder
COPY --from=builder /app/target/release/dokusho /usr/local/bin/dokusho

# Set ownership
RUN chown appuser:appuser /usr/local/bin/dokusho

# Switch to non-root user
USER appuser

# Expose port (adjust if needed)
EXPOSE 8080

# Run the API server
CMD ["dokusho"]
