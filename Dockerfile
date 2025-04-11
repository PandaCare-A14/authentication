# Stage 1: Build
FROM rust:1.76 as builder

WORKDIR /app
COPY . .

RUN cargo build --release

# Stage 2: Runtime
FROM debian:buster-slim

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/pandacare-auth /usr/local/bin/pandacare-auth

ENV PORT=8000

EXPOSE 8000

CMD ["pandacare-auth"]