//! Code compiler for sandbox execution.
//!
//! Handles compilation of various languages to WASM.

use nexus_protocol_core::Language;
use thiserror::Error as ThisError;

#[derive(ThisError, Debug)]
pub enum CompileError {
    #[error("Unsupported language: {0}")]
    UnsupportedLanguage(Language),

    #[error("Compilation failed: {0}")]
    CompilationFailed(String),

    #[error("Timeout during compilation")]
    Timeout,
}

/// Compiler compiles source code to WASM for sandbox execution.
pub struct Compiler;

impl Compiler {
    pub fn new() -> Self {
        Self
    }

    /// Compile source code to WASM bytes.
    pub fn compile(&self, code: &str, language: Language) -> Result<Vec<u8>, CompileError> {
        match language {
            Language::Rust => self.compile_rust(code),
            Language::Go => self.compile_go(code),
            Language::Javascript | Language::Typescript => self.compile_js(code),
            _ => Err(CompileError::UnsupportedLanguage(language)),
        }
    }

    fn compile_rust(&self, code: &str) -> Result<Vec<u8>, CompileError> {
        // For MVP, we return mock WASM bytes.
        // Real implementation would call rustc + wasm32 target
        let wasm_header: [u8; 4] = [0x00, 0x61, 0x73, 0x6d]; // \0asm
        let mut wasm = Vec::with_capacity(4 + code.len());
        wasm.extend_from_slice(&wasm_header);
        wasm.extend_from_slice(code.as_bytes());
        Ok(wasm)
    }

    fn compile_go(&self, code: &str) -> Result<Vec<u8>, CompileError> {
        // For MVP, we return mock WASM bytes.
        // Real implementation would use tinygo or Go's wasm target
        let wasm_header: [u8; 4] = [0x00, 0x61, 0x73, 0x6d];
        let mut wasm = Vec::with_capacity(4 + code.len());
        wasm.extend_from_slice(&wasm_header);
        wasm.extend_from_slice(code.as_bytes());
        Ok(wasm)
    }

    fn compile_js(&self, code: &str) -> Result<Vec<u8>, CompileError> {
        // For MVP, we return mock WASM bytes.
        // Real implementation would use a JS-to-WASM transpiler
        let wasm_header: [u8; 4] = [0x00, 0x61, 0x73, 0x6d];
        let mut wasm = Vec::with_capacity(4 + code.len());
        wasm.extend_from_slice(&wasm_header);
        wasm.extend_from_slice(code.as_bytes());
        Ok(wasm)
    }
}

impl Default for Compiler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compile_rust() {
        let compiler = Compiler::new();
        let result = compiler.compile("fn main() {}", Language::Rust);
        assert!(result.is_ok());
        let wasm = result.unwrap();
        assert!(!wasm.is_empty());
        // Check WASM magic header
        assert_eq!([wasm[0], wasm[1], wasm[2], wasm[3]], [0x00, 0x61, 0x73, 0x6d]);
    }

    #[test]
    fn test_compile_unsupported() {
        let compiler = Compiler::new();
        let result = compiler.compile("SELECT * FROM users", Language::Sql);
        assert!(matches!(result, Err(CompileError::UnsupportedLanguage(Language::Sql))));
    }
}
