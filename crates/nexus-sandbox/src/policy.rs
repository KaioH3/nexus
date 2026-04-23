//! Policy engine for sandbox security.

use nexus_protocol_core::SandboxPolicy;
use std::collections::HashSet;

/// Policy engine validates sandbox operations against configured policies.
#[derive(Debug, Clone)]
pub struct PolicyEngine {
    policy: SandboxPolicy,
}

impl PolicyEngine {
    pub fn new(policy: SandboxPolicy) -> Self {
        Self { policy }
    }

    pub fn policy(&self) -> &SandboxPolicy {
        &self.policy
    }

    /// Check if a syscall is allowed.
    pub fn is_syscall_allowed(&self, syscall: u32) -> bool {
        !self.policy.blocked_syscalls.contains(&syscall)
    }

    /// Check if network access is allowed.
    pub fn is_network_allowed(&self) -> bool {
        self.policy.allowed_network
    }

    /// Check if a path is accessible.
    pub fn is_path_allowed(&self, path: &std::path::Path) -> bool {
        if self.policy.allowed_paths.is_empty() {
            return true; // No restrictions if no paths specified
        }

        for allowed in &self.policy.allowed_paths {
            if path.starts_with(allowed) {
                return true;
            }
        }
        false
    }

    /// Check if an environment variable is allowed.
    pub fn is_env_allowed(&self, var: &str) -> bool {
        if self.policy.allowed_env.is_empty() {
            return true;
        }

        for allowed in &self.policy.allowed_env {
            if var == allowed {
                return true;
            }
        }
        false
    }

    /// Get all blocked syscalls.
    pub fn blocked_syscalls(&self) -> &HashSet<u32> {
        &self.policy.blocked_syscalls
    }

    /// Get max memory in bytes.
    pub fn max_memory_bytes(&self) -> u64 {
        self.policy.max_memory_mb * 1024 * 1024
    }

    /// Get max CPU time in milliseconds.
    pub fn max_cpu_time_ms(&self) -> u64 {
        self.policy.max_cpu_time_ms
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nexus_protocol_core::SandboxPolicy;

    #[test]
    fn test_syscall_blocking() {
        let policy = SandboxPolicy::zero_trust();
        let engine = PolicyEngine::new(policy);

        // clone (56) should be blocked
        assert!(!engine.is_syscall_allowed(56));
        // socket (41) should be blocked
        assert!(!engine.is_syscall_allowed(41));
    }

    #[test]
    fn test_network_allowed() {
        let policy = SandboxPolicy::zero_trust();
        let engine = PolicyEngine::new(policy);
        assert!(!engine.is_network_allowed());

        let policy = SandboxPolicy::development();
        let engine = PolicyEngine::new(policy);
        assert!(engine.is_network_allowed());
    }

    #[test]
    fn test_path_allowed() {
        let policy = SandboxPolicy::ai_generated_code();
        let engine = PolicyEngine::new(policy);

        assert!(engine.is_path_allowed(std::path::Path::new("/tmp")));
        assert!(!engine.is_path_allowed(std::path::Path::new("/etc/passwd")));
    }

    #[test]
    fn test_env_allowed() {
        let policy = SandboxPolicy::zero_trust();
        let _engine = PolicyEngine::new(policy);

        let policy = SandboxPolicy::development();
        let engine = PolicyEngine::new(policy);

        assert!(engine.is_env_allowed("HOME"));
    }

    #[test]
    fn test_resource_limits() {
        let policy = SandboxPolicy::zero_trust();
        let engine = PolicyEngine::new(policy);

        assert_eq!(engine.max_memory_bytes(), 128 * 1024 * 1024);
        assert_eq!(engine.max_cpu_time_ms(), 5_000);
    }
}
