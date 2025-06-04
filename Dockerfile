# Build stage
FROM rust:1.75-alpine as builder

# Install dependencies
RUN apk add --no-cache musl-dev pkgconfig openssl-dev

# Create app user
RUN addgroup -g 1001 -S app && \
    adduser -S -D -H -u 1001 -G app -s /bin/sh app

# Set working directory
WORKDIR /app

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Create dummy source to cache dependencies
RUN mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    echo "pub fn add(left: usize, right: usize) -> usize { left + right }" > src/lib.rs

# Build dependencies
RUN cargo build --release && \
    rm -rf src target/release/deps/raito_proving_service*

# Copy source code
COPY src ./src
COPY data ./data

# Build application
RUN cargo build --release --bin raito-proving-service

# Runtime stage
FROM alpine:3.19

# Install runtime dependencies
RUN apk add --no-cache ca-certificates tzdata && \
    update-ca-certificates

# Create app user
RUN addgroup -g 1001 -S app && \
    adduser -S -D -H -u 1001 -G app -s /bin/sh app

# Create app directory
RUN mkdir -p /app/data && \
    chown -R app:app /app

# Copy binary and data
COPY --from=builder /app/target/release/raito-proving-service /app/
COPY --from=builder /app/data /app/data
COPY --chown=app:app . /app/

# Set ownership
RUN chown -R app:app /app

# Switch to non-root user
USER app

# Set working directory
WORKDIR /app

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD /app/raito-proving-service --help > /dev/null || exit 1

# Expose port
EXPOSE 8080

# Set default environment
ENV RUST_LOG=info
ENV PORT=8080

# Run the application
CMD ["./raito-proving-service"] 