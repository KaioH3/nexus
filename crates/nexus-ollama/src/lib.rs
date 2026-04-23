//! Nexus Ollama - Ollama API client for local LLM inference.
//!
//! Provides REST and WebSocket clients for connecting to local Ollama instances.

pub mod client;
pub mod models;

pub use client::OllamaClient;
pub use models::{ModelInfo, TagsResponse};
