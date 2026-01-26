FROM rust:1.93-slim-bookworm AS builder

WORKDIR /app

RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Cache dependencies by building them first
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release && rm -rf src target/release/deps/discord_bot*

# Build the actual application
COPY src ./src
COPY migrations ./migrations
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

RUN useradd --create-home --shell /bin/sh bot
USER bot
WORKDIR /home/bot

COPY --from=builder /app/target/release/discord-bot ./discord-bot
COPY config ./config

ENTRYPOINT ["./discord-bot"]
