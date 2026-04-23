# Nexus Protocol — Multi-Language SDK Architecture

**Goal:** Adotado por TODOS os desenvolvedores, em QUALQUER linguagem.

---

## The Universal Truth

```
QUALQUER linguagem que pode fazer HTTP/WebSocket pode usar Nexus Protocol.

Não importa se é Python, Go, TypeScript, Rust, Java, C#, Ruby, PHP, Swift, Kotlin...

O protocolo é JSON puro sobre WebSocket.

Você só precisa de:
1. Conectar WebSocket
2. Enviar JSON
3. Receber JSON
4. Pronto.
```

---

## SDK Architecture Overview

```
┌─────────────────────────────────────────────────────────────────────────┐
│                     Nexus Protocol (JSON/WebSocket)                   │
│                                                                     │
│   ┌─────────────────────────────────────────────────────────────┐   │
│   │                    nexus-protocol-core (Rust)                │   │
│   │  - Message types                                             │   │
│   │  - Sandbox policy                                            │   │
│   │  - Capabilities                                             │   │
│   │  - Error codes                                              │   │
│   └─────────────────────────────────────────────────────────────┘   │
│                             │                                       │
│         ┌───────────────────┼───────────────────┐                   │
│         ▼                   ▼                   ▼                   │
│   ┌─────────────┐     ┌─────────────┐     ┌─────────────┐        │
│   │  Rust SDK  │     │   Go SDK    │     │    TS SDK   │        │
│   │  (native)  │     │  (native)  │     │  (native)  │        │
│   └─────────────┘     └─────────────┘     └─────────────┘        │
│         │                   │                   │                   │
│         ▼                   ▼                   ▼                   │
│   ┌─────────────┐     ┌─────────────┐     ┌─────────────┐        │
│   │  Python SDK │     │   Ruby SDK  │     │   Java SDK  │        │
│   │   (PyO3)   │     │  (native)  │     │   (native)  │        │
│   └─────────────┘     └─────────────┘     └─────────────┘        │
│         │                   │                   │                   │
│         ▼                   ▼                   ▼                   │
│   ┌─────────────┐     ┌─────────────┐     ┌─────────────┐        │
│   │  Swift SDK  │     │  Kotlin SDK │     │   C# SDK    │        │
│   │  (native)  │     │  (native)  │     │  (native)  │        │
│   └─────────────┘     └─────────────┘     └─────────────┘        │
│                                                                     │
│   + HTTP/REST fallback (pra qualquer linguagem que quiser)         │
└─────────────────────────────────────────────────────────────────────┘
```

---

## Protocol Specification (Language-Agnostic)

### Connection Flow

```
1. Client connects to: ws://host:port/api/v1/ws
2. Client sends handshake:
   {
     "type": "handshake",
     "version": "0.1.0",
     "api_key": "optional-key",
     "capabilities": {
       "wasm_runtimes": ["wasm3", "wasmer"],
       "ollama": true,
       "streaming": true
     }
   }
3. Server responds:
   {
     "type": "handshake_ack",
     "session_id": "uuid",
     "server_version": "0.1.0",
     "capabilities": { ... }
   }
4. Exchange messages...
```

### Message Types (Complete)

```json
// Execute code
{
  "type": "execute",
  "request_id": "uuid",
  "code": "print('hello')",
  "language": "python",
  "sandbox_policy": {
    "max_memory_mb": 512,
    "max_cpu_time_ms": 30000,
    "allowed_paths": ["/tmp"],
    "allowed_network": false,
    "allowed_env": ["HOME"],
    "blocked_syscalls": [2, 3, 4, 5, 9, 10, 41, 42, 43, 56, 57, 60, 61, 79, 85, 86, 137]
  }
}

// Receive result
{
  "type": "execution_result",
  "request_id": "uuid",
  "exit_code": 0,
  "stdout": "hello\n",
  "stderr": "",
  "execution_time_ms": 10,
  "cache_hit": false
}

// Ollama generate
{
  "type": "ollama_generate",
  "request_id": "uuid",
  "model": "qwen2.5-coder:3b",
  "prompt": "Explain this code",
  "options": {
    "temperature": 0.8,
    "top_p": 0.9,
    "top_k": 40,
    "num_predict": 256
  },
  "stream": true
}

// Stream token
{
  "type": "ollama_token",
  "token": "Hello"
}
```

---

## SDK Implementations

### 1. Rust SDK (native, reference implementation)

