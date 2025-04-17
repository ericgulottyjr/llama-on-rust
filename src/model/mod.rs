use std::sync::Arc;
use anyhow::Result;
use std::env;
use reqwest::Client;
use serde_json::{json, Value};
use log::{info, debug, warn, error};
use crate::web::models::{Message, Role};

// Default constants for token limits
const DEFAULT_MAX_CONTEXT_WINDOW: usize = 4096; // Default maximum context window size
const DEFAULT_SYSTEM_MESSAGE_RESERVE: usize = 200; // Default reserve tokens for system message
const DEFAULT_RESPONSE_RESERVE: usize = 500; // Default reserve tokens for response
const DEFAULT_MIN_TOKENS: usize = 100; // Default minimum tokens for response
const DEFAULT_MAX_TOKENS: usize = 4096; // Default maximum tokens for response

/// Environment variables for configuring the LLM model:
/// 
/// - `MISTRAL_SERVER_URL`: URL of the mistral.rs server (default: "http://localhost:8081")
/// - `MAX_CONTEXT_WINDOW`: Maximum context window size in tokens (default: 4096)
/// - `SYSTEM_MESSAGE_RESERVE`: Tokens reserved for system message (default: 200)
/// - `RESPONSE_RESERVE`: Tokens reserved for response (default: 500)
/// - `MIN_TOKENS`: Minimum tokens for response (default: 100)
/// - `MAX_TOKENS`: Maximum tokens for response (default: 4096)
/// - `TEMPERATURE`: Sampling temperature (default: 0.7)
/// - `TOP_P`: Top-p sampling parameter (default: 0.95)
/// 
/// Note: All token-related values must be positive integers, and the following must hold:
/// - MIN_TOKENS <= MAX_TOKENS
/// - SYSTEM_MESSAGE_RESERVE + RESPONSE_RESERVE < MAX_CONTEXT_WINDOW
/// - MAX_TOKENS <= MAX_CONTEXT_WINDOW

// A wrapper for the mistral.rs server API
pub struct LlamaModel {
    server_url: String,
    client: Client,
    max_context_window: usize,
    system_message_reserve: usize,
    response_reserve: usize,
    min_tokens: usize,
    max_tokens: usize,
}

impl LlamaModel {
    pub async fn new() -> Result<Self> {
        info!("Initializing connection to mistral.rs server");
        
        // Get server URL from environment or use default
        let server_url = env::var("MISTRAL_SERVER_URL")
            .unwrap_or_else(|_| "http://localhost:8081".to_string());
        
        // Get token limits from environment or use defaults
        let max_context_window = env::var("MAX_CONTEXT_WINDOW")
            .ok()
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(DEFAULT_MAX_CONTEXT_WINDOW);
            
        let system_message_reserve = env::var("SYSTEM_MESSAGE_RESERVE")
            .ok()
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(DEFAULT_SYSTEM_MESSAGE_RESERVE);
            
        let response_reserve = env::var("RESPONSE_RESERVE")
            .ok()
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(DEFAULT_RESPONSE_RESERVE);
            
        let min_tokens = env::var("MIN_TOKENS")
            .ok()
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(DEFAULT_MIN_TOKENS);
            
        let max_tokens = env::var("MAX_TOKENS")
            .ok()
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(DEFAULT_MAX_TOKENS);
        
        // Validate token limits
        if min_tokens > max_tokens {
            error!("Invalid token limits: MIN_TOKENS ({}) > MAX_TOKENS ({})", min_tokens, max_tokens);
            return Err(anyhow::anyhow!("Invalid token limits: MIN_TOKENS > MAX_TOKENS"));
        }
        
        if max_tokens > max_context_window {
            error!("Invalid token limits: MAX_TOKENS ({}) > MAX_CONTEXT_WINDOW ({})", max_tokens, max_context_window);
            return Err(anyhow::anyhow!("Invalid token limits: MAX_TOKENS > MAX_CONTEXT_WINDOW"));
        }
        
        let total_reserve = system_message_reserve + response_reserve;
        if total_reserve >= max_context_window {
            error!("Invalid token limits: SYSTEM_MESSAGE_RESERVE ({}) + RESPONSE_RESERVE ({}) >= MAX_CONTEXT_WINDOW ({})", 
                system_message_reserve, response_reserve, max_context_window);
            return Err(anyhow::anyhow!("Invalid token limits: SYSTEM_MESSAGE_RESERVE + RESPONSE_RESERVE >= MAX_CONTEXT_WINDOW"));
        }
        
        // Validate that we have enough space for at least one message
        let min_message_space = max_context_window - total_reserve;
        if min_message_space < 100 {
            error!("Insufficient space for messages: MAX_CONTEXT_WINDOW ({}) - (SYSTEM_MESSAGE_RESERVE ({}) + RESPONSE_RESERVE ({})) = {} < 100", 
                max_context_window, system_message_reserve, response_reserve, min_message_space);
            return Err(anyhow::anyhow!("Insufficient space for messages: less than 100 tokens available after reserves"));
        }
        
        info!("Using mistral.rs server at: {}", server_url);
        info!("Token limits - Context Window: {}, System Reserve: {}, Response Reserve: {}, Min Tokens: {}, Max Tokens: {}", 
            max_context_window, system_message_reserve, response_reserve, min_tokens, max_tokens);
        info!("Available space for messages: {} tokens", min_message_space);
        
        Ok(Self {
            server_url,
            client: Client::new(),
            max_context_window,
            system_message_reserve,
            response_reserve,
            min_tokens,
            max_tokens,
        })
    }
    
