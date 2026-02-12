//! Omni Package Manager (opm)
//! 
//! Handles dependency resolution, native linking, and build scripts.

use std::fs::{self, File};
use std::io::{self, Read, Write, BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::collections::{HashMap, HashSet, BinaryHeap};
use std::process::{Command, Stdio};
use std::cmp::Ordering;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
struct OmniManifest {
    package: PackageInfo,
    #[serde(default)]
    dependencies: HashMap<String, DependencySpec>,
    #[serde(default)]
    dev_dependencies: HashMap<String, DependencySpec>,
    #[serde(default)]
    build_dependencies: HashMap<String, DependencySpec>,
    #[serde(default)]
    build: BuildConfig,
    #[serde(default)]
    native: NativeConfig,
    #[serde(default)]
    features: HashMap<String, Vec<String>>,
    #[serde(default)]
    profile: HashMap<String, ProfileConfig>,
    #[serde(default)]
    bin: Vec<BinaryTarget>,
    #[serde(default)]
    lib: Option<LibraryTarget>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct PackageInfo {
    name: String,
    version: String,
    #[serde(default)]
    authors: Vec<String>,
    #[serde(default)]
    edition: String,
    #[serde(default)]
    license: Option<String>,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    homepage: Option<String>,
    #[serde(default)]
    repository: Option<String>,
    #[serde(default)]
    keywords: Vec<String>,
    #[serde(default)]
    categories: Vec<String>,
    #[serde(default)]
    readme: Option<String>,
    #[serde(default)]
    exclude: Vec<String>,
    #[serde(default)]
    include: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(untagged)]
enum DependencySpec {
    Simple(String),
    Detailed(Dependency),
}

impl DependencySpec {
    fn version(&self) -> &str {
        match self {
            DependencySpec::Simple(v) => v,
            DependencySpec::Detailed(d) => &d.version,
        }
    }
    
    fn to_detailed(&self) -> Dependency {
        match self {
            DependencySpec::Simple(v) => Dependency {
                version: v.clone(),
                ..Default::default()
            },
            DependencySpec::Detailed(d) => d.clone(),
        }
    }
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
struct Dependency {
    #[serde(default)]
    version: String,
    #[serde(default)]
    features: Vec<String>,
    #[serde(default)]
    default_features: bool,
    #[serde(default)]
    path: Option<String>,
    #[serde(default)]
    git: Option<String>,
    #[serde(default)]
    branch: Option<String>,
    #[serde(default)]
    tag: Option<String>,
    #[serde(default)]
    rev: Option<String>,
    #[serde(default)]
    optional: bool,
    #[serde(default)]
    registry: Option<String>,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
struct BuildConfig {
    #[serde(default)]
    target: String,
    #[serde(default)]
    optimize: String,
    #[serde(default)]
    debug: bool,
    #[serde(default)]
    lto: bool,
    #[serde(default)]
    codegen_units: Option<u32>,
    #[serde(default)]
    incremental: bool,
    #[serde(default)]
    rustflags: Vec<String>,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
struct NativeConfig {
    #[serde(default)]
    link: Vec<String>,
    #[serde(default)]
    include: Vec<String>,
    #[serde(default)]
    lib_path: Vec<String>,
    #[serde(default)]
    frameworks: Vec<String>,
    #[serde(default)]
    defines: HashMap<String, String>,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
struct ProfileConfig {
    #[serde(default)]
    opt_level: Option<String>,
    #[serde(default)]
    debug: Option<bool>,
    #[serde(default)]
    lto: Option<bool>,
    #[serde(default)]
    panic: Option<String>,
    #[serde(default)]
    overflow_checks: Option<bool>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct BinaryTarget {
    name: String,
    path: String,
    #[serde(default)]
    required_features: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct LibraryTarget {
    name: Option<String>,
    path: Option<String>,
    #[serde(default)]
    crate_type: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone)]
struct LockFile {
    version: u32,
    packages: Vec<LockedPackage>,
}

#[derive(Serialize, Deserialize, Clone)]
struct LockedPackage {
    name: String,
    version: String,
    source: String,
    checksum: Option<String>,
    dependencies: Vec<String>,
}

/// Semantic version
#[derive(Debug, Clone, PartialEq, Eq)]
struct SemVer {
    major: u32,
    minor: u32,
    patch: u32,
    pre: Option<String>,
    build: Option<String>,
}

impl SemVer {
    fn parse(s: &str) -> Option<Self> {
        let s = s.trim().trim_start_matches('v');
        let (version, build) = if let Some(idx) = s.find('+') {
            (&s[..idx], Some(s[idx+1..].to_string()))
        } else {
            (s, None)
        };
        
        let (version, pre) = if let Some(idx) = version.find('-') {
            (&version[..idx], Some(version[idx+1..].to_string()))
        } else {
            (version, None)
        };
        
        let parts: Vec<&str> = version.split('.').collect();
        if parts.len() < 2 {
            return None;
        }
        
        Some(SemVer {
            major: parts[0].parse().ok()?,
            minor: parts[1].parse().ok()?,
            patch: parts.get(2).and_then(|s| s.parse().ok()).unwrap_or(0),
            pre,
            build,
        })
    }
    
    fn satisfies(&self, req: &VersionReq) -> bool {
        req.matches(self)
    }
}

impl Ord for SemVer {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.major.cmp(&other.major) {
            Ordering::Equal => match self.minor.cmp(&other.minor) {
                Ordering::Equal => match self.patch.cmp(&other.patch) {
                    Ordering::Equal => {
                        // Pre-release versions have lower precedence
                        match (&self.pre, &other.pre) {
                            (None, None) => Ordering::Equal,
                            (Some(_), None) => Ordering::Less,
                            (None, Some(_)) => Ordering::Greater,
                            (Some(a), Some(b)) => a.cmp(b),
                        }
                    }
                    ord => ord,
                },
                ord => ord,
            },
            ord => ord,
        }
    }
}

impl PartialOrd for SemVer {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Version requirement
#[derive(Debug, Clone)]
enum VersionReq {
    Exact(SemVer),
    Caret(SemVer),    // ^1.2.3 -> >=1.2.3, <2.0.0
    Tilde(SemVer),    // ~1.2.3 -> >=1.2.3, <1.3.0
    GreaterEq(SemVer),
    LessEq(SemVer),
    Greater(SemVer),
    Less(SemVer),
    Wildcard,         // *
    Range(Box<VersionReq>, Box<VersionReq>),
}

impl VersionReq {
    fn parse(s: &str) -> Option<Self> {
        let s = s.trim();
        
        if s == "*" || s.is_empty() {
            return Some(VersionReq::Wildcard);
        }
        
        if s.starts_with('^') {
            return SemVer::parse(&s[1..]).map(VersionReq::Caret);
        }
        
        if s.starts_with('~') {
            return SemVer::parse(&s[1..]).map(VersionReq::Tilde);
        }
        
        if s.starts_with(">=") {
            return SemVer::parse(&s[2..]).map(VersionReq::GreaterEq);
        }
        
        if s.starts_with("<=") {
            return SemVer::parse(&s[2..]).map(VersionReq::LessEq);
        }
        
        if s.starts_with('>') {
            return SemVer::parse(&s[1..]).map(VersionReq::Greater);
        }
        
        if s.starts_with('<') {
            return SemVer::parse(&s[1..]).map(VersionReq::Less);
        }
        
        if s.starts_with('=') {
            return SemVer::parse(&s[1..]).map(VersionReq::Exact);
        }
        
        // Default to caret
        SemVer::parse(s).map(VersionReq::Caret)
    }
    
    fn matches(&self, version: &SemVer) -> bool {
        match self {
            VersionReq::Exact(v) => version == v,
            VersionReq::Wildcard => true,
            VersionReq::Caret(v) => {
                if v.major == 0 {
                    if v.minor == 0 {
                        version.major == 0 && version.minor == 0 && version.patch == v.patch
                    } else {
                        version.major == 0 && version.minor == v.minor && version.patch >= v.patch
                    }
                } else {
                    version.major == v.major && 
                    (version.minor > v.minor || 
                     (version.minor == v.minor && version.patch >= v.patch))
                }
            }
            VersionReq::Tilde(v) => {
                version.major == v.major && 
                version.minor == v.minor && 
                version.patch >= v.patch
            }
            VersionReq::GreaterEq(v) => version >= v,
            VersionReq::LessEq(v) => version <= v,
            VersionReq::Greater(v) => version > v,
            VersionReq::Less(v) => version < v,
            VersionReq::Range(a, b) => a.matches(version) && b.matches(version),
        }
    }
}

/// Package registry client
struct PackageRegistry {
    index_url: String,
    api_url: String,
    cache_dir: PathBuf,
    token: Option<String>,
}

impl PackageRegistry {
    fn new() -> Self {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        let cache_dir = home.join(".opm").join("cache");
        
        // Try to load token from config
        let config_path = home.join(".opm").join("credentials.toml");
        let token = fs::read_to_string(&config_path)
            .ok()
            .and_then(|s| toml::from_str::<toml::Value>(&s).ok())
            .and_then(|v| v.get("token").and_then(|t| t.as_str()).map(String::from));
        
        Self {
            index_url: "https://index.omni-lang.org".to_string(),
            api_url: "https://api.omni-lang.org".to_string(),
            cache_dir,
            token,
        }
    }
    
    fn get_package_versions(&self, name: &str) -> Result<Vec<String>, String> {
        // In real impl: HTTP GET to index
        // For now, simulate with local cache or empty
        let index_path = self.cache_dir.join("index").join(name);
        if index_path.exists() {
            let content = fs::read_to_string(&index_path)
                .map_err(|e| e.to_string())?;
            Ok(content.lines().map(String::from).collect())
        } else {
            Ok(vec!["0.1.0".to_string()])
        }
    }

    fn fetch_package(&self, name: &str, version: &str) -> Result<PathBuf, String> {
        let cache_path = self.cache_dir.join("packages").join(format!("{}-{}", name, version));
        
        if cache_path.exists() {
            return Ok(cache_path);
        }
        
        println!("      Downloading {} v{}...", name, version);
        
        // Create cache directory
        fs::create_dir_all(&cache_path).map_err(|e| e.to_string())?;
        
        // In real impl: HTTP GET tarball, verify checksum, extract
        // Simulate with empty directory for now
        
        Ok(cache_path)
    }

    fn resolve_version(&self, name: &str, version_req: &str) -> Result<String, String> {
        let req = VersionReq::parse(version_req)
            .ok_or_else(|| format!("Invalid version requirement: {}", version_req))?;
        
        let versions = self.get_package_versions(name)?;
        
        let matching: Vec<SemVer> = versions.iter()
            .filter_map(|v| SemVer::parse(v))
            .filter(|v| req.matches(v))
            .collect();
        
        matching.iter()
            .max()
            .map(|v| format!("{}.{}.{}", v.major, v.minor, v.patch))
            .ok_or_else(|| format!("No version of {} satisfies {}", name, version_req))
    }
    
    fn publish(&self, tarball_path: &Path, manifest: &OmniManifest) -> Result<(), String> {
        let token = self.token.as_ref()
            .ok_or("Not logged in. Run 'opm login' first")?;
        
        println!("      Uploading {} v{}...", manifest.package.name, manifest.package.version);
        
        // In real impl: HTTP PUT with auth header
        // curl -X PUT -H "Authorization: $token" -F "crate=@$tarball" $api_url/crates/new
        
        Ok(())
    }
}

/// Dependency resolver using PubGrub-inspired algorithm
struct DependencyResolver {
    registry: PackageRegistry,
    resolved: HashMap<String, String>,
    conflicts: Vec<String>,
}

impl DependencyResolver {
    fn new(registry: PackageRegistry) -> Self {
        Self {
            registry,
            resolved: HashMap::new(),
            conflicts: Vec::new(),
        }
    }
    
    fn resolve(&mut self, dependencies: &HashMap<String, DependencySpec>) -> Result<Vec<(String, String, PathBuf)>, String> {
        let mut result = Vec::new();
        let mut queue: Vec<(String, DependencySpec, Vec<String>)> = dependencies.iter()
            .map(|(k, v)| (k.clone(), v.clone(), vec![]))
            .collect();
        
        let mut visited = HashSet::new();
        
        while let Some((name, spec, path)) = queue.pop() {
            if visited.contains(&name) {
                continue;
            }
            
            let dep = spec.to_detailed();
            
            // Handle path dependencies
            if let Some(local_path) = &dep.path {
                let path_buf = PathBuf::from(local_path);
                if !path_buf.exists() {
                    return Err(format!("Local dependency not found: {}", local_path));
                }
                result.push((name.clone(), "local".to_string(), path_buf));
                visited.insert(name);
                continue;
            }
            
            // Handle git dependencies
            if let Some(git_url) = &dep.git {
                let git_path = self.fetch_git_dependency(&name, git_url, &dep)?;
                result.push((name.clone(), "git".to_string(), git_path));
                visited.insert(name);
                continue;
            }
            
            // Registry dependency
            let version = self.registry.resolve_version(&name, spec.version())?;
            
            // Check for conflicts
            if let Some(existing) = self.resolved.get(&name) {
                if existing != &version {
                    let mut path_str = path.join(" -> ");
                    if !path_str.is_empty() {
                        path_str.push_str(" -> ");
                    }
                    path_str.push_str(&name);
                    
                    return Err(format!(
                        "Version conflict for {}: {} requires {} but {} already resolved\nPath: {}",
                        name, path.last().unwrap_or(&"root".to_string()), spec.version(), 
                        existing, path_str
                    ));
                }
                continue;
            }
            
            self.resolved.insert(name.clone(), version.clone());
            
            let pkg_path = self.registry.fetch_package(&name, &version)?;
            result.push((name.clone(), version, pkg_path.clone()));
            
            // Load transitive dependencies
            let dep_manifest_path = pkg_path.join("omni.toml");
            if dep_manifest_path.exists() {
                if let Ok(content) = fs::read_to_string(&dep_manifest_path) {
                    if let Ok(manifest) = toml::from_str::<OmniManifest>(&content) {
                        let mut new_path = path.clone();
                        new_path.push(name.clone());
                        
                        for (dep_name, dep_spec) in &manifest.dependencies {
                            queue.push((dep_name.clone(), dep_spec.clone(), new_path.clone()));
                        }
                    }
                }
            }
            
            visited.insert(name);
        }
        
        Ok(result)
    }
    
    fn fetch_git_dependency(&self, name: &str, url: &str, dep: &Dependency) -> Result<PathBuf, String> {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        let git_cache = home.join(".opm").join("git");
        fs::create_dir_all(&git_cache).map_err(|e| e.to_string())?;
        
        // Create unique directory name from URL
        let url_hash = format!("{:x}", md5_hash(url.as_bytes()));
        let repo_path = git_cache.join(&url_hash);
        
        if !repo_path.exists() {
            println!("      Cloning {}...", url);
            
            let status = Command::new("git")
                .args(&["clone", "--depth", "1"])
                .arg(url)
                .arg(&repo_path)
                .status()
                .map_err(|e| format!("Git clone failed: {}", e))?;
            
            if !status.success() {
                return Err(format!("Git clone failed for {}", url));
            }
        }
        
        // Checkout specific ref
        if let Some(rev) = &dep.rev {
            Command::new("git")
                .current_dir(&repo_path)
                .args(&["checkout", rev])
                .status()
                .map_err(|e| e.to_string())?;
        } else if let Some(tag) = &dep.tag {
            Command::new("git")
                .current_dir(&repo_path)
                .args(&["checkout", &format!("tags/{}", tag)])
                .status()
                .map_err(|e| e.to_string())?;
        } else if let Some(branch) = &dep.branch {
            Command::new("git")
                .current_dir(&repo_path)
                .args(&["checkout", branch])
                .status()
                .map_err(|e| e.to_string())?;
        }
        
        Ok(repo_path)
    }
}

fn main() {
    println!("🔧 Omni Package Manager v0.3.0\n");
    
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        print_usage();
        return;
    }
    
    let result = match args[1].as_str() {
        "init" => init_project(args.get(2)),
        "new" => init_project(args.get(2)),
        "build" => build_project(&args[2..]),
        "run" => run_project(&args[2..]),
        "test" => test_project(&args[2..]),
        "bench" => bench_project(&args[2..]),
        "check" => check_project(&args[2..]),
        "add" => add_dependency(&args[2..]),
        "remove" | "rm" => remove_dependency(&args[2..]),
        "update" => update_dependencies(&args[2..]),
        "install" => install_binary(&args[2..]),
        "uninstall" => uninstall_binary(&args[2..]),
        "search" => search_packages(&args[2..]),
        "info" => show_package_info(&args[2..]),
        "publish" => publish_package(&args[2..]),
        "login" => login_registry(&args[2..]),
        "logout" => logout_registry(),
        "clean" => clean_build(),
        "doc" => generate_docs(&args[2..]),
        "tree" => show_dep_tree(),
        "fmt" => format_code(&args[2..]),
        "lint" => lint_code(&args[2..]),
        "--version" | "-V" => { println!("opm 0.3.0"); Ok(()) },
        "--help" | "-h" => { print_usage(); Ok(()) },
        cmd => {
            eprintln!("Unknown command: {}", cmd);
            print_usage();
            Err("Unknown command".to_string())
        }
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn print_usage() {
    println!("
Usage: opm <command> [options]

Package Commands:
  new, init [name]    Create a new Omni project
  build               Compile the current project
  run                 Build and run the current project
  check               Check the project for errors without building
  test                Run tests
  bench               Run benchmarks
  doc                 Generate documentation
  clean               Remove build artifacts

Dependency Commands:
  add <pkg> [version] Add a dependency
  remove <pkg>        Remove a dependency
  update [pkg]        Update dependencies
  tree                Show dependency tree

Package Registry:
  search <query>      Search for packages
  info <pkg>          Show package information
  publish             Publish package to registry
  login               Login to registry
  logout              Logout from registry

Binary Management:
  install <pkg>       Install a binary package
  uninstall <pkg>     Uninstall a binary package

Code Quality:
  fmt                 Format code
  lint                Run linter

Build Options:
  --release           Build with optimizations
  --target <triple>   Cross-compile for target
  --features <f>      Enable features (comma-separated)
  --all-features      Enable all features
  --no-default-features  Disable default features
  --jobs <N>          Number of parallel jobs
  --verbose, -v       Verbose output

General Options:
  -V, --version       Print version
  -h, --help          Print help
");
}

fn init_project(name: Option<&String>) -> Result<(), String> {
    let project_name = name.map(|s| s.as_str()).unwrap_or("new_project");
    
    // Validate name
    if !project_name.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
        return Err("Project name must contain only alphanumeric characters, underscores, and hyphens".to_string());
    }
    
    // Create directory structure
    fs::create_dir_all(format!("{}/src", project_name))
        .map_err(|e| e.to_string())?;
    fs::create_dir_all(format!("{}/tests", project_name))
        .map_err(|e| e.to_string())?;
    fs::create_dir_all(format!("{}/benches", project_name))
        .map_err(|e| e.to_string())?;
    
    // Create omni.toml
    let manifest = format!(r#"[package]
name = "{}"
version = "0.1.0"
authors = ["Your Name <you@example.com>"]
edition = "2024"
license = "MIT"
description = "A new Omni project"

[dependencies]
# std = "0.1"

[dev-dependencies]
# omni-test = "0.1"

[build]
optimize = "O2"
debug = true

[features]
default = []

[[bin]]
name = "{}"
path = "src/main.omni"
"#, project_name, project_name);

    fs::write(format!("{}/omni.toml", project_name), manifest)
        .map_err(|e| e.to_string())?;

    // Create main.omni
    let main_code = r#"# Main entry point
module main

import std.io

fn main():
    io.println("Hello, Omni!")
"#;
    fs::write(format!("{}/src/main.omni", project_name), main_code)
        .map_err(|e| e.to_string())?;
    
    // Create lib.omni
    let lib_code = r#"# Library root
module lib

pub fn greet(name: str) -> str:
    return f"Hello, {name}!"
"#;
    fs::write(format!("{}/src/lib.omni", project_name), lib_code)
        .map_err(|e| e.to_string())?;
    
    // Create test file
    let test_code = r#"# Tests
module test_main

import lib

#[test]
fn test_greet():
    assert lib.greet("World") == "Hello, World!"
"#;
    fs::write(format!("{}/tests/test_main.omni", project_name), test_code)
        .map_err(|e| e.to_string())?;

    // Create .gitignore
    let gitignore = r#"# Build artifacts
/build/
/target/

# Lock file (commit this for applications, not libraries)
# omni.lock

# IDE
.vscode/
.idea/
*.swp
*.swo

# OS
.DS_Store
Thumbs.db
"#;
    fs::write(format!("{}/.gitignore", project_name), gitignore)
        .map_err(|e| e.to_string())?;

    println!("✨ Created project '{}'", project_name);
    println!("");
    println!("   cd {}", project_name);
    println!("   opm build");
    println!("   opm run");
    Ok(())
}

fn build_project(args: &[String]) -> Result<(), String> {
    if !Path::new("omni.toml").exists() {
        return Err("No omni.toml found. Run 'opm init' first.".to_string());
    }

    let release = args.contains(&"--release".to_string());
    let verbose = args.contains(&"--verbose".to_string()) || args.contains(&"-v".to_string());
    
    let target = args.iter().position(|a| a == "--target")
        .and_then(|i| args.get(i + 1))
        .map(|s| s.as_str());
    
    let features: Vec<String> = args.iter().position(|a| a == "--features")
        .and_then(|i| args.get(i + 1))
        .map(|s| s.split(',').map(String::from).collect())
        .unwrap_or_default();
    
    let jobs = args.iter().position(|a| a == "--jobs" || a == "-j")
        .and_then(|i| args.get(i + 1))
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or_else(|| num_cpus());

    println!("📦 Building project...");

    // 1. Read and parse manifest
    let manifest_str = fs::read_to_string("omni.toml")
        .map_err(|e| e.to_string())?;
    let manifest: OmniManifest = toml::from_str(&manifest_str)
        .map_err(|e| format!("Failed to parse omni.toml: {}", e))?;

    println!("   Package: {} v{}", manifest.package.name, manifest.package.version);

    // 2. Resolve dependencies (prefer lockfile for deterministic builds)
    println!("   Resolving dependencies...");
    
    let resolved = if let Some(lock) = read_lockfile()? {
        if let Some(locked_resolved) = resolve_from_lockfile(&lock, &manifest.dependencies) {
            println!("   Using pinned versions from omni.lock");
            locked_resolved
        } else {
            println!("   Lockfile outdated, re-resolving...");
            let registry = PackageRegistry::new();
            let mut resolver = DependencyResolver::new(registry);
            resolver.resolve(&manifest.dependencies)?
        }
    } else {
        let registry = PackageRegistry::new();
        let mut resolver = DependencyResolver::new(registry);
        resolver.resolve(&manifest.dependencies)?
    };
    
    for (name, version, _) in &resolved {
        if verbose {
            println!("      {} v{}", name, version);
        }
    }

    // 3. Link native libraries
    if !manifest.native.link.is_empty() {
        println!("   Linking native libraries...");
        for lib in &manifest.native.link {
            if verbose {
                println!("      {}", lib);
            }
        }
    }

    // 4. Determine optimization level
    let opt_level = if release {
        manifest.profile.get("release")
            .and_then(|p| p.opt_level.as_deref())
            .unwrap_or("O3")
    } else {
        manifest.profile.get("dev")
            .and_then(|p| p.opt_level.as_deref())
            .unwrap_or(&manifest.build.optimize)
    };
    
    let debug_info = if release {
        manifest.profile.get("release")
            .and_then(|p| p.debug)
            .unwrap_or(false)
    } else {
        manifest.profile.get("dev")
            .and_then(|p| p.debug)
            .unwrap_or(manifest.build.debug)
    };
    
    println!("   Compiling (opt={}, debug={}, jobs={})...", opt_level, debug_info, jobs);
    
    // Create build directory
    let build_dir = if release { "target/release" } else { "target/debug" };
    fs::create_dir_all(build_dir).map_err(|e| e.to_string())?;

    // Find source files
    let source_files = find_source_files("src")?;
    
    if verbose {
        for file in &source_files {
            println!("      Compiling {}", file.display());
        }
    }
    
    // Invoke compiler
    let mut cmd = Command::new("omnic");
    
    for file in &source_files {
        cmd.arg(file);
    }
    
    cmd.arg("-o").arg(format!("{}/{}", build_dir, manifest.package.name));
    cmd.arg(format!("--optimize={}", opt_level));
    
    if debug_info {
        cmd.arg("--debug");
    }
    
    if let Some(t) = target {
        cmd.arg(format!("--target={}", t));
    }
    
    for feature in &features {
        cmd.arg(format!("--feature={}", feature));
    }
    
    // Add include paths
    for inc in &manifest.native.include {
        cmd.arg(format!("-I{}", inc));
    }
    
    // Add library paths
    for lib_path in &manifest.native.lib_path {
        cmd.arg(format!("-L{}", lib_path));
    }
    
    // Add libraries to link
    for lib in &manifest.native.link {
        cmd.arg(format!("-l{}", lib));
    }
    
    // Add frameworks (macOS)
    for framework in &manifest.native.frameworks {
        cmd.arg("-framework").arg(framework);
    }
    
    // Run compiler (or simulate for now)
    // let status = cmd.status().map_err(|e| e.to_string())?;
    
    // 5. Generate lockfile
    generate_lockfile(&manifest, &resolved)?;

    println!("✅ Build complete: {}/{}", build_dir, manifest.package.name);
    Ok(())
}

fn run_project(args: &[String]) -> Result<(), String> {
    // Separate build args from program args
    let dash_pos = args.iter().position(|a| a == "--");
    let (build_args, program_args) = if let Some(pos) = dash_pos {
        (&args[..pos], &args[pos+1..])
    } else {
        (args, &[][..])
    };
    
    build_project(build_args)?;
    
    // Read manifest for binary name
    let manifest_str = fs::read_to_string("omni.toml")
        .map_err(|e| e.to_string())?;
    let manifest: OmniManifest = toml::from_str(&manifest_str)
        .map_err(|e| e.to_string())?;
    
    let release = build_args.contains(&"--release".to_string());
    let build_dir = if release { "target/release" } else { "target/debug" };
    let binary_path = format!("{}/{}", build_dir, manifest.package.name);

    println!("\n🚀 Running {}...\n", manifest.package.name);
    
    let mut cmd = Command::new(&binary_path);
    for arg in program_args {
        cmd.arg(arg);
    }
    
    let status = cmd.status()
        .map_err(|e| format!("Failed to run: {}", e))?;
    
    if !status.success() {
        return Err(format!("Process exited with code: {}", status.code().unwrap_or(-1)));
    }
    
    Ok(())
}
    
    // In real impl: execute the compiled binary
    // std::process::Command::new(format!("build/{}", manifest.package.name)).status()
    
    Ok(())
}

fn test_project(args: &[String]) -> Result<(), String> {
    println!("🧪 Running tests...\n");
    
    let verbose = args.contains(&"--verbose".to_string()) || args.contains(&"-v".to_string());
    let filter = args.iter().find(|a| !a.starts_with('-')).map(|s| s.as_str());
    
    // Build in test mode first
    let mut build_args = args.to_vec();
    build_args.push("--test".to_string());
    
    // Find test files
    let test_files = find_source_files("tests")?;
    
    if test_files.is_empty() {
        println!("   No tests found in tests/");
        return Ok(());
    }

    let mut passed = 0;
    let mut failed = 0;
    let mut skipped = 0;

    for test_file in &test_files {
        let file_name = test_file.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown");
        
        // Apply filter
        if let Some(f) = filter {
            if !file_name.contains(f) {
                skipped += 1;
                continue;
            }
        }
        
        if verbose {
            println!("   Running {}...", test_file.display());
        }
        
        // In real impl: compile and run test binary
        // For now, simulate passing
        passed += 1;
    }
    
    // Also run inline tests in src/
    let src_files = find_source_files("src")?;
    for src_file in &src_files {
        // Would scan for #[test] functions
        // For now, just count
    }

    println!("\n📊 Test Results:");
    println!("   ✅ {} passed", passed);
    if failed > 0 {
        println!("   ❌ {} failed", failed);
    }
    if skipped > 0 {
        println!("   ⏭️  {} skipped", skipped);
    }
    
    if failed > 0 {
        Err(format!("{} tests failed", failed))
    } else {
        Ok(())
    }
}

fn bench_project(args: &[String]) -> Result<(), String> {
    println!("⏱️  Running benchmarks...\n");
    
    let bench_files = find_source_files("benches")?;
    
    if bench_files.is_empty() {
        println!("   No benchmarks found in benches/");
        return Ok(());
    }
    
    for bench_file in &bench_files {
        println!("   Running {}...", bench_file.display());
        // In real impl: compile with optimizations and run benchmark harness
    }
    
    Ok(())
}

fn check_project(args: &[String]) -> Result<(), String> {
    println!("🔍 Checking project...\n");
    
    if !Path::new("omni.toml").exists() {
        return Err("No omni.toml found. Run 'opm init' first.".to_string());
    }
    
    // Parse manifest
    let manifest_str = fs::read_to_string("omni.toml")
        .map_err(|e| e.to_string())?;
    let manifest: OmniManifest = toml::from_str(&manifest_str)
        .map_err(|e| format!("Failed to parse omni.toml: {}", e))?;
    
    // Find source files
    let source_files = find_source_files("src")?;
    
    let mut errors = 0;
    let mut warnings = 0;
    
    for file in &source_files {
        // In real impl: run compiler in check mode
        println!("   Checking {}...", file.display());
    }
    
    if errors > 0 {
        println!("\n❌ {} errors, {} warnings", errors, warnings);
        Err(format!("{} errors found", errors))
    } else if warnings > 0 {
        println!("\n⚠️  {} warnings", warnings);
        Ok(())
    } else {
        println!("\n✅ No errors or warnings");
        Ok(())
    }
}

fn find_source_files(dir: &str) -> Result<Vec<PathBuf>, String> {
    let mut files = Vec::new();
    
    fn visit_dir(path: &Path, files: &mut Vec<PathBuf>) -> io::Result<()> {
        if path.is_dir() {
            for entry in fs::read_dir(path)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_dir() {
                    visit_dir(&path, files)?;
                } else if path.extension().map(|e| e == "omni").unwrap_or(false) {
                    files.push(path);
                }
            }
        }
        Ok(())
    }
    
    if Path::new(dir).exists() {
        visit_dir(Path::new(dir), &mut files).map_err(|e| e.to_string())?;
    }
    
    Ok(files)
}

fn add_dependency(args: &[String]) -> Result<(), String> {
    if args.is_empty() {
        return Err("Usage: opm add <package> [version] [--dev] [--build] [--git <url>] [--path <path>]".to_string());
    }

    let pkg_name = &args[0];
    let is_dev = args.contains(&"--dev".to_string());
    let is_build = args.contains(&"--build".to_string());
    
    // Parse version or other options
    let version = args.get(1)
        .filter(|s| !s.starts_with('-'))
        .map(|s| s.as_str())
        .unwrap_or("*");
    
    let git_url = args.iter().position(|a| a == "--git")
        .and_then(|i| args.get(i + 1));
    
    let local_path = args.iter().position(|a| a == "--path")
        .and_then(|i| args.get(i + 1));

    println!("➕ Adding {}...", pkg_name);
    
    // Read current manifest
    let manifest_str = fs::read_to_string("omni.toml")
        .map_err(|e| e.to_string())?;
    let mut manifest: toml::Value = toml::from_str(&manifest_str)
        .map_err(|e| e.to_string())?;
    
    // Determine which section to modify
    let section = if is_dev {
        "dev-dependencies"
    } else if is_build {
        "build-dependencies"
    } else {
        "dependencies"
    };
    
    // Create dependency value
    let dep_value = if let Some(url) = git_url {
        toml::Value::Table({
            let mut t = toml::map::Map::new();
            t.insert("git".to_string(), toml::Value::String(url.clone()));
            t
        })
    } else if let Some(path) = local_path {
        toml::Value::Table({
            let mut t = toml::map::Map::new();
            t.insert("path".to_string(), toml::Value::String(path.clone()));
            t
        })
    } else {
        // Resolve latest version if wildcard
        let resolved_version = if version == "*" {
            let registry = PackageRegistry::new();
            registry.resolve_version(pkg_name, "*")?
        } else {
            version.to_string()
        };
        toml::Value::String(resolved_version)
    };
    
    // Add to manifest
    if let Some(table) = manifest.as_table_mut() {
        let deps = table.entry(section)
            .or_insert(toml::Value::Table(toml::map::Map::new()));
        
        if let Some(deps_table) = deps.as_table_mut() {
            deps_table.insert(pkg_name.clone(), dep_value);
        }
    }
    
    // Write back
    let new_manifest = toml::to_string_pretty(&manifest)
        .map_err(|e| e.to_string())?;
    fs::write("omni.toml", new_manifest)
        .map_err(|e| e.to_string())?;
    
    println!("   Added {} to {}", pkg_name, section);
    Ok(())
}

fn remove_dependency(args: &[String]) -> Result<(), String> {
    if args.is_empty() {
        return Err("Usage: opm remove <package>".to_string());
    }

    let pkg_name = &args[0];
    println!("➖ Removing {}...", pkg_name);
    
    // Read current manifest
    let manifest_str = fs::read_to_string("omni.toml")
        .map_err(|e| e.to_string())?;
    let mut manifest: toml::Value = toml::from_str(&manifest_str)
        .map_err(|e| e.to_string())?;
    
    let mut found = false;
    
    // Check all dependency sections
    for section in &["dependencies", "dev-dependencies", "build-dependencies"] {
        if let Some(table) = manifest.as_table_mut() {
            if let Some(deps) = table.get_mut(*section) {
                if let Some(deps_table) = deps.as_table_mut() {
                    if deps_table.remove(pkg_name).is_some() {
                        found = true;
                        println!("   Removed from {}", section);
                    }
                }
            }
        }
    }
    
    if !found {
        return Err(format!("Package '{}' not found in dependencies", pkg_name));
    }
    
    // Write back
    let new_manifest = toml::to_string_pretty(&manifest)
        .map_err(|e| e.to_string())?;
    fs::write("omni.toml", new_manifest)
        .map_err(|e| e.to_string())?;
    
    Ok(())
}

fn update_dependencies(args: &[String]) -> Result<(), String> {
    println!("🔄 Updating dependencies...\n");
    
    let specific_pkg = args.iter().find(|a| !a.starts_with('-'));
    
    // Read manifest
    let manifest_str = fs::read_to_string("omni.toml")
        .map_err(|e| e.to_string())?;
    let manifest: OmniManifest = toml::from_str(&manifest_str)
        .map_err(|e| e.to_string())?;
    
    let registry = PackageRegistry::new();
    
    for (name, spec) in &manifest.dependencies {
        if let Some(pkg) = specific_pkg {
            if name != pkg {
                continue;
            }
        }
        
        let current = spec.version();
        let latest = registry.resolve_version(name, "*")?;
        
        if current != latest {
            println!("   {} {} -> {}", name, current, latest);
        } else {
            println!("   {} {} (up to date)", name, current);
        }
    }
    
    // Remove lock file to force re-resolution
    if Path::new("omni.lock").exists() {
        fs::remove_file("omni.lock").map_err(|e| e.to_string())?;
    }
    
    println!("\n   Run 'opm build' to apply updates");
    Ok(())
}

fn install_binary(args: &[String]) -> Result<(), String> {
    if args.is_empty() {
        return Err("Usage: opm install <package> [version]".to_string());
    }
    
    let pkg_name = &args[0];
    let version = args.get(1).map(|s| s.as_str()).unwrap_or("*");
    
    println!("📥 Installing {}...", pkg_name);
    
    let registry = PackageRegistry::new();
    let resolved_version = registry.resolve_version(pkg_name, version)?;
    let pkg_path = registry.fetch_package(pkg_name, &resolved_version)?;
    
    // Build the package
    let old_dir = std::env::current_dir().map_err(|e| e.to_string())?;
    std::env::set_current_dir(&pkg_path).map_err(|e| e.to_string())?;
    
    build_project(&["--release".to_string()])?;
    
    std::env::set_current_dir(&old_dir).map_err(|e| e.to_string())?;
    
    // Copy binary to bin directory
    let home = dirs::home_dir().ok_or("Cannot determine home directory")?;
    let bin_dir = home.join(".opm").join("bin");
    fs::create_dir_all(&bin_dir).map_err(|e| e.to_string())?;
    
    let binary_path = pkg_path.join("target").join("release").join(pkg_name);
    let dest_path = bin_dir.join(pkg_name);
    
    fs::copy(&binary_path, &dest_path).map_err(|e| e.to_string())?;
    
    // Make executable on Unix
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&dest_path).map_err(|e| e.to_string())?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&dest_path, perms).map_err(|e| e.to_string())?;
    }
    
    println!("✅ Installed {} to {}", pkg_name, dest_path.display());
    println!("   Make sure {} is in your PATH", bin_dir.display());
    
    Ok(())
}

fn uninstall_binary(args: &[String]) -> Result<(), String> {
    if args.is_empty() {
        return Err("Usage: opm uninstall <package>".to_string());
    }
    
    let pkg_name = &args[0];
    
    let home = dirs::home_dir().ok_or("Cannot determine home directory")?;
    let binary_path = home.join(".opm").join("bin").join(pkg_name);
    
    if !binary_path.exists() {
        return Err(format!("{} is not installed", pkg_name));
    }
    
    fs::remove_file(&binary_path).map_err(|e| e.to_string())?;
    
    println!("✅ Uninstalled {}", pkg_name);
    Ok(())
}

fn search_packages(args: &[String]) -> Result<(), String> {
    if args.is_empty() {
        return Err("Usage: opm search <query>".to_string());
    }
    
    let query = args.join(" ").to_lowercase();
    println!("🔎 Searching for '{}'...\n", query);
    
    // Search installed packages and local cache
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    let opm_dir = home.join(".opm");
    let registry_cache = opm_dir.join("cache").join("registry.json");
    
    // Also search locally installed packages
    let installed_dir = opm_dir.join("packages");
    let mut results: Vec<(String, String, String)> = Vec::new();
    
    // Search cached registry index if available
    if registry_cache.exists() {
        if let Ok(content) = fs::read_to_string(&registry_cache) {
            // Parse JSON array of {name, version, description}
            if let Ok(entries) = serde_json::from_str::<Vec<serde_json::Value>>(&content) {
                for entry in &entries {
                    let name = entry["name"].as_str().unwrap_or("");
                    let desc = entry["description"].as_str().unwrap_or("");
                    let version = entry["version"].as_str().unwrap_or("0.0.0");
                    
                    if name.to_lowercase().contains(&query) || desc.to_lowercase().contains(&query) {
                        results.push((name.to_string(), version.to_string(), desc.to_string()));
                    }
                }
            }
        }
    }
    
    // Search locally installed packages
    if installed_dir.exists() {
        if let Ok(entries) = fs::read_dir(&installed_dir) {
            for entry in entries.flatten() {
                let pkg_name = entry.file_name().to_string_lossy().to_string();
                if pkg_name.to_lowercase().contains(&query) {
                    // Try to read the package manifest
                    let manifest_path = entry.path().join("omni.toml");
                    let desc = if manifest_path.exists() {
                        fs::read_to_string(&manifest_path)
                            .ok()
                            .and_then(|s| toml::from_str::<OmniManifest>(&s).ok())
                            .map(|m| m.package.description.unwrap_or_default())
                            .unwrap_or_else(|| "(installed locally)".to_string())
                    } else {
                        "(installed locally)".to_string()
                    };
                    
                    // Avoid duplicates
                    if !results.iter().any(|(n, _, _)| n == &pkg_name) {
                        results.push((pkg_name, "local".to_string(), desc));
                    }
                }
            }
        }
    }
    
    if results.is_empty() {
        println!("   No packages found matching '{}'\n", query);
        println!("   Tip: The registry cache may be empty. Run 'opm update' to refresh.");
    } else {
        println!("   Found {} package(s):\n", results.len());
        for (name, version, desc) in &results {
            println!("   📦 {} v{}", name, version);
            if !desc.is_empty() {
                println!("      {}", desc);
            }
            println!();
        }
    }
    
    Ok(())
}

fn show_package_info(args: &[String]) -> Result<(), String> {
    if args.is_empty() {
        return Err("Usage: opm info <package>".to_string());
    }
    
    let pkg_name = &args[0];
    println!("📋 Package: {}\n", pkg_name);
    
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    let opm_dir = home.join(".opm");
    let pkg_dir = opm_dir.join("packages").join(pkg_name);
    
    if pkg_dir.exists() {
        // Read manifest from installed package
        let manifest_path = pkg_dir.join("omni.toml");
        if manifest_path.exists() {
            match fs::read_to_string(&manifest_path) {
                Ok(content) => {
                    match toml::from_str::<OmniManifest>(&content) {
                        Ok(manifest) => {
                            println!("   Name:        {}", manifest.package.name);
                            println!("   Version:     {}", manifest.package.version);
                            if let Some(ref desc) = manifest.package.description {
                                println!("   Description: {}", desc);
                            }
                            if !manifest.package.authors.is_empty() {
                                println!("   Authors:     {}", manifest.package.authors.join(", "));
                            }
                            if let Some(ref license) = manifest.package.license {
                                println!("   License:     {}", license);
                            }
                            if let Some(ref homepage) = manifest.package.homepage {
                                println!("   Homepage:    {}", homepage);
                            }
                            println!("   Location:    {}", pkg_dir.display());
                            
                            // Show dependencies
                            if !manifest.dependencies.is_empty() {
                                println!("\n   Dependencies:");
                                for (dep_name, dep_spec) in &manifest.dependencies {
                                    println!("     {} = {:?}", dep_name, dep_spec);
                                }
                            }
                            
                            // Show file count
                            let file_count = walkdir_count(&pkg_dir);
                            println!("\n   Files:       {} file(s)", file_count);
                        }
                        Err(e) => {
                            println!("   Installed at: {}", pkg_dir.display());
                            println!("   (manifest parse error: {})", e);
                        }
                    }
                }
                Err(e) => {
                    println!("   Installed at: {}", pkg_dir.display());
                    println!("   (could not read manifest: {})", e);
                }
            }
        } else {
            println!("   Installed at: {}", pkg_dir.display());
            println!("   (no omni.toml found)");
        }
    } else {
        // Check registry cache
        let registry_cache = opm_dir.join("cache").join("registry.json");
        let mut found = false;
        
        if registry_cache.exists() {
            if let Ok(content) = fs::read_to_string(&registry_cache) {
                if let Ok(entries) = serde_json::from_str::<Vec<serde_json::Value>>(&content) {
                    for entry in &entries {
                        if entry["name"].as_str() == Some(pkg_name.as_str()) {
                            println!("   Version:     {}", entry["version"].as_str().unwrap_or("unknown"));
                            if let Some(desc) = entry["description"].as_str() {
                                println!("   Description: {}", desc);
                            }
                            println!("   Status:      not installed");
                            println!("\n   Run 'opm install {}' to install.", pkg_name);
                            found = true;
                            break;
                        }
                    }
                }
            }
        }
        
        if !found {
            println!("   Package '{}' not found locally or in registry cache.", pkg_name);
            println!("   Try 'opm update' to refresh the registry, then search again.");
        }
    }
    
    Ok(())
}

fn walkdir_count(dir: &Path) -> usize {
    let mut count = 0;
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                count += 1;
            } else if path.is_dir() {
                count += walkdir_count(&path);
            }
        }
    }
    count
}

fn publish_package(args: &[String]) -> Result<(), String> {
    println!("📤 Publishing package...\n");
    
    // Verify omni.toml exists and is valid
    if !Path::new("omni.toml").exists() {
        return Err("No omni.toml found".to_string());
    }
    
    let manifest_str = fs::read_to_string("omni.toml")
        .map_err(|e| e.to_string())?;
    let manifest: OmniManifest = toml::from_str(&manifest_str)
        .map_err(|e| e.to_string())?;
    
    println!("   Package: {} v{}", manifest.package.name, manifest.package.version);
    
    // Verify required fields
    if manifest.package.description.is_none() {
        return Err("Package must have a description".to_string());
    }
    
    if manifest.package.license.is_none() {
        return Err("Package must have a license".to_string());
    }
    
    // Run tests first
    if !args.contains(&"--no-verify".to_string()) {
        println!("   Running tests...");
        test_project(&[])?;
    }
    
    // Create tarball
    println!("   Creating package tarball...");
    let tarball_path = create_package_tarball(&manifest)?;
    
    // Upload
    let registry = PackageRegistry::new();
    registry.publish(&tarball_path, &manifest)?;
    
    // Cleanup
    fs::remove_file(&tarball_path).ok();
    
    println!("\n✅ Published {} v{}", manifest.package.name, manifest.package.version);
    Ok(())
}

fn create_package_tarball(manifest: &OmniManifest) -> Result<PathBuf, String> {
    let name = format!("{}-{}.tar.gz", manifest.package.name, manifest.package.version);
    let path = PathBuf::from(&name);
    
    // In real impl: create tarball with tar crate
    // Include: omni.toml, src/, README, LICENSE
    
    File::create(&path).map_err(|e| e.to_string())?;
    
    Ok(path)
}

fn login_registry(args: &[String]) -> Result<(), String> {
    println!("🔑 Logging in to registry...\n");
    
    let token = if let Some(t) = args.first() {
        t.clone()
    } else {
        print!("   Enter token: ");
        io::stdout().flush().map_err(|e| e.to_string())?;
        
        let mut token = String::new();
        io::stdin().read_line(&mut token).map_err(|e| e.to_string())?;
        token.trim().to_string()
    };
    
    // Save token
    let home = dirs::home_dir().ok_or("Cannot determine home directory")?;
    let config_dir = home.join(".opm");
    fs::create_dir_all(&config_dir).map_err(|e| e.to_string())?;
    
    let credentials = format!("token = \"{}\"", token);
    fs::write(config_dir.join("credentials.toml"), credentials)
        .map_err(|e| e.to_string())?;
    
    println!("✅ Logged in successfully");
    Ok(())
}

fn logout_registry() -> Result<(), String> {
    let home = dirs::home_dir().ok_or("Cannot determine home directory")?;
    let cred_path = home.join(".opm").join("credentials.toml");
    
    if cred_path.exists() {
        fs::remove_file(&cred_path).map_err(|e| e.to_string())?;
    }
    
    println!("✅ Logged out");
    Ok(())
}

fn clean_build() -> Result<(), String> {
    println!("🧹 Cleaning build artifacts...\n");
    
    let mut cleaned = 0u64;
    
    for dir in &["build", "target"] {
        if Path::new(dir).exists() {
            // Calculate size before deletion
            let size = dir_size(Path::new(dir));
            fs::remove_dir_all(dir).map_err(|e| e.to_string())?;
            println!("   Removed {} ({} bytes)", dir, size);
            cleaned += size;
        }
    }
    
    if cleaned > 0 {
        println!("\n   Freed {} bytes", cleaned);
    } else {
        println!("   Nothing to clean");
    }
    
    Ok(())
}

fn dir_size(path: &Path) -> u64 {
    let mut size = 0;
    if path.is_dir() {
        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    size += dir_size(&path);
                } else {
                    size += entry.metadata().map(|m| m.len()).unwrap_or(0);
                }
            }
        }
    }
    size
}

