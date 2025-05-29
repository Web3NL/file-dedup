//! A minimal file deduplication library
//! 
//! This library provides functionality to find duplicate files using xxHash
//! with size-based pre-filtering for efficiency.

use xxhash_rust::xxh3::Xxh3;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Represents a file with its metadata
#[derive(Debug, Clone)]
pub struct FileInfo {
    pub path: PathBuf,
    pub size: u64,
    pub hash: Option<String>,
}

impl FileInfo {
    pub fn new(path: PathBuf, size: u64) -> Self {
        Self {
            path,
            size,
            hash: None,
        }
    }

    /// Calculate xxHash (XXH3) of the file
    pub fn calculate_hash(&mut self) -> anyhow::Result<&str> {
        if self.hash.is_some() {
            return Ok(self.hash.as_ref().unwrap());
        }

        let mut file = File::open(&self.path)?;
        let mut hasher = Xxh3::new();
        let mut buffer = [0; 8192];

        loop {
            let bytes_read = file.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }
            hasher.update(&buffer[..bytes_read]);
        }

        let hash = format!("{:016x}", hasher.digest());
        self.hash = Some(hash);
        Ok(self.hash.as_ref().unwrap())
    }
}

/// Recursively collect files and group them by size
pub fn collect_files(
    path: &Path,
    files_by_size: &mut HashMap<u64, Vec<FileInfo>>,
    total_files: &mut usize,
    verbose: bool,
) -> anyhow::Result<()> {
    if path.is_file() {
        if let Ok(metadata) = path.metadata() {
            let size = metadata.len();
            if size > 0 { // Skip empty files
                let file_info = FileInfo::new(path.to_path_buf(), size);
                files_by_size.entry(size).or_insert_with(Vec::new).push(file_info);
                *total_files += 1;
                
                if verbose {
                    println!("  Found file: {} ({} bytes)", path.display(), size);
                }
            }
        }
    } else if path.is_dir() {
        for entry in WalkDir::new(path) {
            match entry {
                Ok(entry) => {
                    if entry.file_type().is_file() {
                        if let Ok(metadata) = entry.metadata() {
                            let size = metadata.len();
                            if size > 0 { // Skip empty files
                                let file_info = FileInfo::new(entry.path().to_path_buf(), size);
                                files_by_size.entry(size).or_insert_with(Vec::new).push(file_info);
                                *total_files += 1;
                                
                                if verbose {
                                    println!("  Found file: {} ({} bytes)", entry.path().display(), size);
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Warning: Could not access {}: {}", e.path().unwrap_or(Path::new("unknown")).display(), e);
                }
            }
        }
    }

    Ok(())
}

/// Helper function to collect files for space calculation
pub fn collect_files_for_size_calc(path: &Path) -> Result<Vec<FileInfo>, Box<dyn std::error::Error>> {
    let mut files = Vec::new();
    
    if path.is_file() {
        if let Ok(metadata) = path.metadata() {
            let size = metadata.len();
            if size > 0 {
                files.push(FileInfo::new(path.to_path_buf(), size));
            }
        }
    } else if path.is_dir() {
        for entry in WalkDir::new(path) {
            if let Ok(entry) = entry {
                if entry.file_type().is_file() {
                    if let Ok(metadata) = entry.metadata() {
                        let size = metadata.len();
                        if size > 0 {
                            files.push(FileInfo::new(entry.path().to_path_buf(), size));
                        }
                    }
                }
            }
        }
    }
    
    Ok(files)
}

/// Calculate potential space savings from removing duplicates
pub fn calculate_potential_savings(files: &[FileInfo]) -> u64 {
    let mut files_by_size: HashMap<u64, Vec<&FileInfo>> = HashMap::new();
    
    for file in files {
        files_by_size.entry(file.size).or_insert_with(Vec::new).push(file);
    }
    
    let mut savings = 0u64;
    for (size, files_with_size) in files_by_size {
        if files_with_size.len() > 1 {
            // Assume we can remove all but one copy
            savings += size * (files_with_size.len() as u64 - 1);
        }
    }
    
    savings
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use std::io::Write;
    use tempfile::TempDir;

    fn create_test_file(dir: &std::path::Path, name: &str, content: &[u8]) -> PathBuf {
        let file_path = dir.join(name);
        let mut file = File::create(&file_path).unwrap();
        file.write_all(content).unwrap();
        file_path
    }

    #[test]
    fn test_file_info_hash_calculation() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = create_test_file(temp_dir.path(), "test.txt", b"Hello, World!");
        
        let mut file_info = FileInfo::new(file_path, 13);
        let hash1 = file_info.calculate_hash().unwrap().to_string();
        let hash2 = file_info.calculate_hash().unwrap().to_string();
        
        // Hash should be calculated once and cached
        assert_eq!(hash1, hash2);
        assert!(!hash1.is_empty());
        assert_eq!(hash1.len(), 16); // xxHash (XXH3) hex length (64-bit)
    }

    #[test]
    fn test_identical_files_same_hash() {
        let temp_dir = TempDir::new().unwrap();
        let content = b"This is test content for duplicate detection";
        
        let file1_path = create_test_file(temp_dir.path(), "file1.txt", content);
        let file2_path = create_test_file(temp_dir.path(), "file2.txt", content);
        
        let mut file1 = FileInfo::new(file1_path, content.len() as u64);
        let mut file2 = FileInfo::new(file2_path, content.len() as u64);
        
        let hash1 = file1.calculate_hash().unwrap();
        let hash2 = file2.calculate_hash().unwrap();
        
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_different_files_different_hash() {
        let temp_dir = TempDir::new().unwrap();
        
        let file1_path = create_test_file(temp_dir.path(), "file1.txt", b"Content A");
        let file2_path = create_test_file(temp_dir.path(), "file2.txt", b"Content B");
        
        let mut file1 = FileInfo::new(file1_path, 9);
        let mut file2 = FileInfo::new(file2_path, 9);
        
        let hash1 = file1.calculate_hash().unwrap();
        let hash2 = file2.calculate_hash().unwrap();
        
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_file_info_creation() {
        let path = PathBuf::from("/test/path/file.txt");
        let size = 1024;
        
        let file_info = FileInfo::new(path.clone(), size);
        
        assert_eq!(file_info.path, path);
        assert_eq!(file_info.size, size);
        assert!(file_info.hash.is_none());
    }

    fn create_test_directory_structure() -> TempDir {
        let temp_dir = TempDir::new().unwrap();
        let base_path = temp_dir.path();

        // Create some test files with known duplicates
        fs::create_dir_all(base_path.join("subdir1")).unwrap();
        fs::create_dir_all(base_path.join("subdir2")).unwrap();

        // Identical files (duplicates)
        let content1 = b"This is duplicate content";
        create_test_file(base_path, "original.txt", content1);
        create_test_file(&base_path.join("subdir1"), "copy1.txt", content1);
        create_test_file(&base_path.join("subdir2"), "copy2.txt", content1);

        // Unique files
        create_test_file(base_path, "unique1.txt", b"Unique content 1");
        create_test_file(&base_path.join("subdir1"), "unique2.txt", b"Unique content 2");

        // Different content, same size
        create_test_file(base_path, "same_size1.txt", b"AAAA");
        create_test_file(&base_path.join("subdir2"), "same_size2.txt", b"BBBB");

        temp_dir
    }

    #[test]
    fn test_collect_files_functionality() {
        let temp_dir = create_test_directory_structure();
        let mut files_by_size: HashMap<u64, Vec<FileInfo>> = HashMap::new();
        let mut total_files = 0;

        // Test collecting files from the test directory
        collect_files(temp_dir.path(), &mut files_by_size, &mut total_files, false).unwrap();

        // Should find all 7 files
        assert_eq!(total_files, 7);

        // Should have files grouped by size
        assert!(files_by_size.len() >= 3);

        // Check that files with size 25 (our duplicate content) are properly grouped
        let duplicate_group = files_by_size.get(&25);
        assert!(duplicate_group.is_some());
        assert_eq!(duplicate_group.unwrap().len(), 3);
    }

    #[test]
    fn test_calculate_potential_savings() {
        let files = vec![
            FileInfo::new(PathBuf::from("file1.txt"), 100),
            FileInfo::new(PathBuf::from("file2.txt"), 100), // duplicate
            FileInfo::new(PathBuf::from("file3.txt"), 200),
            FileInfo::new(PathBuf::from("file4.txt"), 200), // duplicate
            FileInfo::new(PathBuf::from("file5.txt"), 200), // duplicate
            FileInfo::new(PathBuf::from("file6.txt"), 300), // unique
        ];

        let savings = calculate_potential_savings(&files);
        // Should save: 1 copy of 100 bytes + 2 copies of 200 bytes = 500 bytes
        assert_eq!(savings, 500);
    }
}
