//! Supported programming languages.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    Rust,
    Go,
    Python,
    Javascript,
    Typescript,
    C,
    Sql,
    Bash,
}

impl Language {
    /// Returns the compiler/binary name for this language.
    pub fn compiler(&self) -> &'static str {
        match self {
            Self::Rust => "rustc",
            Self::Go => "go",
            Self::Python => "python3",
            Self::Javascript => "node",
            Self::Typescript => "npx",
            Self::C => "gcc",
            Self::Sql => "sqlite3",
            Self::Bash => "bash",
        }
    }

    /// Returns the WASM target triple for this language.
    pub fn wasm_target(&self) -> &'static str {
        match self {
            Self::Rust => "wasm32-wasi",
            Self::Go => "wasip1",
            Self::Javascript | Self::Typescript => "wasm32-unknown-unknown",
            _ => "wasm32-unknown-unknown",
        }
    }

    /// Returns true if the language supports direct WASM compilation.
    pub fn can_compile_to_wasm(&self) -> bool {
        matches!(
            self,
            Self::Rust | Self::Go | Self::Javascript | Self::Typescript
        )
    }

    /// Returns the file extension for this language.
    pub fn extension(&self) -> &'static str {
        match self {
            Self::Rust => "rs",
            Self::Go => "go",
            Self::Python => "py",
            Self::Javascript => "js",
            Self::Typescript => "ts",
            Self::C => "c",
            Self::Sql => "sql",
            Self::Bash => "sh",
        }
    }
}

impl std::fmt::Display for Language {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Rust => write!(f, "rust"),
            Self::Go => write!(f, "go"),
            Self::Python => write!(f, "python"),
            Self::Javascript => write!(f, "javascript"),
            Self::Typescript => write!(f, "typescript"),
            Self::C => write!(f, "c"),
            Self::Sql => write!(f, "sql"),
            Self::Bash => write!(f, "bash"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_language_compiler() {
        assert_eq!(Language::Rust.compiler(), "rustc");
        assert_eq!(Language::Go.compiler(), "go");
        assert_eq!(Language::Python.compiler(), "python3");
    }

    #[test]
    fn test_language_wasm_target() {
        assert_eq!(Language::Rust.wasm_target(), "wasm32-wasi");
        assert_eq!(Language::Go.wasm_target(), "wasip1");
    }

    #[test]
    fn test_language_can_compile_to_wasm() {
        assert!(Language::Rust.can_compile_to_wasm());
        assert!(Language::Go.can_compile_to_wasm());
        assert!(!Language::Python.can_compile_to_wasm());
    }

    #[test]
    fn test_language_extension() {
        assert_eq!(Language::Rust.extension(), "rs");
        assert_eq!(Language::Python.extension(), "py");
    }
}
