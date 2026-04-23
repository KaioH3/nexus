//! Test example: Connect to local Ollama via Nexus Protocol types
//!
//! Run with: cargo run --example ollama_test --package nexus-protocol-core
//!
//! This tests the nexus-ollama client against your local Ollama instance.
//! Your available models: qwen2.5-coder:3b, qwen2.5-coder:0.5b, deepseek-coder:1.3b, etc.

use nexus_protocol_core::{message::Message, language::Language, sandbox_policy::SandboxPolicy, capabilities::Capabilities};
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Nexus Protocol + Ollama Test ===\n");
    println!("Available models on your system:");
    println!("  - qwen2.5-coder:3b (3.1B params)");
    println!("  - qwen2.5-coder:0.5b (494M params)");
    println!("  - deepseek-coder:1.3b (1B params)");
    println!("  - opencoder:1.5b (1.9B params)");
    println!("  - yi-coder:1.5b (1.5B params)");
    println!("  - gemma3:4b, gemma3:1b");
    println!("  - llama3.2:1b");
    println!("  - And more...\n");

    // Test 1: Connect to Ollama and list models
    println!("[TEST 1] Connecting to Ollama at http://localhost:11434...");
    let client = nexus_ollama::OllamaClient::new("http://localhost:11434")?;

    let models = client.connect().await?;
    println!("  ✓ Connected! Found {} models:", models.len());
    for model in &models {
        let size_mb = model.size as f64 / 1_000_000.0;
        println!("    - {} ({:.1} MB)", model.name, size_mb);
    }

    // Test 2: Generate with qwen2.5-coder:3b
    println!("\n[TEST 2] Generating with qwen2.5-coder:3b...");
    let response = client.generate(
        "qwen2.5-coder:3b",
        "Write a simple HTTP server in Rust that responds with 'Hello, World!'",
        None,
    ).await?;

    println!("  ✓ Response ({} chars):", response.len());
    println!("  ---\n{}\n---", response.lines().take(10).collect::<Vec<_>>().join("\n"));

    // Test 3: Generate with deepseek-coder
    println!("\n[TEST 3] Generating with deepseek-coder:1.3b...");
    let response = client.generate(
        "deepseek-coder:1.3b",
        "Explain what a goroutine is in Go in one sentence",
        None,
    ).await?;

    println!("  ✓ Response:");
    println!("  {}", response);

    // Test 4: Test Nexus Protocol message types
    println!("\n[TEST 4] Testing Nexus Protocol message serialization...");
    let msg = Message::Execute {
        request_id: Uuid::new_v4(),
        code: "fn main() { println!(\"Hello!\"); }".to_string(),
        language: Language::Rust,
        sandbox_policy: SandboxPolicy::default(),
        model_hint: Some("qwen2.5-coder:3b".to_string()),
    };

    let json = serde_json::to_string_pretty(&msg)?;
    println!("  ✓ Execute message JSON:");
    println!("  {}", json.lines().take(15).collect::<Vec<_>>().join("\n"));

    // Test 5: Capabilities check
    println!("\n[TEST 5] Checking capabilities...");
    let caps = Capabilities::default();
    println!("  ✓ Default capabilities:");
    println!("    - WASM runtimes: {:?}", caps.wasm_runtimes);
    println!("    - Ollama: {}", caps.ollama);
    println!("    - Streaming: {}", caps.streaming);

    println!("\n=== All tests passed! ===");
    println!("\nYour Iris Xe can handle these models well:");
    println!("  - qwen2.5-coder:0.5b (fastest, ~494MB)");
    println!("  - deepseek-coder:1.3b (good balance)");
    println!("  - qwen2.5-coder:3b (best quality but slower)");

    Ok(())
}