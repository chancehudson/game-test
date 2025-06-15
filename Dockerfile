# Multi-stage build for smaller final image
FROM rust:1.87-slim AS builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /app

COPY . ./

# Build the actual application
RUN cargo build --bin=server --features=server --no-default-features --release

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /app

# Copy binary from builder stage
COPY --from=builder /app/target/release/server ./server
COPY --from=builder /app/assets ./assets

# Expose port (adjust as needed)
EXPOSE 1351

# Run the server
CMD ["./server"]
