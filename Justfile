# List available recipes
default:
    @just --list

# Run everything CI runs (fmt-check + clippy + test)
ci: fmt-check clippy test

# Verify the codebase is formatted (fails if not)
fmt-check:
    cargo fmt --all -- --check

# Reformat the codebase in place
fmt:
    cargo fmt --all

# Run clippy lints across the workspace
clippy:
    cargo clippy --workspace --all-targets

# Run the full workspace test suite
test:
    cargo test --workspace --no-fail-fast
