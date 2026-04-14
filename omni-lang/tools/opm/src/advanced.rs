//! Advanced OPM (Omni Package Manager)
//! 
//! SemVer resolution, workspaces, binary caching, build scripts.

#![allow(dead_code)]

use std::collections::{HashMap, HashSet};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum NativeArtifactKind {
    Object,
    Bitcode,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NativeArtifactMetadata {
    pub kind: NativeArtifactKind,
    pub target_triple: String,
    pub path: PathBuf,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub checksum: Option<String>,
}

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
    pub default_members: Vec<PathBuf>,
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
        let mut default_members = Vec::new();

        if let Some(ws) = &manifest.workspace {
            let mut member_paths = Self::expand_patterns(root, &ws.members, false)?;

            let excluded: HashSet<PathBuf> = ws
                .exclude
                .as_ref()
                .map(|patterns| Self::expand_patterns(root, patterns, true))
                .transpose()?
                .unwrap_or_default()
                .into_iter()
                .collect();

            member_paths.retain(|path| !excluded.contains(path));

            default_members = ws
                .default_members
                .as_ref()
                .map(|patterns| Self::expand_patterns(root, patterns, true))
                .transpose()?
                .unwrap_or_else(|| member_paths.clone());

            default_members.retain(|path| member_paths.contains(path));
            default_members.sort();
            default_members.dedup();

            for member_path in &member_paths {
                let member_manifest = Self::load_member_manifest(member_path)?;
                members.push(WorkspaceMember {
                    path: member_path.clone(),
                    manifest: member_manifest,
                });
            }
        }

        Ok(Self {
            root: root.to_path_buf(),
            members,
            default_members,
        })
    }

    fn expand_patterns(
        root: &Path,
        patterns: &[String],
        allow_missing: bool,
    ) -> Result<Vec<PathBuf>, String> {
        let mut resolved = Vec::new();

        for pattern in patterns {
            resolved.extend(Self::expand_pattern(root, pattern, allow_missing)?);
        }

        resolved.sort();
        resolved.dedup();
        Ok(resolved)
    }

    fn expand_pattern(root: &Path, pattern: &str, allow_missing: bool) -> Result<Vec<PathBuf>, String> {
        if let Some(prefix) = pattern.strip_suffix("/*") {
            let base = root.join(prefix);
            if !base.exists() {
                if allow_missing {
                    return Ok(Vec::new());
                }
                return Err(format!(
                    "Workspace member base '{}' does not exist",
                    base.display()
                ));
            }

            let mut members = Vec::new();
            let entries = fs::read_dir(&base)
                .map_err(|e| format!("Failed to read workspace directory {}: {}", base.display(), e))?;

            for entry in entries {
                let entry = entry.map_err(|e| format!("Failed to read workspace entry: {}", e))?;
                let path = entry.path();
                if path.is_dir() && path.join("omni.toml").exists() {
                    members.push(path);
                }
            }

            members.sort();
            return Ok(members);
        }

        let candidate = root.join(pattern);
        let member_path = if candidate.is_file() {
            if candidate.file_name().and_then(|name| name.to_str()) != Some("omni.toml") {
                return if allow_missing {
                    Ok(Vec::new())
                } else {
                    Err(format!(
                        "Workspace member '{}' is not a directory or omni.toml manifest",
                        candidate.display()
                    ))
                };
            }
            candidate.parent().unwrap_or(root).to_path_buf()
        } else {
            candidate
        };

        if !member_path.exists() {
            return if allow_missing {
                Ok(Vec::new())
            } else {
                Err(format!(
                    "Workspace member '{}' does not exist",
                    member_path.display()
                ))
            };
        }

        if !member_path.join("omni.toml").exists() {
            return if allow_missing {
                Ok(Vec::new())
            } else {
                Err(format!(
                    "Workspace member '{}' is missing omni.toml",
                    member_path.display()
                ))
            };
        }

        Ok(vec![member_path])
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

fn find_compiler_manifest(start: &Path) -> Option<PathBuf> {
    for ancestor in start.ancestors() {
        let manifest = ancestor.join("compiler").join("Cargo.toml");
        if manifest.exists() {
            return Some(manifest);
        }
    }

    None
}

fn execute_build_script(
    build_script: &Path,
    package_path: &Path,
    out_dir: &Path,
) -> Result<Output, String> {
    let compiler_command = env::var_os("OMNC").unwrap_or_else(|| "omnc".into());

    let direct_output = Command::new(&compiler_command)
        .current_dir(package_path)
        .env("CARGO_MANIFEST_DIR", package_path)
        .env("OMNI_OUT_DIR", out_dir)
        .env("OUT_DIR", out_dir)
        .arg("--run")
        .arg(build_script)
        .output();

    match direct_output {
        Ok(output) => Ok(output),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
            let compiler_manifest = find_compiler_manifest(package_path).ok_or_else(|| {
                format!(
                    "Failed to locate omnc or compiler/Cargo.toml near {}",
                    package_path.display()
                )
            })?;

            Command::new("cargo")
                .current_dir(package_path)
                .env("CARGO_MANIFEST_DIR", package_path)
                .env("OMNI_OUT_DIR", out_dir)
                .env("OUT_DIR", out_dir)
                .arg("run")
                .arg("--quiet")
                .arg("--manifest-path")
                .arg(&compiler_manifest)
                .arg("--bin")
                .arg("omnc")
                .arg("--")
                .arg("--run")
                .arg(build_script)
                .output()
                .map_err(|e| format!("Failed to run build script via cargo: {}", e))
        }
        Err(err) => Err(format!("Failed to run build script: {}", err)),
    }
}

