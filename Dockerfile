# Stage 1: Build the bot and the server.
FROM rust:latest AS builder

WORKDIR /app

# Copy required files
COPY src src
COPY Cargo.toml Cargo.toml
COPY Cargo.lock Cargo.lock

# Build the Bot
RUN cargo build --release

# Stage 2: Create minimal ubuntu image
FROM ubuntu:latest

# Expose the port for Discord communication
EXPOSE 443

WORKDIR /app

# Copy the .env file from the build context into the Docker image
COPY .env /app/.env

# Copy the built binaries
COPY --from=builder /app/target/release .

CMD ["./osaka-bot"]
