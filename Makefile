.PHONY: run build release clean check fmt lint test all

# Default target
all: check test build

# Run the application in debug mode
run:
	cargo run

# Build in debug mode
build:
	cargo build

# Build for release
release:
	cargo build --release

# Clean build artifacts
clean:
	cargo clean

# Run cargo check
check:
	cargo check

# Format code using rustfmt
fmt:
	cargo fmt --all

# Run clippy for linting
lint:
	cargo clippy -- -D warnings

# Run tests
test:
	cargo test

# Install development dependencies
dev-deps:
	rustup component add rustfmt
	rustup component add clippy

# Watch for changes and run the application
watch:
	cargo watch -x run

# Run all quality checks
quality: fmt lint test
