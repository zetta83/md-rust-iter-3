use crate::infrastructure::jwt::JwtKeys;
use crate::presentation::handler;
use actix_web::{HttpResponse, web};

/*
   - GET /api/health # Health check
       res:    200 { ok: true }
*/
#[actix_web::get("/health")]
async fn health() -> impl actix_web::Responder {
    HttpResponse::Ok().json(serde_json::json!({"status": "ok"}))
}

pub fn configure(keys: &JwtKeys) -> impl Fn(&mut web::ServiceConfig) {
    move |cfg| {
        cfg.service(
            web::scope("/api")
                .service(health)
                .service(handler::auth::configure())
                .service(handler::post::configure(keys)),
        );
    }
}
