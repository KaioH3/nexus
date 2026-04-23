# Nexus Protocol

**The sandbox-first protocol for AI agent security.**

WASM sandbox. 19 dangerous syscalls blocked. Binary protocol. Apache 2.0.

---

## Why Nexus?

MCP (Model Context Protocol) has documented security issues. Nexus Protocol was built from scratch with security as the foundation.

| Feature | MCP | Nexus Protocol |
|---------|-----|----------------|
| Sandbox | ❌ None | ✅ WASM sandbox |
| Blocked syscalls | ❌ 0 | ✅ 19 |
| Protocol | JSON | ✅ Binary |
| Languages | TypeScript | ✅ Rust, Go, Python, TypeScript |
| License | MIT | ✅ Apache 2.0 |

---

## Quick Start

### Rust
```bash
cargo add nexus-protocol-core
```

```rust
use nexus_protocol_core::{Message, SandboxPolicy, Language};

let policy = SandboxPolicy::ai_generated_code();
let msg = Message::Execute {
    request_id: uuid::Uuid::new_v4(),
    code: "fn main() { println!(\"Hello\"); }".to_string(),
    language: Language::Rust,
    sandbox_policy: policy,
    model_hint: None,
};
```

### Python
```bash
pip install nexus-protocol
```

```python
from nexus_protocol import Agent

agent = Agent(model="qwen2.5-coder:3b")  # Local Ollama
result = agent.execute("println!(Hello)")
```

### Go
```go
go get github.com/KaioH3/nexus/sdk/go
```

---

## Security

**MCP's STDIO interface allows arbitrary command execution.** This is documented as "intended behavior" by Anthropic.

**Nexus Protocol security model:**

```rust
// 19 syscalls blocked by default
const BLOCKED_SYSCALLS: &[u32] = &[
    2, 3, 4, 5, 9, 10, 87, // filesystem (open, close, stat, fstat, mmap, munmap, unlink)
    41, 42, 43,            // network (socket, connect, accept)
    56, 57, 59, 60, 61,  // process (clone, fork, execve, exit, wait4)
    79, 85, 86, 137,     // admin (getdents, readlink, mprotect, kexec_load)
];

// Resource limits enforced
pub struct ResourceLimits {
    max_memory_bytes: u64,    // Default: 512 MB
    max_cpu_time_ms: u64,     // Default: 30s
}
```

### CVE Protection

| CVE | Description | Nexus Protection |
|-----|-------------|-----------------|
| CVE-2025-49596 | MCP Inspector RCE | ✅ API key validation (constant-time hash) |
| CVE-2025-68143 | Git MCP prompt injection | ✅ WASM sandbox blocks execve() |
| CVE-2025-34072 | Slack data exfiltration | ✅ Network syscalls blocked (socket=41) |
| CVE-2026-0621 | ReDoS in TypeScript SDK | ✅ Binary protocol has no regex |

### Authentication

Nexus Protocol implements API key validation with security best practices:

```rust
use nexus_protocol_core::ApiKeyConfig;

// Load from NEXUS_API_KEY environment variable
let config = ApiKeyConfig::from_env("NEXUS_API_KEY");

// Or configure explicit keys
let config = ApiKeyConfig::with_keys(&[
    "nexus_sk_prod_key_12345",
    "nexus_sk_dev_key_67890",
]);

// Validate incoming request
config.validate(Some("nexus_sk_prod_key_12345"))?;
```

**Security features:**
- Constant-time comparison (prevents timing attacks)
- Format validation (length, allowed characters)
- Hash-based key storage
- Missing/Invalid API key error codes

For mTLS, deploy behind a reverse proxy (nginx, traefik) with client certificates.

---

## Architecture

```
Clients                    Nexus Protocol                  Backend
┌─────────┐               ┌─────────────────────┐         ┌─────────┐
│ Python  │──────────────▶│  Message Router     │────────▶│ Ollama  │
├─────────┤               │                     │         │ (local) │
│   Go    │──────────────▶│  WASM Sandbox       │         └─────────┘
├─────────┤               │  (19 syscalls block)│
│   TS    │──────────────▶│                     │
└─────────┘               └─────────────────────┘
```

---

## Features

- ✅ WASM sandbox with resource limits
- ✅ 19 blocked dangerous syscalls
- ✅ 3 predefined sandbox policies (zero trust, AI-generated code, development)
- ✅ Multi-language SDK (Python, Go, TS, Rust)
- ✅ Local Ollama support (14 models verified)
- ✅ Binary protocol (no JSON parsing overhead)
- ✅ Streaming token support
- ✅ Capability negotiation
- ✅ Rate limiting (token bucket)
- ✅ Connection pooling
- ✅ Prompt injection guard
- ✅ API key validation (constant-time hash)
- ✅ 60 tests passing

---

## Verified Benchmarks

| Metric | Value | Method |
|--------|-------|--------|
| Binary vs JSON | No parsing overhead | Code inspection |
| Ollama models | 14 working | curl to localhost:11434 |
| Syscalls blocked | 19 | Code inspection |
| Tests | 60 passing | cargo test |

---

## Documentation

- [CONTRIBUTING.md](CONTRIBUTING.md) - How to contribute

---

## License

Apache License 2.0 - fully open, free to use commercially.

---

⭐ Star this repo if you believe AI agents deserve a secure protocol.