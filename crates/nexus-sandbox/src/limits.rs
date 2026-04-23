//! Resource limits for sandbox execution.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    pub max_memory_bytes: u64,
    pub max_cpu_time_ms: u64,
    pub max_disk_bytes: u64,
    pub max_open_files: u32,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_memory_bytes: 512 * 1024 * 1024, // 512 MB
            max_cpu_time_ms: 30_000,             // 30 seconds
            max_disk_bytes: 100 * 1024 * 1024,  // 100 MB
            max_open_files: 16,
        }
    }
}

impl ResourceLimits {
    pub fn new(
        max_memory_mb: u64,
        max_cpu_time_ms: u64,
        max_disk_mb: u64,
        max_open_files: u32,
    ) -> Self {
        Self {
            max_memory_bytes: max_memory_mb * 1024 * 1024,
            max_cpu_time_ms,
            max_disk_bytes: max_disk_mb * 1024 * 1024,
            max_open_files,
        }
    }

    pub fn from_policy(max_memory_mb: u64, max_cpu_time_ms: u64) -> Self {
        Self {
            max_memory_bytes: max_memory_mb * 1024 * 1024,
            max_cpu_time_ms,
            max_disk_bytes: 100 * 1024 * 1024,
            max_open_files: 16,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LimitType {
    Memory,
    CpuTime,
    Disk,
    OpenFiles,
}

impl std::fmt::Display for LimitType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LimitType::Memory => write!(f, "memory"),
            LimitType::CpuTime => write!(f, "cpu_time"),
            LimitType::Disk => write!(f, "disk"),
            LimitType::OpenFiles => write!(f, "open_files"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resource_limits_default() {
        let limits = ResourceLimits::default();
        assert_eq!(limits.max_memory_bytes, 512 * 1024 * 1024);
        assert_eq!(limits.max_cpu_time_ms, 30_000);
    }

    #[test]
    fn test_resource_limits_from_policy() {
        let limits = ResourceLimits::from_policy(256, 10_000);
        assert_eq!(limits.max_memory_bytes, 256 * 1024 * 1024);
        assert_eq!(limits.max_cpu_time_ms, 10_000);
    }

    #[test]
    fn test_resource_limits_new() {
        let limits = ResourceLimits::new(1024, 60_000, 200, 32);
        assert_eq!(limits.max_memory_bytes, 1024 * 1024 * 1024);
        assert_eq!(limits.max_cpu_time_ms, 60_000);
        assert_eq!(limits.max_disk_bytes, 200 * 1024 * 1024);
        assert_eq!(limits.max_open_files, 32);
    }
}
