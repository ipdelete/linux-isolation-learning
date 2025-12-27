.PHONY: clippy fmt build check clean all

# Run clippy linter on all crates
clippy:
	cargo clippy --all-targets --all-features

# Format all code
fmt:
	cargo fmt --all

# Build all crates (debug mode)
build:
	cargo build --all

# Check all crates (faster than build, no binary output)
check:
	cargo check --all

# Remove build artifacts
clean:
	cargo clean

# Run all checks
all: fmt clippy check clean
