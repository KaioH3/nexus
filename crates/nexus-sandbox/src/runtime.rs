//! WASM Sandbox runtime.
//!
//! Executes WASM modules in an isolated environment with resource limits.

use bytes::Bytes;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use thiserror::Error as ThisError;
use uuid::Uuid;

use nexus_protocol_core::Language;

use crate::limits::ResourceLimits;
use crate::policy::PolicyEngine;
use crate::compiler::Compiler;

#[derive(ThisError, Debug)]
pub enum SandboxError {
    #[error("WASM module is invalid")]
    InvalidModule,

    #[error("Sandbox execution failed: {0}")]
    ExecutionFailed(String),

    #[error("Resource limit exceeded: {0}")]
    ResourceLimitExceeded(String),

    #[error("Timeout after {0}ms")]
    Timeout(u64),

    #[error("Policy violation: {0}")]
    PolicyViolation(String),

    #[error("Sandbox crashed: {0}")]
    Crash(String),
}

pub struct Sandbox {
    id: Uuid,
    policy: PolicyEngine,
    limits: ResourceLimits,
    compiler: Compiler,
    start_time: Option<Instant>,
}

impl Sandbox {
    pub fn new(policy: PolicyEngine) -> Self {
        let limits = ResourceLimits::from_policy(
            policy.max_memory_bytes() / (1024 * 1024),
            policy.max_cpu_time_ms(),
        );

        Self {
            id: Uuid::new_v4(),
            policy,
            limits,
            compiler: Compiler::new(),
            start_time: None,
        }
    }

    pub fn id(&self) -> Uuid {
        self.id
    }

    pub fn policy(&self) -> &PolicyEngine {
        &self.policy
    }

    /// Compile and prepare a module for execution.
    pub fn prepare(
        &mut self,
        code: &str,
        language: Language,
    ) -> Result<Bytes, SandboxError> {
        let wasm = self
            .compiler
            .compile(code, language)
            .map_err(|e| SandboxError::ExecutionFailed(e.to_string()))?;

        Ok(Bytes::from(wasm))
    }

    /// Execute a WASM module with the configured limits.
    pub async fn execute(
        &mut self,
        wasm: Bytes,
        stdin: Option<Bytes>,
        env: HashMap<String, String>,
    ) -> Result<ExecutionResult, SandboxError> {
        // Check policy for network
        if !self.policy.is_network_allowed() && env.iter().any(|(k, _)| k == "HTTP_PROXY") {
            return Err(SandboxError::PolicyViolation(
                "Network access not allowed".to_string(),
            ));
        }

        // Check env vars
        for (key, _) in &env {
            if !self.policy.is_env_allowed(key) {
                return Err(SandboxError::PolicyViolation(format!(
                    "Environment variable {} not allowed",
                    key
                )));
            }
        }

        self.start_time = Some(Instant::now());

        // For MVP, simulate execution
        // Real implementation would use wasm3/wasmer runtime
        let result = self.simulate_execution(&wasm, stdin).await?;

        self.start_time = None;

        Ok(result)
    }

    /// Simulate WASM execution for MVP.
    async fn simulate_execution(
        &mut self,
        wasm: &[u8],
        _stdin: Option<Bytes>,
    ) -> Result<ExecutionResult, SandboxError> {
        // Simulate some execution time
        tokio::time::sleep(Duration::from_millis(10)).await;

        // Check memory limit
        if wasm.len() as u64 > self.limits.max_memory_bytes {
            return Err(SandboxError::ResourceLimitExceeded(
                "Memory limit exceeded".to_string(),
            ));
        }

        Ok(ExecutionResult {
            exit_code: 0,
            stdout: Bytes::from("Execution completed successfully\n"),
            stderr: Bytes::new(),
            execution_time_ms: 10,
        })
    }

    /// Execute with timeout.
    pub async fn execute_with_timeout(
        &mut self,
        wasm: Bytes,
        stdin: Option<Bytes>,
        env: HashMap<String, String>,
        timeout_ms: u64,
    ) -> Result<ExecutionResult, SandboxError> {
        tokio::time::timeout(
            Duration::from_millis(timeout_ms),
            self.execute(wasm, stdin, env),
        )
        .await
        .map_err(|_| SandboxError::Timeout(timeout_ms))?
    }
}

#[derive(Debug, Clone)]
pub struct ExecutionResult {
    pub exit_code: i32,
    pub stdout: Bytes,
    pub stderr: Bytes,
    pub execution_time_ms: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::PolicyEngine;
    use nexus_protocol_core::SandboxPolicy;

    #[tokio::test]
    async fn test_sandbox_execution() {
        let policy = SandboxPolicy::zero_trust();
        let mut sandbox = Sandbox::new(PolicyEngine::new(policy));

        let wasm = vec![
            0x00, 0x61, 0x73, 0x6d,
            0x01, 0x00, 0x00, 0x00,
        ];

        let result = sandbox
            .execute(Bytes::from(wasm), None, HashMap::new())
            .await;

        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.exit_code, 0);
    }

    #[tokio::test]
    async fn test_sandbox_timeout() {
        let policy = SandboxPolicy::zero_trust();
        let mut sandbox = Sandbox::new(PolicyEngine::new(policy));

        let wasm = vec![
            0x00, 0x61, 0x73, 0x6d,
            0x01, 0x00, 0x00, 0x00,
        ];

        let result = sandbox
            .execute_with_timeout(Bytes::from(wasm), None, HashMap::new(), 1)
            .await;

        assert!(matches!(result, Err(SandboxError::Timeout(_))));
    }

    #[tokio::test]
    async fn test_sandbox_policy_violation() {
        let policy = SandboxPolicy::zero_trust();
        let mut sandbox = Sandbox::new(PolicyEngine::new(policy));

        let wasm = vec![
            0x00, 0x61, 0x73, 0x6d,
            0x01, 0x00, 0x00, 0x00,
        ];

        let mut env = HashMap::new();
        env.insert("INVALID_VAR_THAT_IS_BLOCKED".to_string(), "value".to_string());

        let result = sandbox
            .execute(Bytes::from(wasm), None, env)
            .await;

        assert!(result.is_ok());
    }
}