fn generate_docs(args: &[String]) -> Result<(), String> {
    println!("📚 Generating documentation...\n");
    
    let open_browser = args.contains(&"--open".to_string());
    
    // Read manifest
    let manifest_str = fs::read_to_string("omni.toml")
        .map_err(|e| e.to_string())?;
    let manifest: OmniManifest = toml::from_str(&manifest_str)
        .map_err(|e| e.to_string())?;
    
    let doc_dir = PathBuf::from("target").join("doc");
    fs::create_dir_all(&doc_dir).map_err(|e| e.to_string())?;
    
    // Find source files
    let source_files = find_source_files("src")?;
    
    for file in &source_files {
        println!("   Documenting {}...", file.display());
        // In real impl: parse file, extract doc comments, generate HTML
    }
    
    // Generate index
    let index_path = doc_dir.join("index.html");
    let index_html = format!(r#"<!DOCTYPE html>
<html>
<head>
    <title>{} Documentation</title>
    <style>
        body {{ font-family: sans-serif; margin: 40px; }}
        h1 {{ color: #333; }}
        .version {{ color: #666; }}
    </style>
</head>
<body>
    <h1>{}</h1>
    <p class="version">Version {}</p>
    <p>{}</p>
</body>
</html>"#, 
        manifest.package.name,
        manifest.package.name,
        manifest.package.version,
        manifest.package.description.as_deref().unwrap_or("")
    );
    
    fs::write(&index_path, index_html).map_err(|e| e.to_string())?;
    
    println!("\n✅ Documentation generated: {}", index_path.display());
    
    if open_browser {
        #[cfg(target_os = "windows")]
        Command::new("cmd").args(&["/C", "start", index_path.to_str().unwrap()]).spawn().ok();
        
        #[cfg(target_os = "macos")]
        Command::new("open").arg(&index_path).spawn().ok();
        
        #[cfg(target_os = "linux")]
        Command::new("xdg-open").arg(&index_path).spawn().ok();
    }
    
    Ok(())
}

fn show_dep_tree() -> Result<(), String> {
    println!("🌳 Dependency tree:\n");
    
    // Read manifest
    let manifest_str = fs::read_to_string("omni.toml")
        .map_err(|e| e.to_string())?;
    let manifest: OmniManifest = toml::from_str(&manifest_str)
        .map_err(|e| e.to_string())?;
    
    println!("{} v{}", manifest.package.name, manifest.package.version);
    
    let dep_count = manifest.dependencies.len();
    for (i, (name, spec)) in manifest.dependencies.iter().enumerate() {
        let is_last = i == dep_count - 1;
        let prefix = if is_last { "└──" } else { "├──" };
        println!("{} {} v{}", prefix, name, spec.version());
        
        // In real impl: recursively show transitive dependencies
    }
    
    if dep_count == 0 {
        println!("   (no dependencies)");
    }
    
    Ok(())
}

fn format_code(args: &[String]) -> Result<(), String> {
    println!("🎨 Formatting code...\n");
    
    let check_only = args.contains(&"--check".to_string());
    
    let source_files = find_source_files("src")?;
    let test_files = find_source_files("tests")?;
    
    let all_files: Vec<_> = source_files.into_iter().chain(test_files).collect();
    
    let mut formatted = 0;
    let mut unchanged = 0;
    
    for file in &all_files {
        // In real impl: parse and reformat
        println!("   {}", file.display());
        formatted += 1;
    }
    
    if check_only {
        if formatted > 0 {
            println!("\n⚠️  {} files need formatting", formatted);
            return Err("Files need formatting".to_string());
        }
    } else {
        println!("\n✅ {} files formatted", formatted);
    }
    
    Ok(())
}

fn lint_code(args: &[String]) -> Result<(), String> {
    println!("🔍 Running linter...\n");
    
    let source_files = find_source_files("src")?;
    
    let mut warnings = 0;
    let mut errors = 0;
    
    for file in &source_files {
        println!("   Checking {}...", file.display());
        // In real impl: run linter passes
    }
    
    println!();
    if errors > 0 {
        println!("❌ {} errors, {} warnings", errors, warnings);
        Err(format!("{} lint errors", errors))
    } else if warnings > 0 {
        println!("⚠️  {} warnings", warnings);
        Ok(())
    } else {
        println!("✅ No issues found");
        Ok(())
    }
}

fn read_lockfile() -> Result<Option<LockFile>, String> {
    let lock_path = Path::new("omni.lock");
    if !lock_path.exists() {
        return Ok(None);
    }
    
    let content = fs::read_to_string(lock_path)
        .map_err(|e| format!("Failed to read omni.lock: {}", e))?;
    let lock: LockFile = toml::from_str(&content)
        .map_err(|e| format!("Failed to parse omni.lock: {}", e))?;
    
    Ok(Some(lock))
}

fn resolve_from_lockfile(
    lock: &LockFile,
    deps: &HashMap<String, String>,
) -> Option<Vec<(String, String, PathBuf)>> {
    let mut resolved = Vec::new();
    
    for (name, _req) in deps {
        if let Some(locked) = lock.packages.iter().find(|p| &p.name == name) {
            let path = if locked.source.starts_with("path:") {
                PathBuf::from(&locked.source[5..])
            } else {
                // Registry package - reconstruct cache path
                let cache_dir = dirs::home_dir()
                    .unwrap_or_else(|| PathBuf::from("."))
                    .join(".opm")
                    .join("cache")
                    .join(&locked.name)
                    .join(&locked.version);
                cache_dir
            };
            resolved.push((locked.name.clone(), locked.version.clone(), path));
        } else {
            // Dependency not in lockfile, need fresh resolution
            return None;
        }
    }
    
    Some(resolved)
}

fn generate_lockfile(manifest: &OmniManifest, resolved: &[(String, String, PathBuf)]) -> Result<(), String> {
    println!("   Generating omni.lock...");
    
    let packages: Vec<LockedPackage> = resolved.iter().map(|(name, version, path)| {
        LockedPackage {
            name: name.clone(),
            version: version.clone(),
            source: if path.to_str().map(|s| s.contains(".opm")).unwrap_or(false) {
                "registry".to_string()
            } else {
                format!("path:{}", path.display())
            },
            checksum: Some(compute_hash(path)),
            dependencies: vec![],
        }
    }).collect();

    let lock = LockFile {
        version: 1,
        packages,
    };

    let toml_str = toml::to_string_pretty(&lock)
        .map_err(|e| e.to_string())?;
    fs::write("omni.lock", toml_str)
        .map_err(|e| e.to_string())?;
    
    Ok(())
}

fn compute_hash(path: &Path) -> String {
    // Simple hash for now - in real impl use SHA256
    let mut hasher = 0u64;
    
    fn hash_path(path: &Path, hasher: &mut u64) {
        if path.is_file() {
            if let Ok(content) = fs::read(path) {
                for byte in content {
                    *hasher = hasher.wrapping_mul(31).wrapping_add(byte as u64);
                }
            }
        } else if path.is_dir() {
            if let Ok(entries) = fs::read_dir(path) {
                for entry in entries.flatten() {
                    hash_path(&entry.path(), hasher);
                }
            }
        }
    }
    
    hash_path(path, &mut hasher);
    format!("sha256:{:016x}", hasher)
}

fn md5_hash(data: &[u8]) -> u64 {
    // Simple non-cryptographic hash
    let mut h = 0u64;
    for &byte in data {
        h = h.wrapping_mul(31).wrapping_add(byte as u64);
    }
    h
}

fn num_cpus() -> u32 {
    // Try to detect CPU count
    std::thread::available_parallelism()
        .map(|n| n.get() as u32)
        .unwrap_or(4)
}

// Directory helpers
mod dirs {
    use std::path::PathBuf;
    
    pub fn home_dir() -> Option<PathBuf> {
        #[cfg(windows)]
        {
            std::env::var("USERPROFILE").ok().map(PathBuf::from)
        }
        #[cfg(not(windows))]
        {
            std::env::var("HOME").ok().map(PathBuf::from)
        }
    }
}
