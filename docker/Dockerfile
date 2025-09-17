# Build stage
FROM rust:1.89-bookworm AS builder

WORKDIR /app

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Copy source code
COPY src ./src
COPY migrations ./migrations

# Build for release
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    libssl3 \
    libpq5 \
    ca-certificates \
    iproute2 \
    net-tools \
    && rm -rf /var/lib/apt/lists/*

# Create app user
RUN useradd -m -u 1000 flowdns

WORKDIR /app

# Copy binary from builder
COPY --from=builder /app/target/release/flowdns /app/flowdns

# Copy configurations and migrations
COPY config ./config
COPY migrations ./migrations

# Create directories for data
RUN mkdir -p /app/data && chown -R flowdns:flowdns /app

# Expose ports
EXPOSE 53/udp 53/tcp 67/udp 68/udp 8080/tcp

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD ["/app/flowdns", "--health-check"]

# Switch to non-root user (Note: DHCP/DNS may require root)
# USER flowdns

# Set environment variables
ENV RUST_LOG=info
ENV DATABASE_URL=postgresql://flowdns:password@postgres/flowdns

# Run the binary
ENTRYPOINT ["/app/flowdns"]