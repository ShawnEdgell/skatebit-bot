# Stage 0: Planner - Create a recipe for dependencies
FROM rust:1.86-alpine AS planner
# Match your project's Rust version
WORKDIR /app
RUN apk add --no-cache build-base openssl-dev pkgconf perl # Keep build deps
ENV OPENSSL_STATIC=1
ENV OPENSSL_DIR=/usr
ENV OPENSSL_LIB_DIR=/usr/lib
ENV OPENSSL_INCLUDE_DIR=/usr/include

# Install cargo-chef
RUN cargo install cargo-chef --locked

COPY . .
# Compute a recipe for dependencies
RUN cargo chef prepare --recipe-path recipe.json

# Stage 1: Cooker - Build dependencies based on the recipe
FROM rust:1.86-alpine AS cooker
WORKDIR /app
RUN apk add --no-cache build-base openssl-dev pkgconf perl # Keep build deps
ENV OPENSSL_STATIC=1
ENV OPENSSL_DIR=/usr
ENV OPENSSL_LIB_DIR=/usr/lib
ENV OPENSSL_INCLUDE_DIR=/usr/include

# Install cargo-chef (needed to cook)
RUN cargo install cargo-chef --locked
# Copy the recipe from the planner stage
COPY --from=planner /app/recipe.json recipe.json
# Build dependencies
RUN cargo chef cook --release --recipe-path recipe.json

# Stage 2: Builder - Build the application code
FROM rust:1.86-alpine AS builder
WORKDIR /app
RUN apk add --no-cache build-base openssl-dev pkgconf perl # Keep build deps
ENV OPENSSL_STATIC=1
ENV OPENSSL_DIR=/usr
ENV OPENSSL_LIB_DIR=/usr/lib
ENV OPENSSL_INCLUDE_DIR=/usr/include

COPY . .
# Copy over the pre-built dependencies from the cooker stage
COPY --from=cooker /app/target target
COPY --from=cooker /usr/local/cargo/registry /usr/local/cargo/registry

# Build the application, using the cached dependencies
RUN cargo build --release --locked

# Stage 3: Runtime - Create a minimal final image (your existing runtime stage)
FROM alpine:latest
RUN apk --no-cache add ca-certificates tzdata
ENV TZ=Etc/UTC
WORKDIR /app
COPY --from=builder /app/target/release/skatebit-bot .
# (Optional) Add a non-root user
# RUN addgroup -S appgroup && adduser -S appuser -G appgroup
# USER appuser
CMD ["./skatebit-bot"]