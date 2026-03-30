// Copyright 2024 Shreyash Jagtap
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#![allow(dead_code)]
//! Module Mode System for Omni
//!
//! Implements the three mutually exclusive per-module profiles from the master canvas:
//! - `script`: interpreter-first dynamic execution
//! - `hosted`: GC-enabled OS-hosted execution with optional static modules
//! - `bare_metal`: no-GC ownership/manual execution with cooperative scheduling
//!
//! Each mode restricts which language features are available.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// The three canonical modes for v0.1.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModuleMode {
    /// Interpreter-first dynamic execution. Dynamic typing, no unsafe, limited stdlib.
    Script,
    /// GC-enabled OS-hosted execution. Full stdlib, JIT optional via annotation.
    Hosted,
    /// No-GC ownership/manual execution. Inline asm allowed, no preemptive threads.
    BareMetal,
}

impl std::fmt::Display for ModuleMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ModuleMode::Script => write!(f, "script"),
            ModuleMode::Hosted => write!(f, "hosted"),
            ModuleMode::BareMetal => write!(f, "bare_metal"),
        }
    }
}

impl std::str::FromStr for ModuleMode {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "script" => Ok(ModuleMode::Script),
            "hosted" => Ok(ModuleMode::Hosted),
            "bare_metal" | "bare-metal" => Ok(ModuleMode::BareMetal),
            _ => Err(format!(
                "Unknown mode '{}'. Valid modes: script, hosted, bare_metal",
                s
            )),
        }
    }
}

/// Features that can be restricted per mode.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Feature {
    /// Garbage collection
    GC,
    /// JIT compilation
    JIT,
    /// AOT compilation
    AOT,
    /// Unsafe blocks
    Unsafe,
    /// Inline assembly
    InlineAsm,
    /// OS preemptive threads
    OsThreads,
    /// Async/await
    Async,
    /// Dynamic typing
    DynamicTyping,
    /// Static typing
    StaticTyping,
    /// Full standard library
    FullStdlib,
    /// Limited standard library (script mode)
    LimitedStdlib,
    /// GPU offloading
    Gpu,
    /// FFI / extern blocks
    FFI,
    /// Ownership / borrow system
    Ownership,
    /// Manual memory management
    ManualMemory,
    /// Cooperative scheduling / fibers
    CooperativeScheduling,
    /// Reflection / runtime introspection
    Reflection,
}

/// Allowed features per mode.
pub fn allowed_features(mode: ModuleMode) -> Vec<Feature> {
    match mode {
        ModuleMode::Script => vec![
            Feature::DynamicTyping,
            Feature::LimitedStdlib,
            Feature::Async,
            Feature::CooperativeScheduling,
            Feature::Reflection,
            // No: GC (uses interpreter's), Unsafe, InlineAsm, OsThreads, GPU, FFI, Ownership
        ],
        ModuleMode::Hosted => vec![
            Feature::GC,
            Feature::JIT,
            Feature::AOT,
            Feature::StaticTyping,
            Feature::DynamicTyping,
            Feature::FullStdlib,
            Feature::OsThreads,
            Feature::Async,
            Feature::CooperativeScheduling,
            Feature::FFI,
            Feature::Ownership,
            Feature::Gpu,
            Feature::Reflection,
            // No: InlineAsm, ManualMemory (GC owns memory)
        ],
        ModuleMode::BareMetal => vec![
            Feature::AOT,
            Feature::StaticTyping,
            Feature::Ownership,
            Feature::ManualMemory,
            Feature::InlineAsm,
            Feature::CooperativeScheduling,
            Feature::FFI,
            Feature::Unsafe,
            // No: GC, JIT, OsThreads, DynamicTyping, FullStdlib, Reflection
        ],
    }
}

/// Check if a specific feature is allowed in the given mode.
pub fn is_feature_allowed(mode: ModuleMode, feature: &Feature) -> bool {
    allowed_features(mode).contains(feature)
}

/// Memory zone types as defined in the master canvas.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MemoryZone {
    /// GC-managed heap (tracing GC with mark-and-sweep)
    GcZone,
    /// Ownership/borrow-checked memory (Rust-style move semantics)
    OwnershipZone,
    /// Manual allocation/deallocation (C-style)
    ManualZone,
    /// Region/arena allocator (batch free)
    RegionZone,
}

