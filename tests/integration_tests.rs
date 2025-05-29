use file_dedup::{collect_files, find_duplicate_groups, FileInfo};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
use tempfile::TempDir;

/// Helper function to create test files in temporary directories
fn create_test_file(dir: &std::path::Path, name: &str, content: &[u8]) -> PathBuf {
    let file_path = dir.join(name);
    let mut file = File::create(&file_path).unwrap();
    file.write_all(content).unwrap();
    file_path
}

/// Create a comprehensive test directory structure for integration testing
fn create_integration_test_structure() -> TempDir {
    let temp_dir = TempDir::new().unwrap();
    let base_path = temp_dir.path();

    // Create directory structure
    fs::create_dir_all(base_path.join("documents")).unwrap();
    fs::create_dir_all(base_path.join("pictures")).unwrap();
    fs::create_dir_all(base_path.join("downloads")).unwrap();
    fs::create_dir_all(base_path.join("nested/deep/folder")).unwrap();

    // Create duplicate files across different directories
    let photo_content = b"FAKE_JPEG_DATA_FOR_TESTING_DUPLICATES_123456789";
    create_test_file(
        base_path.join("pictures").as_path(),
        "photo.jpg",
        photo_content,
    );
    create_test_file(
        base_path.join("downloads").as_path(),
        "photo_copy.jpg",
        photo_content,
    );
    create_test_file(
        base_path.join("documents").as_path(),
        "backup_photo.jpg",
        photo_content,
    );

    // Create document duplicates
    let doc_content = b"This is a test document with some content for duplicate detection testing.";
    create_test_file(
        base_path.join("documents").as_path(),
        "report.txt",
        doc_content,
    );
    create_test_file(
        base_path.join("downloads").as_path(),
        "report_v2.txt",
        doc_content,
    );

    // Create unique files
    create_test_file(
        base_path.join("documents").as_path(),
        "unique_doc.txt",
        b"Unique document content",
    );
    create_test_file(
        base_path.join("pictures").as_path(),
        "unique_image.png",
        b"UNIQUE_PNG_DATA",
    );
    create_test_file(
        base_path.join("nested/deep/folder").as_path(),
        "deep_file.txt",
        b"Deep nested file",
    );

    // Create files with same size but different content (hash collision test)
    create_test_file(
        base_path.join("documents").as_path(),
        "same_size_1.txt",
        b"AAAA",
    );
    create_test_file(
        base_path.join("downloads").as_path(),
        "same_size_2.txt",
        b"BBBB",
    );

    // Create empty files (should be ignored)
    create_test_file(base_path.join("documents").as_path(), "empty1.txt", b"");
    create_test_file(base_path.join("downloads").as_path(), "empty2.txt", b"");

    // Create large duplicate files to test performance
    let large_content =
        b"Large file content for testing performance with bigger files. ".repeat(100);
    create_test_file(
        base_path.join("documents").as_path(),
        "large1.dat",
        &large_content,
    );
    create_test_file(
        base_path.join("downloads").as_path(),
        "large2.dat",
        &large_content,
    );

    temp_dir
}

#[test]
fn test_end_to_end_duplicate_detection() {
    let test_dir = create_integration_test_structure();
    let mut files_by_size: HashMap<u64, Vec<FileInfo>> = HashMap::new();
    let mut total_files = 0;

    // Collect all files
    collect_files(test_dir.path(), &mut files_by_size, &mut total_files, false).unwrap();

    // Should find all non-empty files (ignoring empty files)
    // Files created: photo(3) + doc(2) + unique(3) + same_size(2) + large(2) = 12 files
    assert_eq!(total_files, 12);

    // Find duplicate groups
    let duplicate_groups = find_duplicate_groups(files_by_size, false).unwrap();

    // Should find 3 duplicate groups:
    // 1. Photo files (3 duplicates)
    // 2. Document files (2 duplicates)
    // 3. Large files (2 duplicates)
    assert_eq!(duplicate_groups.len(), 3);

    // Find groups by file count and validate
    let groups_by_count: Vec<_> = duplicate_groups
        .iter()
        .map(|g| (g.files.len(), g.size))
        .collect();

    // Should have one group with 3 files and two groups with 2 files each
    assert!(
        groups_by_count.iter().any(|(count, _)| *count == 3),
        "Should have one group with 3 duplicates"
    );
    assert_eq!(
        groups_by_count
            .iter()
            .filter(|(count, _)| *count == 2)
            .count(),
        2,
        "Should have two groups with 2 duplicates each"
    );
}

