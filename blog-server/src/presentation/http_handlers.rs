use std::sync::Arc;

use actix_web::{HttpResponse, Result, delete, get, post, put, web};
use serde::{Deserialize, Serialize};

use crate::{
    application::{auth_service::AuthService, blog_service::BlogService},
    domain::{
        error::BlogError,
        post::{CreatePostRequest, UpdatePostRequest},
        user::{LoginRequest, RegisterRequest, UserPublic},
    },
    presentation::middleware::UserCtx,
};

#[derive(Serialize)]
struct AuthResponse {
    token: String,
    user: UserPublic,
}

#[derive(Deserialize)]
struct Pagination {
    limit: Option<i64>,
    offset: Option<i64>,
}

pub fn public_routes() -> actix_web::Scope {
    use actix_web_httpauth::middleware::HttpAuthentication;
    use crate::presentation::middleware::jwt_validator;
    web::scope("/api")
        .service(register)
        .service(login)
        .service(get_post)
        .service(list_posts)
        .service(
            web::scope("")
                .wrap(HttpAuthentication::with_fn(jwt_validator))
                .service(create_post)
                .service(update_post)
                .service(delete_post),
        )
}

#[post("/auth/register")]
async fn register(
    auth: web::Data<Arc<AuthService>>,
    payload: web::Json<RegisterRequest>,
) -> Result<HttpResponse> {
    let (token, user) = auth
        .register(&payload.username, &payload.email, &payload.password)
        .await
        .map_err(map_http_error)?;
    Ok(HttpResponse::Created().json(AuthResponse {
        token,
        user: user.into(),
    }))
}

#[post("/auth/login")]
async fn login(auth: web::Data<Arc<AuthService>>, payload: web::Json<LoginRequest>) -> Result<HttpResponse> {
    let (token, user) = auth
        .login(&payload.username, &payload.password)
        .await
        .map_err(map_http_error)?;
    Ok(HttpResponse::Ok().json(AuthResponse {
        token,
        user: user.into(),
    }))
}

#[post("/posts")]
async fn create_post(
    blog: web::Data<Arc<BlogService>>,
    user: UserCtx,
    payload: web::Json<CreatePostRequest>,
) -> Result<HttpResponse> {
    let post = blog
        .create_post(&payload.title, &payload.content, user.user_id)
        .await
        .map_err(map_http_error)?;
    Ok(HttpResponse::Created().json(post))
}

#[get("/posts/{id}")]
async fn get_post(blog: web::Data<Arc<BlogService>>, id: web::Path<i64>) -> Result<HttpResponse> {
    let post = blog.get_post(id.into_inner()).await.map_err(map_http_error)?;
    Ok(HttpResponse::Ok().json(post))
}

#[put("/posts/{id}")]
async fn update_post(
    blog: web::Data<Arc<BlogService>>,
    user: UserCtx,
    id: web::Path<i64>,
    payload: web::Json<UpdatePostRequest>,
) -> Result<HttpResponse> {
    let post = blog
        .update_post(
            id.into_inner(),
            user.user_id,
            payload.title.as_deref(),
            payload.content.as_deref(),
        )
        .await
        .map_err(map_http_error)?;
    Ok(HttpResponse::Ok().json(post))
}

#[delete("/posts/{id}")]
async fn delete_post(blog: web::Data<Arc<BlogService>>, user: UserCtx, id: web::Path<i64>) -> Result<HttpResponse> {
    blog.delete_post(id.into_inner(), user.user_id).await.map_err(map_http_error)?;
    Ok(HttpResponse::NoContent().finish())
}

#[get("/posts")]
async fn list_posts(blog: web::Data<Arc<BlogService>>, query: web::Query<Pagination>) -> Result<HttpResponse> {
    let limit = query.limit.unwrap_or(10).max(1);
    let offset = query.offset.unwrap_or(0).max(0);
    let (posts, total) = blog.list_posts(limit, offset).await.map_err(map_http_error)?;
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "posts": posts, "total": total, "limit": limit, "offset": offset
    })))
}

fn map_http_error(err: BlogError) -> actix_web::Error {
    use actix_web::{error, http::StatusCode};
    tracing::warn!(error = %err, "request failed");
    let (status, msg) = match err {
        BlogError::UserAlreadyExists => (StatusCode::CONFLICT, "user already exists"),
        BlogError::InvalidCredentials | BlogError::Unauthorized => (StatusCode::UNAUTHORIZED, "unauthorized"),
        BlogError::PostNotFound | BlogError::UserNotFound => (StatusCode::NOT_FOUND, "not found"),
        BlogError::Forbidden => (StatusCode::FORBIDDEN, "forbidden"),
        BlogError::Validation(_) => (StatusCode::BAD_REQUEST, "bad request"),
        BlogError::Database(_) => (StatusCode::INTERNAL_SERVER_ERROR, "db error"),
    };
    error::InternalError::new(msg, status).into()
}
