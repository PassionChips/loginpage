# Multi-stage build for Rust application
# Stage 1: Build the application
FROM rust:1.83-slim-bookworm AS builder

# Install required dependencies for Diesel and PostgreSQL
RUN apt-get update && apt-get install -y \
    libpq-dev \
    pkg-config \
    && rm -rf /var/lib/apt/lists/*

# Create a new empty shell project
WORKDIR /app

# Copy manifests
COPY Cargo.toml Cargo.lock ./
COPY diesel.toml ./

# Copy source code
COPY src ./src

# Build for release
RUN cargo build --release --bin run_main

# Stage 2: Runtime image
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    libpq5 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create app user
RUN useradd -m -u 1000 appuser

WORKDIR /app

# Copy the binary from builder
COPY --from=builder /app/target/release/run_main /app/run_main

# Copy static files
COPY --from=builder /app/src/static /app/src/static

# Change ownership to app user
RUN chown -R appuser:appuser /app

# Switch to app user
USER appuser

# Expose port
EXPOSE 3000

# Set environment variables
ENV RUST_LOG=info

# Run the binary
CMD ["/app/run_main"]