//! Phase 10: Security, Sandboxing, and Capability Foundation
//!
//! This module introduces the first concrete capability layer for Omni.
//! It is intentionally small but usable:
//! - opaque capability tokens
//! - grant / revoke / validation logic
//! - capability policies derived from manifest declarations
//! - resource-limited sandbox execution
//! - FFI sandbox API surface for later stack-isolation work

use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CapabilityKind {
    FilesystemRead,
    FilesystemWrite,
    NetworkAccess,
    ProcessSpawn,
    EnvironmentAccess,
    Time,
    Random,
    Custom(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CapabilityToken {
    id: u64,
    kind: CapabilityKind,
}

impl CapabilityToken {
    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn kind(&self) -> &CapabilityKind {
        &self.kind
    }
}

#[derive(Debug, Default)]
pub struct CapabilityAuthority {
    next_id: u64,
}

impl CapabilityAuthority {
    pub fn new() -> Self {
        Self { next_id: 0 }
    }

    pub fn issue(&mut self, kind: CapabilityKind) -> CapabilityToken {
        let token = CapabilityToken {
            id: self.next_id,
            kind,
        };
        self.next_id = self.next_id.saturating_add(1);
        token
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CapabilityErrorKind {
    Missing,
    Revoked,
    PolicyDenied,
    LimitExceeded,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapabilityError {
    kind: CapabilityErrorKind,
    capability: CapabilityKind,
    message: String,
}

impl CapabilityError {
    pub fn new(
        kind: CapabilityErrorKind,
        capability: CapabilityKind,
        message: impl Into<String>,
    ) -> Self {
        Self {
            kind,
            capability,
            message: message.into(),
        }
    }

    pub fn missing(capability: CapabilityKind) -> Self {
        Self::new(
            CapabilityErrorKind::Missing,
            capability.clone(),
            format!("capability {:?} has not been granted", capability),
        )
    }

    pub fn revoked(capability: CapabilityKind) -> Self {
        Self::new(
            CapabilityErrorKind::Revoked,
            capability.clone(),
            format!("capability {:?} has been revoked", capability),
        )
    }

    pub fn policy_denied(capability: CapabilityKind) -> Self {
        Self::new(
            CapabilityErrorKind::PolicyDenied,
            capability.clone(),
            format!("capability {:?} is not declared by policy", capability),
        )
    }

    pub fn limit_exceeded(capability: CapabilityKind) -> Self {
        Self::new(
            CapabilityErrorKind::LimitExceeded,
            capability.clone(),
            format!(
                "capability {:?} exceeds configured resource limits",
                capability
            ),
        )
    }

    pub fn kind(&self) -> &CapabilityErrorKind {
        &self.kind
    }

    pub fn capability(&self) -> &CapabilityKind {
        &self.capability
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for CapabilityError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}: {}", self.kind, self.message)
    }
}

impl Error for CapabilityError {}

#[derive(Debug, Clone, Default)]
pub struct CapabilityGuard {
    granted: HashSet<CapabilityToken>,
    revoked: HashSet<CapabilityToken>,
}

impl CapabilityGuard {
    pub fn new() -> Self {
        Self {
            granted: HashSet::new(),
            revoked: HashSet::new(),
        }
    }

    pub fn with_tokens(tokens: impl IntoIterator<Item = CapabilityToken>) -> Self {
        Self {
            granted: tokens.into_iter().collect(),
            revoked: HashSet::new(),
        }
    }

    pub fn grant(&mut self, token: CapabilityToken) {
        self.revoked.remove(&token);
        self.granted.insert(token);
    }

    pub fn revoke(&mut self, token: &CapabilityToken) -> bool {
        let removed = self.granted.remove(token);
        if removed {
            self.revoked.insert(token.clone());
        }
        removed
    }

    pub fn has(&self, token: &CapabilityToken) -> bool {
        self.granted.contains(token)
    }

    pub fn has_kind(&self, kind: &CapabilityKind) -> bool {
        self.granted.iter().any(|token| token.kind() == kind)
    }

    pub fn revoked_kind(&self, kind: &CapabilityKind) -> bool {
        self.revoked.iter().any(|token| token.kind() == kind)
    }

    pub fn require(&self, kind: &CapabilityKind) -> Result<(), CapabilityError> {
        if self.has_kind(kind) {
            Ok(())
        } else if self.revoked_kind(kind) {
            Err(CapabilityError::revoked(kind.clone()))
        } else {
            Err(CapabilityError::missing(kind.clone()))
        }
    }

    pub fn require_all(&self, kinds: &[CapabilityKind]) -> Result<(), CapabilityError> {
        for kind in kinds {
            self.require(kind)?;
        }
        Ok(())
    }

    pub fn tokens(&self) -> impl Iterator<Item = &CapabilityToken> {
        self.granted.iter()
    }
}

#[derive(Debug, Clone, Default)]
pub struct CapabilityPolicy {
    declared: HashSet<CapabilityKind>,
}

impl CapabilityPolicy {
    pub fn new() -> Self {
        Self {
            declared: HashSet::new(),
        }
    }

    pub fn allow_all() -> Self {
        let mut policy = Self::new();
        policy.declare(CapabilityKind::FilesystemRead);
        policy.declare(CapabilityKind::FilesystemWrite);
        policy.declare(CapabilityKind::NetworkAccess);
        policy.declare(CapabilityKind::ProcessSpawn);
        policy.declare(CapabilityKind::EnvironmentAccess);
        policy.declare(CapabilityKind::Time);
        policy.declare(CapabilityKind::Random);
        policy
    }

    pub fn declare(&mut self, kind: CapabilityKind) {
        self.declared.insert(kind);
    }

    pub fn permits(&self, kind: &CapabilityKind) -> bool {
        self.declared.contains(kind)
    }

    pub fn from_manifest_entries(entries: &HashMap<String, Vec<String>>) -> Self {
        let mut policy = Self::new();

        for (category, permissions) in entries {
            match category.to_ascii_lowercase().as_str() {
                "filesystem" => {
                    for permission in permissions {
                        match permission.to_ascii_lowercase().as_str() {
                            "read" => policy.declare(CapabilityKind::FilesystemRead),
                            "write" => policy.declare(CapabilityKind::FilesystemWrite),
                            other => policy
                                .declare(CapabilityKind::Custom(format!("filesystem:{}", other))),
                        }
                    }
                }
                "network" => {
                    policy.declare(CapabilityKind::NetworkAccess);
                }
                "process" | "subprocess" => {
                    policy.declare(CapabilityKind::ProcessSpawn);
                }
                "environment" | "env" => {
                    policy.declare(CapabilityKind::EnvironmentAccess);
                }
                "time" => {
                    policy.declare(CapabilityKind::Time);
                }
                "random" | "rand" => {
                    policy.declare(CapabilityKind::Random);
                }
                other => {
                    if permissions.is_empty() {
                        policy.declare(CapabilityKind::Custom(other.to_string()));
                    } else {
                        for permission in permissions {
                            policy.declare(CapabilityKind::Custom(format!(
                                "{}:{}",
                                other, permission
                            )));
                        }
                    }
                }
            }
        }

        policy
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceLimits {
    pub max_memory_bytes: u64,
    pub max_cpu_ms: u64,
    pub max_open_files: usize,
    pub allow_network: bool,
    pub allow_process_spawn: bool,
}

impl ResourceLimits {
    pub fn strict() -> Self {
        Self {
            max_memory_bytes: 32 * 1024 * 1024,
            max_cpu_ms: 1_000,
            max_open_files: 16,
            allow_network: false,
            allow_process_spawn: false,
        }
    }

    pub fn permissive() -> Self {
        Self {
            max_memory_bytes: 512 * 1024 * 1024,
            max_cpu_ms: 60_000,
            max_open_files: 256,
            allow_network: true,
            allow_process_spawn: true,
        }
    }

    fn permits(&self, kind: &CapabilityKind) -> bool {
        match kind {
            CapabilityKind::FilesystemRead | CapabilityKind::FilesystemWrite => true,
            CapabilityKind::NetworkAccess => self.allow_network,
            CapabilityKind::ProcessSpawn => self.allow_process_spawn,
            CapabilityKind::EnvironmentAccess => true,
            CapabilityKind::Time | CapabilityKind::Random => true,
            CapabilityKind::Custom(_) => true,
        }
    }
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self::strict()
    }
}

#[derive(Debug, Clone)]
pub struct Sandbox {
    policy: CapabilityPolicy,
    guard: CapabilityGuard,
    limits: ResourceLimits,
}

impl Sandbox {
    pub fn new(policy: CapabilityPolicy, limits: ResourceLimits) -> Self {
        Self {
            policy,
            guard: CapabilityGuard::new(),
            limits,
        }
    }

    pub fn with_guard(
        policy: CapabilityPolicy,
        limits: ResourceLimits,
        guard: CapabilityGuard,
    ) -> Self {
        Self {
            policy,
            guard,
            limits,
        }
    }

    pub fn policy(&self) -> &CapabilityPolicy {
        &self.policy
    }

    pub fn guard(&self) -> &CapabilityGuard {
        &self.guard
    }

    pub fn guard_mut(&mut self) -> &mut CapabilityGuard {
        &mut self.guard
    }

    pub fn limits(&self) -> &ResourceLimits {
        &self.limits
    }

    fn validate(&self, required: &[CapabilityKind]) -> Result<(), CapabilityError> {
        for capability in required {
            if !self.policy.permits(capability) {
                return Err(CapabilityError::policy_denied(capability.clone()));
            }

            if !self.limits.permits(capability) {
                return Err(CapabilityError::limit_exceeded(capability.clone()));
            }

            self.guard.require(capability)?;
        }

        Ok(())
    }

    pub fn execute<R, F>(
        &self,
        required: &[CapabilityKind],
        operation: F,
    ) -> Result<R, CapabilityError>
    where
        F: FnOnce() -> R,
    {
        self.validate(required)?;
        Ok(operation())
    }
}

#[derive(Debug, Clone)]
pub struct FfiSandbox {
    sandbox: Sandbox,
}

impl FfiSandbox {
    pub fn new(policy: CapabilityPolicy, limits: ResourceLimits) -> Self {
        Self {
            sandbox: Sandbox::new(policy, limits),
        }
    }

    pub fn with_guard(
        policy: CapabilityPolicy,
        limits: ResourceLimits,
        guard: CapabilityGuard,
    ) -> Self {
        Self {
            sandbox: Sandbox::with_guard(policy, limits, guard),
        }
    }

    pub fn sandbox(&self) -> &Sandbox {
        &self.sandbox
    }

    pub fn execute<R, F>(
        &self,
        required: &[CapabilityKind],
        operation: F,
    ) -> Result<R, CapabilityError>
    where
        F: FnOnce() -> R,
    {
        self.sandbox.execute(required, operation)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capability_grant_revoke() {
        let mut authority = CapabilityAuthority::new();
        let read_cap = authority.issue(CapabilityKind::FilesystemRead);

        let mut guard = CapabilityGuard::new();
        guard.grant(read_cap.clone());

        assert!(guard.require(&CapabilityKind::FilesystemRead).is_ok());

        assert!(guard.revoke(&read_cap));
        assert_eq!(
            guard
                .require(&CapabilityKind::FilesystemRead)
                .unwrap_err()
                .kind(),
            &CapabilityErrorKind::Revoked
        );
    }

    #[test]
    fn test_capability_policy_from_manifest() {
        let mut entries = HashMap::new();
        entries.insert(
            "filesystem".to_string(),
            vec!["read".to_string(), "write".to_string()],
        );
        entries.insert("network".to_string(), vec!["connect".to_string()]);

        let policy = CapabilityPolicy::from_manifest_entries(&entries);

        assert!(policy.permits(&CapabilityKind::FilesystemRead));
        assert!(policy.permits(&CapabilityKind::FilesystemWrite));
        assert!(policy.permits(&CapabilityKind::NetworkAccess));
    }

    #[test]
    fn test_sandbox_executes_with_capability() {
        let mut authority = CapabilityAuthority::new();
        let network_cap = authority.issue(CapabilityKind::NetworkAccess);

        let mut policy = CapabilityPolicy::new();
        policy.declare(CapabilityKind::NetworkAccess);

        let mut guard = CapabilityGuard::new();
        guard.grant(network_cap);

        let sandbox = Sandbox::with_guard(policy, ResourceLimits::permissive(), guard);
        let value = sandbox
            .execute(&[CapabilityKind::NetworkAccess], || 7usize)
            .unwrap();

        assert_eq!(value, 7);
    }

    #[test]
    fn test_ffi_sandbox_rejects_missing_capability() {
        let policy = CapabilityPolicy::new();
        let ffi_sandbox = FfiSandbox::new(policy, ResourceLimits::strict());

        let result = ffi_sandbox.execute(&[CapabilityKind::ProcessSpawn], || 1usize);
        assert!(matches!(result, Err(error) if error.kind() == &CapabilityErrorKind::PolicyDenied));
    }

    #[test]
    fn test_ffi_sandbox_runs_when_authorized() {
        let mut authority = CapabilityAuthority::new();
        let process_cap = authority.issue(CapabilityKind::ProcessSpawn);

        let mut policy = CapabilityPolicy::allow_all();
        policy.declare(CapabilityKind::ProcessSpawn);

        let mut guard = CapabilityGuard::new();
        guard.grant(process_cap);

        let ffi_sandbox = FfiSandbox::with_guard(policy, ResourceLimits::permissive(), guard);
        let value = ffi_sandbox
            .execute(&[CapabilityKind::ProcessSpawn], || "ok")
            .unwrap();

        assert_eq!(value, "ok");
    }
}
