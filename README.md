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

# Run the service (will automatically create and seed SQLite database)
cargo run --release
```

The service will start on `http://localhost:8080` by default.

### Database Setup

The service uses **SQLite** by default with automatic migrations and seeding:

- **Database file**: `data/raito.db` (auto-created)
- **Migrations**: Run automatically on startup
- **Mock data**: Auto-seeded on first run

#### Environment Variables

| Variable                  | Description                  | Default                | Example                |
| ------------------------- | ---------------------------- | ---------------------- | ---------------------- |
| `DATABASE_URL`            | Database connection string   | `sqlite:data/raito.db` | `sqlite:data/raito.db` |
| `DATABASE_SEED`           | Seed database with mock data | `true`                 | `false`                |
| `DATABASE_RUN_MIGRATIONS` | Run migrations on startup    | `true`                 | `false`                |
| `PORT`                    | Server port                  | `8080`                 | `3000`                 |
| `RUST_LOG`                | Log level                    | `info`                 | `debug`                |

#### Alternative Database Configurations

```bash
# PostgreSQL (for production)
export DATABASE_URL="postgresql://user:password@localhost:5432/raito"

# In-memory SQLite (for testing)
export DATABASE_URL="sqlite::memory:"

# Custom SQLite location
export DATABASE_URL="sqlite:/path/to/custom.db"
```

## 📊 API Endpoints

### Blocks

- `GET /v1/blocks` - List recent blocks with pagination
- `GET /v1/blocks/{height|hash}` - Get block details by height or hash
- `GET /v1/blocks/{height}/proof` - Download STARK proof for a block

### Verification

- `GET /v1/tx/{txid}` - Check transaction inclusion status
- `GET /v1/header/{hash}` - Check block header existence

### Health & Monitoring

- `GET /healthz` - Service health check (includes database connectivity)
- `GET /metrics` - Prometheus metrics
- `GET /docs` - Interactive API documentation (Swagger UI)

### API Documentation

The service automatically generates OpenAPI 3.0 documentation available at:
- Interactive docs: `http://localhost:8080/docs`
- OpenAPI JSON: `http://localhost:8080/api-docs/openapi.json`

## 🔧 Development

### Database Management

```bash
# Run with fresh database
rm -f data/raito.db && cargo run

# Run without seeding mock data
DATABASE_SEED=false cargo run

# Run with debug logging including SQL queries
RUST_LOG=debug,sqlx=debug cargo run
```

### Running Tests

```bash
# Run all tests (uses in-memory database)
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

### Database Schema

The service uses a normalized SQLite schema:

- **blocks** - Bitcoin block information
- **transactions** - Transaction IDs with block associations  
- **proof_files** - STARK proof file metadata
- **block_headers** - Optimized header hash lookups

See `migrations/001_initial.sql` for the complete schema.

### Project Structure

```
├── src/
│   ├── main.rs          # Application entry point
│   ├── lib.rs           # Library root
│   ├── handlers.rs      # HTTP request handlers
│   ├── middleware.rs    # Custom middleware
│   ├── model.rs         # Data models and schemas
│   ├── database.rs      # Database operations and connection management
│   ├── store.rs         # Legacy mock store (for reference)
│   └── error.rs         # Error handling
├── migrations/          # Database migration files
│   └── 001_initial.sql  # Initial schema
├── data/
│   ├── mock_blocks.json # Mock block data
│   └── proofs/          # Mock STARK proof files
├── config/              # Configuration documentation
├── scripts/             # Demo and test scripts
└── .sqlx/               # SQLx query metadata (committed to git)
```

## 📈 Performance

The service is designed to handle:
- 100+ requests/second on a single vCPU
- Sub-300ms P95 latency for block operations
- Concurrent proof downloads
- SQLite supports millions of reads with WAL mode

## 🔒 Security Features

- CORS protection
- Security headers (CSP, HSTS, etc.)
- Input validation with database constraints
- SQL injection protection (compile-time checked queries)
- Rate limiting ready
- Structured JSON logging

## 📦 Docker

```bash
# Build image
docker build -t raito-proving-service .

# Run container with persistent database
docker run -p 8080:8080 -v $(pwd)/data:/app/data raito-proving-service

# Run with custom database URL
docker run -p 8080:8080 -e DATABASE_URL="sqlite:custom.db" raito-proving-service
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
      - DATABASE_URL=sqlite:data/raito.db
      - DATABASE_SEED=false  # Disable seeding in production
    volumes:
      - ./data:/app/data     # Persist database
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8080/healthz"]
      interval: 30s
      timeout: 10s
      retries: 3
```

### Production with PostgreSQL

```yaml
version: '3.8'
services:
  postgres:
    image: postgres:15
    environment:
      POSTGRES_DB: raito
      POSTGRES_USER: raito
      POSTGRES_PASSWORD: ${DB_PASSWORD}
    volumes:
      - postgres_data:/var/lib/postgresql/data

  raito-proving-service:
    image: ghcr.io/raito/proving-service:latest
    depends_on:
      - postgres
    environment:
      - DATABASE_URL=postgresql://raito:${DB_PASSWORD}@postgres:5432/raito
      - DATABASE_SEED=false
    ports:
      - "8080:8080"

volumes:
  postgres_data:
```

## 📚 Mock Data

The service includes realistic mock data for development and testing:

- **5 sample blocks** (heights 869119-869123)
- **STARK proof files** for each block
- **Transaction mappings** for inclusion checks
- **Header hash mappings** for verification

Mock data is automatically loaded from `data/mock_blocks.json` and `data/proofs/` on first startup.

## 🛣️ Roadmap

### Current (MVP)
- ✅ REST API with SQLite database
- ✅ OpenAPI documentation  
- ✅ Health checks and metrics
- ✅ Comprehensive testing
- ✅ Docker containerization
- ✅ Database migrations and seeding

### Next Steps
- [ ] Real STARK proof generation (Cairo + STWO integration)
- [ ] PostgreSQL support for production scaling
- [ ] Job queue for async proof generation
- [ ] WebSocket/SSE for progress updates
- [ ] Authentication and API keys
- [ ] Caching layer (Redis) for performance
- [ ] gRPC API for high-performance use cases

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes
4. Add tests for new functionality
5. Ensure all tests pass (`cargo test`)
6. Format code (`cargo fmt`)
7. Submit a pull request

### Database Changes

When modifying the database schema:

1. Create a new migration file in `migrations/`
2. Test the migration: `rm -f data/raito.db && cargo run`
3. Prepare SQLx queries: `cargo sqlx prepare`
4. Commit the `.sqlx/` directory changes

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
