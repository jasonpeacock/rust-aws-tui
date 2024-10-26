.PHONY: run build release clean check fmt lint test all release-version

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

# Create a new release version
# Usage: make release-version VERSION=x.y.z
release-version:
	@if [ "$(VERSION)" = "" ]; then \
		echo "Please specify a version: make release-version VERSION=x.y.z"; \
		exit 1; \
	fi
	@echo "Creating release v$(VERSION)"
	@git flow release start $(VERSION) || true
	@cargo set-version $(VERSION)
	@git add Cargo.toml Cargo.lock
	@git commit -m "Bump version to $(VERSION)"
	@git flow release finish -m "Release v$(VERSION)" $(VERSION)
	@git push origin main develop --tags

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
	cargo install cargo-edit
	cargo install cargo-watch

# Watch for changes and run the application
watch:
	cargo watch -x run

# Run all quality checks
quality: fmt lint test
