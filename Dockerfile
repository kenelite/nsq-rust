# Multi-stage build for NSQ Rust
FROM rust:1.75-slim as builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /app

# Copy workspace files
COPY Cargo.toml Cargo.lock ./
COPY nsq-protocol/ ./nsq-protocol/
COPY nsq-common/ ./nsq-common/
COPY nsqd/ ./nsqd/
COPY nsqlookupd/ ./nsqlookupd/
COPY nsqadmin/ ./nsqadmin/
COPY tools/ ./tools/

# Build all components
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create nsq user
RUN useradd -r -s /bin/false nsq

# Create directories
RUN mkdir -p /var/lib/nsqd && chown nsq:nsq /var/lib/nsqd

# Copy binaries from builder stage
COPY --from=builder /app/target/release/nsqd /usr/local/bin/
COPY --from=builder /app/target/release/nsqlookupd /usr/local/bin/
COPY --from=builder /app/target/release/nsqadmin /usr/local/bin/
COPY --from=builder /app/target/release/nsq_to_file /usr/local/bin/
COPY --from=builder /app/target/release/to_nsq /usr/local/bin/
COPY --from=builder /app/target/release/nsq_tail /usr/local/bin/
COPY --from=builder /app/target/release/nsq_stat /usr/local/bin/
COPY --from=builder /app/target/release/nsq_to_http /usr/local/bin/
COPY --from=builder /app/target/release/nsq_to_nsq /usr/local/bin/

# Copy UI files
COPY nsqadmin-ui/dist/ /usr/local/share/nsqadmin-ui/

# Set permissions
RUN chmod +x /usr/local/bin/*

# Switch to nsq user
USER nsq

# Expose ports
EXPOSE 4150 4151 4160 4161 4171

# Default command
CMD ["nsqd"]
