# Code Quality and Release Process Integration

This document explains how rustfmt and clippy are integrated into the file-dedup project's development and release workflow.

## Overview

The project now has comprehensive code quality checks integrated at multiple levels:

1. **Development Tools** - For daily development
2. **Pre-commit Hooks** - Automated checks before commits
3. **CI Pipeline** - Automated checks on GitHub
4. **Release Process** - Final quality gates before release

## Development Workflow

### Quick Commands

```bash
# Run all quality checks
make check

# Auto-fix issues where possible
make check-fix

# Individual commands
make format       # Format code with rustfmt
make lint         # Run clippy linting
make test         # Run tests
make build        # Build debug version
make build-release # Build optimized release
```

### Detailed Commands

```bash
# Check formatting only
cargo fmt --all -- --check

# Format code
cargo fmt --all

# Run clippy with strict settings
cargo clippy --all-targets --all-features -- -D warnings

# Auto-fix clippy issues
cargo clippy --all-targets --all-features --fix --allow-dirty --allow-staged -- -D warnings
```

## Pre-commit Hooks

Set up automated checks that run before each commit:

```bash
# Install pre-commit (requires Python)
pip install pre-commit

# Setup hooks
make setup
# or manually: pre-commit install

# Run manually on all files
pre-commit run --all-files
```

The pre-commit configuration (`.pre-commit-config.yaml`) includes:
- `cargo fmt` - Code formatting
- `cargo clippy` - Linting with warnings as errors
- `cargo test` - Test execution
- Basic file checks (trailing whitespace, TOML/YAML validation, etc.)

## CI Integration

The GitHub Actions workflow (`.github/workflows/ci.yml`) automatically runs:

- **Formatting check**: `cargo fmt --all -- --check` (Ubuntu/stable only)
- **Clippy linting**: `cargo clippy --all-targets --all-features -- -D warnings` (all stable builds)
- **Tests**: `cargo test` (all platforms and Rust versions)
- **Security audit**: `cargo audit` (Ubuntu only)

## Release Process

The release script (`scripts/release.sh`) now includes quality gates:

1. **Format check**: Ensures code is properly formatted
2. **Clippy check**: Runs linting with warnings as errors
3. **Test execution**: Ensures all tests pass
4. **Build verification**: Confirms release binary builds and works
5. **Version bump and tag creation**: Only after all checks pass

### Enhanced Release Script

The script will fail and exit if any of these conditions are not met:
- Code is not properly formatted
- Clippy finds any warnings or errors
- Any tests fail
- Release binary doesn't build
- Release binary doesn't execute properly

### Running a Release

```bash
# Use the enhanced release script
./scripts/release.sh 1.2.3

# Or use the Make target
make release VERSION=1.2.3
```

## Quality Standards

### Enforced Standards

1. **Code Formatting**: Must pass `cargo fmt --all -- --check`
2. **Linting**: Must pass `cargo clippy --all-targets --all-features -- -D warnings`
3. **Tests**: All tests must pass
4. **Build**: Must build successfully in release mode

### Additional Checks

The development tools also check for:
- TODO/FIXME comments in source code
- Unused dependencies (if cargo-machete is installed)
- File formatting issues (trailing whitespace, etc.)

## Tool Installation

### Required Tools (included with Rust)
- `rustfmt` - Code formatting
- `clippy` - Linting

### Optional Tools
- `pre-commit` - Pre-commit hooks (requires Python)
- `cargo-machete` - Unused dependency detection
- `cargo-audit` - Security vulnerability scanning

Install all tools:
```bash
make install-tools
```

## Troubleshooting

### Common Issues

1. **Format issues**: Run `cargo fmt --all` or `make check-fix`
2. **Clippy warnings**: Run `cargo clippy --fix --allow-dirty` or `make check-fix`
3. **Pre-commit failures**: Run `make check-fix` then commit again

### Bypassing Checks (Not Recommended)

If you absolutely need to bypass pre-commit hooks:
```bash
git commit --no-verify
```

However, CI will still catch and fail on quality issues.

## Integration Benefits

This integrated approach provides:

1. **Consistent Code Style**: Automated formatting ensures consistent style
2. **Early Bug Detection**: Clippy catches potential issues early
3. **Quality Assurance**: Multiple checkpoints prevent quality regression
4. **Developer Experience**: Easy-to-use commands and automatic fixes
5. **Release Confidence**: Comprehensive checks before releases

## Future Enhancements

Potential additions:
- Code coverage reporting
- Performance benchmarking in CI
- Documentation generation and checks
- Dependency vulnerability scanning integration
- Custom lint rules specific to the project
