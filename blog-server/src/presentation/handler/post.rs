use crate::application::auth_service::AuthenticatedUser;
use crate::application::blog_service::BlogService;
use crate::data::post_repository::PostRepository;
use crate::domain::error::DomainError;
use crate::infrastructure::jwt::JwtKeys;
use crate::presentation::dto::{
    ListPostsResponse, PaginationQuery, PostCreateRequest, PostResponse,
};
use crate::presentation::middleware::JwtAuthMiddleware;
use actix_web::{HttpResponse, web};

/*
   - GET /api/posts # список постов (без JWT, с пагинацией)
      req: { limit=10, offset=0 }
      res:    200 { "posts": [...], "total": N, "limit": 10, "offset": 0 }
*/
#[actix_web::get("")]
async fn handler_get_list_posts(
    blog_srv: web::Data<BlogService<dyn PostRepository>>,
    query: web::Query<PaginationQuery>,
) -> actix_web::Result<HttpResponse, DomainError> {
    let page = query.page.unwrap_or(0);
    let limit = query.limit.unwrap_or(10);

    Ok(HttpResponse::Ok().json(ListPostsResponse {
        posts: blog_srv
            .get_list_posts(page, limit)
            .await?
            .into_iter()
            .map(PostResponse::from)
            .collect(),
        total: blog_srv.get_posts_count().await?,
        limit,
        offset: page,
    }))
}

/*
   - POST /api/posts # создание поста (JWT)
      req: { title, content }
      res:    201 { id, title, content, author_id }
*/
#[actix_web::post("")]
async fn handler_create_post(
    user: AuthenticatedUser,
    blog_srv: web::Data<BlogService<dyn PostRepository>>,
    payload: web::Json<PostCreateRequest>,
) -> actix_web::Result<HttpResponse, DomainError> {
    Ok(HttpResponse::Created().json(PostResponse::from(
        blog_srv
            .create_post(payload.title.as_str(), payload.content.as_str(), user.id)
            .await?,
    )))
}

/*
   - GET /api/posts/{id} # получение поста (без JWT)
      req: { id }
      res:    200 { id, title, content, author_id }
              404 Not Found
*/
#[actix_web::get("/{id}")]
async fn handler_get_post(
    blog_srv: web::Data<BlogService<dyn PostRepository>>,
    path: web::Path<i64>,
) -> actix_web::Result<HttpResponse, DomainError> {
    Ok(HttpResponse::Ok().json(PostResponse::from(
        blog_srv.get_post_by_id(path.into_inner()).await?,
    )))
}

/*
   - PUT /api/posts/{id} # обновление поста (JWT)
      req: { id }
      res:    200 { id, title, content, author_id }
              404 Not Found
              403 Forbidden
*/
#[actix_web::put("/{id}")]
async fn handler_upd_post(
    user: AuthenticatedUser,
    blog_srv: web::Data<BlogService<dyn PostRepository>>,
    path: web::Path<i64>,
    payload: web::Json<PostCreateRequest>,
) -> actix_web::Result<HttpResponse, DomainError> {
    Ok(HttpResponse::Ok().json(PostResponse::from(
        blog_srv
            .update_post(
                path.into_inner(),
                user.id,
                payload.title.as_str(),
                payload.content.as_str(),
            )
            .await?,
    )))
}

/*
  - DELETE /api/posts/{id} # удаление поста (JWT)
      req: { id }
      res:    204 No Content
              404 Not Found
              403 Forbidden
*/
#[actix_web::delete("/{id}")]
async fn handler_delete_post(
    user: AuthenticatedUser,
    blog_srv: web::Data<BlogService<dyn PostRepository>>,
    path: web::Path<i64>,
) -> actix_web::Result<HttpResponse, DomainError> {
    blog_srv.delete_post(path.into_inner(), user.id).await?;
    Ok(HttpResponse::NoContent().finish())
}

pub fn configure(keys: &JwtKeys) -> actix_web::Scope {
    web::scope("/posts")
        .service(handler_get_post)
        .service(handler_get_list_posts)
        .service(
            web::scope("")
                .wrap(JwtAuthMiddleware::new(keys.clone()))
                .service(handler_create_post)
                .service(handler_upd_post)
                .service(handler_delete_post),
        )
}
