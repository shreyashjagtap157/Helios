//! Omni Package Manager (opm)
//!
//! Manages Omni language projects: dependencies, builds, and publishing.
//! Uses `omni.toml` as the project manifest format.

use clap::{Parser, Subcommand};
use base64::{engine::general_purpose::STANDARD as BASE64_STANDARD, Engine as _};
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use log::info;
use semver::Version;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet, hash_map::DefaultHasher};
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;

mod advanced;

// ---------------------------------------------------------------------------
// Manifest types (omni.toml)
// ---------------------------------------------------------------------------
#[derive(Debug, Serialize, Deserialize)]
struct OmniManifest {
    package: PackageInfo,
    #[serde(default)]
    dependencies: toml::map::Map<String, toml::Value>,
    #[serde(default)]
    workspace: Option<WorkspaceConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    signature: Option<ManifestSignature>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct WorkspaceConfig {
    #[serde(default)]
    members: Vec<String>,
    #[serde(default)]
    default_members: Option<Vec<String>>,
    #[serde(default)]
    exclude: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct ManifestSignature {
    algorithm: String,
    key_id: String,
    public_key: String,
    checksum: String,
    signature: String,
}

#[derive(Debug, Clone, Serialize)]
struct ManifestTrustPayload {
    package: PackageInfo,
    dependencies: BTreeMap<String, serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    workspace: Option<WorkspaceConfig>,
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
    Install {
        /// Use existing lockfile without attempting to resolve dependencies
        #[arg(long)]
        locked: bool,
    },
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
const LOCKFILE_NAME: &str = "omni.lock";

fn checksum_for(name: &str, version: &str) -> String {
    let mut hasher = DefaultHasher::new();
    name.hash(&mut hasher);
    version.hash(&mut hasher);
    format!("sha256:{:016x}", hasher.finish())
}

fn checksum_for_content(content: &str) -> String {
    let mut hasher = DefaultHasher::new();
    content.hash(&mut hasher);
    format!("sha256:{:016x}", hasher.finish())
}

fn normalize_lock_version(version_req: &str) -> String {
    let trimmed = version_req.trim();
    if trimmed == "*" {
        return "0.0.0".to_string();
    }

    if let Ok(version) = Version::parse(trimmed) {
        return version.to_string();
    }

    let head = trimmed.split(',').next().unwrap_or(trimmed).trim();
    let clean = head.trim_start_matches(|c| matches!(c, '^' | '~' | '>' | '<' | '='));
    if let Ok(version) = Version::parse(clean) {
        return version.to_string();
    }

    if let Ok(requirement) = semver::VersionReq::parse(trimmed) {
        let mut candidates: Vec<&semver::Comparator> = requirement
            .comparators
            .iter()
            .filter(|comparator| !matches!(comparator.op, semver::Op::Less | semver::Op::LessEq))
            .collect();
        if candidates.is_empty() {
            candidates = requirement.comparators.iter().collect();
        }

        if let Some(highest) = candidates.into_iter().max_by(|a, b| {
            (a.major, a.minor.unwrap_or(0), a.patch.unwrap_or(0), a.pre.clone()).cmp(&(
                b.major,
                b.minor.unwrap_or(0),
                b.patch.unwrap_or(0),
                b.pre.clone(),
            ))
        }) {
            let version = Version::new(
                highest.major,
                highest.minor.unwrap_or(0),
                highest.patch.unwrap_or(0),
            );
            return version.to_string();
        }
    }

    "0.0.0".to_string()
}

fn dependency_request(dep: &toml::Value) -> (String, String) {
    match dep {
        toml::Value::String(s) => (s.clone(), "registry+omni".to_string()),
        toml::Value::Table(table) => {
            let version = table
                .get("version")
                .and_then(|v| v.as_str())
                .unwrap_or("*")
                .to_string();

            if let Some(path) = table.get("path").and_then(|v| v.as_str()) {
                (version, format!("path+{}", path))
            } else if let Some(git) = table.get("git").and_then(|v| v.as_str()) {
                (version, format!("git+{}", git))
            } else {
                (version, "registry+omni".to_string())
            }
        }
        other => (other.to_string(), "registry+omni".to_string()),
    }
}

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

fn canonical_toml_value(value: &toml::Value) -> serde_json::Value {
    match value {
        toml::Value::String(text) => serde_json::Value::String(text.clone()),
        toml::Value::Integer(value) => serde_json::Value::from(*value),
        toml::Value::Float(value) => serde_json::Value::from(*value),
        toml::Value::Boolean(value) => serde_json::Value::from(*value),
        toml::Value::Datetime(value) => serde_json::Value::String(value.to_string()),
        toml::Value::Array(values) => serde_json::Value::Array(
            values.iter().map(canonical_toml_value).collect(),
        ),
        toml::Value::Table(table) => {
            let mut sorted = serde_json::Map::new();
            let ordered: BTreeMap<_, _> = table.iter().collect();
            for (key, value) in ordered {
                sorted.insert(key.clone(), canonical_toml_value(value));
            }
            serde_json::Value::Object(sorted)
        }
    }
}

fn manifest_trust_payload(manifest: &OmniManifest) -> ManifestTrustPayload {
    let dependencies = manifest
        .dependencies
        .iter()
        .map(|(name, value)| (name.clone(), canonical_toml_value(value)))
        .collect();

    ManifestTrustPayload {
        package: manifest.package.clone(),
        dependencies,
        workspace: manifest.workspace.clone(),
    }
}

fn canonical_manifest_bytes(manifest: &OmniManifest) -> Result<Vec<u8>, String> {
    serde_json::to_vec(&manifest_trust_payload(manifest))
        .map_err(|e| format!("Failed to serialize canonical manifest payload: {e}"))
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

fn decode_key_material(encoded: &str) -> Result<Vec<u8>, String> {
    let trimmed = encoded.trim();

    if let Ok(bytes) = BASE64_STANDARD.decode(trimmed) {
        return Ok(bytes);
    }

    if trimmed.len() % 2 == 0 && trimmed.chars().all(|ch| ch.is_ascii_hexdigit()) {
        let mut bytes = Vec::with_capacity(trimmed.len() / 2);
        for chunk in trimmed.as_bytes().chunks(2) {
            let pair = std::str::from_utf8(chunk)
                .map_err(|e| format!("Invalid hex-encoded signing key: {e}"))?;
            let byte = u8::from_str_radix(pair, 16)
                .map_err(|e| format!("Invalid hex-encoded signing key: {e}"))?;
            bytes.push(byte);
        }
        return Ok(bytes);
    }

    Err("Signing key must be base64 or hex encoded".to_string())
}

fn signing_key_from_env() -> Result<(SigningKey, String), String> {
    let encoded = std::env::var("OPM_PUBLISH_PRIVATE_KEY")
        .map_err(|_| "OPM_PUBLISH_PRIVATE_KEY must be set to sign published manifests".to_string())?;
    let key_bytes = decode_key_material(&encoded)?;
    let key_bytes: [u8; 32] = key_bytes
        .try_into()
        .map_err(|_| "OPM_PUBLISH_PRIVATE_KEY must decode to exactly 32 bytes".to_string())?;

    let signing_key = SigningKey::from_bytes(&key_bytes);
    let verifying_key = signing_key.verifying_key();
    let fingerprint = sha256_hex(verifying_key.as_bytes());
    let key_id = std::env::var("OPM_PUBLISH_KEY_ID")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| format!("ed25519:{}", &fingerprint[..16]));

    Ok((signing_key, key_id))
}

fn manifest_signature_for(manifest: &OmniManifest) -> Result<ManifestSignature, String> {
    let (signing_key, key_id) = signing_key_from_env()?;
    let payload_bytes = canonical_manifest_bytes(manifest)?;
    let checksum = sha256_hex(&payload_bytes);
    let signature = signing_key.sign(&payload_bytes);
    let verifying_key = signing_key.verifying_key();

    Ok(ManifestSignature {
        algorithm: "ed25519+sha256".to_string(),
        key_id,
        public_key: BASE64_STANDARD.encode(verifying_key.as_bytes()),
        checksum: format!("sha256:{checksum}"),
        signature: BASE64_STANDARD.encode(signature.to_bytes()),
    })
}

fn sign_manifest(mut manifest: OmniManifest) -> Result<OmniManifest, String> {
    manifest.signature = Some(manifest_signature_for(&manifest)?);
    Ok(manifest)
}

fn verify_manifest_signature(manifest: &OmniManifest, require_signature: bool) -> Result<(), String> {
    let Some(signature) = manifest.signature.as_ref() else {
        if require_signature {
            return Err(format!(
                "Manifest '{}' v{} is missing a cryptographic signature",
                manifest.package.name, manifest.package.version
            ));
        }
        return Ok(());
    };

    if signature.algorithm != "ed25519+sha256" {
        return Err(format!(
            "Unsupported manifest signature algorithm '{}'; expected 'ed25519+sha256'",
            signature.algorithm
        ));
    }

    let payload_bytes = canonical_manifest_bytes(manifest)?;
    let checksum = format!("sha256:{}", sha256_hex(&payload_bytes));
    if signature.checksum != checksum {
        return Err(format!(
            "Manifest checksum mismatch for '{}' v{}: expected {}, found {}",
            manifest.package.name, manifest.package.version, checksum, signature.checksum
        ));
    }

    let public_key_bytes = BASE64_STANDARD
        .decode(signature.public_key.trim())
        .map_err(|e| format!("Invalid manifest public key encoding: {e}"))?;
    let public_key_bytes: [u8; 32] = public_key_bytes
        .as_slice()
        .try_into()
        .map_err(|_| "Manifest public key must decode to exactly 32 bytes".to_string())?;
    let verifying_key = VerifyingKey::from_bytes(&public_key_bytes)
        .map_err(|e| format!("Invalid manifest public key: {e}"))?;

    let signature_bytes = BASE64_STANDARD
        .decode(signature.signature.trim())
        .map_err(|e| format!("Invalid manifest signature encoding: {e}"))?;
    let parsed_signature = Signature::from_slice(&signature_bytes)
        .map_err(|e| format!("Invalid manifest signature bytes: {e}"))?;

    verifying_key
        .verify(&payload_bytes, &parsed_signature)
        .map_err(|e| {
            format!(
                "Manifest signature verification failed for '{}' v{}: {e}",
                manifest.package.name, manifest.package.version
            )
        })
}

fn read_manifest_verified(path: &Path, require_signature: bool) -> Result<OmniManifest, String> {
    let manifest = read_manifest(path)?;
    verify_manifest_signature(&manifest, require_signature)?;
    Ok(manifest)
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ResolvedInstallPackage {
    version: String,
    source: String,
    dependencies: Vec<String>,
}

fn sorted_unique(strings: impl IntoIterator<Item = String>) -> Vec<String> {
    let mut items: Vec<String> = strings.into_iter().collect();
    items.sort();
    items.dedup();
    items
}

fn merge_dependency_edges(existing: &[String], additional: &[String]) -> Vec<String> {
    let mut merged = BTreeSet::new();
    merged.extend(existing.iter().cloned());
    merged.extend(additional.iter().cloned());
    merged.into_iter().collect()
}

fn lock_source_for_path(path: &Path) -> String {
    let canonical = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    format!("path+{}", canonical.display())
}

fn registry_root() -> Option<PathBuf> {
    if let Ok(override_root) = std::env::var("OPM_REGISTRY_ROOT") {
        let trimmed = override_root.trim();
        if !trimmed.is_empty() {
            return Some(PathBuf::from(trimmed));
        }
    }

    if std::env::var("OPM_REGISTRY_URL").is_ok() {
        return None;
    }

    dirs::home_dir().map(|home| home.join(".omni").join("registry"))
}

fn registry_url() -> Option<String> {
    std::env::var("OPM_REGISTRY_URL")
        .ok()
        .map(|value| value.trim().trim_end_matches('/').to_string())
        .filter(|value| !value.is_empty())
}

fn registry_manifest_path(root: &Path, package_name: &str, version: &str) -> PathBuf {
    root.join(package_name).join(version).join(MANIFEST_NAME)
}

fn remote_registry_index_url(base_url: &str, package_name: &str) -> String {
    format!("{}/index/{}", base_url.trim_end_matches('/'), package_name)
}

fn remote_registry_manifest_url(base_url: &str, package_name: &str, version: &str) -> String {
    format!(
        "{}/packages/{}/{}/{}",
        base_url.trim_end_matches('/'),
        package_name,
        version,
        MANIFEST_NAME
    )
}

fn fetch_remote_registry_versions(base_url: &str, package_name: &str) -> Result<Vec<String>, String> {
    let response = reqwest::blocking::get(remote_registry_index_url(base_url, package_name))
        .map_err(|e| format!("Failed to query registry index for '{package_name}': {e}"))?
        .error_for_status()
        .map_err(|e| format!("Registry index returned error for '{package_name}': {e}"))?;

    let body = response
        .text()
        .map_err(|e| format!("Failed to read registry index for '{package_name}': {e}"))?;

    Ok(body
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(String::from)
        .collect())
}

fn resolve_remote_registry_version(base_url: &str, package_name: &str, version_req: &str) -> String {
    let default_version = normalize_lock_version(version_req);
    let requirement = match semver::VersionReq::parse(version_req) {
        Ok(req) => req,
        Err(_) => return default_version,
    };

    let mut candidates = match fetch_remote_registry_versions(base_url, package_name) {
        Ok(versions) => versions
            .into_iter()
            .filter_map(|version| Version::parse(&version).ok())
            .filter(|version| requirement.matches(version))
            .collect::<Vec<_>>(),
        Err(_) => return default_version,
    };

    candidates.sort();
    candidates
        .pop()
        .map(|version| version.to_string())
        .unwrap_or(default_version)
}

fn fetch_remote_registry_manifest(
    base_url: &str,
    package_name: &str,
    version: &str,
) -> Result<OmniManifest, String> {
    let response = reqwest::blocking::get(remote_registry_manifest_url(base_url, package_name, version))
        .map_err(|e| {
            format!(
                "Failed to fetch registry manifest for '{}@{}': {e}",
                package_name, version
            )
        })?
        .error_for_status()
        .map_err(|e| {
            format!(
                "Registry manifest returned error for '{}@{}': {e}",
                package_name, version
            )
        })?;

    let body = response.text().map_err(|e| {
        format!(
            "Failed to read registry manifest for '{}@{}': {e}",
            package_name, version
        )
    })?;

    let manifest = toml::from_str(&body).map_err(|e| {
        format!(
            "Failed to parse registry manifest for '{}@{}': {e}",
            package_name, version
        )
    })?;

    verify_manifest_signature(&manifest, true)?;
    Ok(manifest)
}

fn registry_source_label(package_name: &str, version: &str) -> String {
    if let Some(root) = registry_root() {
        return format!(
            "registry+omni+cache/{}/{}@{}",
            root.display(),
            package_name,
            version
        );
    }

    if let Some(base_url) = registry_url() {
        return format!(
            "registry+omni+remote/{}/{}@{}",
            base_url,
            package_name,
            version
        );
    }

    format!("registry+omni/{}@{}", package_name, version)
}

fn search_local_registry(root: &Path, query: &str) -> Result<Vec<String>, String> {
    if !root.exists() {
        return Ok(Vec::new());
    }

    let needle = query.to_lowercase();
    let mut matches = Vec::new();
    let entries = fs::read_dir(root)
        .map_err(|e| format!("Failed to read local registry root {}: {e}", root.display()))?;

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let Some(name) = path.file_name().and_then(|value| value.to_str()) else {
            continue;
        };
        if name.to_lowercase().contains(&needle) {
            matches.push(name.to_string());
        }
    }

    matches.sort();
    matches.dedup();
    Ok(matches)
}

fn search_remote_registry(base_url: &str, query: &str) -> Result<Vec<String>, String> {
    let encoded = query.trim().replace(' ', "+");
    let url = format!("{}/search?q={}", base_url.trim_end_matches('/'), encoded);
    let response = reqwest::blocking::get(url)
        .map_err(|e| format!("Failed to query remote registry search: {e}"))?
        .error_for_status()
        .map_err(|e| format!("Remote registry search returned error: {e}"))?;

    let body = response
        .text()
        .map_err(|e| format!("Failed to read remote registry search results: {e}"))?;

    let mut matches = body
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(String::from)
        .collect::<Vec<_>>();
    matches.sort();
    matches.dedup();
    Ok(matches)
}

#[derive(Debug, Serialize)]
struct PublishPayload {
    name: String,
    version: String,
    checksum: String,
    manifest: String,
    signature: ManifestSignature,
}

#[derive(Debug, Serialize, Deserialize)]
struct LocalPublishRecord {
    name: String,
    version: String,
    checksum: String,
    manifest_path: String,
    signature: ManifestSignature,
}

fn publish_to_local_registry(
    root: &Path,
    manifest: &OmniManifest,
    manifest_content: &str,
) -> Result<PathBuf, String> {
    let signature = manifest.signature.clone().ok_or_else(|| {
        format!(
            "Refusing to publish unsigned manifest '{}' v{}; signatures are required",
            manifest.package.name, manifest.package.version
        )
    })?;

    let publish_dir = root
        .join(&manifest.package.name)
        .join(&manifest.package.version);

    if publish_dir.exists() && !truthy_env("OPM_PUBLISH_ALLOW_OVERWRITE") {
        return Err(format!(
            "Package {} v{} already exists in local registry (set OPM_PUBLISH_ALLOW_OVERWRITE=1 to replace)",
            manifest.package.name, manifest.package.version
        ));
    }

    fs::create_dir_all(&publish_dir)
        .map_err(|e| format!("Failed to create local publish directory {}: {e}", publish_dir.display()))?;

    let target_manifest = publish_dir.join(MANIFEST_NAME);
    fs::write(&target_manifest, manifest_content)
        .map_err(|e| format!("Failed to write published manifest {}: {e}", target_manifest.display()))?;

    let record = LocalPublishRecord {
        name: manifest.package.name.clone(),
        version: manifest.package.version.clone(),
        checksum: signature.checksum.clone(),
        manifest_path: target_manifest.display().to_string(),
        signature,
    };
    let record_path = publish_dir.join("publish.json");
    fs::write(
        &record_path,
        serde_json::to_string_pretty(&record)
            .map_err(|e| format!("Failed to serialize local publish record: {e}"))?,
    )
    .map_err(|e| format!("Failed to write local publish record {}: {e}", record_path.display()))?;

    Ok(target_manifest)
}

fn publish_to_remote_registry(
    base_url: &str,
    manifest: &OmniManifest,
    manifest_content: &str,
) -> Result<(), String> {
    let signature = manifest.signature.clone().ok_or_else(|| {
        format!(
            "Refusing to publish unsigned manifest '{}' v{} to remote registry",
            manifest.package.name, manifest.package.version
        )
    })?;

    let payload = PublishPayload {
        name: manifest.package.name.clone(),
        version: manifest.package.version.clone(),
        checksum: signature.checksum.clone(),
        manifest: manifest_content.to_string(),
        signature,
    };

    let endpoint = format!("{}/publish", base_url.trim_end_matches('/'));
    let client = reqwest::blocking::Client::new();
    client
        .post(endpoint)
        .json(&payload)
        .send()
        .map_err(|e| format!("Failed to publish package to remote registry: {e}"))?
        .error_for_status()
        .map_err(|e| format!("Remote registry rejected publish request: {e}"))?;

    Ok(())
}

fn resolve_cached_registry_version(root: &Path, package_name: &str, version_req: &str) -> String {
    let default_version = normalize_lock_version(version_req);
    let package_root = root.join(package_name);
    if !package_root.exists() {
        return default_version;
    }

    let requirement = match semver::VersionReq::parse(version_req) {
        Ok(req) => req,
        Err(_) => return default_version,
    };

    let mut candidates = Vec::new();
    let entries = match fs::read_dir(&package_root) {
        Ok(entries) => entries,
        Err(_) => return default_version,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        let Ok(version) = Version::parse(name) else {
            continue;
        };
        if requirement.matches(&version) {
            candidates.push(version);
        }
    }

    candidates.sort();
    candidates
        .pop()
        .map(|version| version.to_string())
        .unwrap_or(default_version)
}

fn insert_resolved_package(
    packages: &mut BTreeMap<String, ResolvedInstallPackage>,
    name: &str,
    candidate: ResolvedInstallPackage,
) -> Result<(), String> {
    if let Some(existing) = packages.get_mut(name) {
        if existing.version != candidate.version || existing.source != candidate.source {
            return Err(format!(
                "Conflict for dependency '{}': {} from {} vs {} from {}",
                name, existing.version, existing.source, candidate.version, candidate.source
            ));
        }

        existing.dependencies = merge_dependency_edges(&existing.dependencies, &candidate.dependencies);
        return Ok(());
    }

    packages.insert(name.to_string(), candidate);
    Ok(())
}

fn collect_lock_packages(
    manifest_dir: &Path,
    dependencies: &toml::map::Map<String, toml::Value>,
    packages: &mut BTreeMap<String, ResolvedInstallPackage>,
    stack: &mut Vec<String>,
) -> Result<Vec<String>, String> {
    let mut resolved_names = Vec::new();
    let dependency_names = sorted_unique(dependencies.keys().cloned());

    for dependency_name in dependency_names {
        let dependency_value = dependencies
            .get(&dependency_name)
            .ok_or_else(|| format!("Dependency '{dependency_name}' disappeared during resolution"))?;
        let (version_req, source) = dependency_request(dependency_value);

        if let Some(path_str) = source.strip_prefix("path+") {
            let dependency_dir = manifest_dir
                .join(path_str)
                .canonicalize()
                .map_err(|e| {
                    format!(
                        "Failed to resolve path dependency '{}' from {}: {e}",
                        dependency_name,
                        manifest_dir.display()
                    )
                })?;

            let dependency_manifest_path = dependency_dir.join(MANIFEST_NAME);
            let dependency_manifest = read_manifest_verified(&dependency_manifest_path, false).map_err(|e| {
                format!(
                    "Failed to load path dependency '{}' from {}: {}",
                    dependency_name,
                    dependency_manifest_path.display(),
                    e
                )
            })?;

            let package_name = dependency_manifest.package.name.clone();
            if stack.contains(&package_name) {
                let mut cycle = stack.clone();
                cycle.push(package_name.clone());
                return Err(format!(
                    "Cyclic path dependency detected: {}",
                    cycle.join(" -> ")
                ));
            }

            stack.push(package_name.clone());
            let child_dependencies = collect_lock_packages(
                &dependency_dir,
                &dependency_manifest.dependencies,
                packages,
                stack,
            )?;
            stack.pop();

            insert_resolved_package(
                packages,
                &package_name,
                ResolvedInstallPackage {
                    version: dependency_manifest.package.version,
                    source: lock_source_for_path(&dependency_dir),
                    dependencies: child_dependencies.clone(),
                },
            )?;
            resolved_names.push(package_name);
        } else {
            let pinned = if source == "registry+omni" {
                if let Some(root) = registry_root() {
                    resolve_cached_registry_version(&root, &dependency_name, &version_req)
                } else if let Some(base_url) = registry_url() {
                    resolve_remote_registry_version(&base_url, &dependency_name, &version_req)
                } else {
                    normalize_lock_version(&version_req)
                }
            } else {
                normalize_lock_version(&version_req)
            };
            let mut resolved_name = dependency_name.clone();
            let mut registry_children = Vec::new();

            if source == "registry+omni" {
                if let Some(root) = registry_root() {
                    let manifest_path = registry_manifest_path(&root, &dependency_name, &pinned);
                    if manifest_path.exists() {
                        let registry_manifest = read_manifest_verified(&manifest_path, true).map_err(|e| {
                            format!(
                                "Failed to load registry package '{}' at {}: {}",
                                dependency_name,
                                manifest_path.display(),
                                e
                            )
                        })?;
                        resolved_name = registry_manifest.package.name.clone();

                        if stack.contains(&resolved_name) {
                            let mut cycle = stack.clone();
                            cycle.push(resolved_name.clone());
                            return Err(format!(
                                "Cyclic registry dependency detected: {}",
                                cycle.join(" -> ")
                            ));
                        }

                        stack.push(resolved_name.clone());
                        registry_children = collect_lock_packages(
                            manifest_path
                                .parent()
                                .ok_or_else(|| {
                                    format!(
                                        "Registry manifest path has no parent: {}",
                                        manifest_path.display()
                                    )
                                })?,
                            &registry_manifest.dependencies,
                            packages,
                            stack,
                        )?;
                        stack.pop();
                    } else if let Some(base_url) = registry_url() {
                        let registry_manifest = fetch_remote_registry_manifest(
                            &base_url,
                            &dependency_name,
                            &pinned,
                        )?;

                        resolved_name = registry_manifest.package.name.clone();

                        if stack.contains(&resolved_name) {
                            let mut cycle = stack.clone();
                            cycle.push(resolved_name.clone());
                            return Err(format!(
                                "Cyclic registry dependency detected: {}",
                                cycle.join(" -> ")
                            ));
                        }

                        stack.push(resolved_name.clone());
                        registry_children = collect_lock_packages(
                            manifest_dir,
                            &registry_manifest.dependencies,
                            packages,
                            stack,
                        )?;
                        stack.pop();
                    }
                } else if let Some(base_url) = registry_url() {
                    let registry_manifest = fetch_remote_registry_manifest(
                        &base_url,
                        &dependency_name,
                        &pinned,
                    )?;

                    resolved_name = registry_manifest.package.name.clone();

                    if stack.contains(&resolved_name) {
                        let mut cycle = stack.clone();
                        cycle.push(resolved_name.clone());
                        return Err(format!(
                            "Cyclic registry dependency detected: {}",
                            cycle.join(" -> ")
                        ));
                    }

                    stack.push(resolved_name.clone());
                    registry_children = collect_lock_packages(
                        manifest_dir,
                        &registry_manifest.dependencies,
                        packages,
                        stack,
                    )?;
                    stack.pop();
                }
            }

            insert_resolved_package(
                packages,
                &resolved_name,
                ResolvedInstallPackage {
                    version: pinned.clone(),
                    source: if source == "registry+omni" {
                        registry_source_label(&resolved_name, &pinned)
                    } else {
                        source
                    },
                    dependencies: registry_children.clone(),
                },
            )?;
            resolved_names.push(resolved_name);
        }
    }

    Ok(sorted_unique(resolved_names))
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
        workspace: None,
        signature: None,
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

fn cmd_install(locked: bool) -> Result<(), String> {
    let path = find_manifest()?;
    let manifest = read_manifest_verified(&path, false)?;
    let manifest_dir = path
        .parent()
        .ok_or_else(|| "Manifest path has no parent directory".to_string())?;
    let lock_path = path
        .parent()
        .ok_or_else(|| "Manifest path has no parent directory".to_string())?
        .join(LOCKFILE_NAME);

    if locked {
        if !lock_path.exists() {
            return Err(format!(
                "--locked requires an existing {} at {}",
                LOCKFILE_NAME,
                lock_path.display()
            ));
        }

        let lockfile = advanced::LockFile::load(&lock_path)
            .map_err(|e| format!("Failed to read existing {}: {e}", LOCKFILE_NAME))?;

        if lockfile.packages.is_empty() && !manifest.dependencies.is_empty() {
            return Err(format!(
                "{} is empty but manifest declares dependencies; run `opm install` first",
                LOCKFILE_NAME
            ));
        }

        println!("Installing dependencies from locked {}:", lock_path.display());
        for package in &lockfile.packages {
            println!(
                "  📦 {} = \"{}\" ({})",
                package.name, package.version, package.source
            );
        }
        println!("✓ Locked install complete");
        return Ok(());
    }

    let mut resolved_packages = BTreeMap::new();
    let mut stack = vec![manifest.package.name.clone()];
    let direct_dependencies = collect_lock_packages(
        manifest_dir,
        &manifest.dependencies,
        &mut resolved_packages,
        &mut stack,
    )?;

    println!("Installing dependencies:");
    for name in &direct_dependencies {
        if let Some(pkg) = resolved_packages.get(name) {
            println!("  📦 {name} = \"{}\" ({})", pkg.version, pkg.source);
        }
    }

    let lockfile = advanced::LockFile {
        version: 1,
        packages: resolved_packages
            .into_iter()
            .map(|(name, pkg)| advanced::LockedPackage {
                checksum: checksum_for(&name, &pkg.version),
                name,
                version: pkg.version,
                source: pkg.source,
                dependencies: pkg.dependencies,
            })
            .collect(),
    };

    lockfile
        .save(&lock_path)
        .map_err(|e| format!("Failed to write {LOCKFILE_NAME}: {e}"))?;

    if direct_dependencies.is_empty() {
        println!("No dependencies to install.");
    }
    println!("✓ Install complete; wrote {}", lock_path.display());
    Ok(())
}

fn build_output_path(project_root: &Path, package_name: &str, release: bool) -> PathBuf {
    let profile = if release { "release" } else { "debug" };
    project_root
        .join("build")
        .join(profile)
        .join(package_name)
        .with_extension("ovm")
}

fn workspace_build_order(
    workspace: &advanced::Workspace,
    selected_paths: &[PathBuf],
) -> Result<Vec<PathBuf>, String> {
    let mut members_by_name: BTreeMap<String, &advanced::WorkspaceMember> = BTreeMap::new();
    let mut selected_names = BTreeSet::new();

    for member in &workspace.members {
        let package_name = member.manifest.package.name.clone();
        if members_by_name.insert(package_name.clone(), member).is_some() {
            return Err(format!(
                "Workspace contains duplicate package name '{package_name}'"
            ));
        }
    }

    for path in selected_paths {
        let member = workspace
            .members
            .iter()
            .find(|candidate| candidate.path == *path)
            .ok_or_else(|| format!("Workspace member '{}' was not loaded", path.display()))?;
        selected_names.insert(member.manifest.package.name.clone());
    }

    if selected_names.is_empty() {
        return Ok(Vec::new());
    }

    let workspace_names: BTreeSet<String> = members_by_name.keys().cloned().collect();
    let mut required_names = selected_names.clone();

    let mut changed = true;
    while changed {
        changed = false;
        let snapshot: Vec<String> = required_names.iter().cloned().collect();

        for package_name in snapshot {
            let member = members_by_name.get(&package_name).ok_or_else(|| {
                format!("Workspace member '{package_name}' was selected but not loaded")
            })?;

            if let Some(dependencies) = &member.manifest.dependencies {
                for dependency_name in dependencies.keys() {
                    if workspace_names.contains(dependency_name)
                        && required_names.insert(dependency_name.clone())
                    {
                        changed = true;
                    }
                }
            }
        }
    }

    let mut indegree: BTreeMap<String, usize> = required_names
        .iter()
        .map(|name| (name.clone(), 0usize))
        .collect();
    let mut dependents: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();

    for package_name in &required_names {
        let member = members_by_name.get(package_name).ok_or_else(|| {
            format!("Workspace member '{package_name}' was selected but not loaded")
        })?;

        if let Some(dependencies) = &member.manifest.dependencies {
            let dependency_names = sorted_unique(dependencies.keys().cloned());
            for dependency_name in dependency_names {
                if required_names.contains(&dependency_name) {
                    dependents
                        .entry(dependency_name.clone())
                        .or_default()
                        .insert(package_name.clone());
                    *indegree.get_mut(package_name).unwrap() += 1;
                }
            }
        }
    }

    let mut ready = indegree
        .iter()
        .filter(|(_, degree)| **degree == 0)
        .map(|(name, _)| name.clone())
        .collect::<BTreeSet<_>>();
    let mut ordered_paths = Vec::new();

    while let Some(package_name) = ready.iter().next().cloned() {
        ready.remove(&package_name);

        let member = members_by_name.get(&package_name).ok_or_else(|| {
            format!("Workspace member '{package_name}' was selected but not loaded")
        })?;
        ordered_paths.push(member.path.clone());

        if let Some(next_packages) = dependents.get(&package_name) {
            for next_package in next_packages {
                let degree = indegree
                    .get_mut(next_package)
                    .expect("dependent package must exist in indegree map");
                *degree -= 1;
                if *degree == 0 {
                    ready.insert(next_package.clone());
                }
            }
        }
    }

    if ordered_paths.len() != required_names.len() {
        let unresolved = indegree
            .into_iter()
            .filter(|(_, degree)| *degree > 0)
            .map(|(name, _)| name)
            .collect::<Vec<_>>();
        return Err(format!(
            "Workspace dependency cycle detected among: {}",
            unresolved.join(" -> ")
        ));
    }

    Ok(ordered_paths)
}

fn resolve_build_units(project_root: &Path) -> Result<Vec<(PathBuf, String)>, String> {
    let workspace = advanced::Workspace::discover(project_root)?;
    if workspace.members.is_empty() {
        let manifest_path = project_root.join(MANIFEST_NAME);
        let manifest = read_manifest(&manifest_path)?;
        return Ok(vec![(project_root.to_path_buf(), manifest.package.name)]);
    }

    let selected_paths = if workspace.default_members.is_empty() {
        workspace
            .members
            .iter()
            .map(|member| member.path.clone())
            .collect::<Vec<_>>()
    } else {
        workspace.default_members.clone()
    };

    let ordered_paths = workspace_build_order(&workspace, &selected_paths)?;
    let mut units = Vec::new();
    for path in ordered_paths {
        let member = workspace
            .members
            .iter()
            .find(|member| member.path == path)
            .ok_or_else(|| {
                format!(
                    "Workspace member '{}' selected for build but not loaded",
                    path.display()
                )
            })?;
        units.push((path, member.manifest.package.name.clone()));
    }
    Ok(units)
}

fn build_script_env_pairs(output: &advanced::BuildOutput) -> Vec<(String, String)> {
    let mut vars = Vec::new();

    vars.push((
        "OMNI_TARGET_TRIPLE".to_string(),
        current_target_triple(),
    ));

    if !output.cfg_flags.is_empty() {
        vars.push((
            "OMNI_CFG_FLAGS".to_string(),
            output.cfg_flags.join(";"),
        ));
    }

    if !output.link_libs.is_empty() {
        vars.push((
            "OMNI_LINK_LIBS".to_string(),
            output.link_libs.join(";"),
        ));
    }

    if !output.link_paths.is_empty() {
        let paths = output
            .link_paths
            .iter()
            .map(|path| path.display().to_string())
            .collect::<Vec<_>>()
            .join(";");
        vars.push(("OMNI_LINK_PATHS".to_string(), paths));
    }

    if let Some(artifact) = &output.native_artifact {
        vars.push((
            "OMNI_NATIVE_ARTIFACT_KIND".to_string(),
            match artifact.kind {
                advanced::NativeArtifactKind::Object => "object".to_string(),
                advanced::NativeArtifactKind::Bitcode => "bitcode".to_string(),
            },
        ));
        vars.push((
            "OMNI_NATIVE_ARTIFACT_TARGET_TRIPLE".to_string(),
            artifact.target_triple.clone(),
        ));
        vars.push((
            "OMNI_NATIVE_ARTIFACT_PATH".to_string(),
            artifact.path.display().to_string(),
        ));
        if let Some(checksum) = &artifact.checksum {
            vars.push((
                "OMNI_NATIVE_ARTIFACT_CHECKSUM".to_string(),
                checksum.clone(),
            ));
        }
    }

    vars
}

fn apply_build_script_env(cmd: &mut Command, output: &advanced::BuildOutput) {
    for (key, value) in build_script_env_pairs(output) {
        cmd.env(key, value);
    }
}

fn sidecar_path_for_output(output_path: &Path) -> PathBuf {
    let file_name = output_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("output");
    output_path.with_file_name(format!("{file_name}.link"))
}

fn parse_link_sidecar(path: &Path) -> Result<(Vec<String>, Vec<String>), String> {
    let content = fs::read_to_string(path)
        .map_err(|e| format!("Failed to read sidecar {}: {e}", path.display()))?;

    let mut libs = Vec::new();
    let mut paths = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        if let Some(value) = trimmed.strip_prefix("link_lib=") {
            libs.push(value.trim().to_string());
            continue;
        }

        if let Some(value) = trimmed.strip_prefix("link_path=") {
            paths.push(value.trim().to_string());
        }
    }

    Ok((libs, paths))
}

fn verify_link_sidecar(
    output_path: &Path,
    build_output: &advanced::BuildOutput,
) -> Result<(), String> {
    if build_output.link_libs.is_empty() && build_output.link_paths.is_empty() {
        return Ok(());
    }

    let sidecar_path = sidecar_path_for_output(output_path);
    if !sidecar_path.exists() {
        return Err(format!(
            "Compiler did not emit expected link sidecar at {}",
            sidecar_path.display()
        ));
    }

    let (libs, paths) = parse_link_sidecar(&sidecar_path)?;
    for expected_lib in &build_output.link_libs {
        if !libs.iter().any(|value| value == expected_lib) {
            return Err(format!(
                "Missing link library directive '{expected_lib}' in {}",
                sidecar_path.display()
            ));
        }
    }

    let path_strings = build_output
        .link_paths
        .iter()
        .map(|path| path.display().to_string())
        .collect::<Vec<_>>();
    for expected_path in &path_strings {
        if !paths.iter().any(|value| value == expected_path) {
            return Err(format!(
                "Missing link path directive '{expected_path}' in {}",
                sidecar_path.display()
            ));
        }
    }

    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
struct BuildArtifactManifest {
    package: String,
    profile: String,
    output: String,
    cfg_flags: Vec<String>,
    link_libs: Vec<String>,
    link_paths: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    native_artifact: Option<advanced::NativeArtifactMetadata>,
    #[serde(default)]
    native_link_plan: Option<String>,
    #[serde(default)]
    native_link_report: Option<String>,
    #[serde(default)]
    native_link_status: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct NativeLinkPlan {
    package: String,
    profile: String,
    linker: String,
    input: String,
    output: String,
    args: Vec<String>,
    link_libs: Vec<String>,
    link_paths: Vec<String>,
    artifact: advanced::NativeArtifactMetadata,
}

#[derive(Debug, Serialize, Deserialize)]
struct NativeLinkExecutionReport {
    package: String,
    profile: String,
    linker: String,
    input: String,
    output: String,
    args: Vec<String>,
    status_code: Option<i32>,
    capture_mode: bool,
}

fn write_build_artifact_manifest(
    project_root: &Path,
    package_name: &str,
    release: bool,
    output_path: &Path,
    cfg_flags: &[String],
    link_libs: &[String],
    link_paths: &[String],
    native_artifact: Option<&advanced::NativeArtifactMetadata>,
    native_link_plan: Option<&Path>,
    native_link_report: Option<&Path>,
    native_link_status: &str,
) -> Result<PathBuf, String> {
    let profile = if release { "release" } else { "debug" };
    let manifest_path = project_root
        .join("build")
        .join(profile)
        .join(format!("{package_name}.artifact.json"));

    let payload = BuildArtifactManifest {
        package: package_name.to_string(),
        profile: profile.to_string(),
        output: output_path.display().to_string(),
        cfg_flags: cfg_flags.to_vec(),
        link_libs: link_libs.to_vec(),
        link_paths: link_paths.to_vec(),
        native_artifact: native_artifact.cloned(),
        native_link_plan: native_link_plan.map(|path| path.display().to_string()),
        native_link_report: native_link_report.map(|path| path.display().to_string()),
        native_link_status: native_link_status.to_string(),
    };

    if let Some(parent) = manifest_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create artifact manifest directory: {e}"))?;
    }

    let content = serde_json::to_string_pretty(&payload)
        .map_err(|e| format!("Failed to serialize artifact manifest: {e}"))?;
    fs::write(&manifest_path, content)
        .map_err(|e| format!("Failed to write artifact manifest {}: {e}", manifest_path.display()))?;

    Ok(manifest_path)
}

fn native_link_output_path(project_root: &Path, package_name: &str, release: bool) -> PathBuf {
    let profile = if release { "release" } else { "debug" };
    let extension = if cfg!(windows) { "exe" } else { "bin" };
    project_root
        .join("build")
        .join(profile)
        .join(format!("{package_name}.native"))
        .with_extension(extension)
}

fn normalize_link_lib_flag(value: &str) -> String {
    let lib = value
        .split_once('=')
        .map(|(_, rest)| rest)
        .unwrap_or(value)
        .trim();
    format!("-l{lib}")
}

fn build_native_link_args(
    input: &Path,
    output: &Path,
    link_libs: &[String],
    link_paths: &[String],
) -> Vec<String> {
    let mut args = Vec::new();
    args.push(input.display().to_string());
    args.push("-o".to_string());
    args.push(output.display().to_string());

    for link_path in link_paths {
        args.push(format!("-L{link_path}"));
    }

    for lib in link_libs {
        args.push(normalize_link_lib_flag(lib));
    }

    args
}

fn truthy_env(name: &str) -> bool {
    matches!(
        std::env::var(name).ok().as_deref().map(str::trim),
        Some("1") | Some("true") | Some("TRUE") | Some("yes") | Some("on")
    )
}

fn falsey_env(name: &str) -> bool {
    matches!(
        std::env::var(name).ok().as_deref().map(str::trim),
        Some("0") | Some("false") | Some("FALSE") | Some("no") | Some("off")
    )
}

fn should_execute_native_link() -> bool {
    !falsey_env("OMNI_NATIVE_LINK_EXECUTE")
}

fn strict_native_link_execution() -> bool {
    truthy_env("OMNI_NATIVE_LINK_STRICT")
}

fn host_target_triple() -> String {
    static HOST_TARGET_TRIPLE: OnceLock<String> = OnceLock::new();

    HOST_TARGET_TRIPLE
        .get_or_init(|| {
            if let Ok(output) = Command::new("rustc").arg("-Vv").output() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                if let Some(host_line) = stdout.lines().find(|line| line.starts_with("host: ")) {
                    return host_line.trim_start_matches("host: ").trim().to_string();
                }
            }

            format!("{}-unknown-{}", std::env::consts::ARCH, std::env::consts::OS)
        })
        .clone()
}

fn current_target_triple() -> String {
    std::env::var("OMNI_TARGET_TRIPLE")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .or_else(|| {
            std::env::var("TARGET")
                .ok()
                .filter(|value| !value.trim().is_empty())
        })
        .or_else(|| {
            std::env::var("HOST")
                .ok()
                .filter(|value| !value.trim().is_empty())
        })
        .unwrap_or_else(host_target_triple)
}

fn validate_native_artifact_target(
    artifact: &advanced::NativeArtifactMetadata,
) -> Result<(), String> {
    let expected = current_target_triple();
    if artifact.target_triple.trim() != expected {
        return Err(format!(
            "No plot armor: native artifact targets '{}' but opm is compiling for '{}'. The linker refuses this crossover arc.",
            artifact.target_triple,
            expected
        ));
    }

    Ok(())
}

fn native_artifact_is_linkable(artifact: &advanced::NativeArtifactMetadata) -> bool {
    matches!(artifact.kind, advanced::NativeArtifactKind::Object)
}

fn native_link_plan_path(project_root: &Path, package_name: &str, release: bool) -> PathBuf {
    let profile = if release { "release" } else { "debug" };
    project_root
        .join("build")
        .join(profile)
        .join(format!("{package_name}.native-link-plan.json"))
}

fn native_link_execution_report_path(
    project_root: &Path,
    package_name: &str,
    release: bool,
) -> PathBuf {
    let profile = if release { "release" } else { "debug" };
    project_root
        .join("build")
        .join(profile)
        .join(format!("{package_name}.native-link-execution.json"))
}

fn write_native_link_plan(
    project_root: &Path,
    package_name: &str,
    release: bool,
    payload: &NativeLinkPlan,
) -> Result<PathBuf, String> {
    if payload.link_libs.is_empty() && payload.link_paths.is_empty() {
        return Err("native link plan requested without any link directives".to_string());
    }

    let manifest_path = native_link_plan_path(project_root, package_name, release);

    if let Some(parent) = manifest_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create native link plan directory: {e}"))?;
    }

    let content = serde_json::to_string_pretty(payload)
        .map_err(|e| format!("Failed to serialize native link plan: {e}"))?;
    fs::write(&manifest_path, content)
        .map_err(|e| format!("Failed to write native link plan {}: {e}", manifest_path.display()))?;

    Ok(manifest_path)
}

fn execute_native_link_plan(
    project_root: &Path,
    package_name: &str,
    release: bool,
    payload: &NativeLinkPlan,
) -> Result<PathBuf, String> {
    let profile = if release { "release" } else { "debug" };
    let report_path = native_link_execution_report_path(project_root, package_name, release);

    validate_native_artifact_target(&payload.artifact)?;

    if !native_artifact_is_linkable(&payload.artifact) {
        return Err(format!(
            "Native artifact '{}' is {:?}; native linker only accepts object artifacts in this stage",
            payload.artifact.path.display(),
            payload.artifact.kind
        ));
    }

    if payload.input.trim() != payload.artifact.path.display().to_string() {
        return Err(format!(
            "Native link plan input '{}' does not match artifact path '{}'",
            payload.input,
            payload.artifact.path.display()
        ));
    }

    if !truthy_env("OMNI_NATIVE_LINK_FORCE") && !payload.artifact.path.exists() {
        return Err(format!(
            "Native link skipped: artifact '{}' does not exist (set OMNI_NATIVE_LINK_FORCE=1 to force)",
            payload.artifact.path.display()
        ));
    }

    if let Some(capture_path) = std::env::var_os("OMNI_NATIVE_LINK_CAPTURE") {
        let capture_report_path = PathBuf::from(&capture_path);
        let report = NativeLinkExecutionReport {
            package: package_name.to_string(),
            profile: profile.to_string(),
            linker: payload.linker.clone(),
            input: payload.input.clone(),
            output: payload.output.clone(),
            args: payload.args.clone(),
            status_code: Some(0),
            capture_mode: true,
        };

        let content = serde_json::to_string_pretty(&report)
            .map_err(|e| format!("Failed to serialize native link execution report: {e}"))?;
        fs::write(&capture_path, content).map_err(|e| {
            format!(
                "Failed to write native link capture {}: {e}",
                capture_report_path.display()
            )
        })?;
        return Ok(capture_report_path);
    }

    let status = Command::new(&payload.linker)
        .args(&payload.args)
        .status()
        .map_err(|e| format!("Failed to invoke native linker '{}': {e}", payload.linker))?;

    let report = NativeLinkExecutionReport {
        package: package_name.to_string(),
        profile: profile.to_string(),
        linker: payload.linker.clone(),
        input: payload.input.clone(),
        output: payload.output.clone(),
        args: payload.args.clone(),
        status_code: status.code(),
        capture_mode: false,
    };

    if let Some(parent) = report_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create native link report directory: {e}"))?;
    }
    let content = serde_json::to_string_pretty(&report)
        .map_err(|e| format!("Failed to serialize native link execution report: {e}"))?;
    fs::write(&report_path, content)
        .map_err(|e| format!("Failed to write native link execution report {}: {e}", report_path.display()))?;

    if !status.success() {
        return Err(format!(
            "Native linker '{}' failed with exit code {:?}",
            payload.linker,
            status.code()
        ));
    }

    Ok(report_path)
}

fn build_single_unit(project_root: &Path, package_name: &str, release: bool) -> Result<(), String> {
    let input = project_root.join("main.omni");
    if !input.exists() {
        return Err(format!("No main.omni found at {}", input.display()));
    }

    let output_path = build_output_path(project_root, package_name, release);
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create build output directory: {e}"))?;
    }