impl std::fmt::Display for MemoryZone {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MemoryZone::GcZone => write!(f, "GC_ZONE"),
            MemoryZone::OwnershipZone => write!(f, "OWNERSHIP_ZONE"),
            MemoryZone::ManualZone => write!(f, "MANUAL_ZONE"),
            MemoryZone::RegionZone => write!(f, "REGION_ZONE"),
        }
    }
}

/// Memory operations that can be checked against allowed zones.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum MemoryOperation {
    /// GC heap allocation (e.g. `new`, array creation, string concat)
    GcAlloc,
    /// Ownership move (variable assignment that transfers ownership)
    OwnershipMove,
    /// Shared borrow (`&T`)
    SharedBorrow,
    /// Mutable borrow (`&mut T`)
    MutableBorrow,
    /// Manual allocation (e.g. `alloc`, `malloc` FFI)
    ManualAlloc,
    /// Manual deallocation (e.g. `free`, `dealloc`)
    ManualDealloc,
    /// Unsafe raw pointer dereference
    UnsafeDeref,
    /// Region allocation (arena bump allocator)
    RegionAlloc,
}

/// Allowed memory zones per mode.
pub fn allowed_zones(mode: ModuleMode) -> Vec<MemoryZone> {
    match mode {
        ModuleMode::Script => vec![MemoryZone::GcZone],
        ModuleMode::Hosted => vec![
            MemoryZone::GcZone,
            MemoryZone::OwnershipZone,
            MemoryZone::RegionZone,
        ],
        ModuleMode::BareMetal => vec![
            MemoryZone::OwnershipZone,
            MemoryZone::ManualZone,
            MemoryZone::RegionZone,
        ],
    }
}

/// Map a memory operation to its required zone.
pub fn required_zone(op: &MemoryOperation) -> MemoryZone {
    match op {
        MemoryOperation::GcAlloc => MemoryZone::GcZone,
        MemoryOperation::OwnershipMove => MemoryZone::OwnershipZone,
        MemoryOperation::SharedBorrow => MemoryZone::OwnershipZone,
        MemoryOperation::MutableBorrow => MemoryZone::OwnershipZone,
        MemoryOperation::ManualAlloc => MemoryZone::ManualZone,
        MemoryOperation::ManualDealloc => MemoryZone::ManualZone,
        MemoryOperation::UnsafeDeref => MemoryZone::ManualZone,
        MemoryOperation::RegionAlloc => MemoryZone::RegionZone,
    }
}

/// Check if a memory operation is allowed in the given mode.
pub fn is_memory_op_allowed(mode: ModuleMode, op: &MemoryOperation) -> bool {
    let zone = required_zone(op);
    allowed_zones(mode).contains(&zone)
}

/// Memory zone checker — validates that memory operations match allowed zones.
pub struct MemoryZoneChecker {
    mode: ModuleMode,
    violations: Vec<String>,
}

impl MemoryZoneChecker {
    pub fn new(mode: ModuleMode) -> Self {
        Self {
            mode,
            violations: Vec::new(),
        }
    }

    /// Check a memory operation against the mode's allowed zones.
    pub fn check_operation(&mut self, op: MemoryOperation, context: &str) {
        if !is_memory_op_allowed(self.mode, &op) {
            let zone = required_zone(&op);
            self.violations.push(format!(
                "Memory operation '{:?}' requires {} which is not allowed in mode '{}' (at: {})",
                op, zone, self.mode, context
            ));
        }
    }

    /// Validate that the module's memory usage is consistent with its mode.
    /// Returns violations found.
    pub fn validate(&self) -> &[String] {
        &self.violations
    }

    /// Returns true if no violations found.
    pub fn is_valid(&self) -> bool {
        self.violations.is_empty()
    }
}

/// Module-checker: validates that a module's used features are compatible with its mode.
pub struct ModuleChecker {
    mode: ModuleMode,
    violations: Vec<String>,
}

impl ModuleChecker {
    pub fn new(mode: ModuleMode) -> Self {
        Self {
            mode,
            violations: Vec::new(),
        }
    }

    /// Check a feature usage against the mode's allowed set.
    pub fn check_feature(&mut self, feature: Feature, context: &str) {
        if !is_feature_allowed(self.mode, &feature) {
            self.violations.push(format!(
                "Feature '{:?}' not allowed in mode '{}' (used at: {})",
                feature, self.mode, context
            ));
        }
    }

    /// Check if the module uses GC features (allocation without ownership).
    pub fn check_gc_usage(&mut self, has_gc_alloc: bool, context: &str) {
        if has_gc_alloc {
            self.check_feature(Feature::GC, context);
        }
    }

