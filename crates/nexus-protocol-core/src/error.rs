//! Error types for Nexus Protocol.

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Error, Serialize, Deserialize)]
#[error("Nexus error: {code:?} - {message}")]
pub struct Error {
    pub code: ErrorCode,
    pub message: String,
    pub request_id: Option<String>,
}

impl Error {
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            request_id: None,
        }
    }

    pub fn with_request_id(mut self, request_id: String) -> Self {
        self.request_id = Some(request_id);
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCode {
    HandshakeFailed,
    VersionMismatch,
    CompilationFailed,
    SandboxViolation,
    SandboxTimeout,
    SandboxOutOfMemory,
    OllamaConnectionFailed,
    OllamaGenerationFailed,
    NetworkBlocked,
    SyscallBlocked,
    FileNotFound,
    PermissionDenied,
    InvalidMessage,
    MissingApiKey,
    InvalidApiKey,
    RateLimited,
    InternalError,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_new() {
        let err = Error::new(ErrorCode::CompilationFailed, "rustc failed");
        assert!(matches!(err.code, ErrorCode::CompilationFailed));
        assert_eq!(err.message, "rustc failed");
        assert!(err.request_id.is_none());
    }

    #[test]
    fn test_error_with_request_id() {
        let err = Error::new(ErrorCode::SandboxTimeout, "timed out")
            .with_request_id("req-123".to_string());
        assert_eq!(err.request_id, Some("req-123".to_string()));
    }
}