```rust
// crates/nexus-sdk/rust/src/lib.rs

pub struct NexusClient {
    ws: WebSocket,
    session_id: Uuid,
}

impl NexusClient {
    pub async fn connect(url: &str, api_key: Option<&str>) -> Result<Self> { ... }

    pub async fn execute(&self, code: &str, language: Language) -> Result<ExecutionResult> { ... }

    pub async fn generate(&self, model: &str, prompt: &str) -> Result<String> { ... }

    pub async fn generate_streaming(&self, model: &str, prompt: &str) -> Result<Stream<String>> { ... }
}

pub async fn example() -> Result<()> {
    let client = NexusClient::connect("ws://localhost:8080", None).await?;
    let result = client.execute("print('hello')", Language::Python).await?;
    println!("{}", result.stdout);
    Ok(())
}
```

### 2. Python SDK (PyO3, high-performance)

```python
# PyO3 extension: nexus_sdk/_native.pydarwin_arm64.so
# Or built via: maturin develop

from nexus_sdk._native import LazyAgent, ExecutionResult

class Agent:
    """
    Pythonic interface with Polars-style ergonomics.

    Usage:
        >>> from nexus import Agent
        >>> agent = Agent(model="qwen2.5-coder:3b")
        >>> result = agent.execute("print('hello')")
    """

    def __init__(
        self,
        model: str = "qwen2.5-coder:3b",
        *,
        tools: Optional[list[str]] = None,
        sandbox: str = "wasm",
    ):
        self._inner = LazyAgent(model=model)

    def execute(self, prompt: str, *, stream: bool = False) -> ExecutionResult | Iterator[str]:
        if stream:
            return self._inner.execute_streaming(prompt)
        return self._inner.execute(prompt)

    # Polars-style lazy API
    def tools(self, *names: str) -> "Agent":
        """Chainable tools configuration."""
        self._inner = self._inner.tools(list(names))
        return self

    def timeout(self, seconds: int) -> "Agent":
        """Chainable timeout configuration."""
        self._inner = self._inner.timeout(seconds * 1000)
        return self

    def system(self, prompt: str) -> "Agent":
        """Chainable system prompt."""
        self._inner = self._inner.system(prompt)
        return self

    # Convenience methods
    def code_review(self, path: str) -> str:
        """One-liner: code review."""
        return self.execute(f"Review code at {path}").output

    def fix_bugs(self, path: str) -> str:
        """One-liner: auto-fix bugs."""
        return self.execute(f"Fix bugs in {path}").output

    def explain(self, code: str) -> str:
        """One-liner: explain code."""
        return self.execute(f"Explain: {code}").output

    def generate_tests(self, path: str) -> str:
        """One-liner: generate tests."""
        return self.execute(f"Generate tests for {path}").output


# Install: pip install nexus-ai
# Run: python -c "from nexus import Agent; print(Agent().execute('hello'))"
```

### 3. Go SDK (already exists, improve)

```go
// crates/nexus-sdk/go/nexus.go

package nexus

type Client struct {
    baseURL string
    apiKey  string
    ws      *websocket.Conn
}

func NewClient(cfg Config) (*Client, error) {
    return &Client{
        baseURL: cfg.BaseURL,
        apiKey:  cfg.APIKey,
    }, nil
}

func (c *Client) Connect(ctx context.Context) error {
    // existing implementation...
}

// NEW: Convenience methods like Python SDK
func (c *Client) CodeReview(path string) (string, error) {
    return c.ExecuteContext(ctx, fmt.Sprintf("Review: %s", path))
}

func (c *Client) FixBugs(path string) (string, error) {
    return c.ExecuteContext(ctx, fmt.Sprintf("Fix: %s", path))
}

func (c *Client) Explain(code string) (string, error) {
    return c.ExecuteContext(ctx, fmt.Sprintf("Explain: %s", code))
}

// NEW: Polars-style lazy builder
type LazyAgent struct {
    client *Client
    model  string
    tools  []Tool
    system string
}

func Lazy(model string) *LazyAgent {
    return &LazyAgent{model: model}
}

func (a *LazyAgent) Tools(names ...string) *LazyAgent {
    a.tools = append(a.tools, names...)
    return a
}

func (a *LazyAgent) System(prompt string) *LazyAgent {
    a.system = prompt
    return a
}

func (a *LazyAgent) Execute(prompt string) (string, error) {
    return a.client.generate(prompt, a.model)
}
```

