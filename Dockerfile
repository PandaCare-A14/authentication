# Stage 1: Build the application
FROM rust:1.86-slim AS builder

# Set working directory
WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y \
    libpq-dev \
    pkg-config \
    build-essential \
    && rm -rf /var/lib/apt/lists/*

# Copy dependency manifest files first (to leverage Docker layer caching)
COPY Cargo.toml Cargo.lock ./

# Pre-build dependencies using a dummy main.rs to speed up rebuilds
RUN mkdir -p src && echo "fn main() {}" > src/main.rs && \
    cargo build --release && rm -rf src

# Copy the actual source code
COPY src src/

# Build the application in release mode
RUN cargo build --release

# Stage 2: Runtime environment
FROM ubuntu:22.04

ENV DEBIAN_FRONTEND=noninteractive

# Install only the runtime dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    libpq5 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Set default logging environment variables
ENV RUST_LOG=info
ENV RUST_BACKTRACE=1

# Set the working directory inside the container
WORKDIR /app

# Copy the compiled binary from the builder stage
COPY --from=builder /app/target/release/pandacare-auth /usr/local/bin/pandacare-auth

# Copy the startup script
COPY start.sh /app/
RUN chmod +x /app/start.sh

# Expose the port used by the application
EXPOSE 8000
HEALTHCHECK CMD curl --fail http://localhost:8000/ || exit 1

# Run the startup script as the container entry point
CMD ["/app/start.sh"]