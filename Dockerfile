# Stage 1: Builder
FROM rust:1.88-alpine as builder

# Instalacja niezbędnych pakietów do kompilacji
RUN apk add --no-cache musl-dev openssl-dev pkgconfig

# Utworzenie nowego pustego projektu
WORKDIR /app
# Kopiowanie plików projektu
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY config.yaml ./

# Budowa w trybie release
RUN cargo build --release

# Stage 2: Runtime
FROM alpine:3.22

# Instalacja niezbędnych bibliotek runtime
RUN apk add --no-cache libgcc

# Utworzenie użytkownika bez uprawnień roota
RUN adduser -D appuser

WORKDIR /app

# Kopiowanie skompilowanego pliku binarnego z etapu budowania
COPY --from=builder /app/target/release/LLM-router /app/LLM-router
COPY --from=builder /app/config.yaml /app/config.yaml

# Zmiana właściciela plików
RUN chown -R appuser:appuser /app

# Przełączenie na użytkownika bez uprawnień roota
USER appuser

# Port na którym działa aplikacja
EXPOSE 8080

# Uruchomienie aplikacji
CMD ["./LLM-router"]
