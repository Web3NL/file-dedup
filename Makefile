# Makefile for file-dedup development
.PHONY: help check check-fix format lint test build clean release install-tools setup

# Default target
help: ## Show this help message
	@echo "File Deduplication Tool - Development Commands"
	@echo "=============================================="
	@echo
	@awk 'BEGIN {FS = ":.*##"} /^[a-zA-Z_-]+:.*##/ { printf "  \033[36m%-15s\033[0m %s\n", $$1, $$2 }' $(MAKEFILE_LIST)

# Quality checks
check: ## Run all code quality checks (format, lint, test)
	@./scripts/check.sh

check-fix: ## Run checks and auto-fix issues where possible
	@./scripts/check.sh --fix

format: ## Format code using rustfmt
	@echo "🎨 Formatting code..."
	@cargo fmt --all

lint: ## Run clippy linting
	@echo "🔍 Running clippy..."
	@cargo clippy --all-targets --all-features -- -D warnings

lint-fix: ## Run clippy with automatic fixes
	@echo "🔧 Running clippy with fixes..."
	@cargo clippy --all-targets --all-features --fix --allow-dirty --allow-staged -- -D warnings

test: ## Run all tests
	@echo "🧪 Running tests..."
	@cargo test

# Build targets
build: ## Build the project
	@echo "🔨 Building project..."
	@cargo build

build-release: ## Build optimized release binary
	@echo "🚀 Building release binary..."
	@cargo build --release

# Utility commands
clean: ## Clean build artifacts
	@echo "🧹 Cleaning build artifacts..."
	@cargo clean

release: ## Create a new release (usage: make release VERSION=1.0.0)
	@if [ -z "$(VERSION)" ]; then \
		echo "❌ VERSION is required. Usage: make release VERSION=1.0.0"; \
		exit 1; \
	fi
	@echo "🚀 Creating release $(VERSION)..."
	@./scripts/release.sh $(VERSION)

# Setup and installation
install-tools: ## Install development tools
	@echo "🛠️  Installing development tools..."
	@echo "Installing rustfmt and clippy..."
	@rustup component add rustfmt clippy
	@echo "Installing pre-commit (requires Python)..."
	@if command -v pip > /dev/null; then \
		pip install pre-commit; \
	else \
		echo "⚠️  pip not found. Install Python and pip to use pre-commit hooks"; \
	fi
	@echo "Installing cargo-machete for unused dependency detection..."
	@cargo install cargo-machete || echo "⚠️  Failed to install cargo-machete"

setup: install-tools ## Setup development environment
	@echo "🔧 Setting up development environment..."
	@if command -v pre-commit > /dev/null; then \
		pre-commit install; \
		echo "✅ Pre-commit hooks installed"; \
	else \
		echo "⚠️  pre-commit not found. Run 'make install-tools' first"; \
	fi

# Documentation
doc: ## Generate and open documentation
	@echo "📚 Generating documentation..."
	@cargo doc --open

# Security
audit: ## Run security audit
	@echo "🔒 Running security audit..."
	@cargo audit
