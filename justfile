# Default recipe - shows available commands
default:
    @just --list

# Run the server
run:
    cargo run

# Run tests
test:
    cargo test --all

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
