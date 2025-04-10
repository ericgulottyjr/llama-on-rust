mod model;
mod web;

use actix_web::{App, HttpServer, web::Data};
use actix_files as fs;
use dotenv::dotenv;
use log::{info, error};
use std::sync::Mutex;
use std::collections::HashMap;
use tera::Tera;

use model::ModelManager;
use web::routes;

// App state structure
struct AppState {
    tera: Tera,
    model: Data<ModelManager>,
    sessions: Mutex<HashMap<uuid::Uuid, Vec<String>>>,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize environment
    dotenv().ok();
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    
    info!("Starting LLaMa web application");
    
    // Initialize the model manager (connection to mistral.rs server)
    let model_manager = match ModelManager::new().await {
        Ok(manager) => {
            info!("Connection to mistral.rs server initialized");
            Data::new(manager)
        },
        Err(e) => {
            error!("Failed to initialize connection to mistral.rs server: {}", e);
            std::process::exit(1);
        }
    };
    
    // Initialize template engine
    let mut tera = match Tera::new("templates/**/*") {
        Ok(t) => t,
        Err(e) => {
            error!("Template parsing error: {}", e);
            std::process::exit(1);
        }
    };
    tera.autoescape_on(vec![".html", ".sql"]);
    
    // Create app state
    let app_state = Data::new(AppState {
        tera,
        model: model_manager.clone(),
        sessions: Mutex::new(HashMap::new()),
    });
    
    // Start web server
    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .app_data(model_manager.clone())
            .configure(routes::configure)
            .service(fs::Files::new("/static", "./static").show_files_listing())
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
