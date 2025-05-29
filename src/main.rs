use clap::Parser;
use file_dedup::{FileInfo, collect_files, collect_files_for_size_calc, calculate_potential_savings, find_duplicate_groups, DuplicateGroup};
use std::collections::HashMap;
use std::path::PathBuf;
use dialoguer::{Confirm, Select};
use std::fs;

/// A minimal file deduplication tool that finds duplicate files using xxHash
#[derive(Parser)]
#[command(author, version, about = "A minimal file deduplication tool with report and interactive modes", long_about = None)]
struct Args {
    /// Paths to scan for duplicates (files or directories)
    #[arg(required = true)]
    paths: Vec<PathBuf>,

    /// Enable verbose output
    #[arg(short, long)]
    verbose: bool,

    /// Enable interactive mode for duplicate resolution
    #[arg(short, long)]
    interactive: bool,
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

    // Find duplicate groups
    let duplicate_groups = find_duplicate_groups(files_by_size, args.verbose)?;

    if duplicate_groups.is_empty() {
        println!("No duplicate files found!");
        return Ok(());
    }

    if args.interactive {
        handle_interactive_mode(duplicate_groups)?;
    } else {
        handle_report_mode(duplicate_groups, &args.paths)?;
    }

    Ok(())
}

fn handle_report_mode(duplicate_groups: Vec<DuplicateGroup>, paths: &[PathBuf]) -> anyhow::Result<()> {
    println!("Found duplicate files:\n");

    let mut total_duplicate_files = 0;

    for (group_idx, group) in duplicate_groups.iter().enumerate() {
        total_duplicate_files += group.files.len();

        println!("Duplicate Group {} (Size: {} bytes, Hash: {}):", 
            group_idx + 1, group.size, &group.hash[..8]);
        
        for (i, file) in group.files.iter().enumerate() {
            let marker = if i == 0 { "[KEEP]" } else { "[DUP] " };
            let parent_dir = file.path.parent()
                .map(|p| p.display().to_string())
                .unwrap_or_else(|| "".to_string());
            let filename = file.path.file_name()
                .map(|f| f.to_string_lossy().to_string())
                .unwrap_or_else(|| file.path.display().to_string());
            
            println!("    Status: {}", marker.trim());
            println!("    Location: {}", parent_dir);
            println!("    Title: {}", filename);
            println!();
        }
        println!();
    }

    // Summary
    println!("Summary:");
    println!("  Found {} duplicate groups", duplicate_groups.len());
    println!("  Total duplicate files: {}", total_duplicate_files);
    println!("  Files that could be removed: {}", total_duplicate_files - duplicate_groups.len());
    
    // Calculate potential space savings
    let mut potential_savings = 0u64;
    for path in paths {
        if let Ok(files) = collect_files_for_size_calc(path) {
            potential_savings += calculate_potential_savings(&files);
        }
    }
    
    if potential_savings > 0 {
        println!("  Potential space savings: {} bytes", potential_savings);
    }

    Ok(())
}

fn handle_interactive_mode(duplicate_groups: Vec<DuplicateGroup>) -> anyhow::Result<()> {
    println!("Found {} duplicate groups. Starting interactive resolution...\n", duplicate_groups.len());

    let mut total_deleted = 0;
    let mut total_space_saved = 0u64;

    for (group_idx, group) in duplicate_groups.iter().enumerate() {
        println!("Duplicate Group {} of {} (Size: {} bytes each)", 
            group_idx + 1, duplicate_groups.len(), group.size);
        println!("Hash: {}", &group.hash[..8]);
        println!();

        // Display all files in the group
        for (i, file) in group.files.iter().enumerate() {
            let parent_dir = file.path.parent()
                .map(|p| p.display().to_string())
                .unwrap_or_else(|| "".to_string());
            let filename = file.path.file_name()
                .map(|f| f.to_string_lossy().to_string())
                .unwrap_or_else(|| file.path.display().to_string());
            
            println!("  {}: {}/{}", i + 1, parent_dir, filename);
        }
        println!();

        // Ask user what to do with this group
        let options = vec![
            "Select files to keep (others will be deleted)",
            "Skip this group (no deletions)",
            "Keep first file, delete all others",
        ];

        let selection = Select::new()
            .with_prompt("What would you like to do with this duplicate group?")
            .items(&options)
            .default(0)
            .interact()?;

        match selection {
            0 => {
                // Interactive selection
                let files_to_delete = select_files_to_delete(&group.files)?;
                if !files_to_delete.is_empty() {
                    if confirm_deletion(&files_to_delete)? {
                        let deleted_count = delete_files(&files_to_delete)?;
                        total_deleted += deleted_count;
                        total_space_saved += group.size * deleted_count as u64;
                    }
                }
            }
            1 => {
                // Skip this group
                println!("Skipping group {}.\n", group_idx + 1);
                continue;
            }
            2 => {
                // Keep first, delete others
                let files_to_delete: Vec<_> = group.files.iter().skip(1).collect();
                if !files_to_delete.is_empty() {
                    if confirm_deletion(&files_to_delete)? {
                        let deleted_count = delete_files(&files_to_delete)?;
                        total_deleted += deleted_count;
                        total_space_saved += group.size * deleted_count as u64;
                    }
                }
            }
            _ => unreachable!(),
        }

        println!();
    }

    // Final summary
    println!("Interactive deduplication complete!");
    println!("  Files deleted: {}", total_deleted);
    println!("  Space saved: {} bytes", total_space_saved);

    Ok(())
}

fn select_files_to_delete(files: &[FileInfo]) -> anyhow::Result<Vec<&FileInfo>> {
    let mut files_to_delete = Vec::new();

    println!("Select files to DELETE (you must keep at least one file):");
    
    for file in files.iter() {
        let parent_dir = file.path.parent()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| "".to_string());
        let filename = file.path.file_name()
            .map(|f| f.to_string_lossy().to_string())
            .unwrap_or_else(|| file.path.display().to_string());
        
        let prompt = format!("Delete: {}/{}", parent_dir, filename);
        
        if Confirm::new()
            .with_prompt(&prompt)
            .default(false)
            .interact()? 
        {
            files_to_delete.push(file);
        }
    }

    // Ensure at least one file is kept
    if files_to_delete.len() >= files.len() {
        println!("Error: You must keep at least one file from each duplicate group!");
        return Ok(Vec::new());
    }

    Ok(files_to_delete)
}

fn confirm_deletion(files_to_delete: &[&FileInfo]) -> anyhow::Result<bool> {
    if files_to_delete.is_empty() {
        return Ok(false);
    }

    println!("\nFiles selected for deletion:");
    for file in files_to_delete {
        println!("  - {}", file.path.display());
    }

    Confirm::new()
        .with_prompt("Are you sure you want to delete these files? This action cannot be undone!")
        .default(false)
        .interact()
        .map_err(|e| anyhow::anyhow!("Failed to get confirmation: {}", e))
}

fn delete_files(files_to_delete: &[&FileInfo]) -> anyhow::Result<usize> {
    let mut deleted_count = 0;

    for file in files_to_delete {
        match fs::remove_file(&file.path) {
            Ok(()) => {
                println!("  Deleted: {}", file.path.display());
                deleted_count += 1;
            }
            Err(e) => {
                eprintln!("  Failed to delete {}: {}", file.path.display(), e);
            }
        }
    }

    Ok(deleted_count)
}
