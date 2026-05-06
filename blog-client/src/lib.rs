pub mod error;
pub mod grpc_client;
pub mod http_client;

pub use grpc_client::connect_channel;

pub mod proto {
    tonic::include_proto!("blog");
}

use error::BlogClientError;
use http_client::{AuthResponse, ListResponse, Post};
use proto::blog_service_client::BlogServiceClient;
use tonic::metadata::MetadataValue;
use tonic::Request;

pub use http_client::User;

#[derive(Clone, Debug)]
pub enum Transport {
    Http(String),
    Grpc(String),
}

#[derive(Clone, Debug)]
pub struct BlogClient {
    pub transport: Transport,
    token: Option<String>,
}

impl BlogClient {
    pub fn new(transport: Transport) -> Self {
        Self { transport, token: None }
    }

    pub fn set_token(&mut self, token: impl Into<String>) {
        self.token = Some(token.into());
    }

    pub fn get_token(&self) -> Option<&str> {
        self.token.as_deref()
    }

    fn bearer(&self) -> Result<&str, BlogClientError> {
        self.token.as_deref().ok_or(BlogClientError::Unauthorized)
    }

    fn auth_header(token: &str) -> Result<MetadataValue<tonic::metadata::Ascii>, BlogClientError> {
        MetadataValue::try_from(format!("Bearer {token}"))
            .map_err(|_| BlogClientError::InvalidRequest("invalid bearer token for metadata".into()))
    }

    pub async fn register(&mut self, username: &str, email: &str, password: &str) -> Result<AuthResponse, BlogClientError> {
        match &self.transport.clone() {
            Transport::Http(base) => {
                let auth = http_client::register(base, username, email, password).await?;
                self.token = Some(auth.token.clone());
                Ok(auth)
            }
            Transport::Grpc(endpoint) => {
                let mut client = BlogServiceClient::connect(endpoint.clone()).await?;
                let resp = client
                    .register(proto::RegisterRequest {
                        username: username.into(),
                        email: email.into(),
                        password: password.into(),
                    })
                    .await
                    .map_err(BlogClientError::from_grpc_status)?;
                let body = resp.into_inner();
                let user_proto = body
                    .user
                    .ok_or_else(|| BlogClientError::InvalidRequest("empty user in auth response".into()))?;
                let user = user_from_proto(user_proto)?;
                self.token = Some(body.token.clone());
                Ok(AuthResponse { token: body.token, user })
            }
        }
    }

    pub async fn login(&mut self, username: &str, password: &str) -> Result<AuthResponse, BlogClientError> {
        match &self.transport.clone() {
            Transport::Http(base) => {
                let auth = http_client::login(base, username, password).await?;
                self.token = Some(auth.token.clone());
                Ok(auth)
            }
            Transport::Grpc(endpoint) => {
                let mut client = BlogServiceClient::connect(endpoint.clone()).await?;
                let resp = client
                    .login(proto::LoginRequest {
                        username: username.into(),
                        password: password.into(),
                    })
                    .await
                    .map_err(BlogClientError::from_grpc_status)?;
                let body = resp.into_inner();
                let user_proto = body
                    .user
                    .ok_or_else(|| BlogClientError::InvalidRequest("empty user in auth response".into()))?;
                let user = user_from_proto(user_proto)?;
                self.token = Some(body.token.clone());
                Ok(AuthResponse { token: body.token, user })
            }
        }
    }

    pub async fn create_post(&mut self, title: &str, content: &str) -> Result<Post, BlogClientError> {
        match &self.transport.clone() {
            Transport::Http(base) => {
                let token = self.bearer()?.to_string();
                http_client::create_post(base, &token, title, content).await
            }
            Transport::Grpc(endpoint) => {
                let token = self.bearer()?.to_string();
                let mut client = BlogServiceClient::connect(endpoint.clone()).await?;
                let mut req = Request::new(proto::CreatePostRequest {
                    title: title.into(),
                    content: content.into(),
                });
                req.metadata_mut()
                    .insert("authorization", Self::auth_header(&token)?);
                let resp = client
                    .create_post(req)
                    .await
                    .map_err(BlogClientError::from_grpc_status)?;
                let proto_post = resp
                    .into_inner()
                    .post
                    .ok_or_else(|| BlogClientError::InvalidRequest("empty post".into()))?;
                post_from_proto(proto_post)
            }
        }
    }

