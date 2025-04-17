use actix_web::{web, HttpResponse, Responder};
use serde_json::json;
use tera::Context;
use uuid::Uuid;
use log::{info, error};
use std::env;

use crate::web::models::{ChatRequest, ChatResponse};
use crate::AppState;

// Index page handler
pub async fn index(data: web::Data<AppState>) -> impl Responder {
    let context = Context::new();
    match data.tera.render("index.html", &context) {
        Ok(html) => HttpResponse::Ok().content_type("text/html").body(html),
        Err(e) => {
            error!("Template error: {}", e);
            HttpResponse::InternalServerError().body("Template error")
        }
    }
}

// Health check endpoint
pub async fn health_check() -> impl Responder {
    HttpResponse::Ok().json(json!({ "status": "ok" }))
}

// Chat API endpoint
pub async fn chat(
    data: web::Data<AppState>,
    req: web::Json<ChatRequest>,
) -> impl Responder {
    // Get default max tokens from environment or use 1000 as default
    let default_max_tokens = env::var("MAX_TOKENS")
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(512);
    
    // Use the requested max_tokens or default
    let max_tokens = req.max_tokens.unwrap_or(default_max_tokens);
    
    let session_id = req.session_id.unwrap_or_else(Uuid::new_v4);
    
    // Create a more specific prompt that encourages detailed responses
    let enhanced_prompt = format!("{}\n\nPlease provide a detailed and comprehensive answer.", 
                req.message);
    
    info!("Chat request from session {}: {} (max_tokens: {})", 
          session_id, req.message, max_tokens);
    
    // Add the new user message to history
    let mut sessions = match data.sessions.lock() {
        Ok(guard) => guard,
        Err(e) => {
            error!("Failed to lock sessions mutex: {}", e);
            return HttpResponse::InternalServerError().json(json!({
                "error": "Internal server error"
            }));
        }
    };
    
    let history = sessions.entry(session_id).or_insert_with(Vec::new);
    
    // Add the new user message (original message, not enhanced)
    history.push(format!("user: {}", req.message.clone()));
    
    // Clone what we need for the future
    let model = data.model.clone();
    let history_clone = history.clone();
    
    // Release the lock before the async operation to avoid blocking
    drop(sessions);
    
    // Generate response
    match model.model.generate_response(&enhanced_prompt, max_tokens, &history_clone).await {
        Ok(response) => {
            // Reacquire lock to update history
            if let Ok(mut sessions) = data.sessions.lock() {
                if let Some(history) = sessions.get_mut(&session_id) {
                    history.push(format!("assistant: {}", response.clone()));
                }
            } else {
                // Not critical if we fail to update history, just log it
                error!("Failed to update session history");
            }
            
            HttpResponse::Ok().json(ChatResponse {
                response,
                session_id,
            })
        }
        Err(e) => {
            error!("Model error: {}", e);
            HttpResponse::InternalServerError().json(json!({
                "error": format!("Failed to generate response: {}", e)
            }))
        }
    }
} 