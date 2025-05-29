# File Dedup

A minimal file deduplication tool that finds duplicate files using xxHash.

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

# Test it on any directory
./target/release/file-dedup ~/Documents

# Or scan multiple directories
./target/release/file-dedup ~/Documents ~/Pictures ~/Downloads
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

Duplicate Group 1 (Size: 2048576 bytes, Hash: a1b2c3d4):
    Status: [KEEP]
    Location: /home/user/Documents
    Title: photo1.jpg

    Status: [DUP]
    Location: /home/user/Pictures
    Title: photo1_copy.jpg

    Status: [DUP]
    Location: /home/user/Downloads
    Title: photo1.jpg


Duplicate Group 2 (Size: 1024 bytes, Hash: f9e8d7c6):
    Status: [KEEP]
    Location: /home/user/Documents
    Title: notes.txt

    Status: [DUP]
    Location: /home/user/Documents/backup
    Title: notes.txt


Summary:
  Found 2 duplicate groups
  Total duplicate files: 5
  Files that could be removed: 3
  Potential space savings: 6291456 bytes
```

## How It Works

1. **File Collection**: Recursively scans specified paths for files
2. **Size Filtering**: Groups files by size (files with different sizes can't be duplicates)
3. **Hash Calculation**: Calculates xxHash (XXH3) only for files with matching sizes
4. **Duplicate Detection**: Groups files with identical hashes as duplicates
5. **Safe Reporting**: Shows results without making any changes to your files

## Options

- `-v, --verbose`: Show detailed progress during scanning
- `-h, --help`: Show help information
- `-V, --version`: Show version information

## Safety

This tool is designed to be completely safe:
- **Read-only**: Never modifies, moves, or deletes any files
- **No false positives**: Uses fast xxHash (XXH3) for accurate duplicate detection
- **Clear marking**: Shows which file would be kept (`[KEEP]`) vs removed (`[DUP]`)

## Performance

The tool is optimized for performance:
- **Size pre-filtering**: Avoids expensive hash calculations for files that can't be duplicates
- **Fast hashing**: Uses xxHash (XXH3) for extremely fast duplicate detection
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
