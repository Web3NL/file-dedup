#!/bin/bash

# Development check script for file-dedup
# Runs formatting, linting, and tests to ensure code quality
# Usage: ./scripts/check.sh [--fix]

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Print colored output
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if --fix flag is provided
FIX_MODE=false
if [ "${1:-}" = "--fix" ]; then
    FIX_MODE=true
    print_status "Running in fix mode - will automatically fix issues where possible"
fi

echo "🔍 File Deduplication Tool - Code Quality Checks"
echo "================================================"
echo

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ]; then
    print_error "This script must be run from the project root directory"
    exit 1
fi

EXIT_CODE=0

# 1. Check/Fix formatting
print_status "1. Checking code formatting..."
if [ "$FIX_MODE" = true ]; then
    if cargo fmt --all; then
        print_success "Code formatted successfully"
    else
        print_error "Failed to format code"
        EXIT_CODE=1
    fi
else
    if cargo fmt --all -- --check; then
        print_success "Code formatting is correct"
    else
        print_error "Code formatting issues found"
        print_warning "Run 'cargo fmt --all' or './scripts/check.sh --fix' to fix them"
        EXIT_CODE=1
    fi
fi
echo

# 2. Run clippy
print_status "2. Running clippy linting..."
if [ "$FIX_MODE" = true ]; then
    print_status "Running clippy with automatic fixes..."
    if cargo clippy --all-targets --all-features --fix --allow-dirty --allow-staged -- -D warnings; then
        print_success "Clippy issues fixed automatically"
    else
        print_warning "Some clippy issues require manual intervention"
        # Run regular clippy to show remaining issues
        cargo clippy --all-targets --all-features -- -D warnings || EXIT_CODE=1
    fi
else
    if cargo clippy --all-targets --all-features -- -D warnings; then
        print_success "Clippy checks passed"
    else
        print_error "Clippy found issues"
        print_warning "Run './scripts/check.sh --fix' to auto-fix some issues"
        EXIT_CODE=1
    fi
fi
echo

# 3. Run tests
print_status "3. Running tests..."
if cargo test; then
    print_success "All tests passed"
else
    print_error "Tests failed"
    EXIT_CODE=1
fi
echo

# 4. Check for common issues
print_status "4. Additional checks..."

# Check for TODO/FIXME comments in main code (excluding this script)
TODO_COUNT=$(find src -name "*.rs" -exec grep -n "TODO\|FIXME" {} + 2>/dev/null | wc -l | xargs || echo "0")
if [ "$TODO_COUNT" -gt 0 ]; then
    print_warning "Found $TODO_COUNT TODO/FIXME comments in source code"
    find src -name "*.rs" -exec grep -n "TODO\|FIXME" {} + 2>/dev/null || true
else
    print_success "No TODO/FIXME comments found"
fi

# Check for unused dependencies (requires cargo-machete)
if command -v cargo-machete > /dev/null 2>&1; then
    if cargo machete; then
        print_success "No unused dependencies found"
    else
        print_warning "Found unused dependencies (install cargo-machete to check: cargo install cargo-machete)"
    fi
else
    print_status "Skipping unused dependency check (cargo-machete not installed)"
fi

echo

# Summary
echo "==============================================="
if [ $EXIT_CODE -eq 0 ]; then
    print_success "🎉 All checks passed! Code is ready for commit/release."
else
    print_error "❌ Some checks failed. Please fix the issues above."
    echo
    print_status "Quick fixes:"
    print_status "  Format code:    cargo fmt --all"
    print_status "  Fix clippy:     cargo clippy --fix --allow-dirty"
    print_status "  Auto-fix all:   ./scripts/check.sh --fix"
fi

exit $EXIT_CODE
