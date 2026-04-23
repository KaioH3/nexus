# Nexus Protocol Python SDK

**As easy as Polars. Works everywhere.**

```bash
pip install nexus-protocol
```

```python
from nexus_protocol import NexusClient, SandboxPolicy

# Async usage (recommended)
async with NexusClient("ws://localhost:8080") as client:
    result = await client.execute("print('Hello!')", language="python")
    print(result.stdout)

# Or sync wrapper
with SyncNexusClient("ws://localhost:8080") as client:
    result = client.execute("print('Hello!')", language="python")
    print(result.stdout)
```

## Features

- **Zero config** - Works out of the box
- **Async native** - Built for asyncio
- **Type-safe** - Full type hints
- **Sandboxed execution** - WASM isolation
- **Ollama integration** - Local LLM support
- **Streaming** - Token-by-token generation

## Quick Start

### Execute Code

```python
from nexus_protocol import NexusClient, SandboxPolicy

async with NexusClient() as client:
    # Python
    result = await client.execute("print('Hello from Nexus!')", language="python")
    
    # Rust (compiles and runs in WASM)
    result = await client.execute('''
fn main() {
    println!("Hello from Nexus!");
}
''', language="rust")
    
    print(result.stdout)
```

### Generate with Ollama

```python
async with NexusClient() as client:
    # Simple generation
    response = await client.generate(
        "Explain async/await in Python",
        model="llama3.2"
    )
    print(response)
    
    # Streaming
    async for token in await client.generate(
        "Write a Fibonacci function",
        model="llama3.2",
        stream=True
    ):
        print(token, end="", flush=True)
```

### Sandbox Policies

```python
from nexus_protocol import SandboxPolicy

# AI-generated code (default)
result = await client.execute(code, language="python")
result = await client.execute(code, language="python", policy=SandboxPolicy.ai_generated_code())

# Zero trust (strictest)
result = await client.execute(code, language="python", policy=SandboxPolicy.zero_trust())

# Development (lenient)
result = await client.execute(code, language="python", policy=SandboxPolicy.development())
```

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│  Your Code                                              │
│                                                         │
│  from nexus_protocol import NexusClient                 │
│                                                         │
│         │                                              │
│         ▼                                              │
│  ┌─────────────────────────────────────────────────┐  │
│  │  Python SDK (websockets + async)                  │  │
│  │  - NexusClient (async)                          │  │
│  │  - SyncNexusClient (sync wrapper)                │  │
│  └─────────────────────────────────────────────────┘  │
│                          │                             │
│                          ▼                             │
│  ┌─────────────────────────────────────────────────┐  │
│  │  Nexus Protocol (WebSocket)                      │  │
│  │  - Handshake + Auth                            │  │
│  │  - Execute / OllamaGenerate                     │  │
│  └─────────────────────────────────────────────────┘  │
│                          │                             │
│         ┌────────────────┼────────────────┐           │
│         ▼                ▼                ▼             │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐   │
│  │ nexus-      │  │ nexus-     │  │ nexus-      │   │
│  │ sandbox     │  │ ollama     │  │ router      │   │
│  │ (WASM)      │  │ (LLM)      │  │ (server)    │   │
│  └─────────────┘  └─────────────┘  └─────────────┘   │
└─────────────────────────────────────────────────────────┘
```

## Comparison

| Feature | Raw WebSocket | Nexus SDK |
|---------|--------------|-----------|
| Connection | Manual | `client.connect()` |
| Handshake | Manual JSON | Auto |
| Error handling | Raw strings | `NexusError` with `ErrorCode` |
| Streaming | Manual parsing | `async for token` |
| Types | `dict` | Full dataclasses |

## License

MIT - free to use, modify, distribute

## Links

- [Documentation](https://docs.nexusprotocol.ai)
- [Protocol Spec](../SPEC.md)
- [Security Analysis](../SECURITY_ANALYSIS.md)
