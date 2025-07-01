# Stage 1: Builder
FROM rust:1.88-alpine as builder

RUN apk add --no-cache musl-dev openssl-dev pkgconfig

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY config.yaml ./

RUN cargo build --release

# Stage 2: Runtime
FROM alpine:3.22

RUN apk add --no-cache libgcc

RUN adduser -D appuser

WORKDIR /app

COPY --from=builder /app/target/release/LLM-router /app/LLM-router
COPY --from=builder /app/config.yaml /app/config.yaml

RUN chown -R appuser:appuser /app

USER appuser

EXPOSE 8080

CMD ["./LLM-router"]