fn parse_build_output(stdout: &str) -> BuildOutput {
    let mut build_output = BuildOutput::default();
    let mut artifact_kind: Option<NativeArtifactKind> = None;
    let mut artifact_target_triple: Option<String> = None;
    let mut artifact_path: Option<PathBuf> = None;
    let mut artifact_checksum: Option<String> = None;

    for line in stdout.lines().map(str::trim) {
        if let Some(lib) = line.strip_prefix("cargo:rustc-link-lib=") {
            build_output.link_libs.push(lib.to_string());
        } else if let Some(path) = line.strip_prefix("cargo:rustc-link-search=") {
            let path = path
                .split_once('=')
                .map(|(_, value)| value)
                .unwrap_or(path);
            build_output.link_paths.push(PathBuf::from(path));
        } else if let Some(flag) = line.strip_prefix("cargo:rustc-cfg=") {
            build_output.cfg_flags.push(flag.to_string());
        } else if let Some(value) = line.strip_prefix("cargo:omni-artifact-kind=") {
            artifact_kind = Some(match value.trim() {
                "object" => NativeArtifactKind::Object,
                "bitcode" => NativeArtifactKind::Bitcode,
                other => {
                    log::warn!("unknown omni artifact kind '{other}', ignoring native artifact metadata");
                    continue;
                }
            });
        } else if let Some(value) = line.strip_prefix("cargo:omni-artifact-target-triple=") {
            artifact_target_triple = Some(value.trim().to_string());
        } else if let Some(value) = line.strip_prefix("cargo:omni-artifact-path=") {
            artifact_path = Some(PathBuf::from(value.trim()));
        } else if let Some(value) = line.strip_prefix("cargo:omni-artifact-checksum=") {
            let trimmed = value.trim();
            if !trimmed.is_empty() {
                artifact_checksum = Some(trimmed.to_string());
            }
        } else if let Some(message) = line.strip_prefix("cargo:warning=") {
            log::warn!("build script warning: {}", message);
        }
    }

    if let (Some(kind), Some(target_triple), Some(path)) = (
        artifact_kind,
        artifact_target_triple,
        artifact_path,
    ) {
        build_output.native_artifact = Some(NativeArtifactMetadata {
            kind,
            target_triple,
            path,
            checksum: artifact_checksum,
        });
    }

    build_output
}

impl BuildScriptRunner {
    /// Compile and execute build.omni in a sandbox
    pub fn run(package_path: &Path) -> Result<BuildOutput, String> {
        let build_script = package_path.join("build.omni");
        
        if !build_script.exists() {
            return Ok(BuildOutput::default());
        }

        log::info!("Running build script for {:?}", package_path);

        let out_dir = package_path.join("target").join("build-scripts");
        fs::create_dir_all(&out_dir).map_err(|e| e.to_string())?;

        let output = execute_build_script(&build_script, package_path, &out_dir)?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!(
                "Build script failed with exit code {:?}: {}",
                output.status.code(),
                stderr.trim()
            ));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(parse_build_output(&stdout))
    }
}

#[derive(Default)]
pub struct BuildOutput {
    pub link_libs: Vec<String>,
    pub link_paths: Vec<PathBuf>,
    pub cfg_flags: Vec<String>,
    pub native_artifact: Option<NativeArtifactMetadata>,
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

#[cfg(test)]
mod tests {
    use super::*;

