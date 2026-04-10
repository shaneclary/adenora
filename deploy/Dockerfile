# Stage 1: Build Rust backend
FROM rust:latest AS rust-builder
WORKDIR /app
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*
COPY Cargo.toml Cargo.lock ./
COPY crates/ crates/
COPY migrations/ migrations/
RUN cargo build --release --bin adenora

# Stage 2: Build frontend
FROM node:22-slim AS web-builder
WORKDIR /app/web
COPY web/package.json web/package-lock.json* ./
RUN npm install --legacy-peer-deps
COPY web/ ./
RUN npm run build

# Stage 3: Runtime
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates curl && rm -rf /var/lib/apt/lists/*

COPY --from=rust-builder /app/target/release/adenora /usr/local/bin/adenora
COPY --from=web-builder /app/web/build /srv/web
COPY migrations/ /app/migrations/

WORKDIR /app
EXPOSE 8080

ENV RUST_LOG=info
HEALTHCHECK --interval=10s --timeout=3s --retries=3 \
    CMD curl -f http://localhost:8080/healthz || exit 1

CMD ["adenora"]
