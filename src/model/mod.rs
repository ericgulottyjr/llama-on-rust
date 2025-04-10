use std::sync::Arc;
use anyhow::Result;
use std::env;
use reqwest::Client;
use serde_json::{json, Value};
use log::{info, debug};
use crate::web::models::{Message, Role};

// A wrapper for the mistral.rs server API
pub struct LlamaModel {
    server_url: String,
    client: Client,
}

impl LlamaModel {
    pub async fn new() -> Result<Self> {
        info!("Initializing connection to mistral.rs server");
        
        // Get server URL from environment or use default
        let server_url = env::var("MISTRAL_SERVER_URL")
            .unwrap_or_else(|_| "http://localhost:8081".to_string());
        
        info!("Using mistral.rs server at: {}", server_url);
        
        Ok(Self {
            server_url,
            client: Client::new(),
        })
    }
    
    pub async fn generate_response(&self, prompt: &str, max_tokens: usize) -> Result<String> {
        info!("Generating response for prompt with max_tokens: {}", max_tokens);
        debug!("Prompt: {}", prompt);
        
        // Read configuration from environment
        let temperature = env::var("TEMPERATURE").ok().and_then(|v| v.parse::<f32>().ok()).unwrap_or(0.7);
        let top_p = env::var("TOP_P").ok().and_then(|v| v.parse::<f32>().ok()).unwrap_or(0.95);
        
        // Ensure max_tokens is within reasonable bounds for mistral.rs
        // Some LLM servers might have internal caps regardless of what we send
        let adjusted_max_tokens = if max_tokens < 100 {
            info!("Increasing max_tokens from {} to minimum of 100", max_tokens);
            100
        } else if max_tokens > 4096 {
            info!("Capping max_tokens from {} to maximum of 4096", max_tokens);
            4096
        } else {
            max_tokens
        };
        
        // Prepare the request to the mistral.rs server
        // Using the OpenAI-compatible API endpoint
        let url = format!("{}/v1/chat/completions", self.server_url);
        
        // Create the message array
        let messages = vec![
            Message {
                role: Role::System,
                content: format!("You are a helpful AI assistant. When responding to the user, please be thorough and detailed in your explanations. Aim to use close to the maximum token length of {} tokens when appropriate for the question.", adjusted_max_tokens),
            },
            Message {
                role: Role::User,
                content: prompt.to_string(),
            }
        ];
        
        // Create the request payload
        let payload = json!({
            "model": "local-model", // This is arbitrary for mistral.rs server
            "messages": messages,
            "temperature": temperature,
            "top_p": top_p,
            "max_tokens": adjusted_max_tokens
        });
        
        info!("Sending request to mistral.rs server with max_tokens: {}", adjusted_max_tokens);
        debug!("Payload: {}", payload);
        
        // Send the request to the server
        let response = self.client.post(&url)
            .json(&payload)
            .send()
            .await?;
        
        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow::anyhow!("API request failed: {}", error_text));
        }
        
        // Parse the response
        let response_json: Value = response.json().await?;
        debug!("Response JSON: {}", response_json);
        
        // Extract the generated text from the response
        let content = response_json
            .get("choices")
            .and_then(|choices| choices.get(0))
            .and_then(|choice| choice.get("message"))
            .and_then(|message| message.get("content"))
            .and_then(|content| content.as_str())
            .ok_or_else(|| anyhow::anyhow!("Failed to extract content from response"))?;
        
        info!("Response length: {} characters", content.len());
        Ok(content.to_string())
    }
}

// Singleton instance for the model
pub struct ModelManager {
    pub model: Arc<LlamaModel>,
}

impl ModelManager {
    pub async fn new() -> Result<Self> {
        let model = LlamaModel::new().await?;
        Ok(Self {
            model: Arc::new(model),
        })
    }
} 