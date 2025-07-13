# Default recipe - shows available commands
default:
    @just --list

# Run the server
run:
    cargo run

# Run tests with nextest
test:
    cargo nextest run --config-file .cargo/nextest.toml --profile local-dev

# Run all tests (including doctests which nextest doesn't handle)
test-all:
    cargo nextest run --config-file .cargo/nextest.toml --profile local-dev
    cargo test --doc

# Run tests with coverage (requires cargo-llvm-cov)
test-coverage:
    cargo llvm-cov nextest --config-file .cargo/nextest.toml --profile coverage --html --open

# Generate coverage in multiple formats for visualization
test-coverage-all:
    #!/usr/bin/env bash
    echo "🧪 Generating comprehensive test coverage reports..."
    echo ""

    # Clean previous coverage data
    cargo llvm-cov clean

    # Generate HTML report (main visual report)
    echo "📊 Generating HTML coverage report..."
    cargo llvm-cov nextest --config-file .cargo/nextest.toml --profile coverage --html

    # Generate JSON report (for tools integration)
    echo "📋 Generating JSON coverage report..."
    cargo llvm-cov nextest --config-file .cargo/nextest.toml --profile coverage --json --output-path target/llvm-cov/coverage.json

    # Generate LCOV report (for CI/CD integration)
    echo "📄 Generating LCOV coverage report..."
    cargo llvm-cov nextest --config-file .cargo/nextest.toml --profile coverage --lcov --output-path target/llvm-cov/coverage.lcov

    # Generate summary report
    echo "📈 Generating coverage summary..."
    cargo llvm-cov nextest --config-file .cargo/nextest.toml --profile coverage --summary-only > target/llvm-cov/summary.txt

    echo ""
    echo "✅ Coverage reports generated:"
    echo "   📊 HTML Report: target/llvm-cov/html/index.html"
    echo "   📋 JSON Report: target/llvm-cov/coverage.json"
    echo "   📄 LCOV Report: target/llvm-cov/coverage.lcov"
    echo "   📈 Summary: target/llvm-cov/summary.txt"
    echo ""
    echo "💡 Use 'just coverage-open' to view in browser"

# Open coverage report in browser
coverage-open:
    #!/usr/bin/env bash
    html_file="target/llvm-cov/html/index.html"
    if [ ! -f "$html_file" ]; then
        echo "❌ Coverage HTML report not found: $html_file"
        echo "💡 Run 'just test-coverage' or 'just test-coverage-all' first"
        exit 1
    fi

    echo "🌐 Opening coverage report in browser..."
    echo "📁 File: $html_file"

    # Try to open in Safari first (best for HTML reports)
    if [ -d "/Applications/Safari.app" ]; then
        open -a "Safari" "$html_file"
    elif [ -d "/Applications/Google Chrome.app" ]; then
        open -a "Google Chrome" "$html_file"
    else
        open "$html_file"
    fi

# Show coverage summary in terminal
coverage-summary:
    #!/usr/bin/env bash
    if [ -f "target/llvm-cov/summary.txt" ]; then
        echo "📊 Test Coverage Summary:"
        echo ""
        cat target/llvm-cov/summary.txt
    else
        echo "❌ Coverage summary not found"
        echo "💡 Run 'just test-coverage-all' first to generate summary"
        echo ""
        echo "🔄 Generating quick summary..."
        cargo llvm-cov nextest --config-file .cargo/nextest.toml --profile coverage --summary-only
    fi

# Watch mode for coverage (auto-regenerate on file changes)
coverage-watch:
    #!/usr/bin/env bash
    if ! command -v cargo-watch >/dev/null 2>&1; then
        echo "❌ cargo-watch not found"
        echo "💡 Install with: cargo install cargo-watch"
        echo "   Or run: just install-test-tools"
        exit 1
    fi

    echo "👀 Starting coverage watch mode..."
    echo "💡 Coverage will regenerate automatically when files change"
    echo "🌐 Open target/llvm-cov/html/index.html in your browser and refresh to see updates"
    echo ""
    cargo watch -x "llvm-cov nextest --config-file .cargo/nextest.toml --profile coverage --html"

