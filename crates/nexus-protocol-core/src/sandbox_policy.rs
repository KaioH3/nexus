//! Sandbox policy configuration.

use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::PathBuf;

/// Sandbox policy defines resource limits and access controls
/// for code execution in the WASM sandbox.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxPolicy {
    pub max_memory_mb: u64,
    pub max_cpu_time_ms: u64,
    pub allowed_paths: Vec<PathBuf>,
    pub allowed_network: bool,
    pub allowed_env: Vec<String>,
    pub blocked_syscalls: HashSet<u32>,
}

impl Default for SandboxPolicy {
    fn default() -> Self {
        Self::ai_generated_code()
    }
}

impl SandboxPolicy {
    /// Zero-trust policy - most restrictive.
    /// No network, minimal memory, short timeout.
    pub fn zero_trust() -> Self {
        Self {
            max_memory_mb: 128,
            max_cpu_time_ms: 5000,
            allowed_paths: vec![],
            allowed_network: false,
            allowed_env: vec![],
            blocked_syscalls: BLOCKED_SYSCALLS.iter().copied().collect(),
        }
    }

    /// Policy for AI-generated code (recommended default).
    /// Moderate limits, no network, minimal env vars.
    pub fn ai_generated_code() -> Self {
        Self {
            max_memory_mb: 512,
            max_cpu_time_ms: 30000,
            allowed_paths: vec![PathBuf::from("/tmp")],
            allowed_network: false,
            allowed_env: vec!["HOME".to_string(), "TMP".to_string()],
            blocked_syscalls: BLOCKED_SYSCALLS.iter().copied().collect(),
        }
    }

    /// Development policy - more lenient.
    /// Network allowed, more memory, longer timeout.
    pub fn development() -> Self {
        Self {
            max_memory_mb: 1024,
            max_cpu_time_ms: 60000,
            allowed_paths: vec![
                PathBuf::from("/tmp"),
                PathBuf::from("/workspace"),
            ],
            allowed_network: true,
            allowed_env: vec![
                "HOME".to_string(),
                "USER".to_string(),
                "PATH".to_string(),
            ],
            blocked_syscalls: BLOCKED_SYSCALLS.iter().copied().collect(),
        }
    }

    pub fn max_memory_mb(mut self, mb: u64) -> Self {
        self.max_memory_mb = mb;
        self
    }

    pub fn max_cpu_time_ms(mut self, ms: u64) -> Self {
        self.max_cpu_time_ms = ms;
        self
    }

    pub fn allow_network(mut self, allow: bool) -> Self {
        self.allowed_network = allow;
        self
    }

    pub fn allowed_paths(mut self, paths: Vec<PathBuf>) -> Self {
        self.allowed_paths = paths;
        self
    }

    pub fn allowed_env(mut self, env: Vec<String>) -> Self {
        self.allowed_env = env;
        self
    }
}

/// Syscalls that are blocked by default in all sandbox policies.
const BLOCKED_SYSCALLS: &[u32] = &[
    // Filesystem operations that could escape sandbox
    2,  // open
    3,  // close
    4,  // stat
    5,  // fstat
    9,  // mmap
    10, // munmap
    85, // readlink
    86, // mprotect
    // Network operations
    41, // socket
    42, // connect
    43, // accept
    // Process operations
    56, // clone
    57, // fork
    60, // exit
    61, // wait4
    // Admin operations
    79, // getdents
    137, // kexec_load
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zero_trust_policy() {
        let policy = SandboxPolicy::zero_trust();
        assert_eq!(policy.max_memory_mb, 128);
        assert!(!policy.allowed_network);
    }

    #[test]
    fn test_ai_generated_code_policy() {
        let policy = SandboxPolicy::ai_generated_code();
        assert_eq!(policy.max_memory_mb, 512);
        assert!(!policy.allowed_network);
        assert!(policy.allowed_env.contains(&"HOME".to_string()));
    }

    #[test]
    fn test_development_policy() {
        let policy = SandboxPolicy::development();
        assert!(policy.allowed_network);
        assert_eq!(policy.max_memory_mb, 1024);
    }

    #[test]
    fn test_policy_builder() {
        let policy = SandboxPolicy::zero_trust()
            .max_memory_mb(256)
            .max_cpu_time_ms(10000);

        assert_eq!(policy.max_memory_mb, 256);
        assert_eq!(policy.max_cpu_time_ms, 10000);
    }
}
