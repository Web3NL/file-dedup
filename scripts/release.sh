#!/bin/bash

# Local release script for file-dedup
# Usage: ./scripts/release.sh <version>
# Example: ./scripts/release.sh 0.5.0

set -e

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

# Check if version is provided
if [ -z "$1" ]; then
    print_error "Usage: $0 <version>"
    print_error "Example: $0 0.5.0"
    exit 1
fi

VERSION="$1"
TAG="v$VERSION"

# Validate version format (basic check)
if ! [[ "$VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+(-[a-zA-Z0-9]+)?$ ]]; then
    print_error "Invalid version format. Use semantic versioning (e.g., 1.0.0, 1.0.0-beta)"
    exit 1
fi

# Check if we're in a git repository
if ! git rev-parse --git-dir > /dev/null 2>&1; then
    print_error "This script must be run from within a git repository"
    exit 1
fi

# Check if working directory is clean
if ! git diff-index --quiet HEAD --; then
    print_error "Working directory is not clean. Commit or stash your changes first."
    exit 1
fi

print_status "Starting release process for version $VERSION"

# Update version in Cargo.toml
print_status "Updating version in Cargo.toml..."
if command -v sed > /dev/null; then
    # macOS/BSD sed
    sed -i '' "s/^version = \".*\"/version = \"$VERSION\"/" Cargo.toml
else
    # GNU sed (Linux)
    sed -i "s/^version = \".*\"/version = \"$VERSION\"/" Cargo.toml
fi

# Verify the change
if grep -q "version = \"$VERSION\"" Cargo.toml; then
    print_success "Updated version in Cargo.toml to $VERSION"
else
    print_error "Failed to update version in Cargo.toml"
    exit 1
fi

# Update Cargo.lock
print_status "Updating Cargo.lock..."
cargo check
print_success "Updated Cargo.lock"

# Run tests
print_status "Running tests..."
if cargo test; then
    print_success "All tests passed"
else
    print_error "Tests failed. Fix them before releasing."
    exit 1
fi

# Build release binary
print_status "Building release binary..."
if cargo build --release; then
    print_success "Release binary built successfully"
else
    print_error "Failed to build release binary"
    exit 1
fi

# Check if binary works
print_status "Testing release binary..."
if ./target/release/file-dedup --help > /dev/null; then
    print_success "Release binary is working"
else
    print_error "Release binary is not working properly"
    exit 1
fi

# Commit version bump
print_status "Committing version bump..."
git add Cargo.toml Cargo.lock
git commit -m "Bump version to $VERSION"
print_success "Version bump committed"

# Create and push tag
print_status "Creating tag $TAG..."
git tag -a "$TAG" -m "Release $TAG"
print_success "Tag $TAG created"

# Ask for confirmation before pushing
echo
print_warning "Ready to push changes and tag to remote repository."
print_warning "This will trigger the release workflow if GitHub Actions is configured."
echo
read -p "Do you want to push now? (y/N): " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    print_status "Pushing changes and tag..."
    # Detect default branch (main or master)
    DEFAULT_BRANCH=$(git symbolic-ref refs/remotes/origin/HEAD 2>/dev/null | sed 's@^refs/remotes/origin/@@' || git branch --show-current)
    git push origin "$DEFAULT_BRANCH"
    git push origin "$TAG"
    print_success "Changes and tag pushed to remote repository"
    echo
    print_success "Release process completed!"
    print_status "If GitHub Actions is configured, the release will be automatically built and published."
    print_status "Check: https://github.com/$(git config --get remote.origin.url | sed 's/.*github.com[:/]\(.*\)\.git.*/\1/')/actions"
else
    print_warning "Changes and tag were not pushed."
    print_warning "You can push manually later with:"
    DEFAULT_BRANCH=$(git symbolic-ref refs/remotes/origin/HEAD 2>/dev/null | sed 's@^refs/remotes/origin/@@' || git branch --show-current)
    print_warning "  git push origin $DEFAULT_BRANCH"
    print_warning "  git push origin $TAG"
fi

echo
print_status "Local release artifacts:"
print_status "  Binary: ./target/release/file-dedup"
print_status "  Version: $VERSION"
print_status "  Tag: $TAG"