# Generate coverage badge (requires installation of additional tools)
coverage-badge:
    #!/usr/bin/env bash
    if [ ! -f "target/llvm-cov/coverage.json" ]; then
        echo "❌ Coverage JSON not found"
        echo "💡 Run 'just test-coverage-all' first"
        exit 1
    fi

    echo "🏷️ Generating coverage badge..."

    # Extract coverage percentage from JSON (requires jq)
    if command -v jq >/dev/null 2>&1; then
        coverage_percent=$(jq -r '.data[0].totals.lines.percent' target/llvm-cov/coverage.json)
        echo "📊 Line Coverage: ${coverage_percent}%"

        # Generate simple badge URL
        badge_color="red"
        if (( $(echo "$coverage_percent >= 80" | bc -l) )); then
            badge_color="brightgreen"
        elif (( $(echo "$coverage_percent >= 60" | bc -l) )); then
            badge_color="yellow"
        elif (( $(echo "$coverage_percent >= 40" | bc -l) )); then
            badge_color="orange"
        fi

        badge_url="https://img.shields.io/badge/coverage-${coverage_percent}%25-${badge_color}"
        echo "🏷️ Badge URL: $badge_url"
        echo "📝 Markdown: [![Coverage](${badge_url})](./target/llvm-cov/html/index.html)"

        # Save to file
        echo "[![Coverage](${badge_url})](./target/llvm-cov/html/index.html)" > target/llvm-cov/coverage-badge.md
        echo "💾 Badge saved to: target/llvm-cov/coverage-badge.md"
    else
        echo "❌ jq not found (needed for JSON parsing)"
        echo "💡 Install with: brew install jq"
        echo "🔄 Showing raw coverage data..."
        head -20 target/llvm-cov/coverage.json
    fi

# Terminal-based visual coverage display
coverage-visual:
    #!/usr/bin/env bash
    if [ ! -f "target/llvm-cov/coverage.json" ]; then
        echo "❌ Coverage JSON not found"
        echo "💡 Run 'just test-coverage-all' first"
        exit 1
    fi

    echo "📊 Visual Coverage Overview:"
    echo ""

    # Extract overall coverage
    if command -v jq >/dev/null 2>&1; then
        line_coverage=$(jq -r '.data[0].totals.lines.percent' target/llvm-cov/coverage.json)
        function_coverage=$(jq -r '.data[0].totals.functions.percent' target/llvm-cov/coverage.json)

        echo "📈 Overall Coverage:"
        echo "   Lines:     $line_coverage%"
        echo "   Functions: $function_coverage%"
        echo ""

        # Visual progress bars
        line_bars=$(echo "scale=0; $line_coverage / 2" | bc)
        function_bars=$(echo "scale=0; $function_coverage / 2" | bc)

        echo "📊 Visual Progress:"
        printf "   Lines:     ["
        for i in $(seq 1 50); do
            if [ $i -le $line_bars ]; then
                printf "█"
            else
                printf "░"
            fi
        done
        printf "] %.1f%%\n" $line_coverage

        printf "   Functions: ["
        for i in $(seq 1 50); do
            if [ $i -le $function_bars ]; then
                printf "█"
            else
                printf "░"
            fi
        done
        printf "] %.1f%%\n" $function_coverage

        echo ""

        # Coverage by file (top 10 highest coverage)
        echo "🏆 Top Coverage Files:"
        jq -r '.data[0].files[] | "\(.filename) \(.summary.lines.percent)%"' target/llvm-cov/coverage.json | \
        sort -k2 -nr | head -5 | \
        while read filename percent; do
            printf "   ✅ %-30s %s\n" "$filename" "$percent"
        done

        echo ""

        # Files needing attention (lowest coverage)
        echo "🎯 Files Needing Attention:"
        jq -r '.data[0].files[] | "\(.filename) \(.summary.lines.percent)%"' target/llvm-cov/coverage.json | \
        sort -k2 -n | head -5 | \
        while read filename percent; do
            coverage_num=$(echo $percent | sed 's/%//')
            if (( $(echo "$coverage_num < 80" | bc -l) )); then
                printf "   🔴 %-30s %s\n" "$filename" "$percent"
            elif (( $(echo "$coverage_num < 90" | bc -l) )); then
                printf "   🟡 %-30s %s\n" "$filename" "$percent"
            else
                printf "   🟢 %-30s %s\n" "$filename" "$percent"
            fi
        done
    else
        echo "❌ jq not found - install with 'just install-coverage-tools'"
    fi

