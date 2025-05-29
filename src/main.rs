use clap::Parser;
use file_dedup::{FileInfo, collect_files, collect_files_for_size_calc, calculate_potential_savings};
use std::collections::HashMap;
use std::path::PathBuf;

/// A minimal file deduplication tool that finds duplicate files using xxHash
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Paths to scan for duplicates (files or directories)
    #[arg(required = true)]
    paths: Vec<PathBuf>,

    /// Enable verbose output
    #[arg(short, long)]
    verbose: bool,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    if args.verbose {
        println!("Starting file deduplication scan...");
        println!("Scanning paths: {:?}", args.paths);
    }

    // Collect all files and group by size
    let mut files_by_size: HashMap<u64, Vec<FileInfo>> = HashMap::new();
    let mut total_files = 0;

    for path in &args.paths {
        if args.verbose {
            println!("Scanning: {}", path.display());
        }

        collect_files(path, &mut files_by_size, &mut total_files, args.verbose)?;
    }

    if args.verbose {
        println!("Found {} files total", total_files);
        println!("Checking for duplicates...");
    }

    // Find duplicates by comparing files with the same size
    let mut duplicates_found = false;
    let mut duplicate_groups = 0;
    let mut total_duplicate_files = 0;

    for (size, mut files) in files_by_size {
        if files.len() < 2 {
            continue; // No duplicates possible
        }

        if args.verbose {
            println!("Checking {} files of size {} bytes", files.len(), size);
        }

        // Calculate hashes for files with the same size
        let mut files_by_hash: HashMap<String, Vec<FileInfo>> = HashMap::new();

        for file in &mut files {
            match file.calculate_hash() {
                Ok(hash) => {
                    files_by_hash.entry(hash.to_string()).or_insert_with(Vec::new).push(file.clone());
                }
                Err(e) => {
                    eprintln!("Warning: Could not hash {}: {}", file.path.display(), e);
                }
            }
        }

        // Report duplicate groups
        for (hash, duplicate_files) in files_by_hash {
            if duplicate_files.len() > 1 {
                if !duplicates_found {
                    println!("Found duplicate files:\n");
                    duplicates_found = true;
                }

                duplicate_groups += 1;
                total_duplicate_files += duplicate_files.len();

                println!("Duplicate Group {} (Size: {} bytes, Hash: {}):", 
                    duplicate_groups, size, &hash[..8]);
                
                for (i, file) in duplicate_files.iter().enumerate() {
                    let marker = if i == 0 { "[KEEP]" } else { "[DUP] " };
                    println!("  {} {}", marker, file.path.display());
                }
                println!();
            }
        }
    }

    // Summary
    if duplicates_found {
        println!("Summary:");
        println!("  Found {} duplicate groups", duplicate_groups);
        println!("  Total duplicate files: {}", total_duplicate_files);
        println!("  Files that could be removed: {}", total_duplicate_files - duplicate_groups);
        
        // Calculate potential space savings
        let mut potential_savings = 0u64;
        for path in &args.paths {
            if let Ok(files) = collect_files_for_size_calc(path) {
                potential_savings += calculate_potential_savings(&files);
            }
        }
        
        if potential_savings > 0 {
            println!("  Potential space savings: {} bytes", potential_savings);
        }
    } else {
        println!("No duplicate files found!");
    }

    Ok(())
}
