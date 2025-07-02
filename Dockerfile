# Stage 1: Builder
FROM rust:1.88-slim AS builder

RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    build-essential \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY config.yml ./

RUN cargo build --release

# Stage 2: Runtime
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    libssl3 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

RUN adduser --disabled-password --gecos '' appuser

WORKDIR /app

COPY --from=builder /app/target/release/LLM-router /app/LLM-router
COPY --from=builder /app/config.yml /app/config.yml

RUN chown -R appuser:appuser /app

USER appuser

EXPOSE 8080

CMD ["./LLM-router"]