# Generate coverage report for external services
coverage-external:
    #!/usr/bin/env bash
    echo "🌐 Coverage Integration Options:"
    echo ""

    if [ ! -f "target/llvm-cov/coverage.lcov" ]; then
        echo "❌ LCOV file not found"
        echo "💡 Run 'just test-coverage-all' first"
        exit 1
    fi

    echo "📄 LCOV file ready for:"
    echo "   🔹 Codecov: https://codecov.io"
    echo "   🔹 Coveralls: https://coveralls.io"
    echo "   🔹 CodeClimate: https://codeclimate.com"
    echo ""

    echo "🚀 Upload commands:"
    echo "   # Codecov"
    echo "   curl -s https://codecov.io/bash | bash -s -- -f target/llvm-cov/coverage.lcov"
    echo ""
    echo "   # Coveralls (requires token)"
    echo "   coveralls --lcov-file target/llvm-cov/coverage.lcov"
    echo ""

    echo "📊 File location: target/llvm-cov/coverage.lcov"
    echo "📐 File size: $(du -h target/llvm-cov/coverage.lcov | cut -f1)"

# Comprehensive coverage dashboard
coverage-dashboard:
    #!/usr/bin/env bash
    echo "🎯 Zephyrite Coverage Dashboard"
    echo "================================"
    echo ""

    # Check if coverage exists
    if [ ! -f "target/llvm-cov/coverage.json" ]; then
        echo "❌ No coverage data found"
        echo "💡 Run 'just test-coverage-all' to generate coverage reports"
        echo ""
        echo "🚀 Quick start:"
        echo "   just test-coverage-all  # Generate all formats"
        echo "   just coverage-open      # Open visual HTML report"
        exit 1
    fi

    # Show visual overview
    just coverage-visual
    echo ""

    # Show available reports
    echo "📊 Available Reports:"
    echo ""
    if [ -f "target/llvm-cov/html/index.html" ]; then
        echo "   🌐 HTML Report:     target/llvm-cov/html/index.html"
        echo "      👀 Open with:   just coverage-open"
    fi

    if [ -f "target/llvm-cov/coverage.json" ]; then
        echo "   📋 JSON Report:    target/llvm-cov/coverage.json"
    fi

    if [ -f "target/llvm-cov/coverage.lcov" ]; then
        echo "   📄 LCOV Report:    target/llvm-cov/coverage.lcov"
    fi

    if [ -f "target/llvm-cov/coverage-badge.md" ]; then
        echo "   🏷️  Coverage Badge: target/llvm-cov/coverage-badge.md"
        echo "      📖 Content:     $(cat target/llvm-cov/coverage-badge.md)"
    fi

    echo ""
    echo "🎮 Quick Actions:"
    echo "   📊 just coverage-open      # Open interactive HTML report"
    echo "   🎯 just coverage-visual    # Terminal visual overview"
    echo "   🏷️  just coverage-badge    # Generate/update badge"
    echo "   👀 just coverage-watch     # Watch mode for development"
    echo "   🌐 just coverage-external  # External service integration"

# Show all coverage files and their purposes
coverage-files:
    #!/usr/bin/env bash
    echo "📁 Coverage Files Overview:"
    echo ""

    coverage_dir="target/llvm-cov"
    if [ ! -d "$coverage_dir" ]; then
        echo "❌ No coverage directory found: $coverage_dir"
        echo "💡 Run 'just test-coverage-all' first"
        exit 1
    fi

    echo "📊 Available Coverage Reports:"
    echo ""

    # Check HTML report
    if [ -f "$coverage_dir/html/index.html" ]; then
        echo "✅ HTML Report (Interactive): $coverage_dir/html/index.html"
        echo "   👀 View with: just coverage-open"
    else
        echo "❌ HTML Report: Not found"
    fi

    # Check JSON report
    if [ -f "$coverage_dir/coverage.json" ]; then
        echo "✅ JSON Report (Machine readable): $coverage_dir/coverage.json"
        echo "   🔧 Use for: CI/CD integration, badges, custom analysis"
    else
        echo "❌ JSON Report: Not found"
    fi

    # Check LCOV report
    if [ -f "$coverage_dir/coverage.lcov" ]; then
        echo "✅ LCOV Report (Industry standard): $coverage_dir/coverage.lcov"
        echo "   🔧 Use for: SonarQube, Codecov, Coveralls integration"
    else
        echo "❌ LCOV Report: Not found"
    fi

    # Check summary
    if [ -f "$coverage_dir/summary.txt" ]; then
        echo "✅ Summary Report (Terminal friendly): $coverage_dir/summary.txt"
        echo "   👀 View with: just coverage-summary"
    else
        echo "❌ Summary Report: Not found"
    fi

    # Check badge
    if [ -f "$coverage_dir/coverage-badge.md" ]; then
        echo "✅ Coverage Badge: $coverage_dir/coverage-badge.md"
    else
        echo "❌ Coverage Badge: Not found (run 'just coverage-badge')"
    fi

    echo ""
    echo "💡 Generate all formats with: just test-coverage-all"

