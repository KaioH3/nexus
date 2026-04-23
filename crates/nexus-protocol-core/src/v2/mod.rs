# Nexus Protocol v2 — GPU-as-a-Service Specification

## Overview

Nexus Protocol v2 extends v1 (MCP-compatible RPC) with a **GPU-as-a-Service** layer: browsers and edge nodes can offload computation to remote GPU workers via a duplex WebSocket stream, using WASM modules as the portable compute artifact.

## Goals

1. **Privacy-first**: Code/data never leave the browser unless explicitly routed to a trusted GPU node
2. **Cross-platform**: Runs on Windows, Linux, macOS, and in-browser (WASM)
3. **Protocol standard**: Open source MIT — anyone can implement servers/clients
4. **Product moat**: Closed-source Vortex IDE uses nexus-protocol v2 as the GPU backend
5. **Dask/Polars model**: Like Dask distributes Python tasks, nexus distributes GPU tasks

## Architecture

```
Browser / Client                    GPU Node (Rust)
┌─────────────────────┐            ┌────────────────────┐
│ WASM Module         │◄──WS──────►│ nexus-gpu-worker   │
│ (WebGPU/CUDA/WASM)  │            │ - WebSocket server  │
└─────────────────────┘            │ - CUDA/WASM runtime │
                                  │ - result stream     │
└─────────────────────┘            └────────────────────┘
```

## Protocol Design

### Transport: WebSocket Duplex Stream

```
Client ──► OPEN { module_wasm: base64, args: json, target: "cuda|wasm|webgpu" }
Client ◄── STREAM { chunk: bytes, metadata: {...} }  (multiple)
Client ◄── DONE { result: bytes, stats: {...} }
Client ◄── ERROR { code: u32, message: string }
```

### Message Types

```rust
enum Message {
    Submit(SubmitTask),
    Stream(StreamChunk),
    Result(TaskResult),
    Error(Error),
    Ping/Pong,
}
```

### Compute Targets

| Target   | Runtime    | Use Case                          |
|----------|------------|-----------------------------------|
| `wasm`   | WASM runtime in browser | Zero-latency local compute |
| `webgpu` | WebGPU in browser        | GPU via browser              |
| `cuda`   | NVIDIA GPU via Rust-CUDA | Full GPU power, cloud/edge   |
| `cpu`    | Rust thread pool         | Fallback                     |

### WASM Module Format

```wat
;; Every WASM module must export:
;; (func $compute (param $input pointer) (result $output pointer))
;; (func $memory_info) -> (pages u32)
```

### Security Model

1. **Sandbox isolation**: WASM modules run in Wasmtime/Firecracker sandbox
2. **No network from compute**: GPU workers cannot initiate outbound connections
3. **Attestation**: Optional TPM-based remote attestation for trusted GPU nodes
4. **User opt-in**: Every task submission requires explicit target selection

## Comparison with Dask/Polars

| Feature           | Dask                      | Nexus Protocol v2                    |
|-------------------|---------------------------|--------------------------------------|
| Language          | Python                    | Any (via WASM)                       |
| Compute location  | Python processes          | Browser + Edge GPU nodes            |
| Communication     | Python IPC / TCP          | WebSocket duplex stream              |
| GPU support       | Yes (via CUDA)            | Yes (CUDA + WebGPU + WASM)           |
| Privacy           | Data on cluster           | Data can stay in browser             |
| Open source       | Yes (BSD)                | Yes (MIT protocol)                  |
| Product moat      | Dask.distributed          | Vortex IDE (closed source)           |

## Directory Structure

```
nexus-protocol/
├── crates/
│   ├── nexus-protocol-core/     # Core protocol (MIT licensed)
│   │   ├── src/
│   │   │   ├── v2/
│   │   │   │   ├── messages.rs
│   │   │   │   ├── transport/
│   │   │   │   │   └── websocket.rs
│   │   │   │   ├── compute/
│   │   │   │   │   ├── wasm.rs
│   │   │   │   ├── webgpu.rs
│   │   │   │   └── cuda.rs
│   │   │   │   └── mod.rs
│   │   │   └── lib.rs
│   │   └── README.md
│   ├── nexus-sandbox/           # WASM sandbox (MIT)
│   ├── nexus-gpu-worker/        # GPU worker implementation
│   │   ├── src/
│   │   │   ├── server.rs        # WebSocket server
│   │   │   ├── runtime/
│   │   │   │   ├── cuda.rs
│   │   │   │   ├── wasm.rs
│   │   │   │   └── mod.rs
│   │   │   └── lib.rs
│   │   └── Cargo.toml
│   └── nexus-sdk/               # Multi-language SDKs (MIT)
│       ├── go/
│       ├── ts/
│       └── python/              # New: Python bindings
├── SPEC.md
├── SECURITY_ANALYSIS.md
└── README.md
```

## SDK Usage Example (Go)

```go
package main

import (
    "context"
    "nexus-sdk/nexus/v2"
)

func main() {
    client, _ := v2.Dial("wss://gpu.example.com/nexus")
    defer client.Close()

    // Option 1: Local WASM in browser
    result, err := client.Submit(v2.Task{
        Module:  localWasmBytes,
        Args:    map[string]any{"input": data},
        Target:  v2.TargetWASM, // runs in browser, zero latency
    })

    // Option 2: Remote CUDA GPU
    result, err = client.Submit(v2.Task{
        Module:  localWasmBytes,
        Args:    map[string]any{"input": hugeData},
        Target:  v2.TargetCUDA, // routed to GPU node
    })

    // Option 3: WebGPU in browser
    result, err = client.Submit(v2.Task{
        Module:  localWasmBytes,
        Args:    map[string]any{"input": data},
        Target:  v2.TargetWebGPU, // GPU via browser API
    })
}
```

## SDK Usage Example (TypeScript)

```typescript
import { NexusClient, Target } from '@nexus-protocol/sdk';

// Browser usage
const client = await NexusClient.connect('wss://gpu.example.com/nexus');

// Stream results for large computations
for await (const chunk of client.submitStreaming({
    module: wasmBytes,
    args: { input: imageData },
    target: Target.WebGPU,
})) {
    updateProgress(chunk);
}
```

## Privacy Design

```
┌─────────────────────────────────────────────────────────────┐
│ Browser                                                     │
│  ┌──────────────┐    WASM module has FULL access to data   │
│  │ Input data   │──────► WASM ──► Result ──► App           │
│  └──────────────┘    Code never leaves browser             │
│                                                             │
│  If target=wasm or target=webgpu: data STAYS local          │
│  If target=cuda: data sent to GPU node (user opt-in)        │
└─────────────────────────────────────────────────────────────┘
```

## Next Steps

- [ ] Implement `nexus-gpu-worker` WebSocket server with CUDA runtime
- [ ] Add Python SDK to `nexus-sdk`
- [ ] Implement WebGPU client shim
- [ ] Create `nexus-runner` CLI for edge deployment
- [ ] Write integration tests with real GPU hardware

## License

Protocol specification and reference implementation: **MIT**

Product (Vortex IDE, managed GPU cloud): **Closed source / Freemium**
