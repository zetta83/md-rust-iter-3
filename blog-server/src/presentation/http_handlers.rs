/*
   # Публичные маршруты (без JWT):
   - GET /api/health # Health check
       res:    200 { ok: true }

   - POST /api/auth/register # регистрация нового пользователя
       req: { username, email, password } // hash_pass with Argon2
       res:    201 { "token": "...", "user": {...} }
               409 UserAlreadyExists

   - POST /api/auth/login # вход в систему
       req: { username, password }
       res:    200 { "token": "...", "user": {...} }
               401 Unauthorized

   - POST /api/posts # создание поста (JWT)
       req: { title, content }
       res:    201 { id, title, content, author_id }

   - GET /api/posts/{id} # получение поста (без JWT)
       req: { id }
       res:    200 { id, title, content, author_id }
               404 Not Found

   - PUT /api/posts/{id} # обновление поста (JWT)
       req: { id }
       res:    200 { id, title, content, author_id }
               404 Not Found
               403 Forbidden

   - DELETE /api/posts/{id} # удаление поста (JWT)
       req: { id }
       res:    204 No Content
               404 Not Found
               403 Forbidden

   - GET /api/posts # список постов (без JWT, с пагинацией)
       req: { limit=10, offset=0 }
       res:    200 { "posts": [...], "total": N, "limit": 10, "offset": 0 }
*/
use crate::infrastructure::jwt::JwtKeys;
use actix_web::{HttpResponse, web};

#[actix_web::get("/health")]
async fn health() -> impl actix_web::Responder {
    HttpResponse::Ok().json(serde_json::json!({"status": "ok"}))
}

// pub fn configure(keys: &JwtKeys) -> impl Fn(&mut web::ServiceConfig) {
pub fn configure() -> impl Fn(&mut web::ServiceConfig) {
    move |cfg| {
        cfg.service(web::scope("/api").service(health));
    }
}
