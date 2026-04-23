//! Capabilities advertisement and negotiation.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Capabilities {
    pub wasm_runtimes: Vec<WasmRuntime>,
    pub ollama: bool,
    pub gguf_loading: bool,
    pub streaming: bool,
    pub sandbox_isolation: bool,
}

impl Default for Capabilities {
    fn default() -> Self {
        Self {
            wasm_runtimes: vec![WasmRuntime::Wasm3],
            ollama: true,
            gguf_loading: true,
            streaming: true,
            sandbox_isolation: true,
        }
    }
}

impl Capabilities {
    pub fn full() -> Self {
        Self {
            wasm_runtimes: vec![
                WasmRuntime::Wasm3,
                WasmRuntime::Wasmer,
                WasmRuntime::Wasmtime,
            ],
            ollama: true,
            gguf_loading: true,
            streaming: true,
            sandbox_isolation: true,
        }
    }

    pub fn client() -> Self {
        Self {
            wasm_runtimes: vec![WasmRuntime::Wasm3],
            ollama: true,
            gguf_loading: true,
            streaming: true,
            sandbox_isolation: true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WasmRuntime {
    Wasm3,
    Wasmer,
    Wasmtime,
    Native,
}

impl std::fmt::Display for WasmRuntime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WasmRuntime::Wasm3 => write!(f, "wasm3"),
            WasmRuntime::Wasmer => write!(f, "wasmer"),
            WasmRuntime::Wasmtime => write!(f, "wasmtime"),
            WasmRuntime::Native => write!(f, "native"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capabilities_default() {
        let caps = Capabilities::default();
        assert!(caps.ollama);
        assert!(caps.streaming);
        assert!(caps.sandbox_isolation);
    }

    #[test]
    fn test_capabilities_full() {
        let caps = Capabilities::full();
        assert!(caps.wasm_runtimes.contains(&WasmRuntime::Wasmer));
    }
}
