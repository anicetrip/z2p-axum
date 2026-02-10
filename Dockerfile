# =====================
# Builder
# =====================
FROM rust:bookworm AS builder
WORKDIR /app

RUN apt-get update && apt-get install -y \
    clang \
    lld \
    binutils \
 && rm -rf /var/lib/apt/lists/*

COPY . .

# SeaORM 不需要 SQLX_OFFLINE
RUN cargo build --release

# 如果你的 binary 名字是 z2p-axum
RUN strip target/release/z2p-axum


# =====================
# Runtime
# =====================
FROM debian:bookworm-slim
WORKDIR /app

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
 && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/z2p-axum /app/z2p-axum
COPY configuration /app/configuration

ENV APP_ENVIRONMENT=production

ENTRYPOINT ["/app/z2p-axum"]
