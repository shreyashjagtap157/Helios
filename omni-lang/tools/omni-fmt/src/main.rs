//! Omni Language Formatter (omni-fmt)
//! 
//! A code formatter for the Omni programming language.
//! Enforces consistent style: indentation, spacing, and alignment.

use anyhow::{Result, Context};
use clap::Parser;
use log::{info, debug, warn};
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

mod formatter;

use formatter::{OmniFormatter, FormatConfig};

#[derive(Parser)]
#[command(name = "omni-fmt")]
#[command(about = "Omni language code formatter")]
#[command(version)]
struct Cli {
    /// Files or directories to format
    #[arg(required = true)]
    paths: Vec<PathBuf>,
    
    /// Check mode - don't write changes, exit with error if changes needed
    #[arg(short, long)]
    check: bool,
    
    /// Number of spaces for indentation (default: 4)
    #[arg(short = 's', long, default_value = "4")]
    indent_spaces: usize,
    
    /// Maximum line width (default: 100)
    #[arg(short = 'w', long, default_value = "100")]
    max_width: usize,
    
    /// Write changes to stdout instead of files
    #[arg(long)]
    stdout: bool,
    
    /// Verbose output
    #[arg(short, long)]
    verbose: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    
    // Initialize logger
    if cli.verbose {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();
    } else {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    }
    
    let config = FormatConfig {
        indent_spaces: cli.indent_spaces,
        max_line_width: cli.max_width,
        align_colons: true,
        sort_imports: true,
        blank_lines_after_imports: 1,
        blank_lines_between_functions: 2,
        trailing_newline: true,
    };
    
    let formatter = OmniFormatter::new(config);
    let mut files_changed = 0;
    let mut files_checked = 0;
    
    for path in &cli.paths {
        if path.is_file() {
            let result = process_file(&formatter, path, cli.check, cli.stdout)?;
            files_checked += 1;
            if result {
                files_changed += 1;
            }
        } else if path.is_dir() {
            for entry in WalkDir::new(path)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.path().extension().map_or(false, |ext| ext == "omni"))
            {
                let result = process_file(&formatter, entry.path(), cli.check, cli.stdout)?;
                files_checked += 1;
                if result {
                    files_changed += 1;
                }
            }
        } else {
            warn!("Path does not exist: {:?}", path);
        }
    }
    
    if cli.check {
        info!("Checked {} files, {} would be reformatted", files_checked, files_changed);
        if files_changed > 0 {
            std::process::exit(1);
        }
    } else {
        info!("Formatted {} files, {} changed", files_checked, files_changed);
    }
    
    Ok(())
}

fn process_file(formatter: &OmniFormatter, path: &Path, check: bool, to_stdout: bool) -> Result<bool> {
    debug!("Processing: {:?}", path);
    
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read {:?}", path))?;
    
    let formatted = formatter.format(&content)?;
    
    if content == formatted {
        debug!("No changes: {:?}", path);
        return Ok(false);
    }
    
    if to_stdout {
        print!("{}", formatted);
    } else if !check {
        fs::write(path, &formatted)
            .with_context(|| format!("Failed to write {:?}", path))?;
        info!("Formatted: {:?}", path);
    } else {
        info!("Would reformat: {:?}", path);
    }
    
    Ok(true)
}
