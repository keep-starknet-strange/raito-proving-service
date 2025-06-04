# Environment Configuration

## Database Settings

| Variable                   | Description                  | Default                | Example                |
| -------------------------- | ---------------------------- | ---------------------- | ---------------------- |
| `DATABASE_URL`             | Database connection string   | `sqlite:data/raito.db` | `sqlite:data/raito.db` |
| `DATABASE_MAX_CONNECTIONS` | Maximum database connections | `10`                   | `10`                   |
| `DATABASE_RUN_MIGRATIONS`  | Run migrations on startup    | `true`                 | `true`                 |
| `DATABASE_SEED`            | Seed database with mock data | `true`                 | `true`                 |

## Server Settings

| Variable   | Description           | Default                                                       | Example |
| ---------- | --------------------- | ------------------------------------------------------------- | ------- |
| `PORT`     | HTTP server port      | `8080`                                                        | `8080`  |
| `RUST_LOG` | Logging configuration | `info,raito_proving_service=debug,tower_http=debug,sqlx=info` | `debug` |

## Database URL Examples

### SQLite (Development/Production)
```bash
DATABASE_URL=sqlite:data/raito.db
```

### SQLite In-Memory (Testing)
```bash
DATABASE_URL=sqlite::memory:
```

### PostgreSQL (Production)
```bash
DATABASE_URL=postgresql://user:password@localhost:5432/raito
```

## Example Configuration

### Development
```bash
PORT=8080
DATABASE_URL=sqlite:data/raito.db
DATABASE_SEED=true
RUST_LOG=debug,sqlx=info
```

### Testing
```bash
DATABASE_URL=sqlite::memory:
DATABASE_SEED=true
RUST_LOG=info
```

### Production
```bash
PORT=8080
DATABASE_URL=sqlite:data/raito.db
DATABASE_SEED=false
DATABASE_MAX_CONNECTIONS=20
RUST_LOG=info,raito_proving_service=debug
``` 