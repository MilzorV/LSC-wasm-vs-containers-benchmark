# syntax=docker/dockerfile:1

FROM rust:latest AS builder

WORKDIR /app

COPY Cargo.toml Cargo.lock* ./
COPY src ./src

RUN cargo build --release

FROM debian:bookworm-slim

WORKDIR /app

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/docker-file-tools /usr/local/bin/docker-file-tools

EXPOSE 8081

ENV HOST=0.0.0.0
ENV PORT=8081

CMD ["docker-file-tools"]
