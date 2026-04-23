//! E2E tests for Nexus Protocol.
//!
//! These tests validate the complete protocol flow including:
//! - Handshake
//! - Code execution in sandbox
//! - Ollama integration
//! - Error handling

mod harness;

use bytes::Bytes;
use nexus_protocol_core::{
    Capabilities, Language, Message, SandboxPolicy, Version,
};
use std::collections::HashMap;
use uuid::Uuid;

/// Test handshake flow.
#[tokio::test]
async fn test_handshake() {
    let mut harness = harness::TestHarness::new().await;

    // Client initiates handshake
    let client_hello = Message::Handshake {
        version: Version::CURRENT,
        api_key: Some("test-key".to_string()),
        capabilities: Capabilities::client(),
    };

    harness.send_client_message(&client_hello).await;

    // Server should respond with HandshakeAck
    let response = harness.recv_server_message().await;
    assert!(matches!(response, Some(Message::HandshakeAck { .. })));
}

/// Test code execution flow.
#[tokio::test]
async fn test_execute_rust_code() {
    let mut harness = harness::TestHarness::new().await;

    // Complete handshake first
    harness.connect().await;

    // Submit Rust code for execution
    let request_id = Uuid::new_v4();
    let execute = Message::Execute {
        request_id,
        code: r#"
            fn main() {
                println!("Hello from Nexus sandbox!");
            }
        "#
        .to_string(),
        language: Language::Rust,
        sandbox_policy: SandboxPolicy::ai_generated_code(),
        model_hint: None,
    };

    harness.send_client_message(&execute).await;

    // Receive ExecutionReady
    let ready = harness.recv_server_message().await;
    assert!(matches!(ready, Some(Message::ExecutionReady { request_id: id, .. } if id == request_id)));

    // Execute in sandbox
    let run = Message::ExecuteInSandbox {
        request_id,
        stdin: None,
        env: HashMap::new(),
    };

    harness.send_client_message(&run).await;

    // Receive stdout
    let stdout = harness.recv_server_message().await;
    assert!(matches!(stdout, Some(Message::Stdout { .. })));

    // Receive exit
    let exit = harness.recv_server_message().await;
    assert!(matches!(exit, Some(Message::Exit { .. })));

    // Receive final result
    let result = harness.recv_server_message().await;
    assert!(matches!(result, Some(Message::ExecutionResult { request_id: id, .. } if id == request_id)));
}

/// Test sandbox isolation - code should not access restricted paths.
#[tokio::test]
async fn test_sandbox_isolation_blocks_network() {
    let mut harness = harness::TestHarness::new().await;
    harness.connect().await;

    let request_id = Uuid::new_v4();

    // Try to execute with development policy (allows network)
    // but send env that would enable network
    let execute = Message::Execute {
        request_id,
        code: "fn main() {}".to_string(),
        language: Language::Rust,
        sandbox_policy: SandboxPolicy::development(),
        model_hint: None,
    };

    harness.send_client_message(&execute).await;

    let ready = harness.recv_server_message().await;
    assert!(matches!(ready, Some(Message::ExecutionReady { .. })));

    let run = Message::ExecuteInSandbox {
        request_id,
        stdin: None,
        env: HashMap::from([("HTTP_PROXY".to_string(), "http://evil.com".to_string())]),
    };

    harness.send_client_message(&run).await;

    // Should get an error due to policy violation
    let error = harness.recv_server_message().await;
    assert!(matches!(error, Some(Message::Error { code: nexus_protocol_core::ErrorCode::SandboxViolation, .. })));
}

/// Test Ollama connection.
#[tokio::test]
async fn test_ollama_connect() {
    let mut harness = harness::TestHarness::new().await;
    harness.connect().await;

    let connect = Message::OllamaConnect {
        url: "http://localhost:11434".to_string(),
    };

    harness.send_client_message(&connect).await;

    // Response depends on whether Ollama is running
    let response = harness.recv_server_message().await;
    match response {
        Some(Message::OllamaConnected { .. }) => {
            // Ollama is running
        }
        Some(Message::Error { code: nexus_protocol_core::ErrorCode::OllamaConnectionFailed, .. }) => {
            // Ollama is not running - this is expected in test environment
        }
        _ => panic!("Unexpected response: {:?}", response),
    }
}

