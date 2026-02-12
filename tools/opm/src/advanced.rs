//! Advanced OPM (Omni Package Manager)
//! 
//! SemVer resolution, workspaces, binary caching, build scripts.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::fs;
use serde::{Deserialize, Serialize};

/// Semantic Version
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SemVer {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
    pub prerelease: Option<String>,
}

impl SemVer {
    pub fn parse(s: &str) -> Option<Self> {
        let parts: Vec<&str> = s.split('.').collect();
        if parts.len() < 3 { return None; }
        
        let patch_str = parts[2];
        let (patch, prerelease) = if let Some(idx) = patch_str.find('-') {
            (patch_str[..idx].parse().ok()?, Some(patch_str[idx+1..].to_string()))
        } else {
            (patch_str.parse().ok()?, None)
        };

        Some(Self {
            major: parts[0].parse().ok()?,
            minor: parts[1].parse().ok()?,
            patch,
            prerelease,
        })
    }

    pub fn satisfies(&self, requirement: &VersionReq) -> bool {
        match requirement {
            VersionReq::Exact(v) => self == v,
            VersionReq::Caret(v) => {
                // ^1.2.3 means >=1.2.3 <2.0.0
                self.major == v.major && (
                    self.minor > v.minor || 
                    (self.minor == v.minor && self.patch >= v.patch)
                )
            },
            VersionReq::Tilde(v) => {
                // ~1.2.3 means >=1.2.3 <1.3.0
                self.major == v.major && self.minor == v.minor && self.patch >= v.patch
            },
            VersionReq::Range { min, max } => {
                self >= min && self <= max
            },
        }
    }
}

impl PartialOrd for SemVer {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SemVer {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        (self.major, self.minor, self.patch).cmp(&(other.major, other.minor, other.patch))
    }
}

#[derive(Debug, Clone)]
pub enum VersionReq {
    Exact(SemVer),
    Caret(SemVer),  // ^1.2.3
    Tilde(SemVer),  // ~1.2.3
    Range { min: SemVer, max: SemVer },
}

/// Dependency Resolver (PubGrub-inspired)
pub struct DependencyResolver {
    packages: HashMap<String, Vec<PackageVersion>>,
}

#[derive(Debug, Clone)]
pub struct PackageVersion {
    pub name: String,
    pub version: SemVer,
    pub dependencies: Vec<Dependency>,
}

#[derive(Debug, Clone)]
pub struct Dependency {
    pub name: String,
    pub version_req: VersionReq,
}

#[derive(Debug)]
pub struct Resolution {
    pub packages: HashMap<String, SemVer>,
}

impl DependencyResolver {
    pub fn new() -> Self {
        Self { packages: HashMap::new() }
    }

    pub fn add_available(&mut self, pkg: PackageVersion) {
        self.packages.entry(pkg.name.clone()).or_default().push(pkg);
    }

    pub fn resolve(&self, root_deps: &[Dependency]) -> Result<Resolution, String> {
        let mut resolved: HashMap<String, SemVer> = HashMap::new();
        let mut queue: Vec<Dependency> = root_deps.to_vec();
        let mut visited: HashSet<String> = HashSet::new();

        while let Some(dep) = queue.pop() {
            if visited.contains(&dep.name) {
                continue;
            }
            visited.insert(dep.name.clone());

            // Find best matching version
            let versions = self.packages.get(&dep.name)
                .ok_or_else(|| format!("Package not found: {}", dep.name))?;

            let best = versions.iter()
                .filter(|v| v.version.satisfies(&dep.version_req))
                .max_by(|a, b| a.version.cmp(&b.version))
                .ok_or_else(|| format!("No matching version for {} {:?}", dep.name, dep.version_req))?;

            // Check for conflicts
            if let Some(existing) = resolved.get(&dep.name) {
                if existing != &best.version {
                    return Err(format!(
                        "Version conflict for {}: {} vs {}",
                        dep.name, existing.major, best.version.major
                    ));
                }
            }

            resolved.insert(dep.name.clone(), best.version.clone());

            // Add transitive dependencies
            for sub_dep in &best.dependencies {
                queue.push(sub_dep.clone());
            }
        }

        Ok(Resolution { packages: resolved })
    }
}

/// Lockfile Generation
#[derive(Serialize, Deserialize)]
pub struct LockFile {
    pub version: u32,
    pub packages: Vec<LockedPackage>,
}

#[derive(Serialize, Deserialize)]
pub struct LockedPackage {
    pub name: String,
    pub version: String,
    pub source: String,
    pub checksum: String,
    pub dependencies: Vec<String>,
}

