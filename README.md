# Zephyrite

> A high-performance, distributed key-value store built in Rust

Zephyrite is a modern key-value database designed for speed, reliability, and scalability.
It combines the performance of in-memory operations with the durability of persistent storage.

- [Zephyrite](#zephyrite)
  - [✨ Current Features](#-current-features)
  - [🚀 Quick Start](#-quick-start)
    - [Prerequisites](#prerequisites)
    - [Installation](#installation)
    - [Running the Server](#running-the-server)
    - [API Usage Examples](#api-usage-examples)
  - [🛠️ Development](#️-development)
    - [Commands](#commands)
    - [API Endpoints](#api-endpoints)
    - [Request/Response Format](#requestresponse-format)
  - [🧪 Testing](#-testing)
    - [Quick Testing](#quick-testing)
    - [Targeted Testing](#targeted-testing)
    - [Advanced Testing](#advanced-testing)
    - [Nextest Profiles](#nextest-profiles)
    - [Test Organization](#test-organization)
    - [Test Features](#test-features)
  - [🔧 Configuration](#-configuration)
    - [Basic Options](#basic-options)
    - [Persistent Storage \& Crash Recovery](#persistent-storage--crash-recovery)
    - [Crash Recovery Behavior](#crash-recovery-behavior)
  - [🗺️ Development Roadmap](#️-development-roadmap)
    - [Phase 1: Foundation ](#phase-1-foundation--complete)
    - [Phase 2: Persistence (WIP)](#phase-2-persistence-wip)
    - [Phase 3: Distribution (Planned)](#phase-3-distribution-planned)
    - [Phase 4: Advanced Features (Planned)](#phase-4-advanced-features-planned)
  - [📚 Documentation](#-documentation)
  - [🤝 Contributing](#-contributing)
  - [📄 License](#-license)
  - [🙋 Support](#-support)

## ✨ Current Features

- **Fast In-Memory Storage**: High-performance key-value operations with metadata tracking
- **Persistent Storage with WAL**: Write-Ahead Logging ensures durability and crash recovery
- **Crash Recovery**: Automatic restoration of data from WAL on server restart
- **HTTP REST API**: Fully functional interface for all CRUD operations
- **Comprehensive Validation**: Robust key and value validation with security checks
- **Structured Logging**: Detailed tracing and observability
- **Error Handling**: Comprehensive error responses with proper HTTP status codes
- **Metadata Tracking**: Automatic timestamps and size tracking for stored values
- **Flexible Storage Options**: Choose between in-memory or persistent storage modes

## 🚀 Quick Start

### Prerequisites

- **Rust 1.85+** (Edition 2024 support)
- **Just** (task runner) - `cargo install just`
- **cargo-nextest** (enhanced test runner) - `just install-nextest`
- **Node.js** (optional) - For automated commit tools and validation

### Installation

```bash
# Clone the repository
git clone https://github.com/yourusername/zephyrite
cd zephyrite

# Install nextest (enhanced test runner)
just install-nextest

# Build the project
just build
```

**Optional:** Install additional testing tools for coverage and watch mode:

```bash
just install-test-tools  # Installs nextest, cargo-llvm-cov, and cargo-watch
```

### Running the Server

```bash
# Start the server with in-memory storage (default port 8080)
just run

# Start with persistent storage and crash recovery
cargo run -- --persistent

# Start with custom WAL file path
cargo run -- --wal-file ./data/my-database.wal

# Start with custom configuration
cargo run -- --port 3000 --log-level debug --persistent --memory-capacity 10000
```

### API Usage Examples

Once the server is running, you can interact with it using curl:

**Health Check:**

```bash
curl -X GET http://localhost:8080/health
# Response: {"status":"ok","version":"0.1.0","service":"Zephyrite"}
```

**Store a Key-Value Pair:**

```bash
curl -X PUT http://localhost:8080/keys/user:john \
  -H "Content-Type: application/json" \
  -d '{"value": "{\"name\":\"John Doe\",\"age\":30}"}'
# Response: 201 Created (for new keys) or 200 OK (for updates)
```

**Retrieve a Value:**

```bash
curl -X GET http://localhost:8080/keys/user:john
# Response: {
#   "key": "user:john",
#   "value": "{\"name\":\"John Doe\",\"age\":30}",
#   "found": true,
#   "size": 28,
#   "created_at": "2025-06-22T10:30:14.050Z",
#   "updated_at": "2025-06-22T10:30:14.050Z"
# }
```

**List All Keys:**

```bash
curl -X GET http://localhost:8080/keys
# Response: {"keys":["user:john","config:app"],"count":2}
```

**Delete a Key:**

```bash
curl -X DELETE http://localhost:8080/keys/user:john
# Response: 204 No Content (if key existed) or 404 Not Found
```

**Error Handling:**

```bash
curl -X GET http://localhost:8080/keys/nonexistent
# Response: {"error":"key_not_found","message":"Key 'nonexistent' not found"}
```

## 🛠️ Development

### Commands

```bash
# Development workflow
just dev      # Format + lint + test (comprehensive)
just run      # Start the server

# Testing (powered by nextest)
just test     # Run tests with nextest (local-dev profile)
just test-fast # Quick test run with fail-fast behavior
just test-all  # Run all tests including doctests
just test-ci   # CI profile with retries and JUnit output

# Code quality
just fmt      # Format code
just lint     # Run linting

# Additional testing commands (run 'just test-help' for full list)
just test-unit        # Unit tests only
just test-integration # Integration tests only
just test-storage     # Storage-specific tests
just test-coverage    # Generate HTML coverage report
just test-watch       # Watch mode for continuous testing
just test-profiles    # Show available nextest profiles
```

### API Endpoints

| Method   | Endpoint      | Description            | Status  |
| -------- | ------------- | ---------------------- | ------- |
| `GET`    | `/health`     | Health check           | ✅ Done |
| `PUT`    | `/keys/{key}` | Store a key-value pair | ✅ Done |
| `GET`    | `/keys/{key}` | Retrieve a value       | ✅ Done |
| `DELETE` | `/keys/{key}` | Delete a key           | ✅ Done |
| `GET`    | `/keys`       | List all keys          | ✅ Done |

### Request/Response Format

**Store a value:**

```http
PUT /keys/mykey
Content-Type: application/json

{
  "value": "myvalue"
}
```

**Response:**

- `201 Created` - New key created
- `200 OK` - Existing key updated
- `400 Bad Request` - Invalid key or value

**Retrieve a value:**

```http
GET /keys/mykey
```

**Response:**

```json
{
  "key": "mykey",
  "value": "myvalue",
  "found": true,
  "size": 7,
  "created_at": "2025-06-22T10:30:14.050Z",
  "updated_at": "2025-06-22T10:30:14.050Z"
}
```

**List keys:**

```http
GET /keys
```

**Response:**

```json
{
  "keys": ["key1", "key2", "key3"],
  "count": 3
}
```

**Delete a key:**

```http
DELETE /keys/mykey
```

**Response:**

- `204 No Content` - Key successfully deleted
- `404 Not Found` - Key does not exist
- `400 Bad Request` - Invalid key format

**Error Response Format:**

```json
{
  "error": "error_code",
  "message": "Human-readable error description"
}
```

## 🧪 Testing

Zephyrite uses **cargo-nextest** for enhanced test execution with better performance, improved output formatting, and advanced CI/CD integration.

### Quick Testing

```bash
# Run all tests (recommended for development)
just test

# Quick iteration with fail-fast behavior
just test-fast

# Run all tests including doctests
just test-all

# Get help with all testing commands
just test-help
```

### Targeted Testing

```bash
# Run only unit tests (excludes integration tests)
just test-unit

# Run only integration tests (HTTP API)
just test-integration

# Run storage-specific tests
just test-storage
```

### Advanced Testing

```bash
# Generate HTML coverage report (opens in browser)
just test-coverage

# Watch mode - auto-rerun tests on file changes
just test-watch

# CI mode with retries and JUnit output
just test-ci

# Show available nextest profiles
just test-profiles
```

### Nextest Profiles

Zephyrite includes several optimized test profiles:

- **local-dev**: Default profile for development with verbose output
- **fast**: Quick profile with fail-fast behavior for rapid iteration
- **integration-only**: Runs integration tests single-threaded to avoid port conflicts
- **ci**: CI profile with automatic retries and JUnit XML output
- **coverage**: Optimized for code coverage collection

### Test Organization

The test suite includes:

- **Unit Tests**: Storage engine, validation, utilities, and WAL operations
- **Integration Tests**: HTTP API endpoints, error scenarios, and end-to-end workflows
- **Documentation Tests**: Code examples in documentation and API contracts
- **Storage Tests**: In-memory and persistent storage with crash recovery scenarios

### Test Features

- **Parallel Execution**: Tests run in separate processes for better isolation
- **Smart Scheduling**: Longer tests start first to optimize total runtime
- **Port Management**: Integration tests run single-threaded to prevent conflicts
- **Automatic Retries**: Flaky tests are automatically retried in CI mode
- **JUnit Output**: XML reports for CI/CD integration
- **Coverage Reports**: HTML coverage reports with detailed metrics

## 🔧 Configuration

Zephyrite supports comprehensive command-line configuration:

### Basic Options

```bash
# Default: port 8080, in-memory storage, info logging
just run

# Custom port
cargo run -- --port 9090

# Debug logging
cargo run -- --log-level debug

# Combined options
cargo run -- --port 3000 --log-level trace
```

**Available log levels:** `trace`, `debug`, `info`, `warn`, `error`

### Persistent Storage & Crash Recovery

```bash
# Enable persistent storage with default WAL file (zephyrite.wal)
cargo run -- --persistent

# Custom WAL file location
cargo run -- --wal-file ./data/database.wal

# Set initial memory capacity (entries)
cargo run -- --persistent --memory-capacity 50000

# Disable WAL checksums (faster writes, less safe)
cargo run -- --persistent --no-checksums

# Full configuration example
cargo run -- \
  --port 8080 \
  --log-level info \
  --wal-file ./data/prod.wal \
  --memory-capacity 100000
```

### Crash Recovery Behavior

When using persistent storage (`--persistent` or `--wal-file`):

1. **On Startup**: Zephyrite automatically reads the WAL file and replays all operations
2. **During Operation**: All write operations (PUT, DELETE, CLEAR) are logged to WAL before execution
3. **On Crash**: Data is preserved in the WAL and will be recovered on next startup
4. **Checksum Verification**: WAL entries are verified for integrity (can be disabled with `--no-checksums`)

The recovery process is automatic and requires no manual intervention.

## 🗺️ Development Roadmap

### Phase 1: Foundation

- [x] Project setup and tooling
- [x] HTTP server with Axum
- [x] In-memory key-value storage
- [x] Complete REST API (GET, PUT, DELETE, LIST)
- [x] Comprehensive input validation
- [x] Error handling and logging
- [x] Full test coverage

### Phase 2: Persistence (WIP)

- [x] Write-Ahead Log (WAL)
- [x] Crash recovery
- [ ] On-disk storage
- [ ] Configuration files
- [ ] Backup and restore

### Phase 3: Distribution (Planned)

- [ ] Node discovery
- [ ] Data replication
- [ ] Consistent hashing
- [ ] Cluster management

### Phase 4: Advanced Features (Planned)

- [ ] Consensus protocol (Raft)
- [ ] Transactions
- [ ] Performance optimizations
- [ ] Metrics and monitoring

## 📚 Documentation

- [Nextest Setup](docs/NEXTEST_SETUP.md) - Comprehensive testing setup and configuration
- [Key Validation Rules](docs/KEY_VALIDATION_RULES.md) - Detailed key validation specifications
- [Conventional Commits Setup](docs/CONVENTIONAL_COMMITS.md) - Detailed commit message guide
- API Documentation (complete - see above)
- Development Guide (coming soon)

## 🤝 Contributing

We welcome contributions! Please see our development setup:

1. **Install prerequisites**: Rust 1.85+, Just, and nextest (`just install-nextest`)
2. **Optional**: Install Node.js for enhanced commit tools and additional testing tools (`just install-test-tools`)
3. **Clone and build**: `git clone ... && just build`
4. **Setup conventional commits**: `just setup-git`
5. **Run tests**: `just test` (or `just test-fast` for quick iteration)
6. **Follow our workflow**: `just dev` before committing (runs format + lint + comprehensive tests)
7. **Use conventional commits**: See `just commit-examples` for format

**Testing workflow**: Use `just test-help` to see all available testing commands. We recommend `just test-fast` during development and `just test-all` before committing.

## 📄 License

MIT License - see [LICENSE](LICENSE) file for details.

## 🙋 Support

- **Issues**: [GitHub Issues](https://github.com/yourusername/zephyrite/issues)
- **Discussions**: [GitHub Discussions](https://github.com/yourusername/zephyrite/discussions)
