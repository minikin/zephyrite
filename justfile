# Default recipe - shows available commands
default:
    @just --list

# Run the server
run:
    cargo run

# Run tests with nextest
test:
    cargo nextest run --profile local-dev

# Run all tests (including doctests which nextest doesn't handle)
test-all:
    cargo nextest run --profile local-dev
    cargo test --doc

# Run tests with coverage (requires cargo-llvm-cov)
test-coverage:
    cargo llvm-cov nextest --profile local-dev --html --open

# Run integration tests only
test-integration:
    cargo nextest run --profile local-dev 'test(/http_server/)'

# Run unit tests only (excluding integration tests)
test-unit:
    cargo nextest run --profile local-dev 'not test(/http_server/)'

# Run storage tests specifically
test-storage:
    cargo nextest run --profile local-dev 'test(/storage/)'

# Run tests in CI mode (with retries and JUnit output)
test-ci:
    cargo nextest run --profile ci

# Watch tests (requires cargo-watch)
test-watch:
    cargo watch -x "nextest run --profile local-dev"

# Show test results from last run
test-results:
    cargo nextest show-config test-groups
    @echo "📊 Test artifacts in target/nextest/"

# Format code
fmt:
    cargo fmt

# Run linting
lint:
    cargo clippy -- -D warnings

# Development workflow (format + lint + test)
dev: fmt lint test-all

# Clean build artifacts (including nextest cache)
clean:
    cargo clean
    rm -rf target/nextest

# Build the project
build:
    cargo build

# Setup git commit template
setup-git:
    git config commit.template .gitmessage
    @echo "✅ Git commit template configured"
    @echo "💡 Now 'git commit' will show the template"

# Interactive commit (if commitizen is available)
commit:
    #!/usr/bin/env bash
    if command -v npx >/dev/null 2>&1 && [ -f "node_modules/.bin/git-cz" ]; then
        npx git-cz
    else
        echo "💡 Using git commit with template..."
        git commit
    fi

# Show commit examples
commit-examples:
    @echo "📝 Conventional Commit Examples for Zephyrite:"
    @echo ""
    @echo "✨ Features:"
    @echo "   feat(server): add HTTP GET endpoint"
    @echo "   feat(storage): implement in-memory storage"
    @echo "   feat(api): add key validation"
    @echo ""
    @echo "🐛 Bug Fixes:"
    @echo "   fix(server): handle empty key validation"
    @echo "   fix(storage): prevent null key insertion"
    @echo ""
    @echo "📚 Documentation:"
    @echo "   docs(readme): update installation instructions"
    @echo "   docs(api): add endpoint documentation"
    @echo ""
    @echo "🧪 Tests:"
    @echo "   test(storage): add memory storage tests"
    @echo "   test(server): add HTTP endpoint tests"
    @echo ""
    @echo "🔧 Chores:"
    @echo "   chore(deps): update tokio to 1.41"
    @echo "   chore: add .gitignore"

# Install nextest (one-time setup)
install-nextest:
    cargo install cargo-nextest --locked
    @echo "✅ cargo-nextest installed"
    @echo "💡 Run 'just test' to use nextest"

# Install additional testing tools
install-test-tools:
    cargo install cargo-nextest --locked
    cargo install cargo-llvm-cov --locked
    cargo install cargo-watch --locked
    @echo "✅ Testing tools installed:"
    @echo "   - cargo-nextest (fast test runner)"
    @echo "   - cargo-llvm-cov (code coverage)"
    @echo "   - cargo-watch (file watching)"

# Show testing help
test-help:
    @echo "🧪 Testing Commands Available:"
    @echo ""
    @echo "Basic Testing:"
    @echo "   just test           - Run tests with nextest (fast)"
    @echo "   just test-all       - Run all tests including doctests"
    @echo "   just test-ci        - Run with CI profile (retries + JUnit)"
    @echo ""
    @echo "Specific Test Types:"
    @echo "   just test-unit      - Unit tests only"
    @echo "   just test-integration - Integration tests only"
    @echo "   just test-storage   - Storage module tests"
    @echo ""
    @echo "Advanced:"
    @echo "   just test-coverage  - Run with coverage report"
    @echo "   just test-watch     - Watch mode (auto-rerun)"
    @echo "   just test-results   - Show test configuration"
    @echo ""
    @echo "Setup:"
    @echo "   just install-nextest     - Install nextest only"
    @echo "   just install-test-tools  - Install all testing tools"
