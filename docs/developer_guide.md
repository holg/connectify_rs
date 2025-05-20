# Connectify Developer Guide

This guide is designed to help new developers get started with the Connectify project. It covers setting up your development environment, building the project, running tests, and contributing to the codebase.

## Table of Contents

- [Prerequisites](#prerequisites)
- [Getting Started](#getting-started)
- [Project Structure](#project-structure)
- [Building the Project](#building-the-project)
- [Running Tests](#running-tests)
- [Development Workflow](#development-workflow)
- [Coding Standards](#coding-standards)
- [Debugging](#debugging)
- [Contributing](#contributing)
- [Troubleshooting](#troubleshooting)

## Prerequisites

Before you begin, make sure you have the following installed:

- **Rust**: The project is built with Rust. Install Rust using [rustup](https://rustup.rs/).
- **Git**: For version control.
- **OpenSSL**: Required for TLS support in HTTP clients.
- **pkg-config**: Required for building native dependencies.
- **Docker** (optional): For containerized development and testing.

### Rust Setup

1. Install Rust using rustup:
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. Install the nightly toolchain (required for some features):
   ```bash
   rustup install nightly
   ```

3. Set the default toolchain to nightly for this project:
   ```bash
   rustup override set nightly
   ```

4. Install required components:
   ```bash
   rustup component add rustfmt clippy
   ```

## Getting Started

1. Clone the repository:
   ```bash
   git clone https://github.com/your-organization/connectify_rs.git
   cd connectify_rs
   ```

2. Create a `.env` file in the project root with the required environment variables:
   ```bash
   cp .env.example .env
   ```

3. Edit the `.env` file to set the required environment variables. See the [Environment Variables documentation](ENVIRONMENT_VARIABLES.md) for details.

4. Build the project:
   ```bash
   cargo build
   ```

5. Run the tests:
   ```bash
   cargo test
   ```

## Project Structure

The Connectify project is organized as a Rust workspace with multiple crates:

- `connectify_backend`: The main application that ties everything together.
- `connectify_common`: Common utilities and abstractions used across the application.
- `connectify_config`: Configuration management.
- `connectify_gcal`: Google Calendar integration.
- `connectify_stripe`: Stripe payment integration.
- `connectify_payrexx`: Payrexx payment integration.
- `connectify_twilio`: Twilio notification integration.
- `connectify_fulfillment`: Fulfillment service.

Each crate follows a similar structure:

- `src/lib.rs`: The entry point for the crate, which exports the public API.
- `src/models.rs`: Data structures and models used by the crate.
- `src/logic.rs`: Core business logic.
- `src/handlers.rs`: HTTP request handlers.
- `src/routes.rs`: Route definitions.
- `src/service.rs`: Service implementations.
- `src/error.rs`: Error types and handling.

For more details on the architecture, see the [Architecture documentation](architecture.md).

## Building the Project

### Debug Build

To build the project in debug mode:

```bash
cargo build
```

### Release Build

To build the project in release mode:

```bash
cargo build --release
```

### Building with Features

The project uses feature flags to enable or disable certain functionality. To build with specific features:

```bash
cargo build --features "gcal stripe twilio"
```

Available features:
- `gcal`: Google Calendar integration
- `stripe`: Stripe payment integration
- `payrexx`: Payrexx payment integration
- `twilio`: Twilio notification integration
- `fulfillment`: Fulfillment service
- `openapi`: OpenAPI documentation generation

### Building for Different Environments

The project supports different environments (development, testing, production) through configuration files. To build for a specific environment:

```bash
RUN_ENV=production cargo build
```

## Running Tests

### Running All Tests

To run all tests:

```bash
cargo test
```

### Running Tests for a Specific Crate

To run tests for a specific crate:

```bash
cargo test -p connectify-gcal
```

### Running a Specific Test

To run a specific test:

```bash
cargo test test_name
```

### Running Tests with Features

To run tests with specific features:

```bash
cargo test --features "gcal stripe twilio"
```

### Running Tests with Logging

To run tests with logging enabled:

```bash
RUST_LOG=debug cargo test
```

## Development Workflow

1. **Create a Feature Branch**: Always create a new branch for your changes:
   ```bash
   git checkout -b feature/your-feature-name
   ```

2. **Make Changes**: Implement your changes, following the coding standards.

3. **Write Tests**: Add tests for your changes to ensure they work as expected.

4. **Run Linters**: Run the linters to ensure your code follows the project's style guidelines:
   ```bash
   cargo clippy
   cargo fmt
   ```

5. **Run Tests**: Run the tests to ensure your changes don't break existing functionality:
   ```bash
   cargo test
   ```

6. **Commit Changes**: Commit your changes with a descriptive commit message:
   ```bash
   git add .
   git commit -m "Add feature: your feature description"
   ```

7. **Push Changes**: Push your changes to the remote repository:
   ```bash
   git push origin feature/your-feature-name
   ```

8. **Create a Pull Request**: Create a pull request on GitHub to merge your changes into the main branch.

## Coding Standards

The Connectify project follows the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/) and the following coding standards:

1. **Code Formatting**: Use `rustfmt` to format your code:
   ```bash
   cargo fmt
   ```

2. **Linting**: Use `clippy` to check for common mistakes and improve your code:
   ```bash
   cargo clippy
   ```

3. **Documentation**: Document all public APIs using doc comments. See the [Code Documentation Guidelines](code_documentation.md) for details.

4. **Error Handling**: Use the `thiserror` crate for defining error types and the `anyhow` crate for propagating errors.

5. **Naming Conventions**: Follow Rust's naming conventions:
   - Use `snake_case` for variables, functions, and modules.
   - Use `CamelCase` for types and traits.
   - Use `SCREAMING_SNAKE_CASE` for constants.

6. **Testing**: Write tests for all public APIs. Use the `#[cfg(test)]` attribute to include test-only code.

## Debugging

### Logging

The project uses the `tracing` crate for logging. To enable logging, set the `RUST_LOG` environment variable:

```bash
RUST_LOG=debug cargo run
```

Log levels:
- `error`: Error conditions that should be addressed.
- `warn`: Warning conditions that might require attention.
- `info`: Informational messages about normal operation.
- `debug`: Detailed information for debugging.
- `trace`: Very detailed information for tracing execution.

### Debugging with VS Code

1. Install the [Rust Analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer) extension.
2. Install the [CodeLLDB](https://marketplace.visualstudio.com/items?itemName=vadimcn.vscode-lldb) extension.
3. Create a `.vscode/launch.json` file with the following content:
   ```json
   {
     "version": "0.2.0",
     "configurations": [
       {
         "type": "lldb",
         "request": "launch",
         "name": "Debug executable",
         "cargo": {
           "args": ["build", "--bin=connectify-backend"],
           "filter": {
             "name": "connectify-backend",
             "kind": "bin"
           }
         },
         "args": [],
         "cwd": "${workspaceFolder}",
         "env": {
           "RUST_LOG": "debug"
         }
       }
     ]
   }
   ```
4. Press F5 to start debugging.

## Contributing

We welcome contributions to the Connectify project! Here's how you can contribute:

1. **Find an Issue**: Look for open issues on the GitHub repository, or create a new one if you've found a bug or have a feature request.

2. **Discuss the Issue**: Discuss the issue with the maintainers to ensure your approach aligns with the project's goals.

3. **Fork the Repository**: Fork the repository to your GitHub account.

4. **Create a Feature Branch**: Create a new branch for your changes:
   ```bash
   git checkout -b feature/your-feature-name
   ```

5. **Make Changes**: Implement your changes, following the coding standards.

6. **Write Tests**: Add tests for your changes to ensure they work as expected.

7. **Run Linters and Tests**: Run the linters and tests to ensure your code follows the project's style guidelines and doesn't break existing functionality:
   ```bash
   cargo clippy
   cargo fmt
   cargo test
   ```

8. **Commit Changes**: Commit your changes with a descriptive commit message:
   ```bash
   git add .
   git commit -m "Add feature: your feature description"
   ```

9. **Push Changes**: Push your changes to your forked repository:
   ```bash
   git push origin feature/your-feature-name
   ```

10. **Create a Pull Request**: Create a pull request on GitHub to merge your changes into the main branch.

11. **Address Review Comments**: Address any review comments from the maintainers.

12. **Merge**: Once your pull request is approved, it will be merged into the main branch.

## Troubleshooting

### Common Issues

#### Compilation Errors

If you encounter compilation errors, try the following:

1. Update your Rust toolchain:
   ```bash
   rustup update
   ```

2. Clean the build artifacts:
   ```bash
   cargo clean
   ```

3. Check for missing dependencies:
   ```bash
   cargo check
   ```

#### Runtime Errors

If you encounter runtime errors, try the following:

1. Enable debug logging:
   ```bash
   RUST_LOG=debug cargo run
   ```

2. Check the configuration:
   ```bash
   cargo run --bin config_watcher -- cargo run
   ```

3. Check for environment variables:
   ```bash
   env | grep CONNECTIFY
   ```

### Getting Help

If you're stuck, you can get help in the following ways:

1. **GitHub Issues**: Create a new issue on the GitHub repository.
2. **Documentation**: Check the documentation in the `docs` directory.
3. **Code Comments**: Look for comments in the code that might explain the behavior.
4. **Ask the Team**: Reach out to the team on the project's communication channels.