.PHONY: build test clean deploy-localnet deploy-devnet deploy-mainnet format lint check help

# Verify code quality (format check + lint + build + test)
check:
	cargo fmt --all -- --check
	cargo clippy
	cargo build-sbf
	cargo test

# Build the program
build:
	cargo build-sbf

# Run tests
test:
	cargo test

# Clean build artifacts
clean:
	cargo clean

# Format code
format:
	cargo fmt --all

# Lint code
lint:
	cargo clippy

# Deploy to localnet (requires validator to be running)
deploy-localnet:
	solana program deploy target/deploy/cascade_protocol_deadline_validator.so --url localhost

# Deploy to devnet
deploy-devnet:
	solana program deploy target/deploy/cascade_protocol_deadline_validator.so --url devnet

# Deploy to mainnet (with confirmation prompt)
deploy-mainnet:
	@echo "WARNING: Deploying to MAINNET. Press Ctrl+C to cancel, Enter to continue..."
	@read confirm
	solana program deploy target/deploy/cascade_protocol_deadline_validator.so --url mainnet-beta

# Help
help:
	@echo "Deadline Validator - Makefile Commands"
	@echo ""
	@echo "  make build            Build the Solana program"
	@echo "  make test             Run all tests"
	@echo "  make clean            Remove build artifacts"
	@echo "  make deploy-localnet  Deploy to localnet"
	@echo "  make deploy-devnet    Deploy to devnet"
	@echo "  make format           Format Rust code"
	@echo "  make lint             Lint Rust code"
	@echo "  make help             Show this help message"
