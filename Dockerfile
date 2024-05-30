FROM lukemathwalker/cargo-chef:latest-rust-1 AS chef
WORKDIR /app

ARG DATABASE_URL
ENV DATABASE_URL=$DATABASE_URL

FROM chef AS planner
COPY . .

RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json

RUN rustup install nightly

# Build dependencies - this is the caching Docker layer!
RUN cargo +nightly chef cook --release --recipe-path recipe.json

# Build application
COPY . .
RUN cargo build --release --bin osaka-bot

# We do not need the Rust toolchain to run the binary!
FROM debian:bookworm-slim AS runtime
WORKDIR /app
COPY --from=builder /app/target/release/osaka-bot /usr/local/bin
ENTRYPOINT ["/usr/local/bin/osaka-bot"]
