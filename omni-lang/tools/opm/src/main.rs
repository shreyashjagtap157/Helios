//! Omni Package Manager (opm)
//!
//! Manages Omni language projects: dependencies, builds, and publishing.
//! Uses `omni.toml` as the project manifest format.

use clap::{Parser, Subcommand};
use log::info;
use semver::Version;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

// ---------------------------------------------------------------------------
// Manifest types (omni.toml)
// ---------------------------------------------------------------------------
#[derive(Debug, Serialize, Deserialize)]
struct OmniManifest {
    package: PackageInfo,
    #[serde(default)]
    dependencies: toml::map::Map<String, toml::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
struct PackageInfo {
    name: String,
    version: String,
    #[serde(default)]
    authors: Vec<String>,
    #[serde(default)]
    edition: Option<String>,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    license: Option<String>,
}

// ---------------------------------------------------------------------------
// CLI definition
// ---------------------------------------------------------------------------
#[derive(Parser)]
#[command(name = "opm")]
#[command(about = "Omni Package Manager – manage Omni projects and dependencies")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Verbose output
    #[arg(short, long, global = true)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new omni.toml project file
    Init {
        /// Project name (defaults to current directory name)
        #[arg(short, long)]
        name: Option<String>,
    },
    /// Add a dependency to omni.toml
    Add {
        /// Package name
        package: String,
        /// Version requirement (e.g. "1.0", "^0.3")
        #[arg(short, long, default_value = "*")]
        version: String,
    },
    /// Remove a dependency from omni.toml
    Remove {
        /// Package name
        package: String,
    },
    /// Install all dependencies from omni.toml
    Install,
    /// Build the project (invokes omnc)
    Build {
        /// Build in release mode
        #[arg(short, long)]
        release: bool,
    },
    /// Build and run the project
    Run {
        /// Extra arguments passed to the program
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },
    /// Publish package to the Omni registry (stub)
    Publish,
    /// Search the Omni package registry (stub)
    Search {
        /// Search query
        query: String,
    },
}

// ---------------------------------------------------------------------------
// Manifest helpers
// ---------------------------------------------------------------------------
const MANIFEST_NAME: &str = "omni.toml";

fn find_manifest() -> Result<PathBuf, String> {
    let cwd = std::env::current_dir().map_err(|e| format!("Cannot get cwd: {e}"))?;
    let path = cwd.join(MANIFEST_NAME);
    if path.exists() {
        Ok(path)
    } else {
        Err(format!(
            "No {MANIFEST_NAME} found in {}. Run `opm init` first.",
            cwd.display()
        ))
    }
}

fn read_manifest(path: &Path) -> Result<OmniManifest, String> {
    let content = fs::read_to_string(path).map_err(|e| format!("Cannot read manifest: {e}"))?;
    toml::from_str(&content).map_err(|e| format!("Invalid manifest: {e}"))
}

fn write_manifest(path: &Path, manifest: &OmniManifest) -> Result<(), String> {
    let content = toml::to_string_pretty(manifest).map_err(|e| format!("Serialize error: {e}"))?;
    fs::write(path, content).map_err(|e| format!("Cannot write manifest: {e}"))
}

// ---------------------------------------------------------------------------
// Subcommand implementations
// ---------------------------------------------------------------------------
fn cmd_init(name: Option<String>) -> Result<(), String> {
    let cwd = std::env::current_dir().map_err(|e| e.to_string())?;
    let manifest_path = cwd.join(MANIFEST_NAME);
    if manifest_path.exists() {
        return Err(format!("{MANIFEST_NAME} already exists in {}", cwd.display()));
    }

    let project_name = name.unwrap_or_else(|| {
        cwd.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("my-project")
            .to_string()
    });

    let manifest = OmniManifest {
        package: PackageInfo {
            name: project_name.clone(),
            version: "0.1.0".into(),
            authors: vec![],
            edition: Some("2025".into()),
            description: None,
            license: None,
        },
        dependencies: toml::map::Map::new(),
    };

    write_manifest(&manifest_path, &manifest)?;

    // Create a default main.omni if it doesn't exist
    let main_path = cwd.join("main.omni");
    if !main_path.exists() {
        fs::write(
            &main_path,
            "// Welcome to Omni!\n\nfn main() {\n    println(\"Hello from Omni!\")\n}\n",
        )
        .map_err(|e| e.to_string())?;
    }

    info!("Created project '{}' with {}", project_name, MANIFEST_NAME);
    println!("✓ Initialized project '{project_name}'");
    Ok(())
}

