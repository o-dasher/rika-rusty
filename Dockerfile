# Stage 1: Build the bot and the server.
FROM rust:latest AS builder

WORKDIR /app

# Copy required files
COPY src src
COPY Cargo.toml Cargo.toml
COPY Cargo.lock Cargo.lock

ARG BUILD_ENV=production

RUN echo "Building in $BUILD_ENV mode";

# Build the binary based on the environment
RUN if [ "$BUILD_ENV" = "production" ]; then \
        cargo build --release; \
    else \
        cargo build; \
    fi

# Stage 2: Create minimal ubuntu image
FROM ubuntu:latest

# Expose the port for Discord communication
EXPOSE 443

WORKDIR /app

# Copy the .env file from the build context into the Docker image
COPY .env /app/.env

# Copy the built binary from the builder image
COPY --from=builder /app/target/releas[e] /app/target/debu[g] .

CMD ["./osaka-bot"]
