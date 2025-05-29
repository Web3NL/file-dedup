# GitHub Release Setup Instructions

This document explains how to set up automated releases for the file-dedup project.

## Prerequisites

1. Your repository must be hosted on GitHub
2. You need admin access to the repository to configure secrets

## Setup Steps

### 1. GitHub Actions is Ready

The GitHub Actions workflows are already configured in `.github/workflows/`:

- `ci.yml` - Runs tests on every push and PR
- `release.yml` - Creates releases when you push version tags

### 2. Configure GitHub Secrets (Optional)

For publishing to crates.io, you need to set up a secret:

1. Go to your repository on GitHub
2. Click **Settings** → **Secrets and variables** → **Actions**
3. Click **New repository secret**
4. Add the following secret:

   **Name:** `CARGO_REGISTRY_TOKEN`
   **Value:** Your crates.io API token

#### Getting a crates.io API Token

1. Go to https://crates.io/me
2. Click **New Token**
3. Give it a name like "file-dedup-releases"
4. Select appropriate scopes (publish is sufficient)
5. Copy the generated token

### 3. Test the Release Process

1. Make sure your code is committed and pushed
2. Use the release script:
   ```bash
   ./scripts/release.sh 0.4.1
   ```

   Or manually create a tag:
   ```bash
   git tag v0.4.1
   git push origin v0.4.1
   ```

3. Check the Actions tab on GitHub to see the workflows running
4. Once complete, check the Releases section for your new release

## What Happens During a Release

When you push a version tag (e.g., `v0.4.1`):

1. **Build Matrix**: Creates binaries for:
   - Linux (glibc x86_64)
   - Linux (musl x86_64) - static binary
   - Windows (x86_64)
   - macOS (Intel x86_64)
   - macOS (Apple Silicon arm64)

2. **Testing**: Runs tests on all platforms

3. **Release Creation**: 
   - Creates a GitHub release
   - Attaches all built binaries
   - Generates changelog from git commits
   - Includes installation instructions

4. **crates.io Publishing**: (if token is configured)
   - Publishes the crate to crates.io
   - Makes it installable via `cargo install file-dedup`

## Release Naming Convention

- Use semantic versioning: `MAJOR.MINOR.PATCH`
- Tags should be prefixed with `v`: `v1.0.0`, `v0.4.1`, etc.
- Pre-releases can use suffixes: `v1.0.0-beta`, `v1.0.0-rc1`

## Troubleshooting

### Build Fails
- Check the Actions logs on GitHub
- Common issues: tests failing, clippy warnings, formatting issues

### crates.io Publishing Fails
- Verify your `CARGO_REGISTRY_TOKEN` secret is correct
- Make sure the version isn't already published
- Check that Cargo.toml has all required fields

### Manual Release
If GitHub Actions fails, you can always create releases manually:

```bash
# Build all targets locally (requires cross-compilation setup)
cargo build --release

# Create release on GitHub web interface
# Upload binary manually
```

## Local Development Workflow

For development without releases:

```bash
# Format code
cargo fmt

# Check for issues
cargo clippy

# Run tests
cargo test

# Build and test locally
cargo build --release
./target/release/file-dedup --help
```
