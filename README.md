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
  - [🔧 Configuration](#-configuration)
  - [🗺️ Development Roadmap](#️-development-roadmap)
    - [Phase 1: Foundation ✅ **Complete**](#phase-1-foundation--complete)
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
- **Node.js** (optional) - For automated commit tools and validation

### Installation

```bash
# Clone the repository
git clone https://github.com/yourusername/zephyrite
cd zephyrite

# Build the project
just build
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
just dev      # Format + lint + test
just run      # Start the server
just test     # Run tests
just fmt      # Format code
just lint     # Run linting
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

```bash
# Run all tests
just test

# Run with coverage
cargo test

# Integration tests are included
# Tests cover HTTP endpoints, storage operations, and error handling
```

The test suite includes:

- **Unit Tests**: Storage engine, validation, and utilities
- **Integration Tests**: HTTP API endpoints and error scenarios
- **Documentation Tests**: Code examples in documentation

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

### Phase 1: Foundation ✅ **Complete**

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

- [Conventional Commits Setup](docs/CONVENTIONAL_COMMITS.md) - Detailed commit message guide
- API Documentation (complete - see above)
- Development Guide (coming soon)

## 🤝 Contributing

We welcome contributions! Please see our development setup:

1. **Install prerequisites**: Rust 1.85+, Just
2. **Optional**: Install Node.js for enhanced commit tools
3. **Clone and build**: `git clone ... && just build`
4. **Setup conventional commits**: `just setup-git`
5. **Run tests**: `just test`
6. **Follow our workflow**: `just dev` before committing
7. **Use conventional commits**: See `just commit-examples` for format

## 📄 License

MIT License - see [LICENSE](LICENSE) file for details.

## 🙋 Support

- **Issues**: [GitHub Issues](https://github.com/yourusername/zephyrite/issues)
- **Discussions**: [GitHub Discussions](https://github.com/yourusername/zephyrite/discussions)
