//! Test harness for E2E tests.
//!
//! Provides a mock server and client for testing the protocol.

use bytes::Bytes;
use futures_util::{SinkExt, StreamExt};
use nexus_protocol_core::{Capabilities, Message, SandboxPolicy, Version, Language, ErrorCode};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{mpsc, Mutex};
use uuid::Uuid;

/// Test harness for running E2E protocol tests.
pub struct TestHarness {
    port: u16,
    shutdown_tx: Option<mpsc::Sender<()>>,
}

impl TestHarness {
    /// Create a new test harness with a mock server.
    pub async fn new() -> Self {
        let port = 0; // Let OS assign a free port
        let listener = TcpListener::bind(("127.0.0.1", port))
            .await
            .expect("Failed to bind TCP listener");

        let addr = listener.local_addr().unwrap();
        let port = addr.port();

        let (shutdown_tx, shutdown_rx) = mpsc::channel::<()>(1);

        // Spawn mock server
        tokio::spawn(async move {
            Self::mock_server(listener, shutdown_rx).await;
        });

        // Give server time to start
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        Self {
            port,
            shutdown_tx: Some(shutdown_tx),
        }
    }

    /// Run the mock server.
    async fn mock_server(listener: TcpListener, mut shutdown_rx: mpsc::Receiver<()>) {
        loop {
            tokio::select! {
                _ = shutdown_rx.recv() => {
                    break;
                }
                result = listener.accept() => {
                    match result {
                        Ok((stream, _)) => {
                            let _ = Self::handle_client(stream).await;
                        }
                        Err(e) => {
                            eprintln!("Accept error: {}", e);
                        }
                    }
                }
            }
        }
    }

    /// Handle a client connection.
    async fn handle_client(mut stream: TcpStream) -> Result<(), std::io::Error> {
        let mut buffer = vec![0u8; 8192];

        loop {
            let n = stream.read(&mut buffer).await?;
            if n == 0 {
                break;
            }

            let data = &buffer[..n];
            let msg: Result<Message, _> = serde_json::from_slice(data);

            match msg {
                Ok(msg) => {
                    let response = Self::handle_message(&msg);
                    if let Some(response) = response {
                        let json = serde_json::to_string(&response).unwrap();
                        let response_bytes = json.as_bytes();
                        stream.write_all(response_bytes).await?;
                        stream.write_all(b"\n").await?;
                    }
                }
                Err(e) => {
                    eprintln!("Failed to parse message: {}", e);
                    break;
                }
            }
        }

        Ok(())
    }

    /// Handle an incoming message and return a response.
    fn handle_message(msg: &Message) -> Option<Message> {
        match msg {
            Message::Handshake { .. } => {
                Some(Message::HandshakeAck {
                    session_id: Uuid::new_v4(),
                    server_version: Version::CURRENT,
                    capabilities: Capabilities::full(),
                })
            }
            Message::Execute { request_id, .. } => {
                // Return mock WASM module
                Some(Message::ExecutionReady {
                    request_id: *request_id,
                    wasm_module: Bytes::from(vec![
                        0x00, 0x61, 0x73, 0x6d, // WASM magic
                        0x01, 0x00, 0x00, 0x00, // Version
                    ]),
                })
            }
            Message::ExecuteInSandbox { request_id, .. } => {
                Some(Message::Stdout {
                    data: Bytes::from("Hello from mock sandbox!\n"),
                })
            }
            _ => None,
        }
    }

    /// Connect to the mock server (completes handshake).
    pub async fn connect(&mut self) {
        let mut stream = TcpStream::connect(("127.0.0.1", self.port))
            .await
            .expect("Failed to connect to mock server");

        let client_hello = Message::Handshake {
            version: Version::CURRENT,
            api_key: Some("test-key".to_string()),
            capabilities: Capabilities::client(),
        };

        let json = serde_json::to_string(&client_hello).unwrap();
        stream.write_all(json.as_bytes()).await?;
        stream.write_all(b"\n").await?;

        // Read response
        let mut response = vec![0u8; 8192];
        let n = stream.read(&mut response).await.expect("Failed to read");
        response.truncate(n);
    }

    /// Send a message to the server.
    pub async fn send_client_message(&mut self, msg: &Message) {
        let mut stream = TcpStream::connect(("127.0.0.1", self.port))
            .await
            .expect("Failed to connect");

        let json = serde_json::to_string(msg).unwrap();
        stream.write_all(json.as_bytes()).await?;
        stream.write_all(b"\n").await?;
    }

    /// Receive a message from the server.
    pub async fn recv_server_message(&mut self) -> Option<Message> {
        let mut stream = TcpStream::connect(("127.0.0.1", self.port))
            .await
            .expect("Failed to connect");

        let mut response = vec![0u8; 8192];
        let n = stream.read(&mut response).await.ok()?;
        if n == 0 {
            return None;
        }
        response.truncate(n);

        serde_json::from_slice(&response).ok()
    }
}

impl Drop for TestHarness {
    fn drop(&mut self) {
        if let Some(shutdown_tx) = self.shutdown_tx.take() {
            let _ = shutdown_tx.try_send(());
        }
    }
}
