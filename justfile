# JsonnetGen Development Tasks
# Run with: just <task>

# Default task - show help
default:
    @just --list

# Build the project
build:
    cargo build

# Build in release mode
build-release:
    cargo build --release

# Run tests
test:
    cargo test

# Run tests with output
test-verbose:
    cargo test -- --nocapture

# Run specific test
test-specific test:
    cargo test {{test}}

# Run integration tests
test-integration:
    cargo test --test integration_test

# Run all tests including integration
test-all:
    cargo test
    cargo test --test integration_test

# Check code formatting
fmt:
    cargo fmt --all -- --check

# Format code
fmt-fix:
    cargo fmt --all

# Run clippy linter
clippy:
    cargo clippy --all-targets --all-features -- -D warnings

# Run clippy with suggestions
clippy-fix:
    cargo clippy --all-targets --all-features --fix -- -D warnings

# Check if code compiles
check:
    cargo check

# Run all checks (fmt, clippy, test)
check-all: fmt clippy test-all

# Install the binary locally
install:
    cargo install --path .

# Clean build artifacts
clean:
    cargo clean

# Update dependencies
update:
    cargo update

# Run the CLI with info command
info:
    cargo run -- info

# Initialize example configuration
init-example:
    cargo run -- init --example

# Validate configuration
validate:
    cargo run -- validate

# Generate from configuration
generate:
    cargo run -- generate

# Generate with verbose output
generate-verbose:
    RUST_LOG=debug cargo run -- generate

# Show lockfile status
lock-status:
    cargo run -- lock --status

# Update lockfile
lock-update:
    cargo run -- lock --update

# Run with custom config
run-with-config config:
    cargo run -- --config {{config}} generate

# Create a new release build
release:
    cargo build --release
    @echo "Release build complete: target/release/jsonnet-gen"

# Run benchmarks (if available)
bench:
    cargo bench

# Generate documentation
doc:
    cargo doc --no-deps --open

# Check for security vulnerabilities
audit:
    cargo audit

# Run cargo outdated to check for dependency updates
outdated:
    cargo outdated

# Create a test CRD file for development
create-test-crd:
    @echo "Creating test CRD file..."
    @mkdir -p test-data
    @echo "apiVersion: apiextensions.k8s.io/v1" > test-data/test-crd.yaml
    @echo "kind: CustomResourceDefinition" >> test-data/test-crd.yaml
    @echo "metadata:" >> test-data/test-crd.yaml
    @echo "  name: tests.example.com" >> test-data/test-crd.yaml
    @echo "spec:" >> test-data/test-crd.yaml
    @echo "  group: example.com" >> test-data/test-crd.yaml
    @echo "  names:" >> test-data/test-crd.yaml
    @echo "    kind: Test" >> test-data/test-crd.yaml
    @echo "    plural: tests" >> test-data/test-crd.yaml
    @echo "  versions:" >> test-data/test-crd.yaml
    @echo "    - name: v1" >> test-data/test-crd.yaml
    @echo "      served: true" >> test-data/test-crd.yaml
    @echo "      storage: true" >> test-data/test-crd.yaml
    @echo "      schema:" >> test-data/test-crd.yaml
    @echo "        openAPIV3Schema:" >> test-data/test-crd.yaml
    @echo "          type: object" >> test-data/test-crd.yaml
    @echo "          properties:" >> test-data/test-crd.yaml
    @echo "            spec:" >> test-data/test-crd.yaml
    @echo "              type: object" >> test-data/test-crd.yaml
    @echo "              properties:" >> test-data/test-crd.yaml
    @echo "                name:" >> test-data/test-crd.yaml
    @echo "                  type: string" >> test-data/test-crd.yaml
    @echo "                replicas:" >> test-data/test-crd.yaml
    @echo "                  type: integer" >> test-data/test-crd.yaml
    @echo "                  minimum: 1" >> test-data/test-crd.yaml
    @echo "                  maximum: 10" >> test-data/test-crd.yaml
    @echo "Test CRD created at test-data/test-crd.yaml"

# Run a complete development workflow
dev-workflow: clean check-all
    @echo "Development workflow complete!"

# Prepare for release
prepare-release: clean check-all build-release
    @echo "Release preparation complete!"

# Show project status
status:
    @echo "=== JsonnetGen Project Status ==="
    @echo "Rust version:"
    @rustc --version
    @echo ""
    @echo "Cargo version:"
    @cargo --version
    @echo ""
    @echo "Git status:"
    @git status --porcelain || echo "Not a git repository"
    @echo ""
    @echo "Recent commits:"
    @git log --oneline -5 || echo "No git history"

# Help task
help:
    @echo "Available tasks:"
    @just --list