    let build_script_output = advanced::BuildScriptRunner::run(project_root)?;
    if !build_script_output.link_libs.is_empty()
        || !build_script_output.link_paths.is_empty()
        || !build_script_output.cfg_flags.is_empty()
    {
        info!(
            "Build script directives ({}): libs={:?}, link_paths={:?}, cfg_flags={:?}",
            package_name,
            build_script_output.link_libs,
            build_script_output.link_paths,
            build_script_output.cfg_flags
        );
    }

    let mode = if release { "--release" } else { "--debug" };
    println!(
        "Building '{}' ({mode}) from {}…",
        package_name,
        input.display()
    );

    let native_artifact = build_script_output.native_artifact.clone();
    if (build_script_output.link_libs.is_empty() && build_script_output.link_paths.is_empty())
        && native_artifact.is_some()
    {
        info!(
            "Build script emitted native artifact metadata for {} but no link directives; skipping native link",
            package_name
        );
    }

    let mut cmd = Command::new("omnc");
    cmd.current_dir(project_root)
        .arg(&input)
        .arg("--output")
        .arg(&output_path)
        .arg("-O")
        .arg(if release { "3" } else { "0" });
    apply_build_script_env(&mut cmd, &build_script_output);

    let status = cmd
        .status()
        .map_err(|e| format!("Failed to invoke omnc: {e}"))?;

