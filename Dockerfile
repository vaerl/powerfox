# Use the official Rust image as the base image
FROM rust:latest as builder

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
COPY .env ./.env

# Build the release version of the application
RUN cargo build --release

# Create a new stage for the final minimal image
FROM debian:buster-slim

# Copy the compiled binary from the builder stage to the final image
COPY --from=builder /usr/src/powerfox/target/release/powerfox /usr/local/bin/powerfox

EXPOSE 3000

# Set the entrypoint command for the container
CMD ["powerfox"]
