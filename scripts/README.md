# Raito Proving Service - Demo Scripts

This directory contains scripts to demonstrate and test the Raito Proving Service API.

## Scripts

### 🚀 `demo.sh` - Complete API Demonstration

**Full-featured demo script that starts the service and showcases all endpoints**

```bash
./scripts/demo.sh
```

**Features:**
- ✅ Automatically builds and starts the service
- ✅ Comprehensive testing of all API endpoints
- ✅ Colored output with detailed explanations
- ✅ Performance testing with concurrent requests
- ✅ Error case demonstrations
- ✅ Service lifecycle management (auto-stop on exit)
- ✅ Real-time service monitoring

**Requirements:**
- `curl` (required)
- `jq` (optional - for pretty JSON formatting)
- `bc` (for performance calculations)

**What it tests:**
- Health check endpoint (`/healthz`)
- Prometheus metrics (`/metrics`)
- API documentation (`/docs`)
- Block listing (`/v1/blocks`)
- Block details (`/v1/blocks/{height|hash}`)
- STARK proof download (`/v1/blocks/{height}/proof`)
- Transaction verification (`/v1/tx/{txid}`)
- Header verification (`/v1/header/{hash}`)
- Error handling and validation
- Concurrent request performance

---

### 🧪 `test_api.sh` - Quick API Test

**Lightweight script for quick API testing (assumes service is running)**

```bash
# First, start the service manually:
cargo run --release &

# Then run the quick test:
./scripts/test_api.sh
```

**Features:**
- ✅ Fast execution (~10 seconds)
- ✅ Tests all core endpoints
- ✅ Compact output with key information
- ✅ Error case validation
- ✅ No service management overhead

**Use cases:**
- Quick validation during development
- CI/CD pipeline integration
- API regression testing
- Manual service verification

---

## Usage Examples

### Run Full Demo

```bash
# Complete demonstration with service management
./scripts/demo.sh

# The script will:
# 1. Build the service
# 2. Start it in background
# 3. Run comprehensive tests
# 4. Keep service running for manual testing
# 5. Clean up on Ctrl+C
```

### Quick Test Against Running Service

```bash
# Terminal 1: Start service
cargo run --release

# Terminal 2: Test API
./scripts/test_api.sh
```

### Development Workflow

```bash
# Make changes to code
# Quick test during development
cargo run --release &
./scripts/test_api.sh
pkill raito-proving-service

# Full demo before commit
./scripts/demo.sh
```

---

## Expected Output Examples

### Health Check
```json
{
  "status": "up",
  "timestamp": 1704067200
}
```

### Blocks List
```json
{
  "blocks": [
    {
      "height": 869123,
      "hash": "0000000000000000000264e1b06b0f6f8b0c7e9e5b8b8f9c9d8b7a6f5e4d3c2b1a",
      "tx_count": 2456,
      "total_fees": 0.12345678,
      "timestamp": 1704067200,
      "verified": true
    }
  ],
  "total": 5,
  "has_next": false,
  "next_cursor": null
}
```

### STARK Proof
```json
{
  "block_height": 869123,
  "block_hash": "0000000000000000000264e1b06b0f6f8b0c7e9e5b8b8f9c9d8b7a6f5e4d3c2b1a",
  "proof_version": "v1.0",
  "stark_proof": {
    "trace": { ... },
    "public_inputs": { ... }
  },
  "verification_key": { ... }
}
```

---

## Script Features

### Color-coded Output
- 🟢 **Green**: Success messages and responses
- 🟡 **Yellow**: Warnings and process updates  
- 🔵 **Blue**: Section headers and info
- 🔴 **Red**: Errors and failures
- 🟣 **Purple**: Headers and highlights

### Error Handling
- Service startup validation
- Endpoint availability checks
- Response format validation
- Graceful error reporting
- Automatic cleanup

### Performance Testing
- Concurrent request handling
- Response time measurement
- Throughput validation
- Load testing simulation

---

## Troubleshooting

### Service Won't Start
```bash
# Check if port is in use
lsof -i :8080

# Kill existing service
pkill raito-proving-service

# Check logs
RUST_LOG=debug cargo run --release
```

### Missing Dependencies
```bash
# Install jq for JSON formatting
brew install jq        # macOS
sudo apt install jq    # Ubuntu

# Install curl
brew install curl      # macOS  
sudo apt install curl  # Ubuntu
```

### Permission Issues
```bash
# Make scripts executable
chmod +x scripts/*.sh

# Or run with bash
bash scripts/demo.sh
```

---

## Integration with CI/CD

### GitHub Actions Example
```yaml
- name: Test API Endpoints
  run: |
    cargo run --release &
    sleep 5
    ./scripts/test_api.sh
    pkill raito-proving-service
```

### Docker Testing
```bash
# Build and test in container
docker build -t raito-test .
docker run -p 8080:8080 raito-test &
./scripts/test_api.sh
```

---

## Next Steps

- **Performance Benchmarking**: Add load testing with tools like `wrk`
- **Integration Tests**: Extend with end-to-end test scenarios
- **Monitoring**: Add health check validation scripts
- **Automation**: Create deployment validation scripts 