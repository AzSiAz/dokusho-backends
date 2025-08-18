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

# Build the worker binary with SQLx offline mode
ENV SQLX_OFFLINE=true
RUN cargo build --release --bin worker

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
COPY --from=builder /app/target/release/worker /usr/local/bin/worker

# Set ownership
RUN chown appuser:appuser /usr/local/bin/worker

# Switch to non-root user
USER appuser

# Run the worker
CMD ["worker"]
