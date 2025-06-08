# Default recipe - shows available commands
default:
    @just --list

# Run the server
run:
    cargo run

# Run tests
test:
    cargo test

# Format code
fmt:
    cargo fmt

# Run linting
lint:
    cargo clippy -- -D warnings

# Development workflow (format + lint + test)
dev: fmt lint test

# Clean build artifacts
clean:
    cargo clean

# Build the project
build:
    cargo build