### 4. TypeScript SDK (already exists, improve)

```typescript
// crates/nexus-sdk/ts/nexus.ts

export class NexusAgent {
    constructor(
        public model: string = "qwen2.5-coder:3b",
        private apiKey?: string
    ) {}

    // Polars-style chainable API
    tools(...names: string[]): this {
        this._tools = names;
        return this;
    }

    system(prompt: string): this {
        this._system = prompt;
        return this;
    }

    timeout(seconds: number): this {
        this._timeout = seconds * 1000;
        return this;
    }

    async execute(prompt: string, stream = false): Promise<string | AsyncGenerator<string>> {
        if (stream) {
            return this._executeStreaming(prompt);
        }
        return this._executeOnce(prompt);
    }

    // One-liners
    async codeReview(path: string): Promise<string> {
        return this.execute(`Review code at ${path}`);
    }

    async fixBugs(path: string): Promise<string> {
        return this.execute(`Fix bugs in ${path}`);
    }

    async explain(code: string): Promise<string> {
        return this.execute(`Explain: ${code}`);
    }
}

// Usage
const agent = new NexusAgent("qwen2.5-coder:3b");
const result = await agent
    .tools("filesystem", "terminal")
    .system("You are a senior developer")
    .execute("Fix bugs in auth.py");
```

### 5. Java SDK (NEW)

```java
// crates/nexus-sdk/java/src/main/java/ai/nexus/sdk/NexusAgent.java

package ai.nexus.sdk;

public class NexusAgent implements AutoCloseable {
    private final NexusClient client;
    private String model = "qwen2.5-coder:3b";
    private List<String> tools = new ArrayList<>();
    private String systemPrompt;

    public NexusAgent(String url, String apiKey) {
        this.client = new NexusClient(url, apiKey);
    }

    // Builder pattern
    public NexusAgent model(String model) {
        this.model = model;
        return this;
    }

    public NexusAgent tools(String... toolNames) {
        this.tools.addAll(Arrays.asList(toolNames));
        return this;
    }

    public NexusAgent system(String prompt) {
        this.systemPrompt = prompt;
        return this;
    }

    public NexusResult execute(String prompt) {
        return client.execute(prompt, model);
    }

    public CompletableFuture<NexusResult> executeAsync(String prompt) {
        return CompletableFuture.supplyAsync(() -> execute(prompt));
    }

    // One-liners
    public String codeReview(String path) {
        return execute("Review code at " + path).getOutput();
    }

    public String fixBugs(String path) {
        return execute("Fix bugs in " + path).getOutput();
    }

    public String explain(String code) {
        return execute("Explain: " + code).getOutput();
    }

    @Override
    public void close() throws Exception {
        client.close();
    }

    public static void main(String[] args) {
        try (var agent = new NexusAgent("ws://localhost:8080", null)) {
            String result = agent
                .model("qwen2.5-coder:3b")
                .tools("filesystem")
                .execute("Say hello");
            System.out.println(result);
        }
    }
}
```

### 6. C# SDK (NEW)

```csharp
// crates/nexus-sdk/csharp/src/NexusAgent.cs

namespace Nexus.Sdk;

public class NexusAgent : IAsyncDisposable
{
    private readonly NexusClient _client;
    public string Model { get; set; } = "qwen2.5-coder:3b";
    public List<string> Tools { get; } = new();
    public string? SystemPrompt { get; set; }

    public NexusAgent(string url, string? apiKey = null)
    {
        _client = new NexusClient(url, apiKey);
    }

    // Builder pattern
    public NexusAgent WithModel(string model)
    {
        Model = model;
        return this;
    }

    public NexusAgent WithTools(params string[] tools)
    {
        Tools.AddRange(tools);
        return this;
    }

    public NexusAgent WithSystem(string prompt)
    {
        SystemPrompt = prompt;
        return this;
    }

    public async Task<NexusResult> ExecuteAsync(string prompt)
    {
        return await _client.ExecuteAsync(prompt, Model);
    }

    // One-liners
    public async Task<string> CodeReviewAsync(string path)
    {
        var result = await ExecuteAsync($"Review code at {path}");
        return result.Output;
    }

    public async Task<string> FixBugsAsync(string path)
    {
        var result = await ExecuteAsync($"Fix bugs in {path}");
        return result.Output;
    }

    public async ValueTask DisposeAsync()
    {
        await _client.DisposeAsync();
    }
}

// Usage
var agent = new NexusAgent("ws://localhost:8080");
var result = await agent
    .WithModel("qwen2.5-coder:3b")
    .WithTools("filesystem", "terminal")
    .ExecuteAsync("Fix bugs in auth.cs");
```

