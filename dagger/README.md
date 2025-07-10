# Dagger.io Integration for Zephyrite

This directory contains Dagger.io integration for Zephyrite.

- [Dagger.io Integration for Zephyrite](#daggerio-integration-for-zephyrite)
  - [What is Dagger.io?](#what-is-daggerio)
  - [Benefits for Zephyrite](#benefits-for-zephyrite)
  - [Implementation](#implementation)
    - [Commands Available](#commands-available)
    - [Prerequisites](#prerequisites)
  - [Quick Start](#quick-start)
  - [Implementation Details](#implementation-details)
    - [Files Structure](#files-structure)
    - [Pipeline Functions](#pipeline-functions)
      - [`testLocal()`](#testlocal)
      - [`release()`](#release)
  - [Relationship to Existing Workflow](#relationship-to-existing-workflow)
    - [Existing Commands (Keep Using)](#existing-commands-keep-using)
    - [New Dagger Commands (Use When)](#new-dagger-commands-use-when)
  - [Docker Requirements](#docker-requirements)
  - [Future Phases](#future-phases)
  - [Troubleshooting](#troubleshooting)
    - [Common Issues](#common-issues)
    - [Performance Tips](#performance-tips)
  - [Contributing](#contributing)

## What is Dagger.io?

Dagger.io is a portable devkit for CI/CD pipelines.
It allows you to run your CI/CD pipelines locally using the same containers and logic as your remote CI systems.

## Benefits for Zephyrite

- **Local-CI Parity**: Run identical pipelines locally vs GitHub Actions
- **Consistent Environment**: Same Rust version, dependencies, and tools everywhere
- **Faster Debugging**: Debug CI issues locally without pushing commits
- **Reproducible Builds**: Consistent release artifacts across environments

## Implementation

### Commands Available

1. **`just dagger-setup`** - One-time setup of Dagger dependencies
2. **`just dagger-test-local`** - Run CI tests locally (mirrors GitHub Actions)
3. **`just dagger-release`** - Build release artifacts with Dagger

### Prerequisites

- **Docker Desktop** - Must be running
- **Go 1.21+** - For running Dagger pipelines
- **Rust 1.85+** - Already required for Zephyrite

## Quick Start

```bash
# 1. Setup Dagger (one-time)
just dagger-setup

# 2. Run CI tests locally
just dagger-test-local

# 3. Build release artifacts
just dagger-release
```

## Implementation Details

### Files Structure

```txt
dagger/
├── main.go          # Dagger pipeline definitions
├── go.mod           # Go module dependencies
├── go.sum           # Go module checksums
└── README.md        # This file
```

### Pipeline Functions

#### `testLocal()`

Mirrors the exact steps from `.github/workflows/rust.yml`:

- Rust 1.85 container
- Install cargo-nextest
- Format check (`cargo fmt --check`)
- Clippy linting (`cargo clippy`)
- Build (`cargo build --verbose`)
- Tests (`cargo nextest run --profile ci`)
- Doctests (`cargo test --doc`)

#### `release()`

Builds release artifacts:

- Rust 1.85 container
- Release build (`cargo build --release`)
- Extract binary to `./target/release/zephyrite`

## Relationship to Existing Workflow

### Existing Commands (Keep Using)

- `just test` - Fast local development testing
- `just dev` - Development workflow (fmt + lint + test)
- `just build` - Regular development builds

### New Dagger Commands (Use When)

- `just dagger-test-local` - Before pushing commits (CI validation)
- `just dagger-release` - Creating release builds
- When debugging CI failures locally

## Docker Requirements

Dagger requires Docker to be running. If you see errors like:

```bash
Cannot connect to the Docker daemon at unix:///var/run/docker.sock
```

1. Install Docker Desktop
2. Start Docker Desktop
3. Run `just dagger-setup` again

## Future Phases

This is Phase 1 of the integration. Future phases may include:

- **Phase 2**: Migrate GitHub Actions to call Dagger functions
- **Phase 3**: Multi-platform builds (ARM64, x86_64)
- **Phase 4**: Advanced caching and optimization

## Troubleshooting

### Common Issues

1. **Docker not running**

   ```bash
   # Check Docker status
   docker info

   # Start Docker Desktop and retry
   just dagger-setup
   ```

2. **Go version too old**

   ```bash
   # Check Go version
   go version

   # Update Go (macOS)
   brew install go
   ```

3. **Module not found errors**

   ```bash
   # Clean and reinstall
   cd dagger
   rm -rf go.mod go.sum
   go mod init zephyrite/dagger
   go mod tidy
   ```

### Performance Tips

- First run downloads containers and may be slow
- Subsequent runs use cached layers and are much faster
- Use `just test` for rapid iteration, `just dagger-test-local` for CI validation

## Contributing

When modifying Dagger pipelines:

1. Test changes locally first
2. Ensure `testLocal()` mirrors `.github/workflows/rust.yml` exactly
3. Update this README if adding new commands
4. Consider impact on existing Just commands
