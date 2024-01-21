# Use the official Rust image as a base
FROM rust:latest as builder

# Install build dependencies
RUN apt-get update && apt-get install -y ca-certificates cmake musl-tools libssl-dev && rm -rf /var/lib/apt/lists/*

# Ensure musl-gcc is properly set up
RUN ln -s /usr/bin/musl-gcc /usr/local/bin/musl-gcc

# Set up Rust for musl target (for a smaller, static binary)
RUN rustup default nightly && rustup update && rustup target add x86_64-unknown-linux-musl
RUN rustup component add clippy llvm-tools-preview

# Set the working directory in the container
WORKDIR /usr/src/yahoo-finance-metrics

# Copy the Cargo manifest files
COPY Cargo.toml Cargo.lock ./

# Copy your source code into the container
COPY src/ ./src/

# Build the application
ENV PKG_CONFIG_ALLOW_CROSS=1
RUN cargo build --target x86_64-unknown-linux-musl --release

# Stage 2: Final Image
FROM alpine:latest

WORKDIR /usr/yahoo-finance-metrics

# Install runtime dependencies including Chromium for headless browsing
RUN apk --no-cache add ca-certificates chromium chromium-chromedriver musl-dev 

# Copy the binary from the builder stage
COPY --from=builder /usr/src/yahoo-finance-metrics/target/x86_64-unknown-linux-musl/release/yahoo-finance-metrics /usr/local/bin/yahoo-finance-metrics

# Expose port
EXPOSE 8080

# Command to run when starting the container
CMD ["yahoo-finance-metrics"]