### 7. Ruby SDK (NEW)

```ruby
# crates/nexus-sdk/ruby/lib/nexus/agent.rb

module Nexus
  class Agent
    attr_accessor :model, :tools, :system_prompt

    def initialize(url: "ws://localhost:8080", api_key: nil)
      @client = Client.new(url, api_key)
      @model = "qwen2.5-coder:3b"
      @tools = []
    end

    # Builder pattern
    def tools(*names)
      @tools.concat(names)
      self
    end

    def system(prompt)
      @system_prompt = prompt
      self
    end

    def execute(prompt, stream: false)
      @client.generate(prompt, @model)
    end

    # One-liners
    def code_review(path)
      execute("Review code at #{path}")
    end

    def fix_bugs(path)
      execute("Fix bugs in #{path}")
    end

    def explain(code)
      execute("Explain: #{code}")
    end
  end
end

# Usage
agent = Nexus::Agent.new
result = agent.tools("filesystem", "terminal")
              .system("You are a senior developer")
              .execute("Fix bugs in auth.rb")
```

### 8. PHP SDK (NEW)

```php
<?php
// crates/nexus-sdk/php/src/Agent.php

namespace Nexus;

class Agent
{
    private Client $client;
    public string $model = "qwen2.5-coder:3b";
    public array $tools = [];
    public ?string $systemPrompt = null;

    public function __construct(
        string $url = "ws://localhost:8080",
        ?string $apiKey = null
    ) {
        $this->client = new Client($url, $apiKey);
    }

    public function withModel(string $model): self
    {
        $this->model = $model;
        return $this;
    }

    public function withTools(...$names): self
    {
        $this->tools = array_merge($this->tools, $names);
        return $this;
    }

    public function withSystem(string $prompt): self
    {
        $this->systemPrompt = $prompt;
        return $this;
    }

    public function execute(string $prompt): Result
    {
        return $this->client->generate($prompt, $this->model);
    }

    public function codeReview(string $path): string
    {
        return $this->execute("Review code at {$path}")->output();
    }

    public function fixBugs(string $path): string
    {
        return $this->execute("Fix bugs in {$path}")->output();
    }
}
```

### 9. Swift SDK (NEW - for Apple platforms)

```swift
// crates/nexus-sdk/swift/Sources/Nexus/Agent.swift

import Foundation

public class NexusAgent: @unchecked Sendable {
    private let client: NexusClient
    public var model: String = "qwen2.5-coder:3b"
    public var tools: [String] = []
    public var systemPrompt: String?

    public init(url: String = "ws://localhost:8080", apiKey: String? = nil) {
        self.client = NexusClient(url: url, apiKey: apiKey)
    }

    @discardableResult
    public func withModel(_ model: String) -> NexusAgent {
        self.model = model
        return self
    }

    @discardableResult
    public func withTools(_ names: String...) -> NexusAgent {
        self.tools.append(contentsOf: names)
        return self
    }

    @discardableResult
    public func withSystem(_ prompt: String) -> NexusAgent {
        self.systemPrompt = prompt
        return self
    }

    public func execute(_ prompt: String) async throws -> NexusResult {
        try await client.generate(prompt, model: model)
    }

    public func codeReview(at path: String) async throws -> String {
        let result = try await execute("Review code at \(path)")
        return result.output
    }

    public func fixBugs(at path: String) async throws -> String {
        let result = try await execute("Fix bugs in \(path)")
        return result.output
    }
}
```

### 10. Kotlin SDK (NEW - for Android/JVM)