    if status.success() {
        verify_link_sidecar(&output_path, &build_script_output)?;

        let (resolved_link_libs, resolved_link_paths) = if build_script_output.link_libs.is_empty()
            && build_script_output.link_paths.is_empty()
        {
            (Vec::new(), Vec::new())
        } else {
            parse_link_sidecar(&sidecar_path_for_output(&output_path))?
        };

        println!("✓ Build succeeded: {}", output_path.display());

        let mut native_link_plan_path: Option<PathBuf> = None;
        let mut native_link_report_path: Option<PathBuf> = None;
        let mut native_link_status = "not-requested".to_string();

        if !resolved_link_libs.is_empty() || !resolved_link_paths.is_empty() {
            let artifact = native_artifact.as_ref().ok_or_else(|| {
                format!(
                    "Build script for '{}' emitted link directives without explicit native artifact metadata",
                    package_name
                )
            })?;

            validate_native_artifact_target(artifact)?;

            let native_link_plan = NativeLinkPlan {
                package: package_name.to_string(),
                profile: if release { "release" } else { "debug" }.to_string(),
                linker: std::env::var("OMNI_NATIVE_LINKER").unwrap_or_else(|_| "cc".to_string()),
                input: artifact.path.display().to_string(),
                output: native_link_output_path(project_root, package_name, release)
                    .display()
                    .to_string(),
                args: build_native_link_args(
                    &artifact.path,
                    &native_link_output_path(project_root, package_name, release),
                    &resolved_link_libs,
                    &resolved_link_paths,
                ),
                link_libs: resolved_link_libs.clone(),
                link_paths: resolved_link_paths.clone(),
                artifact: artifact.clone(),
            };

            let plan_path = write_native_link_plan(
                project_root,
                package_name,
                release,
                &native_link_plan,
            )?;
            native_link_plan_path = Some(plan_path.clone());

            if should_execute_native_link() {
                match execute_native_link_plan(project_root, package_name, release, &native_link_plan) {
                    Ok(execution_report_path) => {
                        native_link_report_path = Some(execution_report_path.clone());
                        native_link_status = "succeeded".to_string();
                        println!(
                            "✓ Native link execution report: {}",
                            execution_report_path.display()
                        );
                    }
                    Err(error) => {
                        native_link_status = if error.contains("does not look like a native/object artifact") {
                            "skipped-non-linkable".to_string()
                        } else {
                            "failed".to_string()
                        };
                        if strict_native_link_execution() {
                            return Err(format!(
                                "Native link execution failed in strict mode: {error}"
                            ));
                        }
                        eprintln!(
                            "⚠ Native link execution failed (continuing): {error}. Set OMNI_NATIVE_LINK_STRICT=1 to fail the build."
                        );
                    }
                }
            } else {
                native_link_status = "disabled".to_string();
                println!("• Native link execution skipped by OMNI_NATIVE_LINK_EXECUTE=0");
            }

            println!("✓ Native link plan: {}", plan_path.display());
        }

        let artifact_manifest_path = write_build_artifact_manifest(
            project_root,
            package_name,
            release,
            &output_path,
            &build_script_output.cfg_flags,
            &resolved_link_libs,
            &resolved_link_paths,
            native_artifact.as_ref(),
            native_link_plan_path.as_deref(),
            native_link_report_path.as_deref(),
            &native_link_status,
        )?;
        println!(
            "✓ Build artifact manifest: {}",
            artifact_manifest_path.display()
        );
        Ok(())
    } else {
        Err(format!(
            "Build failed for '{}' with exit code {:?}",
            package_name,
            status.code()
        ))
    }
}

