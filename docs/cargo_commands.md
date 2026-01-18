# Cargo Commands for Rust Development

## Overview

Cargo is Rust's package manager and build system. It handles many tasks such as building code, downloading dependencies, and running tests. This document summarizes key Cargo commands for development, static analysis, and testing.

## Static Analysis Commands

### cargo check

Analyzes code without building executables:
```bash
# Basic syntax check without producing binaries
cargo check

# Check a specific package
cargo check -p package_name
```

- Checks packages and dependencies for errors
- Much faster than building (skips code generation)
- Use case: Quick feedback during development

### cargo clippy

Advanced linter for catching common mistakes:
```bash
# Run with default settings
cargo clippy

# Treat warnings as errors
cargo clippy -- -D warnings

# Allow specific lint categories
cargo clippy -- -A clippy::style
```

- Contains over 750 lints organized in categories:
  - `clippy::correctness`: Outright wrong code (default: deny)
  - `clippy::suspicious`: Likely wrong code (default: warn)
  - `clippy::style`: Non-idiomatic code (default: warn)
  - `clippy::complexity`: Unnecessarily complex code (default: warn)
  - `clippy::perf`: Performance improvements (default: warn)
- Highly configurable severity levels

### cargo fmt

Format code according to Rust style guidelines:
```bash
# Format all code
cargo fmt

# Check if code is properly formatted (CI-friendly)
cargo fmt -- --check
```

## Testing Commands

### cargo test

Run the test suite:
```bash
# Run all tests
cargo test

# Run tests matching a pattern
cargo test test_name

# Run tests in a specific module
cargo test module::path

# Run tests with output displayed
cargo test -- --nocapture

# Run ignored tests
cargo test -- --ignored

# Run tests on multiple threads
cargo test -- --test-threads=3

# Run tests serially
cargo test -- --test-threads=1
```

Rust supports several test types:
- **Unit tests**: Functions annotated with `#[test]` attribute
- **Integration tests**: Tests in the `tests/` directory
- **Documentation tests**: Code examples in documentation comments

### cargo bench

Run benchmarks for performance testing:
```bash
# Run all benchmarks
cargo bench

# Run specific benchmark
cargo bench benchmark_name
```

## Code Improvement Commands

### cargo fix

Automatically fix compiler warnings:
```bash
# Apply suggested fixes
cargo fix

# Only show what would be fixed
cargo fix --dry-run
```

### cargo doc

Generate and view documentation:
```bash
# Generate documentation
cargo doc

# Generate and open in browser
cargo doc --open
```

## Development Workflow Example

A typical development workflow using Cargo tools:

1. Write code
2. `cargo check` (quick syntax/type checking)
3. `cargo clippy` (deeper static analysis)
4. `cargo test` (run test suite)
5. `cargo fmt` (format code)
6. `cargo build` or `cargo run` (build/run the application)

These tools help maintain code quality, catch errors early, and ensure consistency across Rust codebases.