```kotlin
// crates/nexus-sdk/kotlin/src/main/kotlin/ai/nexus/sdk/Agent.kt

package ai.nexus.sdk

class NexusAgent(
    private val url: String = "ws://localhost:8080",
    private val apiKey: String? = null
) : AutoCloseable {
    private val client = NexusClient(url, apiKey)
    var model: String = "qwen2.5-coder:3b"
    var tools: List<String> = emptyList()
    var systemPrompt: String? = null

    fun withModel(model: String) = apply { this.model = model }
    fun withTools(vararg names: String) = apply { this.tools = names.toList() }
    fun withSystem(prompt: String) = apply { this.systemPrompt = prompt }

    suspend fun execute(prompt: String): NexusResult =
        client.generate(prompt, model)

    suspend fun codeReview(path: String): String =
        execute("Review code at $path").output

    suspend fun fixBugs(path: String): String =
        execute("Fix bugs in $path").output

    override fun close() = client.close()
}

// Usage
suspend fun main() {
    NexusAgent().use { agent ->
        val result = agent
            .withModel("qwen2.5-coder:3b")
            .withTools("filesystem", "terminal")
            .execute("Fix bugs in auth.kt")
        println(result.output)
    }
}
```

---

## HTTP/REST Fallback (Universal)

Para qualquer linguagem que nem WebSocket suporte:

```bash
# REST API (simples HTTP)

# Execute code
curl -X POST https://api.nexus.ai/v1/execute \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $NEXUS_API_KEY" \
  -d '{
    "code": "print('hello')",
    "language": "python"
  }'

# Response
{
  "request_id": "uuid",
  "exit_code": 0,
  "stdout": "hello\n",
  "stderr": "",
  "execution_time_ms": 10
}

# Generate with Ollama
curl -X POST https://api.nexus.ai/v1/generate \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $NEXUS_API_KEY" \
  -d '{
    "model": "qwen2.5-coder:3b",
    "prompt": "Explain this code"
  }'
```

---

## Language-Specific Quick Start Guides

### Python

```bash
pip install nexus-ai

python << 'EOF'
from nexus import Agent

agent = Agent(model="qwen2.5-coder:3b")
result = agent.execute("Say hello")
print(result.output)
EOF
```

### JavaScript/TypeScript

```bash
npm install @nexus-protocol/sdk

typescript << 'EOF'
import { NexusAgent } from '@nexus-protocol/sdk';

const agent = new NexusAgent();
const result = await agent.execute("Say hello");
console.log(result.output);
EOF
```

### Go

```bash
go get github.com/vortex-ia/nexus-protocol/go

go << 'EOF'
package main

import (
    "fmt"
    "github.com/vortex-ia/nexus-protocol/go"
)

func main() {
    agent := nexus.NewAgent(nexus.WithModel("qwen2.5-coder:3b"))
    result := agent.Execute("Say hello")
    fmt.Println(result.Output)
}
EOF
```

### Rust

```bash
cargo add nexus-sdk

rust << 'EOF'
use nexus_sdk::{NexusAgent, Model};

#[tokio::main]
async fn main() {
    let agent = NexusAgent::new(Model::Qwen2_5Coder3b);
    let result = agent.execute("Say hello").await;
    println!("{}", result.output);
}
EOF
```

### Ruby

```bash
gem install nexus

ruby << 'EOF'
require 'nexus'

agent = Nexus::Agent.new
result = agent.execute("Say hello")
puts result.output
EOF
```

### Java

```bash
# Maven dependency
<dependency>
    <groupId>ai.nexus</groupId>
    <artifactId>nexus-sdk</artifactId>
    <version>0.1.0</version>
</dependency>

java << 'EOF'
var agent = new NexusAgent();
var result = agent.execute("Say hello");
System.out.println(result.getOutput());
EOF
```

### C#

```bash
dotnet add package Nexus.Sdk

csharp << 'EOF'
var agent = new NexusAgent();
var result = await agent.ExecuteAsync("Say hello");
Console.WriteLine(result.Output);
EOF
```

---

## Quality Standards (Enterprise-Grade)

### Every SDK Must Have:

| Feature | Rust | Python | Go | TypeScript | Java | C# |
|---------|------|--------|-----|------------|------|-----|
| **Connect** | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| **Execute** | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| **Generate** | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| **Streaming** | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| **One-liners** | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| **Builder/Chainable** | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| **Context manager** | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| **Type-safe** | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| **Error typed** | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| **Tests** | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| **Docs** | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| **CI/CD** | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |

### Testing Requirements

```bash
# Every SDK must pass:
make test          # Unit tests
make e2e          # End-to-end tests
make benchmark     # Performance benchmarks
make docs          # Generate docs
make publish       # Publish to package registry
```

---

## Star Generation per Language

```
Python developers: 10K potential stars
TypeScript developers: 8K potential stars
Go developers: 5K potential stars
Rust developers: 3K potential stars
Java developers: 4K potential stars
C# developers: 3K potential stars
Ruby developers: 2K potential stars
Swift developers: 2K potential stars
Kotlin developers: 2K potential stars
PHP developers: 1K potential stars

TOTAL POTENTIAL: 40K+ stars
(Every language = more reach = more stars)
```