    /// Check if the module uses unsafe blocks.
    pub fn check_unsafe_usage(&mut self, has_unsafe: bool, context: &str) {
        if has_unsafe {
            self.check_feature(Feature::Unsafe, context);
        }
    }

    /// Check if the module uses JIT annotations.
    pub fn check_jit_annotation(&mut self, has_jit: bool, context: &str) {
        if has_jit {
            self.check_feature(Feature::JIT, context);
        }
    }

    /// Get all violations found.
    pub fn violations(&self) -> &[String] {
        &self.violations
    }

    /// Returns true if no violations found.
    pub fn is_valid(&self) -> bool {
        self.violations.is_empty()
    }
}

/// Package manifest (`Omni.toml`) structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageManifest {
    pub package: PackageInfo,
    #[serde(default)]
    pub dependencies: HashMap<String, DependencySpec>,
    #[serde(default)]
    pub build: Option<BuildConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageInfo {
    pub name: String,
    pub version: String,
    #[serde(default = "default_mode")]
    pub mode: String,
    #[serde(default)]
    pub authors: Vec<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default = "default_abi_version")]
    pub abi_version: String,
    #[serde(default = "default_ir_version")]
    pub ir_version: String,
}

fn default_mode() -> String {
    "hosted".to_string()
}

fn default_abi_version() -> String {
    "1.0.0".to_string()
}

fn default_ir_version() -> String {
    "1.0.0".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencySpec {
    pub version: Option<String>,
    #[serde(default)]
    pub features: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildConfig {
    pub target: Option<String>,
    pub optimization: Option<u8>,
}

impl PackageManifest {
    /// Load manifest from an Omni.toml file.
    pub fn load(path: &Path) -> Result<Self, String> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("Cannot read {}: {}", path.display(), e))?;
        let manifest: PackageManifest = toml::from_str(&content)
            .map_err(|e| format!("Cannot parse {}: {}", path.display(), e))?;
        Ok(manifest)
    }

    /// Get the parsed module mode.
    pub fn get_mode(&self) -> Result<ModuleMode, String> {
        self.package.mode.parse()
    }
}

