# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Architecture

The project is structured as a library (`src/lib.rs`) with a CLI frontend (`src/main.rs`):

- **Library (`file_dedup` crate)**: Core deduplication logic including file collection, hashing, and duplicate group detection
- **CLI**: User interface with colored output, interactive prompts, and safety checks

Key architectural components:
- `FileInfo` struct: Represents files with path, size, and lazy-calculated hash
- Size-based pre-filtering: Groups files by size before expensive hash calculations
- xxHash (XXH3) for fast, reliable duplicate detection
- Interactive resolution with safety checks (prevents deleting all copies)

## Development Commands

```bash
# Build and test
cargo build --release
cargo test

# Run integration tests specifically
cargo test --test integration_tests

# Run the tool
cargo run -- ~/Documents
./target/release/file-dedup -v ~/Documents      # Verbose report mode
./target/release/file-dedup -i ~/Documents      # Interactive mode

# Test CLI with multiple paths
cargo run -- ~/Documents ~/Pictures ~/Downloads

# Release management
./scripts/release.sh                            # Automated versioning and release
```

## Key Implementation Notes

- **Performance optimization**: Uses file size pre-filtering to avoid unnecessary hash calculations
- **Safety-first design**: Report mode is default; interactive mode requires explicit confirmation
- **Memory efficiency**: Streams file content for hashing rather than loading entire files
- **Error handling**: Graceful handling of permission errors and inaccessible files
- **Empty file handling**: Skips zero-byte files to focus on meaningful duplicates
- **Security measures**: Skips symlinks to prevent following malicious links outside scan directory
- **TOCTOU protection**: Verifies file size before deletion in interactive mode