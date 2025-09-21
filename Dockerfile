# STAGE 1: Build the application in a dedicated build environment
# Using a recent, patched version of the base image to mitigate vulnerabilities.
FROM rust:alpine AS builder

# Install the linker for our static build target (musl)
RUN apk add --no-cache musl-dev

# Add the musl target to the Rust toolchain for static linking
RUN rustup target add x86_64-unknown-linux-musl

WORKDIR /usr/src/app

# Set the binary name. Make sure this matches the `name` field in your Cargo.toml
ARG APP_NAME=rust_glooo_project

# Copy dependency manifest files
COPY Cargo.toml Cargo.lock ./

# Build dependencies first to leverage Docker's layer caching.
# We build for the musl target to create a static binary.
RUN mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    cargo build --release --target x86_64-unknown-linux-musl && \
    rm -rf src

# Now, copy your actual application source code
COPY src ./src

# Build the application for the musl target. This creates a fully static binary.
RUN cargo build --release --target x86_64-unknown-linux-musl


# STAGE 2: Create the final, minimal production image
# Using a recent, patched version of Alpine for the final image.
FROM alpine:3.20

# Add ca-certificates for making HTTPS requests (good practice)
RUN apk add --no-cache ca-certificates

# Set the binary name. Make sure this matches the `name` field in your Cargo.toml
ARG APP_NAME=rust-glooo-project

# Copy the statically compiled binary from the builder stage.
# Note that the path now includes the target architecture, which is crucial.
COPY --from=builder /usr/src/app/target/x86_64-unknown-linux-musl/release/${APP_NAME} /usr/local/bin/

# Set the command to run your application. This is essential.
# Without CMD, the container will start and immediately exit.
CMD ["rust_glooo_project"]

