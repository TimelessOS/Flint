# Flint Agent Guidelines

## Build/Test Commands
- **Build**: `cargo build --verbose`
- **Release build**: `cargo build --release`
- **Test all**: `cargo test --verbose`
- **Test with features**: `cargo test --verbose --all-features`
- **Single test**: `cargo test <test_name>` or `cargo test <test_name> -- --nocapture`
- **Lint**: `cargo clippy --verbose --all-features`

## Code Style Guidelines

### Rust Edition & Linting
- Use Rust 2024 edition
- Strict clippy rules: all/correctness/suspicious/perf/complexity/style/pedantic/nursery
- `unsafe_code = "forbid"`, `unused_imports = "deny"`

### Error Handling
- Use `anyhow::Result<T>` for all functions
- Use `?` operator for error propagation
- Use `bail!()` and `anyhow!()` for creating errors
- Add comprehensive error documentation in function comments

### Imports & Organization
- Group imports: std, external crates, local modules
- Use feature flags: `#[cfg(feature = "network")]`
- Prefer explicit imports over glob imports

### Naming Conventions
- Functions/variables: `snake_case`
- Types/structs/enums: `PascalCase`
- Constants: `SCREAMING_SNAKE_CASE`
- Modules: `snake_case`

### Code Patterns
- Use `#[derive(Debug, Clone, PartialEq)]` for data types
- Async functions: `#[tokio::main]` for main, regular `async fn` elsewhere
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