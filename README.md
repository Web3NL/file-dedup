# File Dedup

A minimal file deduplication tool that finds duplicate files using SHA-256 hashing.

## Features

- **Safe by default**: Only reports duplicates, never deletes anything
- **Fast detection**: Uses file size pre-filtering before expensive hash calculations
- **Recursive scanning**: Automatically scans subdirectories
- **Clear output**: Groups duplicates and shows which files could be removed
- **Cross-platform**: Works on Windows, macOS, and Linux

## Quick Start

```bash
# Build the tool
cargo build --release

# Run the demo to see it in action
./demo.sh

# Or test it directly on any directory
./target/release/file-dedup ~/Documents
```

## Installation

```bash
# Clone and build
git clone <your-repo>
cd file-dedup
cargo build --release

# The binary will be in target/release/file-dedup
```

## Usage

```bash
# Basic usage - scan one or more paths
file-dedup /path/to/directory

# Scan multiple paths
file-dedup ~/Documents ~/Pictures ~/Downloads

# Verbose output to see progress
file-dedup -v ~/Documents

# Get help
file-dedup --help
```

## Example Output

```
Found duplicate files:

Duplicate Group 1 (Size: 2048576 bytes, Hash: a1b2c3d4e5f6789a):
  [KEEP] /home/user/Documents/photo1.jpg
  [DUP]  /home/user/Pictures/photo1_copy.jpg
  [DUP]  /home/user/Downloads/photo1.jpg

Duplicate Group 2 (Size: 1024 bytes, Hash: f9e8d7c6b5a49876):
  [KEEP] /home/user/Documents/notes.txt
  [DUP]  /home/user/Documents/backup/notes.txt

Summary:
  Found 2 duplicate groups
  Total duplicate files: 5
  Files that could be removed: 3
  Potential space savings: 6291456 bytes
```

## How It Works

1. **File Collection**: Recursively scans specified paths for files
2. **Size Filtering**: Groups files by size (files with different sizes can't be duplicates)
3. **Hash Calculation**: Calculates SHA-256 hashes only for files with matching sizes
4. **Duplicate Detection**: Groups files with identical hashes as duplicates
5. **Safe Reporting**: Shows results without making any changes to your files

## Options

- `-v, --verbose`: Show detailed progress during scanning
- `-h, --help`: Show help information
- `-V, --version`: Show version information

## Safety

This tool is designed to be completely safe:
- **Read-only**: Never modifies, moves, or deletes any files
- **No false positives**: Uses cryptographic hashing (SHA-256) for accurate duplicate detection
- **Clear marking**: Shows which file would be kept (`[KEEP]`) vs removed (`[DUP]`)

## Performance

The tool is optimized for performance:
- **Size pre-filtering**: Avoids expensive hash calculations for files that can't be duplicates
- **Streaming hash calculation**: Processes large files efficiently without loading them entirely into memory
- **Skip empty files**: Ignores zero-byte files to focus on meaningful duplicates

## Future Enhancements

This is a minimal implementation. Potential future features include:
- Deletion/linking capabilities with safety checks
- Multiple hash algorithm support
- File filtering options (size, type, patterns)
- Progress indicators for large scans
- Configuration file support
- JSON/CSV output formats

## License

MIT License - see LICENSE file for details.