#[test]
fn test_cli_report_mode() {
    let test_dir = create_integration_test_structure();

    let output = Command::new("cargo")
        .args(["run", "--", test_dir.path().to_str().unwrap()])
        .output()
        .expect("Failed to run file-dedup");

    assert!(output.status.success(), "CLI should run successfully");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should report duplicate groups
    assert!(stdout.contains("Found duplicate files:"));
    assert!(stdout.contains("Duplicate Group"));
    assert!(stdout.contains("Summary:"));
    assert!(stdout.contains("Found 3 duplicate groups"));
}

#[test]
fn test_cli_verbose_mode() {
    let test_dir = create_integration_test_structure();

    let output = Command::new("cargo")
        .args(["run", "--", "-v", test_dir.path().to_str().unwrap()])
        .output()
        .expect("Failed to run file-dedup with verbose flag");

    assert!(
        output.status.success(),
        "CLI with verbose should run successfully"
    );

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Verbose mode should show scanning progress
    assert!(stdout.contains("Starting file deduplication scan"));
    assert!(stdout.contains("Scanning:"));
    assert!(stdout.contains("Found") && stdout.contains("files total"));
}

#[test]
fn test_cli_no_duplicates_found() {
    let temp_dir = TempDir::new().unwrap();

    // Create only unique files
    create_test_file(temp_dir.path(), "unique1.txt", b"Unique content 1");
    create_test_file(temp_dir.path(), "unique2.txt", b"Unique content 2");
    create_test_file(temp_dir.path(), "unique3.txt", b"Unique content 3");

    let output = Command::new("cargo")
        .args(["run", "--", temp_dir.path().to_str().unwrap()])
        .output()
        .expect("Failed to run file-dedup on unique files");

    assert!(
        output.status.success(),
        "CLI should run successfully on unique files"
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("No duplicate files found!"));
}

#[test]
fn test_symlink_security() {
    let temp_dir = TempDir::new().unwrap();
    let base_path = temp_dir.path();

    // Create a real file
    create_test_file(base_path, "real_file.txt", b"Real file content");

    // Create a symlink pointing outside the directory (if supported on this platform)
    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink;
        let _ = symlink("/etc/passwd", base_path.join("malicious_symlink"));
    }

    let mut files_by_size: HashMap<u64, Vec<FileInfo>> = HashMap::new();
    let mut total_files = 0;

    // Should not follow symlinks and only process the real file
    collect_files(base_path, &mut files_by_size, &mut total_files, false).unwrap();

    // Should only find the real file, not the symlink target
    assert_eq!(total_files, 1);
}

#[test]
fn test_error_handling_permission_denied() {
    // This test verifies graceful handling of permission denied errors
    // Note: This is a basic test - in practice, permission errors are platform-specific
    let temp_dir = TempDir::new().unwrap();

    // Create a file and remove read permissions (Unix only)
    let file_path = create_test_file(temp_dir.path(), "no_read.txt", b"test content");

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&file_path).unwrap().permissions();
        perms.set_mode(0o000); // No permissions
        fs::set_permissions(&file_path, perms).unwrap();
    }

    let mut file_info = FileInfo::new(file_path, 12);
    let result = file_info.calculate_hash();

    // Should return an error, not panic
    assert!(result.is_err());

    #[cfg(unix)]
    {
        // Restore permissions for cleanup
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&file_info.path).unwrap().permissions();
        perms.set_mode(0o644);
        let _ = fs::set_permissions(&file_info.path, perms);
    }
}

#[test]
fn test_multiple_paths_cli() {
    let temp_dir1 = TempDir::new().unwrap();
    let temp_dir2 = TempDir::new().unwrap();

    // Create duplicates across different paths
    let content = b"Cross-path duplicate content";
    create_test_file(temp_dir1.path(), "file1.txt", content);
    create_test_file(temp_dir2.path(), "file2.txt", content);

    let output = Command::new("cargo")
        .args([
            "run",
            "--",
            temp_dir1.path().to_str().unwrap(),
            temp_dir2.path().to_str().unwrap(),
        ])
        .output()
        .expect("Failed to run file-dedup with multiple paths");

    assert!(output.status.success(), "CLI should handle multiple paths");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Found 1 duplicate groups"));
}
