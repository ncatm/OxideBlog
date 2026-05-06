use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{FromRow, PgPool};

use crate::domain::{error::BlogError, post::Post};

#[derive(Debug, Clone, FromRow)]
struct PostRow {
    id: i64,
    title: String,
    content: String,
    author_id: i64,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl From<PostRow> for Post {
    fn from(row: PostRow) -> Self {
        Self {
            id: row.id,
            title: row.title,
            content: row.content,
            author_id: row.author_id,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

#[async_trait]
pub trait PostStore: Send + Sync {
    async fn create(&self, title: &str, content: &str, author_id: i64) -> Result<Post, BlogError>;
    async fn get(&self, id: i64) -> Result<Post, BlogError>;
    async fn update(&self, id: i64, title: Option<&str>, content: Option<&str>) -> Result<Post, BlogError>;
    async fn delete(&self, id: i64) -> Result<(), BlogError>;
    async fn list(&self, limit: i64, offset: i64) -> Result<(Vec<Post>, i64), BlogError>;
}

pub struct PostRepository {
    pool: PgPool,
}

impl PostRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl PostStore for PostRepository {
    async fn create(&self, title: &str, content: &str, author_id: i64) -> Result<Post, BlogError> {
        let row = sqlx::query_as::<_, PostRow>(
            r#"INSERT INTO posts (title, content, author_id)
               VALUES ($1, $2, $3)
               RETURNING id, title, content, author_id, created_at, updated_at"#,
        )
        .bind(title)
        .bind(content)
        .bind(author_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| BlogError::Database(e.to_string()))?;
        Ok(row.into())
    }

    async fn get(&self, id: i64) -> Result<Post, BlogError> {
        let row = sqlx::query_as::<_, PostRow>(
            r#"SELECT id, title, content, author_id, created_at, updated_at
               FROM posts WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| BlogError::Database(e.to_string()))?;
        row.map(Into::into).ok_or(BlogError::PostNotFound)
    }

    async fn update(&self, id: i64, title: Option<&str>, content: Option<&str>) -> Result<Post, BlogError> {
        let row = sqlx::query_as::<_, PostRow>(
            r#"UPDATE posts
               SET title = COALESCE($2, title),
                   content = COALESCE($3, content),
                   updated_at = NOW()
               WHERE id = $1
               RETURNING id, title, content, author_id, created_at, updated_at"#,
        )
        .bind(id)
        .bind(title)
        .bind(content)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| BlogError::Database(e.to_string()))?;
        row.map(Into::into).ok_or(BlogError::PostNotFound)
    }

    async fn delete(&self, id: i64) -> Result<(), BlogError> {
        let affected = sqlx::query("DELETE FROM posts WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| BlogError::Database(e.to_string()))?
            .rows_affected();
        if affected == 0 {
            return Err(BlogError::PostNotFound);
        }
        Ok(())
    }

    async fn list(&self, limit: i64, offset: i64) -> Result<(Vec<Post>, i64), BlogError> {
        let posts = sqlx::query_as::<_, PostRow>(
            r#"SELECT id, title, content, author_id, created_at, updated_at
               FROM posts
               ORDER BY created_at DESC
               LIMIT $1 OFFSET $2"#,
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| BlogError::Database(e.to_string()))?
        .into_iter()
        .map(Into::into)
        .collect();

        let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM posts")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| BlogError::Database(e.to_string()))?;
        Ok((posts, total))
    }
}