    fn unique_temp_dir(prefix: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "{prefix}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("time")
                .as_nanos()
        ))
    }

    fn write_package_manifest(dir: &Path, name: &str, version: &str) {
        fs::create_dir_all(dir).expect("create package dir");
        fs::write(
            dir.join("omni.toml"),
            format!(
                "[package]\nname = \"{}\"\nversion = \"{}\"\n",
                name, version
            ),
        )
        .expect("write package manifest");
    }

    #[test]
    fn parse_build_output_parses_directives() {
        let output = parse_build_output(
            "cargo:rustc-link-lib=static=foo\n\
             cargo:rustc-link-search=native=/tmp/omni\n\
             cargo:rustc-cfg=feature=demo\n\
             cargo:omni-artifact-kind=object\n\
             cargo:omni-artifact-target-triple=x86_64-unknown-linux-gnu\n\
             cargo:omni-artifact-path=/tmp/omni/build/demo.o\n\
             cargo:omni-artifact-checksum=sha256:deadbeef\n\
             cargo:warning=ignored\n",
        );

        assert_eq!(output.link_libs, vec!["static=foo".to_string()]);
        assert_eq!(output.link_paths, vec![PathBuf::from("/tmp/omni")]);
        assert_eq!(output.cfg_flags, vec!["feature=demo".to_string()]);
        let artifact = output.native_artifact.expect("expected native artifact metadata");
        assert!(matches!(artifact.kind, NativeArtifactKind::Object));
        assert_eq!(artifact.target_triple, "x86_64-unknown-linux-gnu");
        assert_eq!(artifact.path, PathBuf::from("/tmp/omni/build/demo.o"));
        assert_eq!(artifact.checksum.as_deref(), Some("sha256:deadbeef"));
    }

    #[test]
    fn find_compiler_manifest_walks_ancestors() {
        let root = std::env::temp_dir().join(format!("omni-opm-test-{}", std::process::id()));
        let compiler_dir = root.join("compiler");
        let nested_package = root.join("packages").join("demo");

        fs::create_dir_all(&compiler_dir).expect("create compiler dir");
        fs::create_dir_all(&nested_package).expect("create nested package dir");
        fs::write(compiler_dir.join("Cargo.toml"), "[package]\nname = \"compiler\"\nversion = \"0.1.0\"\n")
            .expect("write compiler manifest");

        let manifest = find_compiler_manifest(&nested_package).expect("should find compiler manifest");
        assert_eq!(manifest, compiler_dir.join("Cargo.toml"));

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn workspace_discover_respects_excludes_and_default_members() {
        let root = unique_temp_dir("omni-workspace-test");
        let packages = root.join("packages");
        let tools = root.join("tools");

        fs::create_dir_all(&packages).expect("create packages dir");
        fs::create_dir_all(&tools).expect("create tools dir");

        write_package_manifest(&packages.join("app"), "app", "0.1.0");
        write_package_manifest(&packages.join("lib"), "lib", "0.1.0");
        write_package_manifest(&packages.join("skip"), "skip", "0.1.0");
        write_package_manifest(&tools.join("demo"), "demo", "0.1.0");
        fs::create_dir_all(packages.join("notes")).expect("create non-package dir");

        fs::write(
            root.join("omni.toml"),
            r#"[package]
name = "workspace-root"
version = "0.1.0"

[workspace]
members = ["packages/*", "tools/demo"]
default_members = ["packages/app", "tools/demo"]
exclude = ["packages/skip"]
"#,
        )
        .expect("write root manifest");

        let workspace = Workspace::discover(&root).expect("discover workspace");

        let member_names: Vec<_> = workspace
            .members
            .iter()
            .map(|member| member.manifest.package.name.as_str())
            .collect();
        assert_eq!(member_names, vec!["app", "lib", "demo"]);

        let default_names: Vec<_> = workspace
            .default_members
            .iter()
            .map(|path| {
                path.file_name()
                    .and_then(|name| name.to_str())
                    .expect("default member name")
                    .to_string()
            })
            .collect();
        assert_eq!(default_names, vec!["app".to_string(), "demo".to_string()]);

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn workspace_discover_errors_on_missing_explicit_member() {
        let root = unique_temp_dir("omni-workspace-missing");
        fs::create_dir_all(&root).expect("create workspace root");

        fs::write(
            root.join("omni.toml"),
            r#"[package]
name = "workspace-root"
version = "0.1.0"

[workspace]
members = ["packages/missing"]
"#,
        )
        .expect("write root manifest");

        let result = Workspace::discover(&root);
        assert!(
            matches!(result, Err(message) if message.contains("does not exist")),
            "expected missing-member error"
        );

        let _ = fs::remove_dir_all(&root);
    }
}