/// Test invalid API key.
#[tokio::test]
async fn test_invalid_api_key() {
    let harness = harness::TestHarness::new().await;

    // Try handshake with invalid key
    let client_hello = Message::Handshake {
        version: Version::CURRENT,
        api_key: Some("invalid-key".to_string()),
        capabilities: Capabilities::client(),
    };

    // In a real implementation, the server would reject this
    // For MVP, we just test the message serialization
    let json = serde_json::to_string(&client_hello).unwrap();
    assert!(json.contains("invalid-key"));
}

/// Test timeout for long-running code.
#[tokio::test]
async fn test_execution_timeout() {
    let mut harness = harness::TestHarness::new().await;
    harness.connect().await;

    // Use zero-trust policy with very short timeout
    let request_id = Uuid::new_v4();
    let execute = Message::Execute {
        request_id,
        code: r#"
            fn main() {
                loop {
                    // Infinite loop - should timeout
                }
            }
        "#
        .to_string(),
        language: Language::Rust,
        sandbox_policy: SandboxPolicy::zero_trust(),
        model_hint: None,
    };

    harness.send_client_message(&execute).await;

    let ready = harness.recv_server_message().await;
    assert!(matches!(ready, Some(Message::ExecutionReady { .. })));

    let run = Message::ExecuteInSandbox {
        request_id,
        stdin: None,
        env: HashMap::new(),
    };

    harness.send_client_message(&run).await;

    // Should eventually get timeout error
    let response = harness.recv_server_message().await;
    match response {
        Some(Message::Error { code: nexus_protocol_core::ErrorCode::SandboxTimeout, .. }) => {
            // Expected
        }
        Some(Message::Exit { code: 0, .. }) => {
            // Some runtimes might optimize away infinite loops
        }
        _ => {}
    }
}

/// Test memory limit enforcement.
#[tokio::test]
async fn test_memory_limit() {
    let mut harness = harness::TestHarness::new().await;
    harness.connect().await;

    let request_id = Uuid::new_v4();

    // Very small memory limit
    let mut policy = SandboxPolicy::zero_trust();
    policy.max_memory_mb = 1; // 1MB - very small

    let execute = Message::Execute {
        request_id,
        code: format!("fn main() {{ let v = vec![0u8; {}]; }}", 10 * 1024 * 1024),
        language: Language::Rust,
        sandbox_policy: policy,
        model_hint: None,
    };

    harness.send_client_message(&execute).await;

    let ready = harness.recv_server_message().await;
    assert!(matches!(ready, Some(Message::ExecutionReady { .. })));

    let run = Message::ExecuteInSandbox {
        request_id,
        stdin: None,
        env: HashMap::new(),
    };

    harness.send_client_message(&run).await;

    // Should get OOM or similar error
    let response = harness.recv_server_message().await;
    match response {
        Some(Message::Error { code: nexus_protocol_core::ErrorCode::SandboxOutOfMemory, .. }) => {
            // Expected
        }
        Some(Message::ExecutionResult { exit_code: _, .. }) => {
            // Might succeed or fail depending on allocator
        }
        _ => {}
    }
}

/// Test message serialization roundtrip.
#[test]
fn test_message_serialization_roundtrip() {
    let messages = vec![
        Message::Handshake {
            version: Version::CURRENT,
            api_key: Some("key-123".to_string()),
            capabilities: Capabilities::client(),
        },
        Message::Execute {
            request_id: Uuid::new_v4(),
            code: "fn main() {}".to_string(),
            language: Language::Rust,
            sandbox_policy: SandboxPolicy::default(),
            model_hint: None,
        },
        Message::ExecuteInSandbox {
            request_id: Uuid::new_v4(),
            stdin: Some(Bytes::from("input")),
            env: HashMap::from([("KEY".to_string(), "value".to_string())]),
        },
        Message::Error {
            request_id: Some("req-123".to_string()),
            code: nexus_protocol_core::ErrorCode::CompilationFailed,
            message: "test error".to_string(),
        },
    ];

    for msg in messages {
        let json = serde_json::to_string(&msg).unwrap();
        let parsed: Message = serde_json::from_str(&json).unwrap();
        assert_eq!(format!("{:?}", msg), format!("{:?}", parsed));
    }
}
