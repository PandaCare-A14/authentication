# Stage 1: Build
FROM rust:1.76-slim as builder

WORKDIR /app
COPY . .

RUN apt-get update && apt-get install -y libpq-dev pkg-config build-essential ca-certificates

RUN cargo build --release

# Stage 2: Runtime
FROM debian:buster-slim

RUN apt-get update && apt-get install -y libpq-dev ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/pandacare-auth /usr/local/bin/pandacare-auth

EXPOSE 8000
CMD ["pandacare-auth"]