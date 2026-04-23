//! Ollama HTTP + WebSocket client.

use anyhow::{Context, Result};
use futures_util::{SinkExt, StreamExt};
use reqwest::Client;
use std::time::Duration;
use thiserror::Error;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use url::Url;

use crate::models::{
    GenerateOptions, GenerateRequest, GenerateResponse, ModelInfo, StreamingResponse,
    TagsResponse,
};

#[derive(Error, Debug)]
pub enum OllamaError {
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),

    #[error("Request failed: {0}")]
    RequestFailed(String),

    #[error("Model not found: {0}")]
    ModelNotFound(String),

    #[error("Generation failed: {0}")]
    GenerationFailed(String),

    #[error("WebSocket error: {0}")]
    WebSocketError(String),
}

pub struct OllamaClient {
    base_url: Url,
    http_client: Client,
}

impl OllamaClient {
    pub fn new(base_url: &str) -> Result<Self> {
        let base_url = Url::parse(base_url)
            .context("Invalid Ollama base URL")?;

        let http_client = Client::builder()
            .timeout(Duration::from_secs(60))
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self {
            base_url,
            http_client,
        })
    }

    pub async fn connect(&self) -> Result<Vec<ModelInfo>, OllamaError> {
        let url = self.base_url.join("/api/tags").unwrap();

        let response = self
            .http_client
            .get(url.as_str())
            .send()
            .await
            .map_err(|e| OllamaError::ConnectionFailed(e.to_string()))?;

        if !response.status().is_success() {
            return Err(OllamaError::ConnectionFailed(format!(
                "HTTP {}",
                response.status()
            )));
        }

        let tags: TagsResponse = response
            .json()
            .await
            .map_err(|e| OllamaError::RequestFailed(e.to_string()))?;

        Ok(tags.models)
    }

    pub async fn generate(
        &self,
        model: &str,
        prompt: &str,
        options: Option<GenerateOptions>,
    ) -> Result<String, OllamaError> {
        let url = self.base_url.join("/api/generate").unwrap();

        let request = GenerateRequest {
            model: model.to_string(),
            prompt: prompt.to_string(),
            system: None,
            template: None,
            context: None,
            stream: Some(false),
            options,
        };

        let response = self
            .http_client
            .post(url.as_str())
            .json(&request)
            .send()
            .await
            .map_err(|e| OllamaError::RequestFailed(e.to_string()))?;

        if !response.status().is_success() {
            return Err(OllamaError::GenerationFailed(format!(
                "HTTP {}",
                response.status()
            )));
        }

        let gen_response: GenerateResponse = response
            .json()
            .await
            .map_err(|e| OllamaError::RequestFailed(e.to_string()))?;

        Ok(gen_response.response)
    }

    pub async fn generate_streaming(
        &self,
        model: &str,
        prompt: &str,
        options: Option<GenerateOptions>,
    ) -> Result<impl futures_util::Stream<Item = String>, OllamaError> {
        let ws_url = self
            .base_url
            .join("/api/generate")
            .unwrap()
            .to_string()
            .replace("http://", "ws://")
            .replace("https://", "wss://");

        let (ws_stream, _) = connect_async(&ws_url)
            .await
            .map_err(|e| OllamaError::WebSocketError(e.to_string()))?;

        let (mut write, read) = ws_stream.split();

        let request = GenerateRequest {
            model: model.to_string(),
            prompt: prompt.to_string(),
            system: None,
            template: None,
            context: None,
            stream: Some(true),
            options,
        };

        write
            .send(Message::Text(serde_json::to_string(&request).unwrap()))
            .await
            .map_err(|e| OllamaError::WebSocketError(e.to_string()))?;

        let stream = read.map(|msg| {
            let msg = msg.unwrap();
            let text = msg.into_text().unwrap();
            let response: StreamingResponse = serde_json::from_str(&text).unwrap();
            response.response
        });

        Ok(stream)
    }

    pub async fn health_check(&self) -> bool {
        let url = self.base_url.join("/api/tags").unwrap();

        self.http_client
            .get(url.as_str())
            .send()
            .await
            .is_ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = OllamaClient::new("http://localhost:11434");
        assert!(client.is_ok());
    }

    #[test]
    fn test_invalid_url() {
        let client = OllamaClient::new("not a valid url");
        assert!(client.is_err());
    }
}
