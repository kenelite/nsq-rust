# Makefile for NSQ Rust
.PHONY: help build test clean docker-build docker-run docker-test lint format check

# Default target
help:
	@echo "NSQ Rust Makefile"
	@echo ""
	@echo "Available targets:"
	@echo "  build          Build all components"
	@echo "  test           Run all tests"
	@echo "  clean          Clean build artifacts"
	@echo "  docker-build   Build Docker images"
	@echo "  docker-run     Run Docker containers"
	@echo "  docker-test    Run tests in Docker"
	@echo "  lint           Run clippy linter"
	@echo "  format         Format code with rustfmt"
	@echo "  check          Run cargo check"
	@echo "  ui-build       Build NSQAdmin UI"
	@echo "  ui-dev         Start UI development server"
	@echo "  release        Build release binaries"
	@echo "  install        Install binaries to system"
	@echo "  uninstall      Remove installed binaries"
	@echo ""
	@echo "Examples:"
	@echo "  make build"
	@echo "  make test"
	@echo "  make docker-build"
	@echo "  make docker-run"

# Build all components
build:
	@echo "Building NSQ Rust components..."
	cargo build

# Build release version
release:
	@echo "Building release version..."
	cargo build --release

# Run all tests
test:
	@echo "Running tests..."
	cargo test

# Run integration tests
test-integration:
	@echo "Running integration tests..."
	cargo test --test integration

# Run compatibility tests
test-compatibility:
	@echo "Running compatibility tests..."
	cargo test --test compatibility

# Clean build artifacts
clean:
	@echo "Cleaning build artifacts..."
	cargo clean
	rm -rf target/
	rm -rf nsqadmin-ui/dist/
	rm -rf nsqadmin-ui/node_modules/

# Run clippy linter
lint:
	@echo "Running clippy linter..."
	cargo clippy -- -D warnings

# Format code with rustfmt
format:
	@echo "Formatting code..."
	cargo fmt

# Run cargo check
check:
	@echo "Running cargo check..."
	cargo check

# Build UI
ui-build:
	@echo "Building NSQAdmin UI..."
	cd nsqadmin-ui && npm install && npm run build

# Start UI development server
ui-dev:
	@echo "Starting UI development server..."
	cd nsqadmin-ui && npm install && npm run dev

# Build Docker images
docker-build:
	@echo "Building Docker images..."
	./scripts/docker-build.sh

# Build specific Docker image
docker-build-nsqd:
	@echo "Building NSQD Docker image..."
	./scripts/docker-build.sh --nsqd

docker-build-nsqlookupd:
	@echo "Building NSQLookupd Docker image..."
	./scripts/docker-build.sh --nsqlookupd

docker-build-nsqadmin:
	@echo "Building NSQAdmin Docker image..."
	./scripts/docker-build.sh --nsqadmin

# Run Docker containers
docker-run:
	@echo "Running Docker containers..."
	./scripts/docker-run.sh --env dev --action up

docker-run-prod:
	@echo "Running production Docker containers..."
	./scripts/docker-run.sh --env prod --action up

docker-run-test:
	@echo "Running test Docker containers..."
	./scripts/docker-run.sh --env test --action up

# Run tests in Docker
docker-test:
	@echo "Running tests in Docker..."
	./scripts/docker-run.sh --env test --action up
	docker-compose -f docker-compose.test.yml exec test-runner cargo test

# Stop Docker containers
docker-stop:
	@echo "Stopping Docker containers..."
	./scripts/docker-run.sh --env dev --action down

docker-stop-prod:
	@echo "Stopping production Docker containers..."
	./scripts/docker-run.sh --env prod --action down

docker-stop-test:
	@echo "Stopping test Docker containers..."
	./scripts/docker-run.sh --env test --action down

# View Docker logs
docker-logs:
	@echo "Viewing Docker logs..."
	./scripts/docker-run.sh --env dev --action logs

# Install binaries to system
install: release
	@echo "Installing NSQ Rust binaries..."
	sudo cp target/release/nsqd /usr/local/bin/
	sudo cp target/release/nsqlookupd /usr/local/bin/
	sudo cp target/release/nsqadmin /usr/local/bin/
	sudo cp target/release/nsq_to_file /usr/local/bin/
	sudo cp target/release/to_nsq /usr/local/bin/
	sudo cp target/release/nsq_tail /usr/local/bin/
	sudo cp target/release/nsq_stat /usr/local/bin/
	sudo cp target/release/nsq_to_http /usr/local/bin/
	sudo cp target/release/nsq_to_nsq /usr/local/bin/
	@echo "Installation complete!"

# Remove installed binaries
uninstall:
	@echo "Removing NSQ Rust binaries..."
	sudo rm -f /usr/local/bin/nsqd
	sudo rm -f /usr/local/bin/nsqlookupd
	sudo rm -f /usr/local/bin/nsqadmin
	sudo rm -f /usr/local/bin/nsq_to_file
	sudo rm -f /usr/local/bin/to_nsq
	sudo rm -f /usr/local/bin/nsq_tail
	sudo rm -f /usr/local/bin/nsq_stat
	sudo rm -f /usr/local/bin/nsq_to_http
	sudo rm -f /usr/local/bin/nsq_to_nsq
	@echo "Uninstallation complete!"

# Development setup
dev-setup:
	@echo "Setting up development environment..."
	rustup component add rustfmt clippy
	cargo install cargo-watch cargo-expand cargo-audit cargo-outdated
	cd nsqadmin-ui && npm install
	@echo "Development setup complete!"

# Run development server
dev:
	@echo "Starting development servers..."
	@echo "Starting NSQLookupd..."
	cargo run -p nsqlookupd -- --tcp-address=127.0.0.1:4160 --http-address=127.0.0.1:4161 &
	@echo "Starting NSQD..."
	cargo run -p nsqd -- --tcp-address=127.0.0.1:4150 --http-address=127.0.0.1:4151 --lookupd-tcp-address=127.0.0.1:4160 --lookupd-http-address=127.0.0.1:4161 &
	@echo "Starting NSQAdmin..."
	cargo run -p nsqadmin -- --http-address=127.0.0.1:4171 --lookupd-http-address=127.0.0.1:4161 &
	@echo "Development servers started!"
	@echo "NSQAdmin: http://localhost:4171"
	@echo "NSQD: http://localhost:4151"
	@echo "NSQLookupd: http://localhost:4161"

# Stop development servers
dev-stop:
	@echo "Stopping development servers..."
	pkill -f nsqlookupd || true
	pkill -f nsqd || true
	pkill -f nsqadmin || true
	@echo "Development servers stopped!"

# Benchmark
bench:
	@echo "Running benchmarks..."
	cargo bench

# Security audit
audit:
	@echo "Running security audit..."
	cargo audit

# Check for outdated dependencies
outdated:
	@echo "Checking for outdated dependencies..."
	cargo outdated

# Generate documentation
docs:
	@echo "Generating documentation..."
	cargo doc --no-deps --open

# Package release
package: release
	@echo "Packaging release..."
	mkdir -p dist
	tar -czf dist/nsq-rust-linux-amd64.tar.gz -C target/release nsqd nsqlookupd nsqadmin nsq_to_file to_nsq nsq_tail nsq_stat nsq_to_http nsq_to_nsq
	@echo "Release packaged: dist/nsq-rust-linux-amd64.tar.gz"

# Full CI pipeline
ci: clean format lint check test test-integration test-compatibility
	@echo "CI pipeline completed successfully!"

# Full release pipeline
release-pipeline: clean format lint check test test-integration test-compatibility release package
	@echo "Release pipeline completed successfully!"
