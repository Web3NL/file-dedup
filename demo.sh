#!/bin/bash
# Demo script for file-dedup tool

echo "File Deduplication Tool Demo"
echo "============================"
echo

# Build the tool if needed
echo "Building the tool..."
cargo build --release > /dev/null 2>&1

echo "Creating demo files with known duplicates..."

# Create demo directory
mkdir -p demo_example/{documents,backup,downloads}

# Create some content
echo "This is a common file that gets duplicated everywhere." > demo_example/documents/important.txt
echo "Meeting notes from 2025-05-29" > demo_example/documents/notes.txt
echo "Configuration file content" > demo_example/documents/config.conf

# Create duplicates
cp demo_example/documents/important.txt demo_example/backup/important_backup.txt
cp demo_example/documents/important.txt demo_example/downloads/important.txt
cp demo_example/documents/notes.txt demo_example/backup/notes_copy.txt

# Create some unique files
echo "This is unique content." > demo_example/documents/unique1.txt
echo "Another unique file." > demo_example/backup/unique2.txt

echo "Demo directory structure:"
find demo_example -type f -exec ls -lh {} \; | awk '{print $9 " (" $5 ")"}'
echo

echo "Running file-dedup tool:"
echo "$ ./target/release/file-dedup demo_example/"
echo
./target/release/file-dedup demo_example/

echo
echo "With verbose output:"
echo "$ ./target/release/file-dedup -v demo_example/"
echo
./target/release/file-dedup -v demo_example/

echo
echo "Demo completed! You can explore the demo_example/ directory."
echo "To clean up: rm -rf demo_example/"
