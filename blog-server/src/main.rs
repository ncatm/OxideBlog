mod application;
mod data;
mod domain;
mod infrastructure;
mod presentation;

use std::sync::Arc;

use actix_cors::Cors;
use actix_web::{App, HttpServer, web};
use application::{auth_service::AuthService, blog_service::BlogService};
use data::{post_repository::PostRepository, user_repository::UserRepository};
use dotenvy::dotenv;
use infrastructure::{database, jwt::JwtService, logging};
use presentation::{grpc_service::BlogGrpcService, http_handlers};
use tonic::transport::Server;
use tracing::info;

pub mod proto {
    tonic::include_proto!("blog");
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    logging::init();
    dotenv().ok();

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost/blog_db".to_string());
    let jwt_secret =
        std::env::var("JWT_SECRET").unwrap_or_else(|_| "change_me_to_a_long_secret_key_123456".to_string());

    let pool = database::create_pool(&database_url).await?;
    database::run_migrations(&pool).await?;

    let user_repo = Arc::new(UserRepository::new(pool.clone()));
    let post_repo = Arc::new(PostRepository::new(pool.clone()));
    let jwt = Arc::new(JwtService::new(&jwt_secret)?);
    let auth_service = Arc::new(AuthService::new(user_repo.clone(), jwt.clone()));
    let blog_service = Arc::new(BlogService::new(post_repo.clone()));

    let grpc = BlogGrpcService::new(auth_service.clone(), blog_service.clone(), jwt.clone());
    let grpc_addr = "0.0.0.0:50051".parse()?;
    let http_addr = "0.0.0.0:8080";

    let http = HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(auth_service.clone()))
            .app_data(web::Data::new(blog_service.clone()))
            .app_data(web::Data::new(jwt.clone()))
            .wrap(
                Cors::default()
                    .allow_any_origin()
                    .allow_any_method()
                    .allow_any_header()
                    .max_age(3600),
            )
            .service(http_handlers::public_routes())
    })
    .bind(http_addr)?
    .run();

    let grpc = Server::builder()
        .add_service(proto::blog_service_server::BlogServiceServer::new(grpc))
        .serve(grpc_addr);

    info!("HTTP server listening on {http_addr}");
    info!("gRPC server listening on 0.0.0.0:50051");

    tokio::select! {
        res = http => res.map_err(anyhow::Error::from),
        res = grpc => res.map_err(anyhow::Error::from),
    }
}
