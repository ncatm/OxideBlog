use std::time::Duration;

use reqwest::{Client, Response, StatusCode};
use serde::{Deserialize, Serialize};

use crate::error::BlogClientError;

fn http() -> Result<Client, BlogClientError> {
    Ok(Client::builder().timeout(Duration::from_secs(30)).build()?)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub email: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: User,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Post {
    pub id: i64,
    pub title: String,
    pub content: String,
    pub author_id: i64,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListResponse {
    pub posts: Vec<Post>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

async fn map_json<T: serde::de::DeserializeOwned>(res: Response) -> Result<T, BlogClientError> {
    let status = res.status();
    if status == StatusCode::NOT_FOUND {
        return Err(BlogClientError::NotFound);
    }
    if status == StatusCode::UNAUTHORIZED {
        return Err(BlogClientError::Unauthorized);
    }
    if !status.is_success() {
        let body = res.text().await.unwrap_or_default();
        return Err(BlogClientError::InvalidRequest(format!("HTTP {status}: {body}")));
    }
    Ok(res.json().await?)
}

async fn map_empty(res: Response) -> Result<(), BlogClientError> {
    let status = res.status();
    if status == StatusCode::NOT_FOUND {
        return Err(BlogClientError::NotFound);
    }
    if status == StatusCode::UNAUTHORIZED {
        return Err(BlogClientError::Unauthorized);
    }
    if status == StatusCode::FORBIDDEN {
        return Err(BlogClientError::Unauthorized);
    }
    if !status.is_success() {
        let body = res.text().await.unwrap_or_default();
        return Err(BlogClientError::InvalidRequest(format!("HTTP {status}: {body}")));
    }
    Ok(())
}

pub async fn register(base: &str, username: &str, email: &str, password: &str) -> Result<AuthResponse, BlogClientError> {
    let res = http()?
        .post(format!("{base}/api/auth/register"))
        .json(&serde_json::json!({ "username": username, "email": email, "password": password }))
        .send()
        .await?;
    map_json(res).await
}

pub async fn login(base: &str, username: &str, password: &str) -> Result<AuthResponse, BlogClientError> {
    let res = http()?
        .post(format!("{base}/api/auth/login"))
        .json(&serde_json::json!({ "username": username, "password": password }))
        .send()
        .await?;
    map_json(res).await
}

pub async fn create_post(base: &str, token: &str, title: &str, content: &str) -> Result<Post, BlogClientError> {
    let res = http()?
        .post(format!("{base}/api/posts"))
        .header("Authorization", format!("Bearer {token}"))
        .json(&serde_json::json!({ "title": title, "content": content }))
        .send()
        .await?;
    map_json(res).await
}

pub async fn get_post(base: &str, id: i64) -> Result<Post, BlogClientError> {
    let res = http()?.get(format!("{base}/api/posts/{id}")).send().await?;
    map_json(res).await
}

pub async fn update_post(
    base: &str,
    token: &str,
    id: i64,
    title: &str,
    content: Option<&str>,
) -> Result<Post, BlogClientError> {
    let body = serde_json::json!({
        "title": title,
        "content": content,
    });
    let res = http()?
        .put(format!("{base}/api/posts/{id}"))
        .header("Authorization", format!("Bearer {token}"))
        .json(&body)
        .send()
        .await?;
    map_json(res).await
}

pub async fn delete_post(base: &str, token: &str, id: i64) -> Result<(), BlogClientError> {
    let res = http()?
        .delete(format!("{base}/api/posts/{id}"))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await?;
    map_empty(res).await
}

pub async fn list_posts(base: &str, limit: i64, offset: i64) -> Result<ListResponse, BlogClientError> {
    let res = http()?
        .get(format!("{base}/api/posts"))
        .query(&[("limit", limit.to_string()), ("offset", offset.to_string())])
        .send()
        .await?;
    map_json(res).await
}