# Run integration tests only
test-integration:
    cargo nextest run --config-file .cargo/nextest.toml --profile integration-only

# Run unit tests only (excluding integration tests)
test-unit:
    cargo nextest run --config-file .cargo/nextest.toml --profile local-dev -E 'not test(health_check)'

# Run storage tests specifically
test-storage:
    cargo nextest run --config-file .cargo/nextest.toml --profile local-dev -E 'test(storage)'

# Run tests with fast profile (quick feedback)
test-fast:
    cargo nextest run --config-file .cargo/nextest.toml --profile fast

# Run tests in CI mode (with retries and JUnit output)
test-ci:
    cargo nextest run --config-file .cargo/nextest.toml --profile ci

# Watch tests (requires cargo-watch)
test-watch:
    cargo watch -x "nextest run --config-file .cargo/nextest.toml --profile local-dev"

# Show test results from last run
test-results:
    cargo nextest show-config test-groups
    @echo "📊 Test artifacts in target/nextest/"

# View JUnit XML reports
test-junit:
    #!/usr/bin/env bash
    echo "📋 Available JUnit XML Reports:"
    echo ""
    for profile in default local-dev ci coverage; do
        junit_file="target/nextest/$profile/target/nextest"
        case $profile in
            "default") junit_file="$junit_file/junit.xml" ;;
            "local-dev") junit_file="$junit_file/local-junit.xml" ;;
            "ci") junit_file="$junit_file/ci-junit.xml" ;;
            "coverage") junit_file="$junit_file/coverage-junit.xml" ;;
        esac
        if [ -f "$junit_file" ]; then
            echo "✅ $profile: $junit_file"
        else
            echo "❌ $profile: not found"
        fi
    done
    echo ""
    echo "💡 To view local-dev JUnit XML:"
    echo "   just test-junit-view local-dev"

# View specific JUnit XML report
test-junit-view profile="local-dev":
    #!/usr/bin/env bash
    case "{{profile}}" in
        "default") file="target/nextest/default/target/nextest/junit.xml" ;;
        "local-dev") file="target/nextest/local-dev/target/nextest/local-junit.xml" ;;
        "ci") file="target/nextest/ci/target/nextest/ci-junit.xml" ;;
        "coverage") file="target/nextest/coverage/target/nextest/coverage-junit.xml" ;;
        *) echo "❌ Unknown profile: {{profile}}"; exit 1 ;;
    esac

    if [ ! -f "$file" ]; then
        echo "❌ JUnit XML not found: $file"
        echo "💡 Run tests with the {{profile}} profile first"
        exit 1
    fi

    echo "📄 Viewing JUnit XML for {{profile}} profile:"
    echo "📁 File: $file"
    echo ""

    # Show summary
    if command -v xmllint >/dev/null 2>&1; then
        echo "📊 Test Summary:"
        xmllint --xpath "//testsuite/@*" "$file" 2>/dev/null || echo "Basic XML parsing..."
        echo ""
        echo "📋 Formatted XML:"
        xmllint --format "$file"
    else
        echo "📊 Raw XML Content:"
        cat "$file"
    fi

# Open JUnit XML report in web browser
test-junit-browser profile="local-dev" browser="":
    #!/usr/bin/env bash
    case "{{profile}}" in
        "default") file="target/nextest/default/target/nextest/junit.xml" ;;
        "local-dev") file="target/nextest/local-dev/target/nextest/local-junit.xml" ;;
        "ci") file="target/nextest/ci/target/nextest/ci-junit.xml" ;;
        "coverage") file="target/nextest/coverage/target/nextest/coverage-junit.xml" ;;
        *) echo "❌ Unknown profile: {{profile}}"; exit 1 ;;
    esac

    if [ ! -f "$file" ]; then
        echo "❌ JUnit XML not found: $file"
        echo "💡 Run tests with the {{profile}} profile first"
        exit 1
    fi

    echo "🌐 Opening JUnit XML in browser for {{profile}} profile..."
    echo "📁 File: $file"

    # Choose browser
    if [ "{{browser}}" != "" ]; then
        open -a "{{browser}}" "$file"
    elif command -v safari >/dev/null 2>&1 || [ -d "/Applications/Safari.app" ]; then
        open -a "Safari" "$file"
    elif command -v chrome >/dev/null 2>&1 || [ -d "/Applications/Google Chrome.app" ]; then
        open -a "Google Chrome" "$file"
    elif command -v firefox >/dev/null 2>&1 || [ -d "/Applications/Firefox.app" ]; then
        open -a "Firefox" "$file"
    else
        echo "💡 Opening with default application (might be Xcode)..."
        open "$file"
    fi

