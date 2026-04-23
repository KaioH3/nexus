//! Ollama model information types.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub name: String,
    pub model: String,
    pub modified_at: String,
    pub size: u64,
    pub digest: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagsResponse {
    pub models: Vec<ModelInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateRequest {
    pub model: String,
    pub prompt: String,
    pub system: Option<String>,
    pub template: Option<String>,
    pub context: Option<Vec<u64>>,
    pub stream: Option<bool>,
    pub options: Option<GenerateOptions>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateOptions {
    pub temperature: Option<f32>,
    pub top_p: Option<f32>,
    pub top_k: Option<u32>,
    pub num_predict: Option<u32>,
    pub stop: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateResponse {
    pub model: String,
    pub response: String,
    pub done: bool,
    pub context: Option<Vec<u64>>,
    pub total_duration: Option<u64>,
    pub load_duration: Option<u64>,
    pub prompt_eval_count: Option<u32>,
    pub prompt_eval_duration: Option<u64>,
    pub eval_count: Option<u32>,
    pub eval_duration: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamingResponse {
    pub model: String,
    pub response: String,
    pub done: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_info_deserialization() {
        let json = r#"{
            "name": "llama3.1:latest",
            "model": "llama3.1",
            "modified_at": "2024-01-01T00:00:00Z",
            "size": 12345678,
            "digest": "sha256:abc123"
        }"#;

        let model: ModelInfo = serde_json::from_str(json).unwrap();
        assert_eq!(model.name, "llama3.1:latest");
        assert_eq!(model.size, 12345678);
    }

    #[test]
    fn test_tags_response_deserialization() {
        let json = r#"{
            "models": [
                {
                    "name": "llama3.1:8b",
                    "model": "llama3.1",
                    "modified_at": "2024-01-01T00:00:00Z",
                    "size": 8200000000,
                    "digest": "sha256:def456"
                }
            ]
        }"#;

        let response: TagsResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.models.len(), 1);
        assert_eq!(response.models[0].name, "llama3.1:8b");
    }
}