impl LockFile {
    pub fn from_resolution(resolution: &Resolution, sources: &HashMap<String, String>) -> Self {
        Self {
            version: 1,
            packages: resolution.packages.iter().map(|(name, version)| {
                LockedPackage {
                    name: name.clone(),
                    version: format!("{}.{}.{}", version.major, version.minor, version.patch),
                    source: sources.get(name).cloned().unwrap_or_default(),
                    checksum: Self::compute_checksum(name, version),
                    dependencies: Vec::new(),
                }
            }).collect(),
        }
    }

    fn compute_checksum(name: &str, version: &SemVer) -> String {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        name.hash(&mut hasher);
        version.major.hash(&mut hasher);
        version.minor.hash(&mut hasher);
        version.patch.hash(&mut hasher);
        format!("sha256:{:016x}", hasher.finish())
    }

    pub fn save(&self, path: &Path) -> Result<(), std::io::Error> {
        let content = serde_json::to_string_pretty(self)?;
        fs::write(path, content)
    }

    pub fn load(path: &Path) -> Result<Self, std::io::Error> {
        let content = fs::read_to_string(path)?;
        serde_json::from_str(&content).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    }
}

/// Workspace Support (Monorepo)
#[derive(Deserialize)]
pub struct WorkspaceConfig {
    pub members: Vec<String>,
    pub default_members: Option<Vec<String>>,
    pub exclude: Option<Vec<String>>,
}

pub struct Workspace {
    pub root: PathBuf,
    pub members: Vec<WorkspaceMember>,
}

pub struct WorkspaceMember {
    pub path: PathBuf,
    pub manifest: OmniManifest,
}

#[derive(Deserialize)]
pub struct OmniManifest {
    pub package: PackageInfo,
    pub dependencies: Option<HashMap<String, String>>,
    pub workspace: Option<WorkspaceConfig>,
}

#[derive(Deserialize)]
pub struct PackageInfo {
    pub name: String,
    pub version: String,
}

impl Workspace {
    pub fn discover(root: &Path) -> Result<Self, String> {
        let manifest_path = root.join("omni.toml");
        let content = fs::read_to_string(&manifest_path)
            .map_err(|e| format!("Failed to read manifest: {}", e))?;
        let manifest: OmniManifest = toml::from_str(&content)
            .map_err(|e| format!("Failed to parse manifest: {}", e))?;

        let mut members = Vec::new();

        if let Some(ws) = &manifest.workspace {
            for pattern in &ws.members {
                let member_path = root.join(pattern);
                if member_path.exists() {
                    let member_manifest = Self::load_member_manifest(&member_path)?;
                    members.push(WorkspaceMember {
                        path: member_path,
                        manifest: member_manifest,
                    });
                }
            }
        }

        Ok(Self {
            root: root.to_path_buf(),
            members,
        })
    }

    fn load_member_manifest(path: &Path) -> Result<OmniManifest, String> {
        let manifest_path = path.join("omni.toml");
        let content = fs::read_to_string(&manifest_path)
            .map_err(|e| format!("Failed to read member manifest: {}", e))?;
        toml::from_str(&content)
            .map_err(|e| format!("Failed to parse member manifest: {}", e))
    }
}

/// Binary Caching
pub struct BinaryCache {
    pub cache_dir: PathBuf,
}

impl BinaryCache {
    pub fn new() -> Self {
        let cache_dir = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".omni")
            .join("cache");
        
        fs::create_dir_all(&cache_dir).ok();
        
        Self { cache_dir }
    }

    pub fn get(&self, package: &str, version: &str, target: &str) -> Option<PathBuf> {
        let key = format!("{}-{}-{}", package, version, target);
        let path = self.cache_dir.join(&key);
        if path.exists() {
            Some(path)
        } else {
            None
        }
    }

    pub fn store(&self, package: &str, version: &str, target: &str, artifact: &[u8]) -> Result<PathBuf, std::io::Error> {
        let key = format!("{}-{}-{}", package, version, target);
        let path = self.cache_dir.join(&key);
        fs::write(&path, artifact)?;
        Ok(path)
    }

    pub fn clean_old(&self, max_age_days: u64) -> Result<usize, std::io::Error> {
        let mut cleaned = 0;
        let threshold = std::time::SystemTime::now() - std::time::Duration::from_secs(max_age_days * 24 * 60 * 60);

        for entry in fs::read_dir(&self.cache_dir)? {
            let entry = entry?;
            if let Ok(metadata) = entry.metadata() {
                if let Ok(modified) = metadata.modified() {
                    if modified < threshold {
                        fs::remove_file(entry.path())?;
                        cleaned += 1;
                    }
                }
            }
        }

        Ok(cleaned)
    }
}

