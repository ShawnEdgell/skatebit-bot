# Stage 1: Builder - Compile the Rust application
FROM rust:1.86-alpine AS builder

# Install C build tools, OpenSSL development libraries, pkgconf, and perl.
RUN apk add --no-cache build-base openssl-dev pkgconf perl

# Set environment variables to encourage static linking of OpenSSL
# and help openssl-sys find the libraries on Alpine.
ENV OPENSSL_STATIC=1
ENV OPENSSL_DIR=/usr
ENV OPENSSL_LIB_DIR=/usr/lib
ENV OPENSSL_INCLUDE_DIR=/usr/include

WORKDIR /usr/src/skatebit-bot

# Copy Cargo.toml and Cargo.lock to cache dependencies
COPY Cargo.toml Cargo.lock ./

# Create a dummy main.rs and build an empty project to cache dependencies.
RUN mkdir src && \
    echo "fn main() {println!(\"Dummy build for dependency caching\")}" > src/main.rs && \
    echo "Building dummy project to cache dependencies..." && \
    cargo build --release && \
    echo "Dummy project build complete. Cleaning up..." && \
    rm -rf src target

# Now copy your actual application source code
COPY ./src ./src

# Build your actual application binary in release mode
RUN echo "Building actual application..." && \
    cargo build --release && \
    echo "Actual application build complete."

# Stage 2: Runtime - Create a minimal final image
FROM alpine:latest

# Add ca-certificates for making HTTPS calls
RUN apk --no-cache add ca-certificates
# If OpenSSL was statically linked, we might not need runtime openssl libs.
# If dynamic linking happened or some part still needs it, you might need:
# RUN apk add --no-cache libssl3 libcrypto3 # Or just 'openssl'

WORKDIR /app

# Copy only the compiled binary from the builder stage's release target directory.
COPY --from=builder /usr/src/skatebit-bot/target/release/skatebit-bot .

# CMD to run the application
CMD ["./skatebit-bot"]
