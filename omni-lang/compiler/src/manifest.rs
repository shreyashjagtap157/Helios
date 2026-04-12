#![allow(dead_code)]

use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct PackageManifest {
    pub name: String,
    pub version: String,
    pub authors: Vec<String>,
    pub mode: String,
    pub abi_version: String,
    pub ir_version: String,
    pub dependencies: HashMap<String, Dependency>,
    pub build: BuildConfig,
    pub features: HashMap<String, Vec<String>>,
}

#[derive(Debug, Clone)]
pub struct Dependency {
    pub version: String,
    pub features: Vec<String>,
    pub source: Option<String>,
}

#[derive(Debug, Clone)]
pub struct BuildConfig {
    pub target: String,
    pub optimization: u8,
    pub output_dir: Option<String>,
}

impl Default for BuildConfig {
    fn default() -> Self {
        Self {
            target: "x86_64-pc-windows-msvc".to_string(),
            optimization: 2,
            output_dir: None,
        }
    }
}

impl PackageManifest {
    pub fn parse(content: &str) -> Result<Self, String> {
        let mut name = String::new();
        let mut version = "0.1.0".to_string();
        let mut authors = Vec::new();
        let mut mode = "standalone".to_string();
        let mut abi_version = "1.0.0".to_string();
        let mut ir_version = "1.0.0".to_string();
        let mut dependencies = HashMap::new();
        let mut build = BuildConfig::default();
        let mut features = HashMap::new();

        let mut in_dependencies = false;
        let mut in_build = false;
        let mut in_features = false;

        for line in content.lines() {
            let line = line.trim();

            if line.starts_with('[') {
                in_dependencies = false;
                in_build = false;
                in_features = false;

                if line == "[package]" {
                    continue;
                } else if line == "[dependencies]" {
                    in_dependencies = true;
                } else if line == "[build]" {
                    in_build = true;
                } else if line == "[features]" {
                    in_features = true;
                }
                continue;
            }

            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            if let Some((key, value)) = line.split_once('=') {
                let key = key.trim();
                let value = value.trim().trim_matches('"');

                if in_dependencies {
                    if let Some(dep) = parse_dependency(value) {
                        dependencies.insert(key.to_string(), dep);
                    }
                } else if in_build {
                    match key {
                        "target" => build.target = value.to_string(),
                        "optimization" => {
                            if let Ok(opt) = value.parse() {
                                build.optimization = opt;
                            }
                        }
                        "output" => build.output_dir = Some(value.to_string()),
                        _ => {}
                    }
                } else if in_features {
                    features.insert(
                        key.to_string(),
                        value.split(',').map(|s| s.trim().to_string()).collect(),
                    );
                } else {
                    match key {
                        "name" => name = value.to_string(),
                        "version" => version = value.to_string(),
                        "authors" => {
                            authors = value.split(',').map(|s| s.trim().to_string()).collect()
                        }
                        "mode" => mode = value.to_string(),
                        "abi_version" => abi_version = value.to_string(),
                        "ir_version" => ir_version = value.to_string(),
                        _ => {}
                    }
                }
            }
        }

        if name.is_empty() {
            return Err("Package name is required".to_string());
        }

        Ok(Self {
            name,
            version,
            authors,
            mode,
            abi_version,
            ir_version,
            dependencies,
            build,
            features,
        })
    }

    pub fn from_file(path: &Path) -> Result<Self, String> {
        let content =
            fs::read_to_string(path).map_err(|e| format!("Failed to read manifest: {}", e))?;
        Self::parse(&content)
    }
}

fn parse_dependency(value: &str) -> Option<Dependency> {
    let mut parts = value.split(',');
    let version = parts.next()?.trim().to_string();

    let mut features = Vec::new();
    let mut source = None;

    for part in parts {
        let part = part.trim();
        if part.starts_with("features") {
            if let Some(f) = part.split('=').nth(1) {
                features = f
                    .trim_matches(['[', ']', ' '])
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .collect();
            }
        } else if part.starts_with("git") || part.starts_with("path") {
            source = Some(part.to_string());
        }
    }

    Some(Dependency {
        version,
        features,
        source,
    })
}

impl std::fmt::Display for PackageManifest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Package: {} v{}", self.name, self.version)?;
        writeln!(f, "Mode: {}", self.mode)?;
        writeln!(f, "ABI: {}, IR: {}", self.abi_version, self.ir_version)?;

        if !self.dependencies.is_empty() {
            writeln!(f, "Dependencies:")?;
            for (name, dep) in &self.dependencies {
                writeln!(f, "  {}: {}", name, dep.version)?;
            }
        }

        writeln!(
            f,
            "Build: {} (opt {})",
            self.build.target, self.build.optimization
        )?;

        Ok(())
    }
}
