# Stage 1: Build the application
FROM rust:latest AS builder

# Set the working directory inside the container
WORKDIR /usr/src/app

# Copy the Cargo.toml and Cargo.lock to leverage Docker cache
COPY Cargo.toml Cargo.lock ./

# Pre-fetch dependencies
RUN cargo fetch

# Copy the entire project directory
COPY . .

# Build the application in release mode
RUN cargo build --release

# Stage 2: Create a lightweight image with the compiled binary
FROM debian:bookworm-slim

# Install only the necessary libraries for your binary
RUN apt-get update && apt-get install -y \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Copy the compiled binary from the builder stage
COPY --from=builder /usr/src/app/target/release/rust-cli-wallet /usr/local/bin/rust-cli-wallet

# Copy required directories into the container
COPY --from=builder /usr/src/app/abis /app/abis
COPY --from=builder /usr/src/app/config /app/config

# Set the working directory inside the container
WORKDIR /app

# Set the entrypoint to the CLI application
ENTRYPOINT ["rust-cli-wallet"]
