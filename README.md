# File Dedup

A minimal file deduplication tool that finds duplicate files using xxHash.

## Features

- **Two modes**: Report-only mode (safe by default) and interactive deletion mode
- **Fast detection**: Uses file size pre-filtering before expensive hash calculations
- **Recursive scanning**: Automatically scans subdirectories
- **Clear output**: Groups duplicates and shows which files could be removed with emoji-enhanced formatting
- **Interactive resolution**: Choose which duplicates to keep or delete on a per-group basis
- **Safety checks**: Confirmation prompts and prevents deleting all copies of a file
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
git clone https://github.com/your-username/file-dedup
cd file-dedup
cargo build --release

# The binary will be in target/release/file-dedup
```

## Usage

```bash
# Basic usage - scan one or more paths (report only)
file-dedup /path/to/directory

# Scan multiple paths
file-dedup ~/Documents ~/Pictures ~/Downloads

# Verbose output to see progress
file-dedup -v ~/Documents

# Interactive mode - choose which duplicates to delete
file-dedup -i ~/Documents

# Interactive mode with verbose output
file-dedup -i -v ~/Documents

# Get help
file-dedup --help
```

## Example Output

### Report Mode (Default)
```
🔍 Found duplicate files:

📁 Duplicate Group 1/1 (52 B) Hash: fc7c9033

    📄 Status: KEEP
    📍 Location: test_docs/Pictures
    🏷️ Title: photo1_copy.jpg

    📄 Status: DUP
    📍 Location: test_docs/Documents
    🏷️ Title: photo1.jpg

    📄 Status: DUP
    📍 Location: test_docs/Downloads
    🏷️ Title: photo1.jpg



🔍 Summary:
ℹ️ Found 1 duplicate groups
ℹ️ Total duplicate files: 3
⚠️ Files that could be removed: 2
✅ Potential space savings: 104 B
```

### Interactive Mode
```
🔍 Found 1 duplicate groups. Starting interactive resolution...

📁 Duplicate Group 1/1 (52 B) Hash: fc7c9033

  📄 1:
    📍 test_docs/Pictures
    🏷️ photo1_copy.jpg

  📄 2:
    📍 test_docs/Documents
    🏷️ photo1.jpg

  📄 3:
    📍 test_docs/Downloads
    🏷️ photo1.jpg


🤔 What would you like to do with this duplicate group?:
> Select files to keep (others will be deleted)
  Skip this group (no deletions)
  Keep first file, delete all others

⚠️ Files selected for deletion:
  🗑️ test_docs/Documents/photo1.jpg
  🗑️ test_docs/Downloads/photo1.jpg

⚠️ Are you sure you want to delete these files? This action cannot be undone! yes
✅ Deleted: test_docs/Documents/photo1.jpg
✅ Deleted: test_docs/Downloads/photo1.jpg

✅ Interactive deduplication complete!
ℹ️ Files deleted: 2
✅ Space saved: 104 B
```

## How It Works

1. **File Collection**: Recursively scans specified paths for files
2. **Size Filtering**: Groups files by size (files with different sizes can't be duplicates)
3. **Hash Calculation**: Calculates xxHash (XXH3) only for files with matching sizes
4. **Duplicate Detection**: Groups files with identical hashes as duplicates
5. **Safe Reporting**: Shows results without making any changes to your files

## Options

- `-v, --verbose`: Show detailed progress during scanning
- `-i, --interactive`: Enable interactive mode for duplicate resolution
- `--no-color`: Disable colored output
- `-h, --help`: Show help information
- `-V, --version`: Show version information

## Safety

This tool offers two modes with different safety levels:

### Report Mode (Default)
- **Read-only**: Never modifies, moves, or deletes any files
- **No false positives**: Uses fast xxHash (XXH3) for accurate duplicate detection
- **Clear marking**: Shows which file would be kept (`[KEEP]`) vs removed (`[DUP]`)

### Interactive Mode (`-i, --interactive`)
- **User-controlled**: Only deletes files after explicit user confirmation
- **Group-by-group**: Handles duplicates one group at a time for careful review
- **Safety checks**: 
  - Prevents deleting all copies of a file (at least one must be kept)
  - Requires explicit confirmation before any deletions
  - Shows exactly which files will be deleted before proceeding
- **Reversible decisions**: Can skip any group without making changes

## Performance

The tool is optimized for performance:
- **Size pre-filtering**: Avoids expensive hash calculations for files that can't be duplicates
- **Fast hashing**: Uses xxHash (XXH3) for extremely fast duplicate detection
- **Streaming hash calculation**: Processes large files efficiently without loading them entirely into memory
- **Skip empty files**: Ignores zero-byte files to focus on meaningful duplicates

## License

MIT License - see LICENSE file for details.
