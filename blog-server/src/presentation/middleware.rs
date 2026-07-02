use crate::application::auth_service::{AuthService, extract_user_from_token};
use crate::data::user_repository::UserRepository;
use crate::infrastructure::jwt::JwtKeys;
use actix_web::dev::{Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::{Error, HttpMessage, web};
use futures_util::future::LocalBoxFuture;
use std::cell::RefCell;
use std::future::{Ready, ready};
use std::rc::Rc;
use std::task::{Context, Poll};

pub struct JwtAuthMiddleware {
    keys: JwtKeys,
}

impl JwtAuthMiddleware {
    pub fn new(keys: JwtKeys) -> Self {
        Self { keys }
    }
}

impl<S, B> Transform<S, ServiceRequest> for JwtAuthMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = JwtAuthService<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(JwtAuthService {
            service: Rc::new(RefCell::new(service)),
            keys: self.keys.clone(),
        }))
    }
}

pub struct JwtAuthService<S> {
    service: Rc<RefCell<S>>,
    keys: JwtKeys,
}

impl<S, B> Service<ServiceRequest> for JwtAuthService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&self, ctx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.borrow_mut().poll_ready(ctx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let keys = self.keys.clone();
        let service = Rc::clone(&self.service);
        let auth_service = req
            .app_data::<web::Data<AuthService<dyn UserRepository>>>()
            .cloned();
        let auth_header = req
            .headers()
            .get(actix_web::http::header::AUTHORIZATION)
            .and_then(|h| h.to_str().ok())
            .map(str::to_owned);

        Box::pin(async move {
            let auth_service = auth_service
                .ok_or_else(|| actix_web::error::ErrorInternalServerError("AuthService missing"))?;

            let header = auth_header.ok_or_else(|| {
                actix_web::error::ErrorUnauthorized("missing authorization header")
            })?;
            let token = header.strip_prefix("Bearer ").ok_or_else(|| {
                actix_web::error::ErrorUnauthorized("invalid authorization header")
            })?;

            let user = extract_user_from_token(token, &keys, auth_service.get_ref()).await?;
            req.extensions_mut().insert(user);

            let fut = {
                let svc = service.borrow_mut();
                svc.call(req)
            };

            fut.await
        })
    }
}
