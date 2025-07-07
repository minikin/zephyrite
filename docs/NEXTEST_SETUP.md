# Nextest Setup for Zephyrite

This document describes the cargo-nextest setup for the Zephyrite project.

- [Nextest Setup for Zephyrite](#nextest-setup-for-zephyrite)
  - [Overview](#overview)
  - [Configuration](#configuration)
    - [Profiles](#profiles)
      - [`default`](#default)
      - [`fast`](#fast)
      - [`integration-only`](#integration-only)
      - [`local-dev`](#local-dev)
      - [`ci`](#ci)
      - [`coverage`](#coverage)
  - [Usage](#usage)
    - [Running Tests](#running-tests)
    - [VS Code Tasks](#vs-code-tasks)
    - [Test Groups](#test-groups)
    - [Test Overrides](#test-overrides)
  - [CI Integration](#ci-integration)
  - [JUnit Output](#junit-output)
  - [Filtering Tests](#filtering-tests)
  - [Watch Mode](#watch-mode)
  - [Performance Benefits](#performance-benefits)
  - [Migration from cargo test](#migration-from-cargo-test)
  - [Troubleshooting](#troubleshooting)
    - [Config file not found](#config-file-not-found)
    - [Profile not found](#profile-not-found)
    - [Tests hanging](#tests-hanging)
    - [Port conflicts](#port-conflicts)

## Overview

[cargo-nextest](https://nexte.st/) is a next-generation test runner for Rust that provides:

- Better parallelization and performance
- Improved output formatting
- Enhanced CI/CD integration
- Advanced test grouping and filtering capabilities

## Configuration

The nextest configuration is located in `.cargo/nextest.toml` and includes several profiles:

### Profiles

#### `default`

- Standard profile for regular development
- Moderate timeouts and retry settings
- Outputs JUnit XML for CI integration

#### `fast`

- Quick profile for development iteration
- Fail-fast behavior for quick feedback
- Shorter timeouts (5s slow timeout)

#### `integration-only`

- Profile designed for running integration tests only
- Single-threaded execution for integration tests
- Longer timeouts (60s slow timeout)

#### `local-dev`

- Profile optimized for local development
- Shows all output including successful tests
- Shorter timeouts for faster feedback

#### `ci`

- Profile optimized for CI environments
- Automatic retry on failure (2 retries)
- More generous timeouts
- Stores both success and failure output

#### `coverage`

- Profile optimized for code coverage collection
- Very long timeouts (120s)
- Minimal output for clean coverage reports

## Usage

### Running Tests

```bash
# Run all tests with default profile
cargo nextest run --config-file .cargo/nextest.toml

# Run with specific profile
cargo nextest run --config-file .cargo/nextest.toml --profile fast

# Run only integration tests
cargo nextest run --config-file .cargo/nextest.toml -E 'test(health_check_works)'

# Run all http_server tests
cargo nextest run --config-file .cargo/nextest.toml -E 'test(health_check) or test(put_and_get) or test(delete_)'
```

### VS Code Tasks

The following VS Code tasks are available (Ctrl+Shift+P → "Tasks: Run Task"):

- **cargo nextest run**: Run all tests with default profile
- **cargo nextest run (fast)**: Quick test run for development
- **cargo nextest run (integration-only)**: Run only integration tests
- **cargo nextest run (local-dev)**: Development profile with verbose output
- **cargo nextest watch**: Watch mode for continuous testing

### Test Groups

The configuration defines test groups for better organization:

- **integration**: HTTP server tests that require special handling
  - Limited to 1 thread to avoid port conflicts
  - Longer timeouts for server startup

### Test Overrides

Specific tests have custom configurations:

- `setup_test_server`: 1 retry, 45s timeout
- `persistent_storage`: 20s timeout
- Storage tests: 15s timeout
- WAL tests: 25s timeout
- Compaction tests: 30s timeout

## CI Integration

For CI environments, use the `ci` profile:

```bash
cargo nextest run --config-file .cargo/nextest.toml --profile ci
```

This profile:

- Enables automatic retries for flaky tests
- Uses more generous timeouts
- Stores both success and failure output for debugging
- Generates JUnit XML reports

## JUnit Output

JUnit XML reports are generated for different profiles:

- Default: `target/nextest/junit.xml`
- CI: `target/nextest/ci-junit.xml`
- Local Dev: `target/nextest/local-junit.xml`
- Coverage: `target/nextest/coverage-junit.xml`

## Filtering Tests

You can filter tests using the `-E` (expression) flag:

```bash
# Run only unit tests (exclude integration tests)
cargo nextest run --config-file .cargo/nextest.toml -E 'not test(health_check)'

# Run only storage-related tests
cargo nextest run --config-file .cargo/nextest.toml -E 'test(storage)'

# Run tests matching a pattern
cargo nextest run --config-file .cargo/nextest.toml -E 'test(persistent)'
```

## Watch Mode

For continuous testing during development:

```bash
cargo watch -x "nextest run --config-file .cargo/nextest.toml --profile local-dev"
```

Or use the VS Code task "cargo nextest watch".

## Performance Benefits

Nextest provides several performance improvements over `cargo test`:

1. **Better parallelization**: Tests run in separate processes
2. **Smarter scheduling**: Longer tests start first
3. **Cancellation**: Failed tests don't block others
4. **Efficient output**: Only relevant information is shown

## Migration from cargo test

Most `cargo test` commands can be replaced with `cargo nextest run`:

```bash
# Before
cargo test
cargo test --lib
cargo test test_name

# After
cargo nextest run --config-file .cargo/nextest.toml
cargo nextest run --config-file .cargo/nextest.toml --lib
cargo nextest run --config-file .cargo/nextest.toml -E 'test(test_name)'
```

## Troubleshooting

### Config file not found

Always use `--config-file .cargo/nextest.toml` to ensure the configuration is loaded.

### Profile not found

Check that the profile name matches exactly and the config file is valid TOML.

### Tests hanging

Check the timeout settings in the configuration. Integration tests may need longer timeouts.

### Port conflicts

Integration tests are configured to run single-threaded to avoid port conflicts.