fn cmd_build(release: bool) -> Result<(), String> {
    let manifest_path = find_manifest()?;
    let project_root = manifest_path
        .parent()
        .ok_or_else(|| "Manifest path has no parent directory".to_string())?
        .to_path_buf();
    let build_units = resolve_build_units(&project_root)?;
    for (unit_root, package_name) in build_units {
        build_single_unit(&unit_root, &package_name, release)?;
    }

    Ok(())
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
    let manifest = sign_manifest(read_manifest(&path)?)?;
    let manifest_content = toml::to_string_pretty(&manifest)
        .map_err(|e| format!("Failed to serialize signed manifest for publishing: {e}"))?;

    println!(
        "Publishing {} v{} …",
        manifest.package.name, manifest.package.version
    );

    if let Some(base_url) = registry_url() {
        publish_to_remote_registry(&base_url, &manifest, &manifest_content)?;
        println!("✓ Published to remote registry: {base_url}");
        return Ok(());
    }

    if let Some(root) = registry_root() {
        let target = publish_to_local_registry(&root, &manifest, &manifest_content)?;
        println!("✓ Published to local registry cache: {}", target.display());
        return Ok(());
    }

    Err(
        "No registry configured. Set OPM_REGISTRY_ROOT for local publish or OPM_REGISTRY_URL for remote publish."
            .to_string(),
    )
}

