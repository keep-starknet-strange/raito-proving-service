# Raito Proving Service

A trust-minimized backend service that generates, stores and serves STARK proofs, block headers and transaction-inclusion metadata for Bitcoin.

## 🚀 Quick Start

### Prerequisites

- Rust 1.70+ 
- Cargo

### Running the Service

```bash
# Clone the repository
git clone https://github.com/keep-starknet-strange/raito-proving-service
cd raito-proving-service

# Build the project
cargo build --release

# Run the service
cargo run --release
```

The service will start on `http://localhost:8080` by default.

### Environment Variables

- `PORT` - Server port (default: 8080)
- `RUST_LOG` - Log level (default: info)

## 📊 API Endpoints

### Blocks

- `GET /v1/blocks` - List recent blocks with pagination
- `GET /v1/blocks/{height|hash}` - Get block details by height or hash
- `GET /v1/blocks/{height}/proof` - Download STARK proof for a block

### Verification

- `GET /v1/tx/{txid}` - Check transaction inclusion status
- `GET /v1/header/{hash}` - Check block header existence

### Health & Monitoring

- `GET /healthz` - Service health check
- `GET /metrics` - Prometheus metrics
- `GET /docs` - Interactive API documentation (Swagger UI)

### API Documentation

The service automatically generates OpenAPI 3.0 documentation available at:
- Interactive docs: `http://localhost:8080/docs`
- OpenAPI JSON: `http://localhost:8080/api-docs/openapi.json`

## 🔧 Development

### Running Tests

```bash
# Run all tests
cargo test

# Run with coverage
cargo test --coverage

# Run specific test
cargo test test_health_check
```

### Code Quality

```bash
# Format code
cargo fmt

# Lint code
cargo clippy

# Security audit
cargo audit
```

### Project Structure

```
├── src/
│   ├── main.rs          # Application entry point
│   ├── lib.rs           # Library root
│   ├── handlers.rs      # HTTP request handlers
│   ├── middleware.rs    # Custom middleware
│   ├── model.rs         # Data models and schemas
│   ├── store.rs         # Mock data store
│   └── error.rs         # Error handling
├── data/
│   ├── mock_blocks.json # Mock block data
│   └── proofs/          # Mock STARK proof files
├── tests/               # Integration tests
└── docs/                # Additional documentation
```

## 📈 Performance

The service is designed to handle:
- 100+ requests/second on a single vCPU
- Sub-300ms P95 latency for block operations
- Concurrent proof downloads

## 🔒 Security Features

- CORS protection
- Security headers (CSP, HSTS, etc.)
- Input validation
- Rate limiting ready
- Structured JSON logging

## 📦 Docker

```bash
# Build image
docker build -t raito-proving-service .

# Run container
docker run -p 8080:8080 raito-proving-service
```

## 🚀 Deployment

### Using Docker Compose

```yaml
version: '3.8'
services:
  raito-proving-service:
    image: ghcr.io/raito/proving-service:latest
    ports:
      - "8080:8080"
    environment:
      - RUST_LOG=info
      - PORT=8080
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8080/healthz"]
      interval: 30s
      timeout: 10s
      retries: 3
```

### Kubernetes

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: raito-proving-service
spec:
  replicas: 2
  selector:
    matchLabels:
      app: raito-proving-service
  template:
    metadata:
      labels:
        app: raito-proving-service
    spec:
      containers:
      - name: service
        image: ghcr.io/raito/proving-service:latest
        ports:
        - containerPort: 8080
        env:
        - name: PORT
          value: "8080"
        livenessProbe:
          httpGet:
            path: /healthz
            port: 8080
          initialDelaySeconds: 10
          periodSeconds: 30
```

## 📚 Mock Data

The service currently serves mock data for development and testing:

- **5 sample blocks** (heights 869119-869123)
- **STARK proof files** for each block
- **Transaction mappings** for inclusion checks
- **Header hash mappings** for verification

Mock data is loaded from `data/mock_blocks.json` and `data/proofs/` directory.

## 🛣️ Roadmap

### Current (MVP)
- ✅ REST API with mocked data
- ✅ OpenAPI documentation
- ✅ Health checks and metrics
- ✅ Comprehensive testing
- ✅ Docker containerization

### Next Steps
- [ ] Real STARK proof generation (Cairo + STWO integration)
- [ ] Job queue for async proof generation
- [ ] WebSocket/SSE for progress updates
- [ ] PostgreSQL + S3 persistence
- [ ] Authentication and API keys
- [ ] gRPC API for high-performance use cases

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes
4. Add tests for new functionality
5. Ensure all tests pass (`cargo test`)
6. Format code (`cargo fmt`)
7. Submit a pull request

### Code Standards

- Follow Rust best practices and idioms
- Maintain test coverage above 90%
- Use structured logging with appropriate levels
- Document public APIs with examples
- Keep functions small and focused

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🔗 Related Projects

- [Raito](https://github.com/keep-starknet-strange/raito) - Bitcoin consensus client in Cairo
- [Shinigami](https://github.com/keep-starknet-strange/shinigami) - Bitcoin Script verification
- [ZeroSync](https://github.com/ZeroSync/ZeroSync) - Inspiration for the project

## 📞 Contact

- [Telegram](https://t.me/ShinigamiStarknet)
- [GitHub Issues](https://github.com/keep-starknet-strange/raito-proving-service/issues)

---

Built with ❤️ by the Raito team