/// Generate module metadata JSON for resolver input.
pub fn generate_module_metadata(
    manifest: &PackageManifest,
    used_features: &[Feature],
) -> serde_json::Value {
    let mode = manifest.get_mode().unwrap_or(ModuleMode::Hosted);
    serde_json::json!({
        "module": manifest.package.name,
        "version": manifest.package.version,
        "mode": mode.to_string(),
        "abi_version": manifest.package.abi_version,
        "ir_version": manifest.package.ir_version,
        "used_features": used_features.iter().map(|f| format!("{:?}", f)).collect::<Vec<_>>(),
        "allowed_features": allowed_features(mode).iter().map(|f| format!("{:?}", f)).collect::<Vec<_>>(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_script_mode_allows_dynamic_typing() {
        assert!(is_feature_allowed(
            ModuleMode::Script,
            &Feature::DynamicTyping
        ));
    }

    #[test]
    fn test_script_mode_disallows_gc() {
        assert!(!is_feature_allowed(ModuleMode::Script, &Feature::GC));
    }

    #[test]
    fn test_hosted_mode_allows_gc() {
        assert!(is_feature_allowed(ModuleMode::Hosted, &Feature::GC));
    }

    #[test]
    fn test_hosted_mode_allows_jit() {
        assert!(is_feature_allowed(ModuleMode::Hosted, &Feature::JIT));
    }

    #[test]
    fn test_bare_metal_disallows_gc() {
        assert!(!is_feature_allowed(ModuleMode::BareMetal, &Feature::GC));
    }

    #[test]
    fn test_bare_metal_allows_inline_asm() {
        assert!(is_feature_allowed(
            ModuleMode::BareMetal,
            &Feature::InlineAsm
        ));
    }

    #[test]
    fn test_bare_metal_allows_ownership() {
        assert!(is_feature_allowed(
            ModuleMode::BareMetal,
            &Feature::Ownership
        ));
    }

    #[test]
    fn test_module_checker_catches_violation() {
        let mut checker = ModuleChecker::new(ModuleMode::BareMetal);
        checker.check_gc_usage(true, "line 42: heap allocation");
        assert!(!checker.is_valid());
        assert_eq!(checker.violations().len(), 1);
        assert!(checker.violations()[0].contains("GC"));
    }

    #[test]
    fn test_module_checker_passes() {
        let mut checker = ModuleChecker::new(ModuleMode::Hosted);
        checker.check_gc_usage(true, "line 42");
        checker.check_jit_annotation(true, "line 10");
        assert!(checker.is_valid());
    }

    #[test]
    fn test_mode_from_str() {
        assert_eq!("script".parse::<ModuleMode>().unwrap(), ModuleMode::Script);
        assert_eq!("hosted".parse::<ModuleMode>().unwrap(), ModuleMode::Hosted);
        assert_eq!(
            "bare_metal".parse::<ModuleMode>().unwrap(),
            ModuleMode::BareMetal
        );
        assert!("invalid".parse::<ModuleMode>().is_err());
    }

    #[test]
    fn test_feature_coverage() {
        // Each mode should have distinct feature sets
        let script = allowed_features(ModuleMode::Script);
        let hosted = allowed_features(ModuleMode::Hosted);
        let bare = allowed_features(ModuleMode::BareMetal);

        // Hosted has the most features
        assert!(hosted.len() > script.len());
        assert!(hosted.len() > bare.len());

        // Bare metal is the most restrictive in some ways
        assert!(!bare.contains(&Feature::GC));
        assert!(!bare.contains(&Feature::JIT));
    }

    // Memory zone tests
    #[test]
    fn test_script_allows_gc_zone_only() {
        assert!(is_memory_op_allowed(
            ModuleMode::Script,
            &MemoryOperation::GcAlloc
        ));
        assert!(!is_memory_op_allowed(
            ModuleMode::Script,
            &MemoryOperation::ManualAlloc
        ));
        assert!(!is_memory_op_allowed(
            ModuleMode::Script,
            &MemoryOperation::OwnershipMove
        ));
    }

    #[test]
    fn test_hosted_allows_gc_and_ownership() {
        assert!(is_memory_op_allowed(
            ModuleMode::Hosted,
            &MemoryOperation::GcAlloc
        ));
        assert!(is_memory_op_allowed(
            ModuleMode::Hosted,
            &MemoryOperation::OwnershipMove
        ));
        assert!(is_memory_op_allowed(
            ModuleMode::Hosted,
            &MemoryOperation::SharedBorrow
        ));
        assert!(!is_memory_op_allowed(
            ModuleMode::Hosted,
            &MemoryOperation::ManualAlloc
        ));
    }

    #[test]
    fn test_bare_metal_allows_ownership_and_manual() {
        assert!(is_memory_op_allowed(
            ModuleMode::BareMetal,
            &MemoryOperation::OwnershipMove
        ));
        assert!(is_memory_op_allowed(
            ModuleMode::BareMetal,
            &MemoryOperation::ManualAlloc
        ));
        assert!(is_memory_op_allowed(
            ModuleMode::BareMetal,
            &MemoryOperation::ManualDealloc
        ));
        assert!(!is_memory_op_allowed(
            ModuleMode::BareMetal,
            &MemoryOperation::GcAlloc
        ));
    }

    #[test]
    fn test_memory_zone_checker() {
        let mut checker = MemoryZoneChecker::new(ModuleMode::BareMetal);
        checker.check_operation(MemoryOperation::GcAlloc, "line 10: new Object()");
        assert!(!checker.is_valid());
        assert!(checker.validate()[0].contains("GC_ZONE"));
    }

    #[test]
    fn test_memory_zone_checker_valid() {
        let mut checker = MemoryZoneChecker::new(ModuleMode::Hosted);
        checker.check_operation(MemoryOperation::GcAlloc, "line 10");
        checker.check_operation(MemoryOperation::SharedBorrow, "line 20");
        assert!(checker.is_valid());
    }

    #[test]
    fn test_allowed_zones_coverage() {
        let script_zones = allowed_zones(ModuleMode::Script);
        let hosted_zones = allowed_zones(ModuleMode::Hosted);
        let bare_zones = allowed_zones(ModuleMode::BareMetal);

        // Script is GC only
        assert_eq!(script_zones.len(), 1);
        assert!(script_zones.contains(&MemoryZone::GcZone));

        // Hosted has GC + ownership + region
        assert!(hosted_zones.contains(&MemoryZone::GcZone));
        assert!(hosted_zones.contains(&MemoryZone::OwnershipZone));

        // Bare metal has ownership + manual + region, no GC
        assert!(!bare_zones.contains(&MemoryZone::GcZone));
        assert!(bare_zones.contains(&MemoryZone::ManualZone));
    }
}