fn cmd_search(query: &str) -> Result<(), String> {
    println!("Searching for '{query}'…");

    let mut results = Vec::new();
    if let Some(root) = registry_root() {
        results.extend(search_local_registry(&root, query)?);
    }

    if results.is_empty() {
        if let Some(base_url) = registry_url() {
            match search_remote_registry(&base_url, query) {
                Ok(remote_results) => results = remote_results,
                Err(error) => {
                    println!("⚠ Remote registry search unavailable: {error}");
                }
            }
        }
    }

    if results.is_empty() {
        println!("  No results.");
    } else {
        for package in results {
            println!("  {package}");
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::sync::{Mutex, OnceLock};
    use std::thread;

    fn global_test_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

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

    const TEST_SIGNING_KEY_BASE64: &str = "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=";

    fn set_test_signing_key() -> Option<String> {
        let previous = std::env::var("OPM_PUBLISH_PRIVATE_KEY").ok();
        std::env::set_var("OPM_PUBLISH_PRIVATE_KEY", TEST_SIGNING_KEY_BASE64);
        previous
    }

    fn restore_test_signing_key(previous: Option<String>) {
        if let Some(value) = previous {
            std::env::set_var("OPM_PUBLISH_PRIVATE_KEY", value);
        } else {
            std::env::remove_var("OPM_PUBLISH_PRIVATE_KEY");
        }
    }

    fn signed_manifest_string(manifest_toml: &str) -> String {
        let previous_key = set_test_signing_key();
        let manifest: OmniManifest = toml::from_str(manifest_toml).expect("parse manifest for signing");
        let signed_manifest = sign_manifest(manifest).expect("sign test manifest");
        restore_test_signing_key(previous_key);
        toml::to_string_pretty(&signed_manifest).expect("serialize signed manifest")
    }

    fn write_signed_manifest(path: &Path, manifest_toml: &str) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("create signed manifest parent directory");
        }
        fs::write(path, signed_manifest_string(manifest_toml)).expect("write signed manifest");
    }

    fn sample_native_artifact(path: &Path) -> advanced::NativeArtifactMetadata {
        advanced::NativeArtifactMetadata {
            kind: advanced::NativeArtifactKind::Object,
            target_triple: current_target_triple(),
            path: path.to_path_buf(),
            checksum: None,
        }
    }

    #[test]
    fn test_parse_manifest() {
        let toml_content = r#"
[package]
name = "test-pkg"
version = "0.1.0"
authors = ["Test Author"]
description = "A test package"
edition = "2021"
"#;
        let manifest: OmniManifest = toml::from_str(toml_content).unwrap();
        assert_eq!(manifest.package.name, "test-pkg");
        assert_eq!(manifest.package.version, "0.1.0");
    }

    #[test]
    fn test_version_comparison() {
        let v1 = Version::parse("1.0.0").unwrap();
        let v2 = Version::parse("2.0.0").unwrap();
        assert!(v1 < v2);
    }

    #[test]
    fn test_semver_parse() {
        assert!(Version::parse("1.2.3").is_ok());
        assert!(Version::parse("0.1.0-alpha").is_ok());
        assert!(Version::parse("invalid").is_err());
    }

    #[test]
    fn test_manifest_default_values() {
        let toml_content = r#"
[package]
name = "minimal"
version = "0.1.0"
"#;
        let manifest: OmniManifest = toml::from_str(toml_content).unwrap();
        assert_eq!(manifest.package.name, "minimal");
        assert!(manifest.package.description.is_none());
        assert!(manifest.dependencies.is_empty());
    }

    #[test]
    fn test_normalize_lock_version() {
        assert_eq!(normalize_lock_version("^1.2.3"), "1.2.3");
        assert_eq!(normalize_lock_version("~0.8.0"), "0.8.0");
        assert_eq!(normalize_lock_version(">=1.0"), "1.0.0");
        assert_eq!(normalize_lock_version(">=1.2, <2.0"), "1.2.0");
        assert_eq!(normalize_lock_version("*"), "0.0.0");
    }

    #[test]
    fn test_dependency_request_from_table() {
        let mut table = toml::Table::new();
        table.insert(
            "git".to_string(),
            toml::Value::String("https://example.org/x".to_string()),
        );
        table.insert(
            "version".to_string(),
            toml::Value::String("^1.2.0".to_string()),
        );
        let dep = toml::Value::Table(table);

        let (version, source) = dependency_request(&dep);
        assert_eq!(version, "^1.2.0");
        assert_eq!(source, "git+https://example.org/x");
    }

    #[test]
    fn test_cmd_install_writes_lockfile() {
        let _guard = global_test_lock().lock().expect("lock");
        let old_cwd = std::env::current_dir().expect("cwd");
        let root = unique_temp_dir("opm-install-test");
        let app_dir = root.join("app");
        let local_dep_dir = root.join("local_dep");
        let transitive_dep_dir = root.join("transitive_dep");
        let registry_root = root.join("registry-empty");

        fs::create_dir_all(&app_dir).expect("create app dir");
        fs::create_dir_all(&local_dep_dir).expect("create local dep dir");
        fs::create_dir_all(&transitive_dep_dir).expect("create transitive dep dir");
        fs::create_dir_all(&registry_root).expect("create empty registry root");

        fs::write(
            app_dir.join(MANIFEST_NAME),
            r#"[package]
name = "demo"
version = "0.1.0"

[dependencies]
serde = "^1.0.0"
local_dep = { version = "0.2.0", path = "../local_dep" }
"#,
        )
        .expect("write manifest");
        fs::write(
            local_dep_dir.join(MANIFEST_NAME),
            r#"[package]
name = "local_dep"
version = "0.2.0"

[dependencies]
transitive_dep = { path = "../transitive_dep" }
"#,
        )
        .expect("write local dep manifest");
        fs::write(
            transitive_dep_dir.join(MANIFEST_NAME),
            r#"[package]
name = "transitive_dep"
version = "1.5.0"
"#,
        )
        .expect("write transitive dep manifest");

        std::env::set_var("OPM_REGISTRY_ROOT", &registry_root);
        std::env::set_current_dir(&app_dir).expect("set temp cwd");
        let result = cmd_install(false);
        std::env::set_current_dir(&old_cwd).expect("restore cwd");
        std::env::remove_var("OPM_REGISTRY_ROOT");

        assert!(result.is_ok(), "install failed: {:?}", result.err());

        let lock_path = app_dir.join(LOCKFILE_NAME);
        assert!(lock_path.exists(), "lockfile should exist");

        let lockfile = advanced::LockFile::load(&lock_path).expect("load lockfile");
        assert_eq!(lockfile.version, 1);
        assert_eq!(lockfile.packages.len(), 3);
        let package_names: Vec<_> = lockfile.packages.iter().map(|p| p.name.as_str()).collect();
        assert_eq!(package_names, vec!["local_dep", "serde", "transitive_dep"]);
        assert!(lockfile.packages.iter().any(|p| p.name == "serde"));
        assert!(lockfile.packages.iter().any(|p| p.name == "local_dep"));
        assert!(lockfile.packages.iter().any(|p| p.name == "transitive_dep"));
        let local_dep = lockfile
            .packages
            .iter()
            .find(|p| p.name == "local_dep")
            .expect("local dep in lockfile");
        assert_eq!(local_dep.dependencies, vec!["transitive_dep".to_string()]);

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn test_cmd_install_detects_transitive_path_conflicts() {
        let _guard = global_test_lock().lock().expect("lock");
        let old_cwd = std::env::current_dir().expect("cwd");
        let root = unique_temp_dir("opm-conflict-test");
        let app_dir = root.join("app");
        let dep_a_dir = root.join("dep_a");
        let dep_b_dir = root.join("dep_b");
        let shared_v1_dir = root.join("shared_v1");
        let shared_v2_dir = root.join("shared_v2");

        for dir in [&app_dir, &dep_a_dir, &dep_b_dir, &shared_v1_dir, &shared_v2_dir] {
            fs::create_dir_all(dir).expect("create test dir");
        }

        fs::write(
            app_dir.join(MANIFEST_NAME),
            r#"[package]
name = "demo"
version = "0.1.0"

[dependencies]
dep_a = { path = "../dep_a" }
dep_b = { path = "../dep_b" }
"#,
        )
        .expect("write root manifest");
        fs::write(
            dep_a_dir.join(MANIFEST_NAME),
            r#"[package]
name = "dep_a"
version = "0.1.0"

[dependencies]
shared = { path = "../shared_v1" }
"#,
        )
        .expect("write dep_a manifest");
        fs::write(
            dep_b_dir.join(MANIFEST_NAME),
            r#"[package]
name = "dep_b"
version = "0.1.0"

[dependencies]
shared = { path = "../shared_v2" }
"#,
        )
        .expect("write dep_b manifest");
        fs::write(
            shared_v1_dir.join(MANIFEST_NAME),
            r#"[package]
name = "shared"
version = "1.0.0"
"#,
        )
        .expect("write shared_v1 manifest");
        fs::write(
            shared_v2_dir.join(MANIFEST_NAME),
            r#"[package]
name = "shared"
version = "2.0.0"
"#,
        )
        .expect("write shared_v2 manifest");

        std::env::set_current_dir(&app_dir).expect("set temp cwd");
        let result = cmd_install(false);
        std::env::set_current_dir(&old_cwd).expect("restore cwd");

        assert!(
            matches!(result, Err(ref message) if message.contains("Conflict for dependency 'shared'")),
            "expected transitive conflict error, got {:?}",
            result
        );

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn test_cmd_install_resolves_registry_transitive_dependencies_from_cache() {
        let _guard = global_test_lock().lock().expect("lock");
        let old_cwd = std::env::current_dir().expect("cwd");
        let root = unique_temp_dir("opm-registry-test");
        let app_dir = root.join("app");
        let registry_dir = root.join("registry");

        fs::create_dir_all(&app_dir).expect("create app dir");
        fs::create_dir_all(registry_dir.join("serde").join("1.0.0"))
            .expect("create serde registry dir");
        fs::create_dir_all(registry_dir.join("itoa").join("1.0.0"))
            .expect("create itoa registry dir");

        fs::write(
            app_dir.join(MANIFEST_NAME),
            r#"[package]
name = "demo"
version = "0.1.0"

[dependencies]
serde = "^1.0.0"
"#,
        )
        .expect("write app manifest");

        write_signed_manifest(
            &registry_dir.join("serde").join("1.0.0").join(MANIFEST_NAME),
            r#"[package]
name = "serde"
version = "1.0.0"

[dependencies]
itoa = "1.0.0"
"#,
        );

        write_signed_manifest(
            &registry_dir.join("itoa").join("1.0.0").join(MANIFEST_NAME),
            r#"[package]
name = "itoa"
version = "1.0.0"
"#,
        );

        std::env::set_var("OPM_REGISTRY_ROOT", &registry_dir);
        std::env::set_current_dir(&app_dir).expect("set temp cwd");
        let result = cmd_install(false);
        std::env::set_current_dir(&old_cwd).expect("restore cwd");
        std::env::remove_var("OPM_REGISTRY_ROOT");

        assert!(result.is_ok(), "install failed: {:?}", result.err());

        let lockfile = advanced::LockFile::load(&app_dir.join(LOCKFILE_NAME)).expect("load lockfile");
        let serde = lockfile
            .packages
            .iter()
            .find(|package| package.name == "serde")
            .expect("serde package present");
        assert_eq!(serde.dependencies, vec!["itoa".to_string()]);
        assert!(lockfile.packages.iter().any(|package| package.name == "itoa"));

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn test_resolve_cached_registry_version_picks_highest_matching() {
        let root = unique_temp_dir("opm-registry-select");
        let serde_root = root.join("serde");
        fs::create_dir_all(serde_root.join("1.0.0")).expect("create 1.0.0");
        fs::create_dir_all(serde_root.join("1.2.5")).expect("create 1.2.5");
        fs::create_dir_all(serde_root.join("2.0.0")).expect("create 2.0.0");

        let selected = resolve_cached_registry_version(&root, "serde", "^1.0.0");
        assert_eq!(selected, "1.2.5");

        let selected = resolve_cached_registry_version(&root, "serde", ">=1.0, <2.0");
        assert_eq!(selected, "1.2.5");

        let selected = resolve_cached_registry_version(&root, "serde", ">=2.0.0");
        assert_eq!(selected, "2.0.0");

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn test_cmd_install_resolves_registry_transitive_dependencies_from_remote_index() {
        let _guard = global_test_lock().lock().expect("lock");
        let old_cwd = std::env::current_dir().expect("cwd");
        let root = unique_temp_dir("opm-remote-registry-test");
        let app_dir = root.join("app");

        fs::create_dir_all(&app_dir).expect("create app dir");
        fs::write(
            app_dir.join(MANIFEST_NAME),
            r#"[package]
name = "demo"
version = "0.1.0"

[dependencies]
serde = "^1.0.0"
"#,
        )
        .expect("write app manifest");

        let serde_manifest = signed_manifest_string(
            "[package]\nname = \"serde\"\nversion = \"1.2.0\"\n\n[dependencies]\nitoa = \"1.0.0\"\n",
        );
        let itoa_manifest = signed_manifest_string(
            "[package]\nname = \"itoa\"\nversion = \"1.0.0\"\n",
        );

        let listener = TcpListener::bind("127.0.0.1:0").expect("bind test registry");
        let registry_url = format!("http://{}", listener.local_addr().expect("addr"));
        let server = thread::spawn(move || {
            let serde_manifest = serde_manifest;
            let itoa_manifest = itoa_manifest;
            for _ in 0..4 {
                let (mut stream, _) = listener.accept().expect("accept request");
                let mut buffer = [0_u8; 4096];
                let bytes_read = stream.read(&mut buffer).expect("read request");
                let request = String::from_utf8_lossy(&buffer[..bytes_read]);
                let path = request
                    .lines()
                    .next()
                    .and_then(|line| line.split_whitespace().nth(1))
                    .unwrap_or("/");

                let body = match path {
                    "/index/serde" => "1.0.0\n1.2.0\n".to_string(),
                    "/packages/serde/1.2.0/omni.toml" => serde_manifest.clone(),
                    "/index/itoa" => "1.0.0\n".to_string(),
                    "/packages/itoa/1.0.0/omni.toml" => itoa_manifest.clone(),
                    _ => String::new(),
                };

                let status = if body.is_empty() {
                    "404 Not Found"
                } else {
                    "200 OK"
                };
                let response = format!(
                    "HTTP/1.1 {status}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                    body.len()
                );
                stream.write_all(response.as_bytes()).expect("write response");
            }
        });

        std::env::set_var("OPM_REGISTRY_URL", &registry_url);
        std::env::remove_var("OPM_REGISTRY_ROOT");
        std::env::set_current_dir(&app_dir).expect("set temp cwd");

        let result = cmd_install(false);

        std::env::set_current_dir(&old_cwd).expect("restore cwd");
        std::env::remove_var("OPM_REGISTRY_URL");

        assert!(result.is_ok(), "install failed: {:?}", result.err());

        let lockfile = advanced::LockFile::load(&app_dir.join(LOCKFILE_NAME)).expect("load lockfile");
        assert!(lockfile.packages.iter().any(|package| package.name == "serde"));
        assert!(lockfile.packages.iter().any(|package| package.name == "itoa"));
        let serde = lockfile
            .packages
            .iter()
            .find(|package| package.name == "serde")
            .expect("serde package present");
        assert_eq!(serde.version, "1.2.0");
        assert_eq!(serde.dependencies, vec!["itoa".to_string()]);

        server.join().expect("registry server should stop cleanly");
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn test_cmd_install_locked_requires_existing_lockfile() {
        let _guard = global_test_lock().lock().expect("lock");
        let old_cwd = std::env::current_dir().expect("cwd");
        let root = unique_temp_dir("opm-install-locked-missing");
        let app_dir = root.join("app");

        fs::create_dir_all(&app_dir).expect("create app dir");
        fs::write(
            app_dir.join(MANIFEST_NAME),
            r#"[package]
name = "demo"
version = "0.1.0"

[dependencies]
serde = "^1.0.0"
"#,
        )
        .expect("write app manifest");

        std::env::set_current_dir(&app_dir).expect("set temp cwd");
        let result = cmd_install(true);
        std::env::set_current_dir(&old_cwd).expect("restore cwd");

        assert!(
            matches!(result, Err(ref msg) if msg.contains("--locked requires an existing")),
            "expected missing lockfile error, got {:?}",
            result
        );

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn test_cmd_install_locked_uses_existing_lockfile() {
        let _guard = global_test_lock().lock().expect("lock");
        let old_cwd = std::env::current_dir().expect("cwd");
        let root = unique_temp_dir("opm-install-locked");
        let app_dir = root.join("app");

        fs::create_dir_all(&app_dir).expect("create app dir");
        fs::write(
            app_dir.join(MANIFEST_NAME),
            r#"[package]
name = "demo"
version = "0.1.0"

[dependencies]
serde = "^1.0.0"
"#,
        )
        .expect("write app manifest");

        let lock = advanced::LockFile {
            version: 1,
            packages: vec![advanced::LockedPackage {
                name: "serde".to_string(),
                version: "1.2.0".to_string(),
                source: "registry+omni+remote/http://registry.example/serde@1.2.0".to_string(),
                checksum: checksum_for("serde", "1.2.0"),
                dependencies: Vec::new(),
            }],
        };
        lock.save(&app_dir.join(LOCKFILE_NAME)).expect("write lockfile");

        std::env::set_current_dir(&app_dir).expect("set temp cwd");
        let result = cmd_install(true);
        std::env::set_current_dir(&old_cwd).expect("restore cwd");

        assert!(result.is_ok(), "locked install failed: {:?}", result.err());
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn test_registry_source_label_prefers_remote_url_when_configured() {
        let _guard = global_test_lock().lock().expect("lock");
        std::env::set_var("OPM_REGISTRY_URL", "http://registry.example");
        std::env::remove_var("OPM_REGISTRY_ROOT");

        let source = registry_source_label("serde", "1.2.0");
        assert_eq!(
            source,
            "registry+omni+remote/http://registry.example/serde@1.2.0"
        );

        std::env::remove_var("OPM_REGISTRY_URL");
    }

    #[test]
    fn test_registry_source_label_uses_local_cache_root_when_set() {
        let _guard = global_test_lock().lock().expect("lock");
        let root = unique_temp_dir("opm-registry-source");
        fs::create_dir_all(&root).expect("create root");
        std::env::set_var("OPM_REGISTRY_ROOT", &root);
        std::env::remove_var("OPM_REGISTRY_URL");

        let source = registry_source_label("serde", "1.2.0");
        assert!(source.contains("registry+omni+cache/"));
        assert!(source.ends_with("/serde@1.2.0"));

        std::env::remove_var("OPM_REGISTRY_ROOT");
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn test_build_output_path_layout() {
        let root = PathBuf::from("/tmp/omni-opm");
        let debug_path = build_output_path(&root, "demo", false);
        let release_path = build_output_path(&root, "demo", true);

        assert_eq!(debug_path, root.join("build").join("debug").join("demo.ovm"));
        assert_eq!(
            release_path,
            root.join("build").join("release").join("demo.ovm")
        );
    }

    #[test]
    fn test_build_script_env_pairs_include_cfg_link_data() {
        let output = advanced::BuildOutput {
            cfg_flags: vec!["feature=demo".to_string(), "my_flag".to_string()],
            link_libs: vec!["static=foo".to_string()],
            link_paths: vec![PathBuf::from("/tmp/omni")],
            native_artifact: None,
        };

        let env = build_script_env_pairs(&output);
        assert!(env
            .iter()
            .any(|(k, v)| k == "OMNI_CFG_FLAGS" && v == "feature=demo;my_flag"));
        assert!(env
            .iter()
            .any(|(k, v)| k == "OMNI_LINK_LIBS" && v == "static=foo"));
        assert!(env
            .iter()
            .any(|(k, v)| k == "OMNI_LINK_PATHS" && v.contains("/tmp/omni")));
    }

    #[test]
    fn test_parse_link_sidecar_reads_directives() {
        let root = unique_temp_dir("opm-link-sidecar-parse");
        fs::create_dir_all(&root).expect("create temp dir");
        let sidecar = root.join("demo.ovm.link");

        fs::write(
            &sidecar,
            "# omni link directives\nlink_lib=static=foo\nlink_path=/tmp/omni\n",
        )
        .expect("write sidecar");

        let (libs, paths) = parse_link_sidecar(&sidecar).expect("parse sidecar");
        assert_eq!(libs, vec!["static=foo".to_string()]);
        assert_eq!(paths, vec!["/tmp/omni".to_string()]);

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn test_verify_link_sidecar_validates_expected_directives() {
        let root = unique_temp_dir("opm-link-sidecar-verify");
        fs::create_dir_all(&root).expect("create temp dir");
        let output = root.join("demo.ovm");
        let sidecar = sidecar_path_for_output(&output);

        fs::write(
            &sidecar,
            "# omni link directives\nlink_lib=static=foo\nlink_path=/tmp/omni\n",
        )
        .expect("write sidecar");

        let build_output = advanced::BuildOutput {
            cfg_flags: Vec::new(),
            link_libs: vec!["static=foo".to_string()],
            link_paths: vec![PathBuf::from("/tmp/omni")],
            native_artifact: None,
        };

        assert!(verify_link_sidecar(&output, &build_output).is_ok());

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn test_resolve_build_units_non_workspace_returns_root_package() {
        let root = unique_temp_dir("opm-build-units-single");
        fs::create_dir_all(&root).expect("create root");
        fs::write(
            root.join(MANIFEST_NAME),
            r#"[package]
name = "single"
version = "0.1.0"
"#,
        )
        .expect("write manifest");

        let units = resolve_build_units(&root).expect("resolve units");
        assert_eq!(units, vec![(root.clone(), "single".to_string())]);

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn test_resolve_build_units_workspace_prefers_default_members() {
        let root = unique_temp_dir("opm-build-units-workspace");
        let packages = root.join("packages");
        fs::create_dir_all(&packages).expect("create packages dir");

        let app_dir = packages.join("app");
        let lib_dir = packages.join("lib");
        fs::create_dir_all(&app_dir).expect("create app dir");
        fs::create_dir_all(&lib_dir).expect("create lib dir");

        fs::write(
            root.join(MANIFEST_NAME),
            r#"[package]
name = "workspace-root"
version = "0.1.0"

[workspace]
members = ["packages/*"]
default_members = ["packages/app"]
"#,
        )
        .expect("write root manifest");

        fs::write(
            app_dir.join(MANIFEST_NAME),
            r#"[package]
name = "app"
version = "0.1.0"
"#,
        )
        .expect("write app manifest");
        fs::write(
            lib_dir.join(MANIFEST_NAME),
            r#"[package]
name = "lib"
version = "0.1.0"
"#,
        )
        .expect("write lib manifest");

        let units = resolve_build_units(&root).expect("resolve units");
        assert_eq!(units, vec![(app_dir.clone(), "app".to_string())]);

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn test_write_build_artifact_manifest_writes_expected_payload() {
        let root = unique_temp_dir("opm-artifact-manifest");
        fs::create_dir_all(&root).expect("create root");
        let output = root.join("build").join("debug").join("demo.ovm");

        let manifest_path = write_build_artifact_manifest(
            &root,
            "demo",
            false,
            &output,
            &["feature=demo".to_string()],
            &["static=foo".to_string()],
            &["/tmp/omni".to_string()],
            None,
            None,
            None,
            "not-requested",
        )
        .expect("write artifact manifest");

        let content = fs::read_to_string(&manifest_path).expect("read artifact manifest");
        let parsed: BuildArtifactManifest =
            serde_json::from_str(&content).expect("parse artifact manifest");

        assert_eq!(parsed.package, "demo");
        assert_eq!(parsed.profile, "debug");
        assert!(parsed.output.ends_with("demo.ovm"));
        assert_eq!(parsed.cfg_flags, vec!["feature=demo".to_string()]);
        assert_eq!(parsed.link_libs, vec!["static=foo".to_string()]);
        assert_eq!(parsed.link_paths, vec!["/tmp/omni".to_string()]);
        assert_eq!(parsed.native_link_status, "not-requested");
        assert!(parsed.native_link_plan.is_none());
        assert!(parsed.native_link_report.is_none());

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn test_normalize_link_lib_flag_handles_directive_forms() {
        assert_eq!(normalize_link_lib_flag("static=z"), "-lz");
        assert_eq!(normalize_link_lib_flag("dylib=ssl"), "-lssl");
        assert_eq!(normalize_link_lib_flag("crypto"), "-lcrypto");
    }

    #[test]
    fn test_write_native_link_plan_writes_expected_payload() {
        let root = unique_temp_dir("opm-native-link-plan");
        fs::create_dir_all(&root).expect("create root");
        let input = root.join("build").join("debug").join("demo.ovm");
        let output = native_link_output_path(&root, "demo", false);

        let payload = NativeLinkPlan {
            package: "demo".to_string(),
            profile: "debug".to_string(),
            linker: "cc".to_string(),
            input: input.display().to_string(),
            output: output.display().to_string(),
            args: build_native_link_args(
                &input,
                &output,
                &["static=foo".to_string(), "ssl".to_string()],
                &["/tmp/omni".to_string()],
            ),
            link_libs: vec!["static=foo".to_string(), "ssl".to_string()],
            link_paths: vec!["/tmp/omni".to_string()],
            artifact: sample_native_artifact(&input),
        };

        let plan_path = write_native_link_plan(
            &root,
            "demo",
            false,
            &payload,
        )
        .expect("write native link plan");

        let content = fs::read_to_string(&plan_path).expect("read native link plan");
        let parsed: NativeLinkPlan = serde_json::from_str(&content).expect("parse native link plan");

        assert_eq!(parsed.package, "demo");
        assert_eq!(parsed.profile, "debug");
        assert!(parsed.input.ends_with("demo.ovm"));
        if cfg!(windows) {
            assert!(parsed.output.ends_with("demo.exe"));
        } else {
            assert!(parsed.output.ends_with("demo.bin"));
        }
        assert!(parsed.args.iter().any(|arg| arg == "-lfoo"));
        assert!(parsed.args.iter().any(|arg| arg == "-lssl"));
        assert!(parsed.args.iter().any(|arg| arg == "-L/tmp/omni"));

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn test_execute_native_link_plan_capture_mode_writes_report() {
        let _guard = global_test_lock().lock().expect("lock");
        let root = unique_temp_dir("opm-native-link-execute");
        fs::create_dir_all(&root).expect("create root");
        let input = root.join("build").join("debug").join("demo.o");
        let output = native_link_output_path(&root, "demo", false);
        let capture_path = root.join("capture.json");

        let payload = NativeLinkPlan {
            package: "demo".to_string(),
            profile: "debug".to_string(),
            linker: "nonexistent-linker".to_string(),
            input: input.display().to_string(),
            output: output.display().to_string(),
            args: build_native_link_args(
                &input,
                &output,
                &["static=foo".to_string()],
                &["/tmp/omni".to_string()],
            ),
            link_libs: vec!["static=foo".to_string()],
            link_paths: vec!["/tmp/omni".to_string()],
            artifact: sample_native_artifact(&input),
        };

        if let Some(parent) = input.parent() {
            fs::create_dir_all(parent).expect("create input parent directory");
        }
        fs::write(&input, b"object-bytes").expect("write fake object file");

        std::env::set_var("OMNI_NATIVE_LINK_CAPTURE", &capture_path);
        let report_path = execute_native_link_plan(&root, "demo", false, &payload)
            .expect("capture execution should succeed");
        std::env::remove_var("OMNI_NATIVE_LINK_CAPTURE");

        assert_eq!(report_path, capture_path);

        let content = fs::read_to_string(&capture_path).expect("read capture report");
        let parsed: NativeLinkExecutionReport = serde_json::from_str(&content)
            .expect("parse execution report");
        assert!(parsed.capture_mode);
        assert_eq!(parsed.status_code, Some(0));
        assert_eq!(parsed.linker, "nonexistent-linker");
        assert!(parsed.args.iter().any(|arg| arg == "-lfoo"));

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn test_native_artifact_linkability_uses_explicit_kind_contract() {
        let object_artifact = advanced::NativeArtifactMetadata {
            kind: advanced::NativeArtifactKind::Object,
            target_triple: current_target_triple(),
            path: PathBuf::from("build/demo.o"),
            checksum: None,
        };
        let bitcode_artifact = advanced::NativeArtifactMetadata {
            kind: advanced::NativeArtifactKind::Bitcode,
            target_triple: current_target_triple(),
            path: PathBuf::from("build/demo.bc"),
            checksum: None,
        };

        assert!(native_artifact_is_linkable(&object_artifact));
        assert!(!native_artifact_is_linkable(&bitcode_artifact));
    }

    #[test]
    fn test_should_execute_native_link_defaults_enabled() {
        let _guard = global_test_lock().lock().expect("lock");
        std::env::remove_var("OMNI_NATIVE_LINK_EXECUTE");
        assert!(should_execute_native_link());
    }

    #[test]
    fn test_should_execute_native_link_honors_falsey_values() {
        let _guard = global_test_lock().lock().expect("lock");
        std::env::set_var("OMNI_NATIVE_LINK_EXECUTE", "0");
        assert!(!should_execute_native_link());
        std::env::set_var("OMNI_NATIVE_LINK_EXECUTE", "off");
        assert!(!should_execute_native_link());
        std::env::remove_var("OMNI_NATIVE_LINK_EXECUTE");
    }

    #[test]
    fn test_strict_native_link_execution_flag() {
        let _guard = global_test_lock().lock().expect("lock");
        std::env::remove_var("OMNI_NATIVE_LINK_STRICT");
        assert!(!strict_native_link_execution());
        std::env::set_var("OMNI_NATIVE_LINK_STRICT", "1");
        assert!(strict_native_link_execution());
        std::env::remove_var("OMNI_NATIVE_LINK_STRICT");
    }

    #[test]
    fn test_search_local_registry_filters_by_query() {
        let root = unique_temp_dir("opm-search-local");
        fs::create_dir_all(root.join("serde")).expect("create serde package dir");
        fs::create_dir_all(root.join("serde_json")).expect("create serde_json package dir");
        fs::create_dir_all(root.join("itoa")).expect("create itoa package dir");

        let results = search_local_registry(&root, "serde").expect("local search should succeed");
        assert_eq!(results, vec!["serde".to_string(), "serde_json".to_string()]);

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn test_search_remote_registry_parses_line_results() {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind test server");
        let base_url = format!("http://{}", listener.local_addr().expect("addr"));

        let server = thread::spawn(move || {
            let (mut stream, _) = listener.accept().expect("accept request");
            let mut buffer = [0_u8; 2048];
            let bytes_read = stream.read(&mut buffer).expect("read request");
            let request = String::from_utf8_lossy(&buffer[..bytes_read]);
            let path = request
                .lines()
                .next()
                .and_then(|line| line.split_whitespace().nth(1))
                .unwrap_or("/");

            let body = if path == "/search?q=serde" {
                "serde\nserde_json\n"
            } else {
                ""
            };
            let status = if body.is_empty() {
                "404 Not Found"
            } else {
                "200 OK"
            };
            let response = format!(
                "HTTP/1.1 {status}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                body.len()
            );
            stream.write_all(response.as_bytes()).expect("write response");
        });

        let results = search_remote_registry(&base_url, "serde")
            .expect("remote search should succeed");
        assert_eq!(results, vec!["serde".to_string(), "serde_json".to_string()]);

        server.join().expect("server should stop cleanly");
    }

    #[test]
    fn test_cmd_publish_writes_manifest_to_local_registry() {
        let _guard = global_test_lock().lock().expect("lock");
        let old_cwd = std::env::current_dir().expect("cwd");
        let root = unique_temp_dir("opm-publish-local");
        let app_dir = root.join("app");
        let registry_dir = root.join("registry");

        fs::create_dir_all(&app_dir).expect("create app dir");
        fs::create_dir_all(&registry_dir).expect("create registry dir");
        fs::write(
            app_dir.join(MANIFEST_NAME),
            r#"[package]
name = "demo"
version = "0.1.0"
"#,
        )
        .expect("write app manifest");

        std::env::set_var("OPM_REGISTRY_ROOT", &registry_dir);
        std::env::remove_var("OPM_REGISTRY_URL");
        let previous_key = set_test_signing_key();
        std::env::set_current_dir(&app_dir).expect("set temp cwd");

        let result = cmd_publish();

        std::env::set_current_dir(&old_cwd).expect("restore cwd");
        std::env::remove_var("OPM_REGISTRY_ROOT");
        restore_test_signing_key(previous_key);

        assert!(result.is_ok(), "publish failed: {:?}", result.err());
        let published_manifest = registry_dir.join("demo").join("0.1.0").join(MANIFEST_NAME);
        assert!(published_manifest.exists(), "published manifest should exist");
        let publish_record = registry_dir.join("demo").join("0.1.0").join("publish.json");
        assert!(publish_record.exists(), "publish metadata should exist");

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn test_cmd_publish_rejects_duplicate_without_overwrite() {
        let _guard = global_test_lock().lock().expect("lock");
        let old_cwd = std::env::current_dir().expect("cwd");
        let root = unique_temp_dir("opm-publish-duplicate");
        let app_dir = root.join("app");
        let registry_dir = root.join("registry");

        fs::create_dir_all(&app_dir).expect("create app dir");
        fs::create_dir_all(registry_dir.join("demo").join("0.1.0"))
            .expect("create published dir");
        fs::write(
            app_dir.join(MANIFEST_NAME),
            r#"[package]
name = "demo"
version = "0.1.0"
"#,
        )
        .expect("write app manifest");

        std::env::set_var("OPM_REGISTRY_ROOT", &registry_dir);
        std::env::remove_var("OPM_REGISTRY_URL");
        std::env::remove_var("OPM_PUBLISH_ALLOW_OVERWRITE");
        let previous_key = set_test_signing_key();
        std::env::set_current_dir(&app_dir).expect("set temp cwd");

        let result = cmd_publish();

        std::env::set_current_dir(&old_cwd).expect("restore cwd");
        std::env::remove_var("OPM_REGISTRY_ROOT");
        restore_test_signing_key(previous_key);

        assert!(
            matches!(result, Err(ref msg) if msg.contains("already exists")),
            "expected duplicate publish rejection, got {:?}",
            result
        );

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn test_publish_to_remote_registry_posts_payload() {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind test server");
        let base_url = format!("http://{}", listener.local_addr().expect("addr"));

        let server = thread::spawn(move || {
            let (mut stream, _) = listener.accept().expect("accept request");
            let mut buffer = [0_u8; 8192];
            let bytes_read = stream.read(&mut buffer).expect("read request");
            let request = String::from_utf8_lossy(&buffer[..bytes_read]);
            let is_publish = request.starts_with("POST /publish ");
            let has_manifest = request.contains("\"manifest\"");

            let status = if is_publish && has_manifest {
                "200 OK"
            } else {
                "400 Bad Request"
            };
            let response = format!(
                "HTTP/1.1 {status}\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
            );
            stream.write_all(response.as_bytes()).expect("write response");
        });

        let previous_key = set_test_signing_key();
        let manifest = sign_manifest(
            toml::from_str(
                r#"[package]
name = "demo"
version = "0.1.0"
"#,
            )
            .expect("parse manifest"),
        )
        .expect("sign manifest");
        restore_test_signing_key(previous_key);

        let manifest_content = toml::to_string_pretty(&manifest).expect("serialize manifest");

        let result = publish_to_remote_registry(
            &base_url,
            &manifest,
            &manifest_content,
        );
        assert!(result.is_ok(), "remote publish failed: {:?}", result.err());

        server.join().expect("server should stop cleanly");
    }
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
        Commands::Install { locked } => cmd_install(*locked),
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
