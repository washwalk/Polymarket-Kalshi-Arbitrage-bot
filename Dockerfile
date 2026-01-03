# --- STAGE 1: Build & Cache Warming ---
FROM rust:latest AS builder

# Install system dependencies for Rust and Python
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    python3 \
    python3-pip \
    && rm -rf /var/lib/apt/lists/*

# Install Python requirements for the cache script
RUN pip3 install aiohttp --break-system-packages

WORKDIR /app

# Copy only the dependency files first (optimizes Docker layer caching)
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs && cargo build --release

# Copy the rest of the source code
COPY . .

# Warm the Polymarket sports cache (required by the bot at startup)
# Note: This requires network access during build
RUN python3 scripts/build_sports_cache.py

# Final compilation of the actual bot
RUN cargo build --release

# --- STAGE 2: Lightweight Runtime ---
FROM debian:bookworm-slim

# Install SSL certificates (required for connecting to Kalshi/Poly APIs)
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the binary and required JSON caches from the builder stage
COPY --from=builder /app/target/release/prediction-market-arbitrage .
COPY --from=builder /app/.clob_market_cache.json .
COPY --from=builder /app/kalshi_team_cache.json .

# Ensure the binary has execution permissions
RUN chmod +x ./prediction-market-arbitrage

# Set environment to production
ENV RUST_LOG=arb_bot=info
ENV DRY_RUN=1

# Run the bot
CMD ["./prediction-market-arbitrage"]