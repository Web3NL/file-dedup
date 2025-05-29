use clap::Parser;
use colored::*;
use dialoguer::{Confirm, Select};
use file_dedup::{
    calculate_potential_savings, collect_files, collect_files_for_size_calc, find_duplicate_groups,
    DuplicateGroup, FileInfo,
};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

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

    /// Disable colored output
    #[arg(long)]
    no_color: bool,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    // Disable colored output if requested
    if args.no_color {
        colored::control::set_override(false);
    }

    if args.verbose {
        print_header("Starting file deduplication scan...");
        print_info(&format!("Scanning paths: {:?}", args.paths));
    }

    // Collect all files and group by size
    let mut files_by_size: HashMap<u64, Vec<FileInfo>> = HashMap::new();
    let mut total_files = 0;

    for path in &args.paths {
        if args.verbose {
            print_info(&format!("Scanning: {}", path.display()));
        }

        collect_files(path, &mut files_by_size, &mut total_files, args.verbose)?;
    }

    if args.verbose {
        print_success(&format!("Found {} files total", total_files));
        print_header("Checking for duplicates...");
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

fn handle_report_mode(
    duplicate_groups: Vec<DuplicateGroup>,
    paths: &[PathBuf],
) -> anyhow::Result<()> {
    print_header("Found duplicate files:\n");

    let mut total_duplicate_files = 0;

    for (group_idx, group) in duplicate_groups.iter().enumerate() {
        total_duplicate_files += group.files.len();

        print_duplicate_group_header(group_idx, duplicate_groups.len(), group.size, &group.hash);
        println!();

        for (i, file) in group.files.iter().enumerate() {
            let marker = if i == 0 {
                "KEEP".green().bold()
            } else {
                "DUP".red().bold()
            };
            let parent_dir = file
                .path
                .parent()
                .map(|p| p.display().to_string())
                .unwrap_or_else(|| "".to_string());
            let filename = file
                .path
                .file_name()
                .map(|f| f.to_string_lossy().to_string())
                .unwrap_or_else(|| file.path.display().to_string());

            println!("    {} Status: {}", "📄".blue(), marker);
            println!("    {} Location: {}", "📍".yellow(), parent_dir.dimmed());
            println!("    {} Title: {}", "🏷️".cyan(), filename.bold());
            println!();
        }
        println!();
    }

    // Summary
    println!();
    print_header("Summary:");
    print_info(&format!(
        "Found {} duplicate groups",
        duplicate_groups.len()
    ));
    print_info(&format!("Total duplicate files: {}", total_duplicate_files));
    print_warning(&format!(
        "Files that could be removed: {}",
        total_duplicate_files - duplicate_groups.len()
    ));

    // Calculate potential space savings
    let mut potential_savings = 0u64;
    for path in paths {
        match collect_files_for_size_calc(path) {
            Ok(files) => {
                potential_savings =
                    potential_savings.saturating_add(calculate_potential_savings(&files));
            }
            Err(e) => {
                eprintln!(
                    "Warning: Could not calculate savings for {}: {}",
                    path.display(),
                    e
                );
            }
        }
    }

    if potential_savings > 0 {
        print_success(&format!(
            "Potential space savings: {}",
            format_file_size(potential_savings)
        ));
    }

    Ok(())
}

fn handle_interactive_mode(duplicate_groups: Vec<DuplicateGroup>) -> anyhow::Result<()> {
    print_header(&format!(
        "Found {} duplicate groups. Starting interactive resolution...",
        duplicate_groups.len()
    ));
    println!();

    let mut total_deleted = 0;
    let mut total_space_saved = 0u64;

    for (group_idx, group) in duplicate_groups.iter().enumerate() {
        print_duplicate_group_header(group_idx, duplicate_groups.len(), group.size, &group.hash);
        println!();

        // Display all files in the group
        for (i, file) in group.files.iter().enumerate() {
            println!("  {} {}:", "📄".blue(), format!("{}", i + 1).bold().white());
            println!(
                "    {} {}",
                "📍".yellow(),
                file.path
                    .parent()
                    .map(|p| p.display().to_string())
                    .unwrap_or_else(|| "/".to_string())
                    .dimmed()
            );
            println!(
                "    {} {}",
                "🏷️".cyan(),
                file.path
                    .file_name()
                    .map(|f| f.to_string_lossy().to_string())
                    .unwrap_or_else(|| file.path.display().to_string())
                    .bold()
            );
            println!();
        }
        println!();

        // Ask user what to do with this group
        let options = vec![
            "Select files to keep (others will be deleted)",
            "Skip this group (no deletions)",
            "Keep first file, delete all others",
        ];

        let selection = Select::new()
            .with_prompt(format!(
                "{} What would you like to do with this duplicate group?",
                "🤔".bold()
            ))
            .items(&options)
            .default(0)
            .interact()?;

        match selection {
            0 => {
                // Interactive selection
                let files_to_delete = select_files_to_delete(&group.files)?;
                if !files_to_delete.is_empty() && confirm_deletion(&files_to_delete)? {
                    let deleted_count = delete_files(&files_to_delete)?;
                    total_deleted += deleted_count;
                    total_space_saved = total_space_saved
                        .saturating_add(group.size.saturating_mul(deleted_count as u64));
                }
            }
            1 => {
                // Skip this group
                print_warning(&format!("Skipping group {}", group_idx + 1));
                println!();
                continue;
            }
            2 => {
                // Keep first, delete others
                let files_to_delete: Vec<_> = group.files.iter().skip(1).collect();
                if !files_to_delete.is_empty() && confirm_deletion(&files_to_delete)? {
                    let deleted_count = delete_files(&files_to_delete)?;
                    total_deleted += deleted_count;
                    total_space_saved = total_space_saved
                        .saturating_add(group.size.saturating_mul(deleted_count as u64));
                }
            }
            _ => unreachable!(),
        }

        println!();
    }

    // Final summary
    println!();
    print_success("Interactive deduplication complete!");
    print_info(&format!("Files deleted: {}", total_deleted));
    print_success(&format!(
        "Space saved: {}",
        format_file_size(total_space_saved)
    ));

    Ok(())
}

fn select_files_to_delete(files: &[FileInfo]) -> anyhow::Result<Vec<&FileInfo>> {
    let mut files_to_delete = Vec::new();

    print_warning("Select files to DELETE (you must keep at least one file):");
    println!();

    for file in files.iter() {
        let parent_dir = file
            .path
            .parent()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| "".to_string());
        let filename = file
            .path
            .file_name()
            .map(|f| f.to_string_lossy().to_string())
            .unwrap_or_else(|| file.path.display().to_string());

        let prompt = format!(
            "{} Delete: {}/{}",
            "🗑️".red(),
            parent_dir.dimmed(),
            filename.bold()
        );

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
        print_error("You must keep at least one file from each duplicate group!");
        return Ok(Vec::new());
    }

    Ok(files_to_delete)
}

