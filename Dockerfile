# Use the official Rust image as the base image
FROM rust:latest as builder
ENV SQLX_OFFLINE=true
ENV PKG_CONFIG_ALLOW_CROSS=1

# Create a new directory for the application code
WORKDIR /usr/src/powerfox

# Copy the Cargo.toml and Cargo.lock files to the container
COPY Cargo.toml Cargo.lock ./

# Build a dummy project to cache dependencies
RUN mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    cargo build --release

# Copy the application source code to the container
COPY src ./src
COPY .env sqlx-data.json ./

# Build the release version of the application
RUN cargo build --release

# Create a new stage for the final minimal image
FROM debian:bullseye

# Copy the compiled binary from the builder stage to the final image
COPY --from=builder /usr/src/powerfox/target/release/powerfox /usr/local/bin/powerfox

# Install system dependencies (if required by your application)
RUN apt-get update && apt-get install -y pkg-config libssl-dev neovim

EXPOSE 3000

# Set the entrypoint command for the container
CMD ["bash"]