# Show available nextest profiles and configuration
test-profiles:
    @echo "📋 Available Nextest Profiles:"
    @echo ""
    @echo "  default        - Standard profile for regular development"
    @echo "  fast           - Quick profile with fail-fast behavior"
    @echo "  integration-only - Run integration tests only (single-threaded)"
    @echo "  local-dev      - Optimized for local development (verbose output)"
    @echo "  ci             - CI profile with retries and generous timeouts"
    @echo "  coverage       - Optimized for code coverage collection"
    @echo ""
    @echo "💡 Use 'just test-fast' for quick iteration or 'just test-ci' for thorough testing"

# Quick shortcuts for opening JUnit XML in specific browsers
test-safari profile="local-dev":
    just test-junit-browser {{profile}} "Safari"

test-chrome profile="local-dev":
    just test-junit-browser {{profile}} "Google Chrome"

test-firefox profile="local-dev":
    just test-junit-browser {{profile}} "Firefox"

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
    @echo ""
    @echo "📦 Additional recommended tools for coverage visualization:"
    @echo "   jq (JSON processing): brew install jq"
    @echo "   bc (calculations): brew install bc"
    @echo ""
    @echo "💡 Install these with: just install-coverage-tools"

# Dagger.io integration commands (Phase 1)

# Setup Dagger dependencies
dagger-setup:
    #!/usr/bin/env bash
    echo "🔧 Setting up Dagger.io dependencies..."
    echo ""

    # Check if Docker is running
    if ! docker info >/dev/null 2>&1; then
        echo "❌ Docker is not running or not installed"
        echo "💡 Please install Docker Desktop and ensure it's running:"
        echo "   - macOS: https://docs.docker.com/desktop/mac/install/"
        echo "   - Linux: https://docs.docker.com/engine/install/"
        echo "   - Windows: https://docs.docker.com/desktop/windows/install/"
        exit 1
    fi

    # Check if Go is installed
    if ! command -v go >/dev/null 2>&1; then
        echo "❌ Go is not installed"
        echo "💡 Please install Go 1.21+:"
        echo "   - macOS: brew install go"
        echo "   - Linux: https://golang.org/doc/install"
        echo "   - Windows: https://golang.org/doc/install"
        exit 1
    fi

    # Check Go version
    go_version=$(go version | awk '{print $3}' | sed 's/go//')
    required_version="1.21"
    if [ "$(printf '%s\n' "$required_version" "$go_version" | sort -V | head -n1)" != "$required_version" ]; then
        echo "❌ Go version $go_version is too old (requires $required_version+)"
        echo "💡 Please update Go to version $required_version or higher"
        exit 1
    fi

    # Initialize Go module if needed
    if [ ! -f "dagger/go.mod" ]; then
        echo "📦 Initializing Dagger Go module..."
        cd dagger && go mod init zephyrite/dagger
    fi

    # Install dependencies
    echo "📥 Installing Dagger dependencies..."
    cd dagger && go mod tidy

    echo ""
    echo "✅ Dagger setup complete!"
    echo "💡 You can now use 'just dagger-test-local' and 'just dagger-release'"

# Run CI tests locally with Dagger (mirrors .github/workflows/rust.yml)
dagger-test-local:
    #!/usr/bin/env bash
    echo "🧪 Running Zephyrite CI tests locally with Dagger..."
    echo "💡 This mirrors the exact CI pipeline from GitHub Actions"
    echo ""

    # Check if Docker is running
    if ! docker info >/dev/null 2>&1; then
        echo "❌ Docker is not running"
        echo "💡 Please start Docker Desktop and run 'just dagger-setup'"
        exit 1
    fi

    # Check if dagger directory exists
    if [ ! -d "dagger" ]; then
        echo "❌ Dagger directory not found"
        echo "💡 Please run 'just dagger-setup' first"
        exit 1
    fi

    # Run the Dagger pipeline
    cd dagger && go run main.go test-local