fn confirm_deletion(files_to_delete: &[&FileInfo]) -> anyhow::Result<bool> {
    if files_to_delete.is_empty() {
        return Ok(false);
    }

    println!();
    print_warning("Files selected for deletion:");
    for file in files_to_delete {
        println!(
            "  {} {}",
            "🗑️".red(),
            file.path.display().to_string().dimmed()
        );
    }
    println!();

    Confirm::new()
        .with_prompt(format!(
            "{} Are you sure you want to delete these files? This action cannot be undone!",
            "⚠️".red().bold()
        ))
        .default(false)
        .interact()
        .map_err(|e| anyhow::anyhow!("Failed to get confirmation: {}", e))
}

fn delete_files(files_to_delete: &[&FileInfo]) -> anyhow::Result<usize> {
    let mut deleted_count = 0;

    for file in files_to_delete {
        // Verify file still exists and has expected size (TOCTOU protection)
        if let Ok(metadata) = fs::metadata(&file.path) {
            if metadata.len() != file.size {
                print_error(&format!(
                    "File {} changed size, skipping deletion",
                    file.path.display()
                ));
                continue;
            }
        } else {
            print_error(&format!(
                "File {} no longer exists, skipping",
                file.path.display()
            ));
            continue;
        }

        match fs::remove_file(&file.path) {
            Ok(()) => {
                print_success(&format!("Deleted: {}", file.path.display()));
                deleted_count += 1;
            }
            Err(e) => {
                print_error(&format!("Failed to delete {}: {}", file.path.display(), e));
            }
        }
    }

    Ok(deleted_count)
}

// Pretty printing helper functions
fn format_file_size(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{} {}", bytes, UNITS[unit_index])
    } else {
        format!("{:.1} {}", size, UNITS[unit_index])
    }
}

fn print_header(text: &str) {
    println!("{} {}", "🔍".blue(), text.bold().blue());
}

fn print_success(text: &str) {
    println!("{} {}", "✅".green(), text.green());
}

fn print_warning(text: &str) {
    println!("{} {}", "⚠️".yellow(), text.yellow());
}

fn print_error(text: &str) {
    println!("{} {}", "❌".red(), text.red());
}

fn print_info(text: &str) {
    println!("{} {}", "ℹ️".cyan(), text.cyan());
}

fn print_duplicate_group_header(group_idx: usize, total_groups: usize, size: u64, hash: &str) {
    println!(
        "{} {} {} {} {}",
        "📁".bold(),
        "Duplicate Group".bold().magenta(),
        format!("{}/{}", group_idx + 1, total_groups).bold().white(),
        format!("({})", format_file_size(size)).dimmed(),
        format!("Hash: {}", hash.get(..8).unwrap_or(hash)).dimmed()
    );
}
