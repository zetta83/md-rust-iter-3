use crate::application::auth_service::AuthService;
use crate::data::user_repository::UserRepository;
use crate::domain::error::DomainError;
use crate::presentation::dto::{AuthResponse, LoginRequest, RegisterRequest, User};
use actix_web::{HttpResponse, web};

/*
   - POST /api/auth/register # регистрация нового пользователя
      req: { username, email, password } // hash_pass with Argon2
      res:    201 { "token": "...", "user": {...} }
              409 UserAlreadyExists
*/
#[actix_web::post("/register")]
async fn handler_register(
    user_srv: web::Data<AuthService<dyn UserRepository>>,
    payload: web::Json<RegisterRequest>,
) -> actix_web::Result<HttpResponse, DomainError> {
    let user = user_srv
        .register(
            payload.username.clone(),
            payload.email.clone(),
            payload.password.clone(),
        )
        .await?;

    Ok(HttpResponse::Created().json(AuthResponse {
        token: user_srv.generate_token(&user)?,
        user: User {
            id: user.id,
            username: user.username.clone(),
            email: user.email.clone(),
        },
    }))
}

/*
   - POST /api/auth/login # вход в систему
      req: { username, password }
      res:    200 { "token": "...", "user": {...} }
              401 Unauthorized
*/
#[actix_web::post("/login")]
async fn handler_login(
    user_srv: web::Data<AuthService<dyn UserRepository>>,
    payload: web::Json<LoginRequest>,
) -> actix_web::Result<HttpResponse, DomainError> {
    let user = user_srv
        .login(payload.username.clone(), payload.password.clone())
        .await?;

    Ok(HttpResponse::Ok().json(AuthResponse {
        token: user_srv.generate_token(&user)?,
        user: User {
            id: user.id,
            username: user.username.clone(),
            email: user.email.clone(),
        },
    }))
}

pub fn configure() -> actix_web::Scope {
    web::scope("/auth")
        .service(handler_register)
        .service(handler_login)
}