# Build release artifacts with Dagger
dagger-release:
    #!/usr/bin/env bash
    echo "🚀 Building Zephyrite release artifacts with Dagger..."
    echo ""

    # Check if Docker is running
    if ! docker info >/dev/null 2>&1; then
        echo "❌ Docker is not running"
        echo "💡 Please start Docker Desktop and run 'just dagger-setup'"
        exit 1
    fi

    # Check if dagger directory exists
    if [ ! -d "dagger" ]; then
        echo "❌ Dagger directory not found"
        echo "💡 Please run 'just dagger-setup' first"
        exit 1
    fi

    # Run the Dagger pipeline
    cd dagger && go run main.go release

    echo ""
    echo "📦 Release artifacts:"
    echo "   Binary: ./target/release/zephyrite"
    echo "💡 You can now distribute or test the release binary"

# Install coverage visualization tools (macOS)
install-coverage-tools:
    #!/usr/bin/env bash
    echo "📦 Installing coverage visualization tools..."
    echo ""

    # Check if Homebrew is available
    if ! command -v brew >/dev/null 2>&1; then
        echo "❌ Homebrew not found"
        echo "💡 Install Homebrew first: https://brew.sh"
        echo "   Or install manually:"
        echo "   - jq: https://github.com/jqlang/jq"
        echo "   - bc: usually pre-installed on macOS"
        exit 1
    fi

    # Install jq for JSON processing
    if ! command -v jq >/dev/null 2>&1; then
        echo "📥 Installing jq..."
        brew install jq
    else
        echo "✅ jq already installed"
    fi

    # Install bc for calculations (usually pre-installed)
    if ! command -v bc >/dev/null 2>&1; then
        echo "📥 Installing bc..."
        brew install bc
    else
        echo "✅ bc already installed"
    fi

    echo ""
    echo "✅ Coverage tools ready!"
    echo "💡 Now you can use: just coverage-badge"

# Show testing help
test-help:
    @echo "🧪 Testing Commands Available:"
    @echo ""
    @echo "Basic Testing:"
    @echo "   just test           - Run tests with nextest (local-dev profile)"
    @echo "   just test-all       - Run all tests including doctests"
    @echo "   just test-fast      - Quick test run with fast profile"
    @echo "   just test-ci        - Run with CI profile (retries + JUnit)"
    @echo ""
    @echo "Specific Test Types:"
    @echo "   just test-unit      - Unit tests only (excludes integration)"
    @echo "   just test-integration - Integration tests only"
    @echo "   just test-storage   - Storage module tests"
    @echo ""
    @echo "Advanced:"
    @echo "   just test-watch     - Watch mode (auto-rerun on changes)"
    @echo "   just test-results   - Show test configuration"
    @echo "   just test-profiles  - Show available nextest profiles"
    @echo "   just test-junit     - List available JUnit XML reports"
    @echo "   just test-junit-view [profile] - View JUnit XML (default: local-dev)"
    @echo "   just test-junit-browser [profile] [browser] - Open JUnit XML in browser"
    @echo "   just test-safari/chrome/firefox [profile] - Quick browser shortcuts"
    @echo ""
    @echo "Coverage & Visualization:"
    @echo "   just test-coverage  - Generate HTML coverage report and open"
    @echo "   just test-coverage-all - Generate all coverage formats (HTML/JSON/LCOV)"
    @echo "   just coverage-dashboard - Complete coverage overview with visuals"
    @echo "   just coverage-open  - Open coverage HTML report in browser"
    @echo "   just coverage-summary - Show coverage summary in terminal"
    @echo "   just coverage-visual - Terminal visual coverage with progress bars"
    @echo "   just coverage-watch - Watch mode for coverage (auto-regenerate)"
    @echo "   just coverage-badge - Generate coverage badge for README"
    @echo "   just coverage-files - List all coverage reports and their uses"
    @echo "   just coverage-external - Setup for external coverage services"
    @echo ""
    @echo "Setup:"
    @echo "   just install-nextest        - Install nextest only"
    @echo "   just install-test-tools     - Install all testing tools"
    @echo "   just install-coverage-tools - Install coverage visualization tools"
    @echo ""
    @echo "Dagger Integration (Phase 1):"
    @echo "   just dagger-test-local - Run CI tests locally with Dagger"
    @echo "   just dagger-release    - Build release artifacts with Dagger"
    @echo "   just dagger-setup      - Setup Dagger dependencies"