    pub async fn get_post(&mut self, id: i64) -> Result<Post, BlogClientError> {
        match &self.transport.clone() {
            Transport::Http(base) => http_client::get_post(base, id).await,
            Transport::Grpc(endpoint) => {
                let mut client = BlogServiceClient::connect(endpoint.clone()).await?;
                let resp = client
                    .get_post(proto::GetPostRequest { id })
                    .await
                    .map_err(BlogClientError::from_grpc_status)?;
                let proto_post = resp.into_inner().post.ok_or(BlogClientError::NotFound)?;
                post_from_proto(proto_post)
            }
        }
    }

    pub async fn update_post(&mut self, id: i64, title: &str, content: Option<&str>) -> Result<Post, BlogClientError> {
        match &self.transport.clone() {
            Transport::Http(base) => {
                let token = self.bearer()?.to_string();
                http_client::update_post(base, &token, id, title, content).await
            }
            Transport::Grpc(endpoint) => {
                let token = self.bearer()?.to_string();
                let mut client = BlogServiceClient::connect(endpoint.clone()).await?;
                let mut req = Request::new(proto::UpdatePostRequest {
                    id,
                    title: title.into(),
                    content: content.unwrap_or("").to_string(),
                });
                req.metadata_mut()
                    .insert("authorization", Self::auth_header(&token)?);
                let resp = client
                    .update_post(req)
                    .await
                    .map_err(BlogClientError::from_grpc_status)?;
                let proto_post = resp
                    .into_inner()
                    .post
                    .ok_or_else(|| BlogClientError::InvalidRequest("empty post".into()))?;
                post_from_proto(proto_post)
            }
        }
    }

    pub async fn delete_post(&mut self, id: i64) -> Result<(), BlogClientError> {
        match &self.transport.clone() {
            Transport::Http(base) => {
                let token = self.bearer()?.to_string();
                http_client::delete_post(base, &token, id).await
            }
            Transport::Grpc(endpoint) => {
                let token = self.bearer()?.to_string();
                let mut client = BlogServiceClient::connect(endpoint.clone()).await?;
                let mut req = Request::new(proto::DeletePostRequest { id });
                req.metadata_mut()
                    .insert("authorization", Self::auth_header(&token)?);
                client
                    .delete_post(req)
                    .await
                    .map_err(BlogClientError::from_grpc_status)?;
                Ok(())
            }
        }
    }

    pub async fn list_posts(&mut self, limit: i64, offset: i64) -> Result<ListResponse, BlogClientError> {
        match &self.transport.clone() {
            Transport::Http(base) => http_client::list_posts(base, limit, offset).await,
            Transport::Grpc(endpoint) => {
                let mut client = BlogServiceClient::connect(endpoint.clone()).await?;
                let resp = client
                    .list_posts(proto::ListPostsRequest { limit, offset })
                    .await
                    .map_err(BlogClientError::from_grpc_status)?;
                let body = resp.into_inner();
                let mut posts = Vec::with_capacity(body.posts.len());
                for p in body.posts {
                    posts.push(post_from_proto(p)?);
                }
                Ok(ListResponse {
                    posts,
                    total: body.total,
                    limit: body.limit,
                    offset: body.offset,
                })
            }
        }
    }
}

fn user_from_proto(u: proto::User) -> Result<User, BlogClientError> {
    let created_at = chrono::DateTime::parse_from_rfc3339(&u.created_at)
        .map(|dt| dt.with_timezone(&chrono::Utc))
        .map_err(|e| BlogClientError::InvalidRequest(e.to_string()))?;
    Ok(User {
        id: u.id,
        username: u.username,
        email: u.email,
        created_at,
    })
}

fn post_from_proto(p: proto::Post) -> Result<Post, BlogClientError> {
    let created_at = chrono::DateTime::parse_from_rfc3339(&p.created_at)
        .map(|dt| dt.with_timezone(&chrono::Utc))
        .map_err(|e| BlogClientError::InvalidRequest(e.to_string()))?;
    let updated_at = chrono::DateTime::parse_from_rfc3339(&p.updated_at)
        .map(|dt| dt.with_timezone(&chrono::Utc))
        .map_err(|e| BlogClientError::InvalidRequest(e.to_string()))?;
    Ok(Post {
        id: p.id,
        title: p.title,
        content: p.content,
        author_id: p.author_id,
        created_at,
        updated_at,
    })
}
