# Use the official Rust image as the base image
FROM rust:latest as builder
ENV SQLX_OFFLINE=true
ENV PKG_CONFIG_ALLOW_CROSS=1

# Create a new directory for the application code
WORKDIR /usr/src/powerfox

# Copy the application source code to the container
COPY . .

# Build the release version of the application
RUN cargo build --release

# Create a new stage for the final minimal image
FROM debian:bookworm-slim

# Copy the compiled binary from the builder stage to the final image
COPY --from=builder /usr/src/powerfox/target/release/powerfox /usr/local/bin/powerfox

# make certificates work
RUN apt-get update && apt-get install -y ca-certificates && update-ca-certificates

EXPOSE 3000

# Set the entrypoint command for the container
CMD ["powerfox"]
