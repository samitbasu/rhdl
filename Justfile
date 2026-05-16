# List available recipes
default:
    @just --list

# Run everything CI runs (fmt-check + clippy + test + doctest)
ci: fmt-check clippy test doctest

# Verify the codebase is formatted (fails if not)
fmt-check:
    cargo fmt --all -- --check

# Reformat the codebase in place
fmt:
    cargo fmt --all

# Run clippy lints across the workspace
clippy:
    cargo clippy --workspace --all-targets

# Run the workspace test suite via nextest (faster, parallelizes test binaries)
test:
    cargo nextest run --workspace --no-fail-fast

# Run doctests (nextest does not handle these)
doctest:
    cargo test --doc --workspace --no-fail-fast
