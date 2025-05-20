# Connectify Testing Guide

This document provides an overview of the testing strategy for the Connectify project, including how to run tests, how to add new tests, and best practices for testing.

## Table of Contents

- [Testing Strategy](#testing-strategy)
- [Types of Tests](#types-of-tests)
- [Running Tests](#running-tests)
- [Adding New Tests](#adding-new-tests)
- [Test Fixtures](#test-fixtures)
- [Continuous Integration](#continuous-integration)
- [Code Coverage](#code-coverage)
- [Performance Benchmarks](#performance-benchmarks)

## Testing Strategy

The Connectify project follows a comprehensive testing strategy that includes:

- **Unit Tests**: Test individual components in isolation
- **Integration Tests**: Test interactions between components
- **End-to-End Tests**: Test complete user flows
- **Property-Based Tests**: Test properties of functions using randomly generated inputs
- **Contract Tests**: Test interactions with external services
- **Performance Benchmarks**: Test performance of critical paths

The goal is to maintain at least 80% code coverage across the codebase, with critical paths having close to 100% coverage.

## Types of Tests

### Unit Tests

Unit tests are located in the same file as the code they test, in a `#[cfg(test)]` module. For example:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_some_function() {
        // Test code here
    }
}
```

### Integration Tests

Integration tests are located in the `tests` directory of each crate. They test the public API of the crate.

### End-to-End Tests

End-to-End tests are located in the `tests` directory of each crate, with filenames like `e2e_tests.rs`. They test complete user flows.

### Property-Based Tests

Property-based tests use the `proptest` crate to generate random inputs and test properties of functions. They are located in files with names like `logic_proptest.rs`.

### Contract Tests

Contract tests verify that our code interacts correctly with external services. They are located in the `tests` directory with filenames like `contract_tests.rs`.

### Performance Benchmarks

Performance benchmarks use the `criterion` crate to measure the performance of critical paths. They are located in the `benches` directory of each crate.

## Running Tests

To run all tests:

```bash
cargo test
```

To run tests for a specific crate:

```bash
cargo test -p connectify-gcal
```

To run a specific test:

```bash
cargo test test_name
```

To run benchmarks:

```bash
cargo bench
```

## Adding New Tests

When adding new functionality, follow these steps:

1. Write unit tests for the new code
2. Update or add integration tests if the public API changes
3. Update or add end-to-end tests if user flows change
4. Consider adding property-based tests for complex logic
5. Update contract tests if interactions with external services change
6. Add performance benchmarks for performance-critical code

## Test Fixtures

Common test fixtures and factory functions are available in the `tests/fixtures.rs` file. Use these to create test data consistently across tests.

Example:

```rust
use connectify_gcal::tests::fixtures;

#[test]
fn test_something() {
    let config = fixtures::create_mock_config();
    let event = fixtures::create_test_calendar_event(1, 60, "Test Event", Some("Description"));
    // Test code here
}
```

## Continuous Integration

The project uses GitHub Actions for continuous integration. The workflow is defined in `.github/workflows/rust-tests.yml` and includes:

- Running all tests
- Checking code formatting with `rustfmt`
- Running the Clippy linter
- Generating code coverage reports
- Running benchmarks

The CI pipeline runs on every push to the main, master, and develop branches, as well as on pull requests to these branches.

## Code Coverage

Code coverage is tracked using `cargo-tarpaulin` and reported to codecov.io. To generate a coverage report locally:

```bash
cargo install cargo-tarpaulin
cargo tarpaulin --out Html
```

The coverage report will be available in `tarpaulin-report.html`.

## Performance Benchmarks

Performance benchmarks use the `criterion` crate. To run benchmarks:

```bash
cargo bench
```

Benchmark results are stored in the `target/criterion` directory and can be viewed in a web browser.

When making changes that might affect performance, run benchmarks before and after the changes to ensure performance doesn't regress.