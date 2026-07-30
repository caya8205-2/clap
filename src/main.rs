//! Mass Rename Tool (Rust port)
//!
//! Renames every file in a folder to a new sequential name,
//! numbered starting from 1, ordered by last-modified time
//! (oldest -> newest).
//! Safe to run multiple times on the same folder (numbers won't
//! skip or collide), because it does a two-phase rename through
//! unique UUID-based temp names.
//!
//! Usage:
//!     clap                            -> interactive mode (asks for folder & name)
//!     clap -p ./folder -n "Photo"      -> runs immediately, no prompts

use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use clap::Parser;
use uuid::Uuid;

#[derive(Parser, Debug)]
#[command(
    name = "clap",
    version,
    about = "Rename every file in a folder to 'Name 1', 'Name 2', etc, ordered by mtime."
)]
struct Args {
    /// Folder whose files should be renamed (e.g. C:\Users\Caya\Pictures\test or ./photos)
    #[arg(short = 'p', long = "path")]
    folder: Option<PathBuf>,

    /// New name prefix for every file (e.g. "Photo" -> "Photo 1.jpg", "Photo 2.jpg", ...)
    #[arg(short = 'n', long = "name")]
    new_name: Option<String>,

    /// Only print the rename plan without actually renaming anything
    #[arg(long)]
    dry_run: bool,
}

struct RenamePlan {
    old_filename: String,
    temp_filename: String,
    final_filename: String,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let folder = match args.folder {
        Some(f) => f,
        None => PathBuf::from(
            prompt("Folder path (e.g. C:\\Users\\Caya\\Pictures\\test or .): ")?
                .trim()
                .trim_matches('"'),
        ),
    };

    let new_name = match args.new_name {
        Some(n) => n,
        None => prompt("New name prefix for all files (e.g. Photo, Screenshot, neuro): ")?
            .trim()
            .trim_matches('"')
            .to_string(),
    };

    clap(&folder, &new_name, args.dry_run)
}

fn prompt(label: &str) -> Result<String> {
    print!("{label}");
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input)
}

fn clap(folder: &Path, new_name: &str, dry_run: bool) -> Result<()> {
    if !folder.is_dir() {
        bail!("Folder not found: {}", folder.display());
    }

    // Collect files (not directories), sorted by mtime oldest -> newest.
    let mut entries: Vec<(PathBuf, String, std::time::SystemTime)> = fs::read_dir(folder)
        .with_context(|| format!("Failed to read folder: {}", folder.display()))?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_file())
        .filter_map(|e| {
            let path = e.path();
            let filename = path.file_name()?.to_string_lossy().to_string();
            let mtime = e.metadata().ok()?.modified().ok()?;
            Some((path, filename, mtime))
        })
        .collect();

    if entries.is_empty() {
        println!("No files found in this folder.");
        return Ok(());
    }

    entries.sort_by_key(|(_, _, mtime)| *mtime);

    // --- Phase 1: plan unique temp names ---
    let mut plans: Vec<RenamePlan> = Vec::with_capacity(entries.len());
    for (path, filename, _) in &entries {
        let ext = path
            .extension()
            .map(|e| format!(".{}", e.to_string_lossy()))
            .unwrap_or_default();
        let temp_filename = format!("__tmp_{}{}", Uuid::new_v4().simple(), ext);
        plans.push(RenamePlan {
            old_filename: filename.clone(),
            temp_filename,
            final_filename: String::new(), // filled in during phase 2
        });
    }

    if dry_run {
        for (index, ((path, _, _), plan)) in entries.iter().zip(plans.iter_mut()).enumerate() {
            let ext = path
                .extension()
                .map(|e| format!(".{}", e.to_string_lossy()))
                .unwrap_or_default();
            plan.final_filename = format!("{} {}{}", new_name, index + 1, ext);
            println!("[dry-run] {}  ->  {}", plan.old_filename, plan.final_filename);
        }
        println!(
            "\n[dry-run] {} file(s) would be renamed (no changes made).",
            plans.len()
        );
        return Ok(());
    }

    // Actually move to temp names first, so re-running this on the same folder,
    // or having a final name collide with an existing file, stays safe.
    for ((path, _, _), plan) in entries.iter().zip(plans.iter()) {
        let temp_path = folder.join(&plan.temp_filename);
        fs::rename(path, &temp_path)
            .with_context(|| format!("Failed to rename '{}' to a temp name", plan.old_filename))?;
    }

    // --- Phase 2: rename from temp names to the final sequential names ---
    let mut renamed = 0usize;
    for (index, plan) in plans.iter_mut().enumerate() {
        let temp_path = folder.join(&plan.temp_filename);
        let ext = Path::new(&plan.old_filename)
            .extension()
            .map(|e| format!(".{}", e.to_string_lossy()))
            .unwrap_or_default();

        plan.final_filename = format!("{} {}{}", new_name, index + 1, ext);
        let final_path = folder.join(&plan.final_filename);

        fs::rename(&temp_path, &final_path)
            .with_context(|| format!("Failed to rename to final name: {}", plan.final_filename))?;

        println!("{}  ->  {}", plan.old_filename, plan.final_filename);
        renamed += 1;
    }

    println!("\nDone. {renamed} file(s) renamed successfully.");
    Ok(())
}
