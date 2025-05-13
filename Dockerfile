FROM rust:alpine3.21 as builder

WORKDIR /usr/src/pandacare-auth
COPY . .
RUN cargo install --path .

FROM alpine:latest as runner
RUN apk add --no-cache build-base

COPY --from=builder /usr/local/cargo/bin/pandacare-auth /usr/local/bin/pandacare-auth
CMD ["pandacare-auth"]