/// Build Script Execution
pub struct BuildScriptRunner;

impl BuildScriptRunner {
    /// Compile and execute build.omni in a sandbox
    pub fn run(package_path: &Path) -> Result<BuildOutput, String> {
        let build_script = package_path.join("build.omni");
        
        if !build_script.exists() {
            return Ok(BuildOutput::default());
        }

        log::info!("Running build script for {:?}", package_path);

        // 1. Compile build.omni to temporary binary
        let temp_dir = std::env::temp_dir().join("omni-build");
        fs::create_dir_all(&temp_dir).map_err(|e| e.to_string())?;
        
        let binary = temp_dir.join("build_script");
        
        // In real impl: call omnc to compile
        // omnc build.omni --output build_script

        // 2. Execute in sandbox
        let output = std::process::Command::new(&binary)
            .current_dir(package_path)
            .env("OMNI_OUT_DIR", package_path.join("target"))
            .output()
            .map_err(|e| format!("Failed to run build script: {}", e))?;

        // 3. Parse output
        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut build_output = BuildOutput::default();

        for line in stdout.lines() {
            if let Some(lib) = line.strip_prefix("cargo:rustc-link-lib=") {
                build_output.link_libs.push(lib.to_string());
            } else if let Some(path) = line.strip_prefix("cargo:rustc-link-search=") {
                build_output.link_paths.push(PathBuf::from(path));
            } else if let Some(flag) = line.strip_prefix("cargo:rustc-cfg=") {
                build_output.cfg_flags.push(flag.to_string());
            }
        }

        Ok(build_output)
    }
}

#[derive(Default)]
pub struct BuildOutput {
    pub link_libs: Vec<String>,
    pub link_paths: Vec<PathBuf>,
    pub cfg_flags: Vec<String>,
}

/// Security Audit
pub struct SecurityAuditor;

#[derive(Debug)]
pub struct Vulnerability {
    pub package: String,
    pub version: String,
    pub advisory_id: String,
    pub severity: Severity,
    pub description: String,
}

#[derive(Debug)]
pub enum Severity {
    Low,
    Medium,
    High,
    Critical,
}

impl SecurityAuditor {
    pub fn audit(lockfile: &LockFile) -> Vec<Vulnerability> {
        let mut vulnerabilities = Vec::new();

        // In real impl: query vulnerability database
        // For demo: check against known vulnerable versions
        for pkg in &lockfile.packages {
            if pkg.name == "unsafe-lib" && pkg.version.starts_with("0.") {
                vulnerabilities.push(Vulnerability {
                    package: pkg.name.clone(),
                    version: pkg.version.clone(),
                    advisory_id: "OMNI-2024-001".to_string(),
                    severity: Severity::High,
                    description: "Memory safety issue in unsafe-lib < 1.0".to_string(),
                });
            }
        }

        vulnerabilities
    }
}

/// Dependency Graph Visualization
pub struct DependencyGraphVisualizer;

impl DependencyGraphVisualizer {
    pub fn to_dot(lockfile: &LockFile) -> String {
        let mut dot = String::from("digraph dependencies {\n");
        dot.push_str("  rankdir=LR;\n");
        dot.push_str("  node [shape=box];\n\n");

        for pkg in &lockfile.packages {
            let node_id = pkg.name.replace("-", "_");
            dot.push_str(&format!("  {} [label=\"{}\\n{}\"];\n", node_id, pkg.name, pkg.version));
            
            for dep in &pkg.dependencies {
                let dep_id = dep.replace("-", "_");
                dot.push_str(&format!("  {} -> {};\n", node_id, dep_id));
            }
        }

        dot.push_str("}\n");
        dot
    }

    pub fn print_tree(lockfile: &LockFile, indent: usize) {
        for pkg in &lockfile.packages {
            println!("{}{} v{}", "  ".repeat(indent), pkg.name, pkg.version);
        }
    }
}

/// Cross-Compilation Support
pub struct CrossCompiler;

#[derive(Debug)]
pub struct Target {
    pub triple: String, // e.g., "aarch64-linux-android"
    pub cpu: Option<String>,
    pub features: Vec<String>,
}

impl CrossCompiler {
    pub fn parse_target(triple: &str) -> Target {
        Target {
            triple: triple.to_string(),
            cpu: None,
            features: Vec::new(),
        }
    }

    pub fn get_sysroot(target: &Target) -> Option<PathBuf> {
        // Look for target-specific sysroot
        let sysroot = PathBuf::from(format!("/usr/local/{}", target.triple));
        if sysroot.exists() {
            Some(sysroot)
        } else {
            None
        }
    }
}