    // Helper function to estimate token count (rough approximation)
    fn estimate_tokens(text: &str) -> usize {
        // Rough approximation: 1 token ≈ 4 characters
        // This is a simple estimation - in production you might want to use a proper tokenizer
        (text.len() / 4).max(1)
    }

    pub async fn generate_response(&self, prompt: &str, max_tokens: usize, history: &[String]) -> Result<String> {
        info!("Generating response for prompt with max_tokens: {}", max_tokens);
        debug!("Prompt: {}", prompt);
        
        // Read configuration from environment
        let temperature = env::var("TEMPERATURE").ok().and_then(|v| v.parse::<f32>().ok()).unwrap_or(0.7);
        let top_p = env::var("TOP_P").ok().and_then(|v| v.parse::<f32>().ok()).unwrap_or(0.95);
        
        // Adjust max_tokens to be within configured bounds
        let adjusted_max_tokens = if max_tokens < self.min_tokens {
            info!("Increasing max_tokens from {} to minimum of {}", max_tokens, self.min_tokens);
            self.min_tokens
        } else if max_tokens > self.max_tokens {
            info!("Capping max_tokens from {} to maximum of {}", max_tokens, self.max_tokens);
            self.max_tokens
        } else {
            max_tokens
        };
        
        // Calculate available tokens for history
        let system_tokens = self.system_message_reserve;
        let response_tokens = self.response_reserve;
        let prompt_tokens = Self::estimate_tokens(prompt);
        let available_history_tokens = self.max_context_window.saturating_sub(system_tokens + response_tokens + prompt_tokens);
        
        // Create the message array starting with system message
        let mut messages = vec![
            Message {
                role: Role::System,
                content: format!("You are a helpful AI assistant. When responding to the user, please be thorough and detailed in your explanations. Aim to use close to the maximum token length of {} tokens when appropriate for the question.", adjusted_max_tokens),
            }
        ];
        
        // Add conversation history with token limit
        let mut total_history_tokens = 0;
        let mut truncated_history = Vec::new();
        
        // Process history in reverse to keep most recent messages
        for message in history.iter().rev() {
            let message_tokens = Self::estimate_tokens(message);
            
            if total_history_tokens + message_tokens > available_history_tokens {
                warn!("Conversation history truncated due to token limit. Available: {}, Needed: {}", 
                    available_history_tokens, total_history_tokens + message_tokens);
                break;
            }
            
            total_history_tokens += message_tokens;
            truncated_history.push(message.clone());
        }
        
        // Reverse back to original order
        truncated_history.reverse();
        
        // Add truncated history to messages
        for message in truncated_history {
            let (role, content) = if message.starts_with("user: ") {
                (Role::User, message.trim_start_matches("user: ").to_string())
            } else if message.starts_with("assistant: ") {
                (Role::Assistant, message.trim_start_matches("assistant: ").to_string())
            } else {
                continue; // Skip malformed messages
            };
            
            messages.push(Message {
                role,
                content,
            });
        }
        
        // Add the current message
        messages.push(Message {
            role: Role::User,
            content: prompt.to_string(),
        });
        
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
        let response = self.client.post(&format!("{}/v1/chat/completions", self.server_url))
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