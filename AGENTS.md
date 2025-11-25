# Flint Agent Guidelines

## Build/Test Commands
- **Build**: `cargo build --verbose`
- **Test**: `cargo test --verbose --all-features`
- **Lint**: `cargo clippy --verbose --all-features`

## Code Style Guidelines

### Rust Edition & Linting
- Use Rust 2024 edition

### Error Handling
- Use `anyhow::Result<T>` for all functions
- Use `?` operator for error propagation
- Use `bail!()` and `anyhow!()` for creating errors
- Add comprehensive error documentation in function comments

### Imports & Organization
- Group imports: std, external crates, local modules
- Use feature flags: `#[cfg(feature = "network")]`
- Prefer explicit imports over glob imports

### Code Patterns
- CLI: Use `clap` with derive macros for commands
- Serialization: Use `serde` with YAML format
- Documentation: Comprehensive doc comments with error descriptions
- Testing: Unit tests with `#[test]` and `#[cfg(test)]` modules

### Security & Best Practices
- No unsafe code allowed
- Use path canonicalization for security
- Validate user inputs and paths
- Sign manifests with cryptographic keys
- Use temp directories for tests
