# Stage 1: Build
FROM rust:1.86-slim AS builder

# Set working directory
WORKDIR /app

# Install only the necessary build dependencies
RUN apt-get update && apt-get install -y \
    libpq-dev \
    pkg-config \
    build-essential \
    && rm -rf /var/lib/apt/lists/*

# Copy Cargo.toml and Cargo.lock first to leverage Docker cache for dependencies
COPY Cargo.toml Cargo.lock ./

# Create a dummy main.rs to build dependencies
RUN mkdir -p src && \
    echo "fn main() {}" > src/main.rs && \
    cargo build --release && \
    rm -rf src

# Copy the actual source code
COPY src src/

# Rebuild with actual source code
RUN cargo build --release

# Stage 2: Runtime - Using Ubuntu 22.04 with newer GLIBC
FROM ubuntu:22.04

# Avoid prompts from apt
ENV DEBIAN_FRONTEND=noninteractive

# Install only the necessary runtime dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    libpq5 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Copy the compiled binary from the builder stage
COPY --from=builder /app/target/release/pandacare-auth /usr/local/bin/pandacare-auth

# Expose the necessary port
EXPOSE 8000

# Command to run the application
CMD ["pandacare-auth"]