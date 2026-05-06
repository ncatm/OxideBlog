use std::sync::Arc;

use tonic::{Request, Response, Status};

use crate::{
    application::{auth_service::AuthService, blog_service::BlogService},
    domain::error::BlogError,
    infrastructure::jwt::JwtService,
    proto::{
        AuthResponse, CreatePostRequest, DeletePostRequest, DeletePostResponse, GetPostRequest, ListPostsRequest,
        ListPostsResponse, LoginRequest, Post as ProtoPost, PostResponse, RegisterRequest, UpdatePostRequest, User,
        blog_service_server::BlogService as BlogServiceTrait,
    },
};

pub struct BlogGrpcService {
    auth: Arc<AuthService>,
    blog: Arc<BlogService>,
    jwt: Arc<JwtService>,
}

impl BlogGrpcService {
    pub fn new(auth: Arc<AuthService>, blog: Arc<BlogService>, jwt: Arc<JwtService>) -> Self {
        Self { auth, blog, jwt }
    }
}

#[tonic::async_trait]
impl BlogServiceTrait for BlogGrpcService {
    async fn register(&self, request: Request<RegisterRequest>) -> Result<Response<AuthResponse>, Status> {
        let r = request.into_inner();
        let (token, user) = self
            .auth
            .register(&r.username, &r.email, &r.password)
            .await
            .map_err(map_grpc_error)?;
        Ok(Response::new(AuthResponse {
            token,
            user: Some(User {
                id: user.id,
                username: user.username,
                email: user.email,
                created_at: user.created_at.to_rfc3339(),
            }),
        }))
    }

    async fn login(&self, request: Request<LoginRequest>) -> Result<Response<AuthResponse>, Status> {
        let r = request.into_inner();
        let (token, user) = self.auth.login(&r.username, &r.password).await.map_err(map_grpc_error)?;
        Ok(Response::new(AuthResponse {
            token,
            user: Some(User {
                id: user.id,
                username: user.username,
                email: user.email,
                created_at: user.created_at.to_rfc3339(),
            }),
        }))
    }

    async fn create_post(&self, request: Request<CreatePostRequest>) -> Result<Response<PostResponse>, Status> {
        let user_id = authorized_user_id(request.metadata(), &self.jwt)?;
        let r = request.into_inner();
        let post = self
            .blog
            .create_post(&r.title, &r.content, user_id)
            .await
            .map_err(map_grpc_error)?;
        Ok(Response::new(PostResponse { post: Some(to_proto_post(post)) }))
    }

    async fn get_post(&self, request: Request<GetPostRequest>) -> Result<Response<PostResponse>, Status> {
        let post = self.blog.get_post(request.into_inner().id).await.map_err(map_grpc_error)?;
        Ok(Response::new(PostResponse { post: Some(to_proto_post(post)) }))
    }

    async fn update_post(&self, request: Request<UpdatePostRequest>) -> Result<Response<PostResponse>, Status> {
        let user_id = authorized_user_id(request.metadata(), &self.jwt)?;
        let r = request.into_inner();
        let title = if r.title.is_empty() {
            None
        } else {
            Some(r.title.as_str())
        };
        let content = if r.content.is_empty() {
            None
        } else {
            Some(r.content.as_str())
        };
        let post = self
            .blog
            .update_post(r.id, user_id, title, content)
            .await
            .map_err(map_grpc_error)?;
        Ok(Response::new(PostResponse { post: Some(to_proto_post(post)) }))
    }

    async fn delete_post(&self, request: Request<DeletePostRequest>) -> Result<Response<DeletePostResponse>, Status> {
        let user_id = authorized_user_id(request.metadata(), &self.jwt)?;
        self.blog
            .delete_post(request.into_inner().id, user_id)
            .await
            .map_err(map_grpc_error)?;
        Ok(Response::new(DeletePostResponse { success: true }))
    }

    async fn list_posts(&self, request: Request<ListPostsRequest>) -> Result<Response<ListPostsResponse>, Status> {
        let r = request.into_inner();
        let limit = if r.limit > 0 { r.limit } else { 10 };
        let offset = r.offset.max(0);
        let (posts, total) = self.blog.list_posts(limit, offset).await.map_err(map_grpc_error)?;
        Ok(Response::new(ListPostsResponse {
            posts: posts.into_iter().map(to_proto_post).collect(),
            total,
            limit,
            offset,
        }))
    }
}

fn authorized_user_id(md: &tonic::metadata::MetadataMap, jwt: &JwtService) -> Result<i64, Status> {
    let auth = md
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| Status::unauthenticated("missing authorization"))?;
    let token = auth.trim_start_matches("Bearer ").trim();
    let claims = jwt.verify_token(token).map_err(|_| Status::unauthenticated("invalid token"))?;
    Ok(claims.user_id)
}

fn to_proto_post(post: crate::domain::post::Post) -> ProtoPost {
    ProtoPost {
        id: post.id,
        title: post.title,
        content: post.content,
        author_id: post.author_id,
        created_at: post.created_at.to_rfc3339(),
        updated_at: post.updated_at.to_rfc3339(),
    }
}

fn map_grpc_error(err: BlogError) -> Status {
    match err {
        BlogError::UserAlreadyExists => Status::already_exists("user exists"),
        BlogError::InvalidCredentials | BlogError::Unauthorized => Status::unauthenticated("unauthorized"),
        BlogError::PostNotFound | BlogError::UserNotFound => Status::not_found("not found"),
        BlogError::Forbidden => Status::permission_denied("forbidden"),
        BlogError::Validation(m) => Status::invalid_argument(m),
        BlogError::Database(m) => Status::internal(m),
    }
}
