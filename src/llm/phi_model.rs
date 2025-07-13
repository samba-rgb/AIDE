use anyhow::{Result, Context};
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct OllamaRequest {
    model: String,
    prompt: String,
    stream: bool,
}

#[derive(Deserialize)]
struct OllamaResponse {
    response: String,
}

pub struct PhiModel {
    client: Client,
    base_url: String,
    model_name: String,
}

impl PhiModel {
    /// Create a new PhiModel with configurable base_url and model_name
    pub async fn new(base_url: impl Into<String>, model_name: impl Into<String>) -> Result<Self> {
        let client = Client::new();
        Ok(Self {
            client,
            base_url: base_url.into(),
            model_name: model_name.into(),
        })
    }
    
    pub async fn generate_command(&self, prompt: &str) -> Result<String> {
        let system_prompt = "You are a command-line expert. Convert natural language requests into exact shell commands. Return only the command, no explanation.";
        let full_prompt = format!("{}\n\nUser request: {}\nCommand:", system_prompt, prompt);
        
        let request = OllamaRequest {
            model: self.model_name.clone(),
            prompt: full_prompt,
            stream: false,
        };
        
        let response = self.client
            .post(&format!("{}/api/generate", self.base_url))
            .json(&request)
            .send()
            .await
            .context("Failed to send request to Ollama")?;
        
        let ollama_response: OllamaResponse = response
            .json()
            .await
            .context("Failed to parse Ollama response")?;
        
        Ok(ollama_response.response.trim().to_string())
    }
}