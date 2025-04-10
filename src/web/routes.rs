use actix_web::web;
use crate::web::handlers;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
            .route("/chat", web::post().to(handlers::chat))
    )
    .route("/", web::get().to(handlers::index))
    .route("/health", web::get().to(handlers::health_check));
} 