---

## Directory Structure

```
nexus-protocol/
├── crates/
│   ├── nexus-protocol-core/      # Rust core (MIT)
│   ├── nexus-sandbox/            # WASM sandbox (MIT)
│   ├── nexus-ollama/             # Ollama client (MIT)
│   ├── nexus-sdk/
│   │   ├── rust/                # Rust SDK (native)
│   │   ├── python/               # Python SDK (PyO3)
│   │   ├── go/                   # Go SDK
│   │   ├── ts/                   # TypeScript SDK
│   │   ├── java/                 # Java SDK (NEW)
│   │   ├── csharp/              # C# SDK (NEW)
│   │   ├── ruby/                 # Ruby SDK (NEW)
│   │   ├── php/                  # PHP SDK (NEW)
│   │   ├── swift/               # Swift SDK (NEW)
│   │   └── kotlin/              # Kotlin SDK (NEW)
│   └── nexus-router/            # HTTP server
├── SPEC.md
├── SECURITY_AUDIT.md
├── STAR_STRATEGY.md
└── PLATFORM_STRATEGY.md
```

---

## The Killer README (All Languages)

```markdown
# Nexus Protocol — The Universal AI Agent Protocol

<p align="center">
  <img src="logo.svg" width="200"/>
</p>

<p align="center">
  <a href="https://github.com/vortex-ia/nexus-protocol">
    <img src="https://img.shields.io/github/stars/vortex-ia/nexus-protocol"/>
  </a>
  <a href="https://pypi.org/project/nexus-ai/">
    <img src="https://img.shields.io/pypi/v/nexus-ai"/>
  </a>
</p>

## One Protocol, Every Language

| Language | Install | One-liner |
|----------|---------|-----------|
| **Python** | `pip install nexus-ai` | `Agent().execute("hello")` |
| **TypeScript** | `npm install @nexus-protocol/sdk` | `new Agent().execute("hello")` |
| **Go** | `go get github.com/vortex-ia/nexus-protocol/go` | `Agent().Execute("hello")` |
| **Rust** | `cargo add nexus-sdk` | `Agent::new().execute("hello")` |
| **Java** | Maven/Gradle | `new Agent().execute("hello")` |
| **C#** | `dotnet add package Nexus.Sdk` | `new Agent().Execute("hello")` |
| **Ruby** | `gem install nexus` | `Agent.new.execute("hello")` |
| **PHP** | `composer require nexus/sdk` | `(new Agent())->execute("hello")` |
| **Swift** | Swift Package Manager | `Agent().execute("hello")` |
| **Kotlin** | Gradle/Maven | `Agent().execute("hello")` |

## Why Nexus?

```
LangChain = Confuso demais
MCP = Inseguro demais
Nexus = Faz sentido. Funciona. É rápido.
```

## Quick Start (Python)

```python
pip install nexus-ai

python << 'EOF'
from nexus import Agent

agent = Agent(model="qwen2.5-coder:3b")
result = agent.execute("Say hello")
print(result.output)
EOF
```

## Features

- **Universal** — Works in any language with WebSocket or HTTP
- **Fast** — Rust-powered, 10-100x faster than LangChain
- **Secure** — WASM sandbox, no host access
- **Local-first** — Run models locally with Ollama
- **Streaming** — Token-by-token output
- **Enterprise** — RBAC, SSO, audit logs

## License

MIT — free to use, modify, distribute

## Stars

[![Stars](https://img.shields.io/github/stars/vortex-ia/nexus-protocol)](https://github.com/vortex-ia/nexus-protocol/stargazers)

Se foi útil, ⭐ estrela!
```

---

## Summary

| Language | SDK Status | Package Manager |
|----------|-----------|----------------|
| **Python** | PyO3 (high-perf) | PyPI |
| **TypeScript** | Native | npm |
| **Go** | Native | go get |
| **Rust** | Native | cargo |
| **Java** | Native | Maven |
| **C#** | Native | NuGet |
| **Ruby** | Native | gem |
| **PHP** | Native | Composer |
| **Swift** | Native | SPM |
| **Kotlin** | Native | Gradle |
| **HTTP** | Fallback | - |

**Every developer in every language can use Nexus.**
