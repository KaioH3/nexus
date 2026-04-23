//! Message types for Nexus Protocol.
//!
//! All messages follow a tagged enum format for easy serialization/deserialization
//! and efficient routing in the router.

use bytes::Bytes;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::capabilities::Capabilities;
use crate::error::ErrorCode;
use crate::language::Language;
use crate::sandbox_policy::SandboxPolicy;
use crate::version::Version;

/// All messages in the Nexus Protocol.
///
/// Messages are serialized as JSON with a "type" tag field.
/// Example: `{"type": "handshake", "version": "0.1.0", ...}`
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Message {
    // ========================================================================
    // Connection messages
    // ========================================================================

    /// Client initiates handshake.
    Handshake {
        version: Version,
        api_key: Option<String>,
        capabilities: Capabilities,
    },

    /// Server acknowledges handshake.
    HandshakeAck {
        session_id: Uuid,
        server_version: Version,
        capabilities: Capabilities,
    },

    // ========================================================================
    // Execution messages
    // ========================================================================

    /// Client requests code execution.
    Execute {
        request_id: Uuid,
        code: String,
        language: Language,
        sandbox_policy: SandboxPolicy,
        model_hint: Option<String>,
    },

    /// Server confirms compilation, WASM module ready.
    ExecutionReady {
        request_id: Uuid,
        wasm_module: Bytes,
    },

    /// Client triggers execution in sandbox.
    ExecuteInSandbox {
        request_id: Uuid,
        stdin: Option<Bytes>,
        env: HashMap<String, String>,
    },

    /// Server sends stdout data.
    Stdout {
        data: Bytes,
    },

    /// Server sends stderr data.
    Stderr {
        data: Bytes,
    },

    /// Process exited.
    Exit {
        code: i32,
        duration_ms: u64,
    },

    /// Final execution result.
    ExecutionResult {
        request_id: Uuid,
        exit_code: i32,
        stdout: Bytes,
        stderr: Bytes,
        execution_time_ms: u64,
        cache_hit: bool,
    },

    // ========================================================================
    // Ollama messages
    // ========================================================================

    /// Connect to local Ollama instance.
    OllamaConnect {
        url: String,
    },

    /// Ollama connection established.
    OllamaConnected {
        models: Vec<String>,
    },

    /// Generate text with Ollama.
    OllamaGenerate {
        request_id: Uuid,
        model: String,
        prompt: String,
        options: GenerateOptions,
        stream: bool,
    },

    /// Streamed token from Ollama.
    OllamaToken {
        token: String,
    },

    /// Ollama generation complete.
    OllamaDone {
        stats: GenerationStats,
    },

    // ========================================================================
    // Error messages
    // ========================================================================

    /// Generic error.
    Error {
        request_id: Option<String>,
        code: ErrorCode,
        message: String,
    },
}

/// Generation options for LLM inference.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateOptions {
    pub temperature: Option<f32>,
    pub top_p: Option<f32>,
    pub top_k: Option<u32>,
    pub repeat_penalty: Option<f32>,
    pub num_predict: Option<u32>,
}

impl Default for GenerateOptions {
    fn default() -> Self {
        Self {
            temperature: Some(0.8),
            top_p: Some(0.9),
            top_k: Some(40),
            repeat_penalty: None,
            num_predict: Some(256),
        }
    }
}

/// Statistics for LLM generation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationStats {
    pub model: String,
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
    pub duration_ms: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_handshake_serialization() {
        let msg = Message::Handshake {
            version: Version::CURRENT,
            api_key: Some("key-123".to_string()),
            capabilities: Capabilities::default(),
        };

        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"handshake\""));
        assert!(json.contains("\"version\""));

        let parsed: Message = serde_json::from_str(&json).unwrap();
        assert!(matches!(parsed, Message::Handshake { .. }));
    }

    #[test]
    fn test_execute_serialization() {
        let msg = Message::Execute {
            request_id: Uuid::new_v4(),
            code: "fn main() {}".to_string(),
            language: Language::Rust,
            sandbox_policy: SandboxPolicy::default(),
            model_hint: None,
        };

        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"execute\""));
        assert!(json.contains("\"rust\""));

        let parsed: Message = serde_json::from_str(&json).unwrap();
        assert!(matches!(parsed, Message::Execute { language: Language::Rust, .. }));
    }

    #[test]
    fn test_error_serialization() {
        let msg = Message::Error {
            request_id: Some("req-123".to_string()),
            code: ErrorCode::CompilationFailed,
            message: "rustc: unable to find compiler".to_string(),
        };

        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"error\""));
        assert!(json.contains("\"compilation_failed\""));

        let parsed: Message = serde_json::from_str(&json).unwrap();
        assert!(matches!(parsed, Message::Error { code: ErrorCode::CompilationFailed, .. }));
    }

    #[test]
    fn test_generate_options_default() {
        let opts = GenerateOptions::default();
        assert_eq!(opts.temperature, Some(0.8));
        assert_eq!(opts.num_predict, Some(256));
    }
}