fn cmd_add(package: &str, version: &str) -> Result<(), String> {
    let path = find_manifest()?;
    let mut manifest = read_manifest(&path)?;

    // Validate version string
    if version != "*" {
        let clean = version.trim_start_matches('^').trim_start_matches('~');
        Version::parse(clean).map_err(|e| format!("Invalid version '{version}': {e}"))?;
    }

    manifest
        .dependencies
        .insert(package.to_string(), toml::Value::String(version.to_string()));
    write_manifest(&path, &manifest)?;
    println!("✓ Added {package} = \"{version}\"");
    Ok(())
}

fn cmd_remove(package: &str) -> Result<(), String> {
    let path = find_manifest()?;
    let mut manifest = read_manifest(&path)?;

    if manifest.dependencies.remove(package).is_none() {
        return Err(format!("Package '{package}' not found in dependencies"));
    }
    write_manifest(&path, &manifest)?;
    println!("✓ Removed {package}");
    Ok(())
}

fn cmd_install() -> Result<(), String> {
    let path = find_manifest()?;
    let manifest = read_manifest(&path)?;

    if manifest.dependencies.is_empty() {
        println!("No dependencies to install.");
        return Ok(());
    }

    println!("Installing dependencies:");
    for (name, version) in &manifest.dependencies {
        let ver_str = match version {
            toml::Value::String(s) => s.clone(),
            other => other.to_string(),
        };
        // Stub: in the future this would download from the Omni package registry
        println!("  📦 {name} = \"{ver_str}\"  (stub – registry not yet available)");
    }
    println!("✓ Install complete (registry integration pending)");
    Ok(())
}

fn cmd_build(release: bool) -> Result<(), String> {
    let _path = find_manifest()?;
    let mode = if release { "--release" } else { "--debug" };
    println!("Building project ({mode})…");

    let status = Command::new("omnc")
        .arg("build")
        .arg(mode)
        .status()
        .map_err(|e| format!("Failed to invoke omnc: {e}"))?;

    if status.success() {
        println!("✓ Build succeeded");
        Ok(())
    } else {
        Err(format!("Build failed with exit code {:?}", status.code()))
    }
}

fn cmd_run(args: &[String]) -> Result<(), String> {
    cmd_build(false)?;

    let path = find_manifest()?;
    let manifest = read_manifest(&path)?;

    println!("Running {}…", manifest.package.name);

    let mut cmd = Command::new("omnc");
    cmd.arg("run");
    for a in args {
        cmd.arg(a);
    }
    let status = cmd.status().map_err(|e| format!("Failed to run: {e}"))?;

    if !status.success() {
        return Err(format!("Program exited with code {:?}", status.code()));
    }
    Ok(())
}

fn cmd_publish() -> Result<(), String> {
    let path = find_manifest()?;
    let manifest = read_manifest(&path)?;
    println!(
        "Publishing {} v{} …",
        manifest.package.name, manifest.package.version
    );
    println!("⚠ Registry not yet available. Publishing is a stub.");
    Ok(())
}

fn cmd_search(query: &str) -> Result<(), String> {
    println!("Searching for '{query}'…");
    println!("⚠ Registry not yet available. Search is a stub.");
    println!("  No results.");
    Ok(())
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------
#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    if cli.verbose {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();
    } else {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("warn")).init();
    }

    let result = match &cli.command {
        Commands::Init { name } => cmd_init(name.clone()),
        Commands::Add { package, version } => cmd_add(package, version),
        Commands::Remove { package } => cmd_remove(package),
        Commands::Install => cmd_install(),
        Commands::Build { release } => cmd_build(*release),
        Commands::Run { args } => cmd_run(args),
        Commands::Publish => cmd_publish(),
        Commands::Search { query } => cmd_search(query),
    };

    if let Err(e) = result {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}
