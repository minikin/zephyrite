# Zephyrite

> A high-performance, distributed key-value store built in Rust

Zephyrite is a modern key-value database designed for speed, reliability, and scalability.
It combines the performance of in-memory operations with the durability of persistent storage.

## ✨ Planned Features

- **Fast In-Memory Storage**: High-performance key-value operations
- **HTTP REST API**: Simple, clean interface for all operations
- **Write-Ahead Log**: Persistence and crash recovery
- **Distributed Clustering**: Multi-node data distribution
- **Consensus Protocol**: Strong consistency guarantees

## 🚀 Quick Start

### Prerequisites

- **Rust 1.85+** (Edition 2024 support)
- **Just** (task runner) - `cargo install just`

### Installation

```bash
# Clone the repository
git clone https://github.com/yourusername/zephyrite
cd zephyrite

# Build the project (Phase 1 in development)
just build

# Note: Server functionality is currently being developed
```

### Current Status

🚧 **Phase 1 Development in Progress**

We're currently building the foundation of Zephyrite step by step. The basic project structure is set up with:

- ✅ Rust Edition 2024 project structure
- ✅ Development tooling (justfile, linting)
- ✅ Professional README and documentation
- 🚧 **Next**: Basic HTTP server and in-memory storage

### Coming Soon

Once Phase 1 is complete, you'll be able to:

```bash
# Start the server
just run

# Test basic operations
curl -X PUT http://127.0.0.1:8080/keys/hello \
  -H "Content-Type: application/json" \
  -d '{"value": "world"}'
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

### API Endpoints (Planned)

| Method   | Endpoint      | Description            | Status     |
| -------- | ------------- | ---------------------- | ---------- |
| `GET`    | `/`           | Health check           | 🚧 Phase 1 |
| `PUT`    | `/keys/{key}` | Store a key-value pair | 🚧 Phase 1 |
| `GET`    | `/keys/{key}` | Retrieve a value       | 🚧 Phase 1 |
| `DELETE` | `/keys/{key}` | Delete a key           | 🚧 Phase 1 |
| `GET`    | `/keys`       | List all keys          | 🚧 Phase 1 |

### Request/Response Format (Planned)

**Store a value:**

```bash
PUT /keys/mykey
Content-Type: application/json

{
  "value": "myvalue"
}
```

**Response:**

```
201 Created
```

**Retrieve a value:**

```bash
GET /keys/mykey
```

**Response:**

```json
{
  "key": "mykey",
  "value": "myvalue",
  "found": true
}
```

_Note: API is currently in development as part of Phase 1_

## 🧪 Testing

```bash
# Currently: Run project structure tests
just test

# Coming in Phase 1: Integration tests
# just test && curl http://127.0.0.1:8080/
```

## 📚 Project Structure

```
zephyrite/
├── src/
│   ├── main.rs           # CLI entry point (Phase 1)
│   ├── lib.rs            # Library root (Phase 1)
│   ├── config.rs         # Configuration (Phase 1)
│   ├── server/           # HTTP server (Phase 1)
│   │   └── mod.rs
│   └── storage/          # Storage engines (Phase 1)
│       ├── mod.rs
│       ├── engine.rs     # Storage trait (Phase 1)
│       ├── memory.rs     # In-memory impl (Phase 1)
│       └── error.rs      # Error types (Phase 1)
├── tests/                # Integration tests (Phase 1)
├── examples/             # Usage examples (Phase 1)
├── Cargo.toml           # Dependencies ✅
├── justfile             # Task commands ✅
└── README.md           # This file ✅
```

✅ = Complete | 🚧 = In development

## 🔧 Configuration

Currently, Zephyrite uses simple command-line configuration:

```bash
# Custom port
cargo run -- --port 9090

# Debug logging
cargo run -- --log-level debug
```

## 🗺️ Development Roadmap

### Phase 1: Foundation 🚧 **In Progress**

- [x] Project setup with Rust Edition 2024
- [x] Development tooling (justfile, linting)
- [x] Professional documentation
- [ ] **Next**: Basic HTTP server
- [ ] **Next**: In-memory key-value storage
- [ ] **Next**: REST API (GET, PUT, DELETE, LIST)
- [ ] **Next**: Comprehensive testing

### Phase 2: Persistence (Planned)

- [ ] Write-Ahead Log (WAL)
- [ ] Crash recovery
- [ ] On-disk storage
- [ ] Configuration files

### Phase 3: Distribution (Planned)

- [ ] Node discovery
- [ ] Data replication
- [ ] Consistent hashing

### Phase 4: Advanced Features (Planned)

- [ ] Consensus protocol (Raft)
- [ ] Transactions
- [ ] Performance optimizations

## 🤝 Contributing

We welcome contributions! Please see our development setup:

1. **Install prerequisites**: Rust 1.85+, Just
2. **Clone and build**: `git clone ... && just build`
3. **Run tests**: `just test`
4. **Follow our workflow**: `just dev` before committing

## 📄 License

MIT License - see [LICENSE](LICENSE) file for details.

## 🙋 Support

- **Issues**: [GitHub Issues](https://github.com/yourusername/zephyrite/issues)
- **Discussions**: [GitHub Discussions](https://github.com/yourusername/zephyrite/discussions)

**Zephyrite**: Fast, reliable, and built with Rust 🦀
