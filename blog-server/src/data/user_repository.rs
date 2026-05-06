use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{FromRow, PgPool};

use crate::domain::{error::BlogError, user::User};

#[derive(Debug, Clone, FromRow)]
struct UserRow {
    id: i64,
    username: String,
    email: String,
    password_hash: String,
    created_at: DateTime<Utc>,
}

impl From<UserRow> for User {
    fn from(row: UserRow) -> Self {
        Self {
            id: row.id,
            username: row.username,
            email: row.email,
            password_hash: row.password_hash,
            created_at: row.created_at,
        }
    }
}

#[async_trait]
pub trait UserStore: Send + Sync {
    async fn create(&self, username: &str, email: &str, password_hash: &str) -> Result<User, BlogError>;
    async fn find_by_username(&self, username: &str) -> Result<User, BlogError>;
    async fn find_by_email(&self, email: &str) -> Result<User, BlogError>;
}

pub struct UserRepository {
    pool: PgPool,
}

impl UserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl UserStore for UserRepository {
    async fn create(&self, username: &str, email: &str, password_hash: &str) -> Result<User, BlogError> {
        let row = sqlx::query_as::<_, UserRow>(
            r#"INSERT INTO users (username, email, password_hash)
               VALUES ($1, $2, $3)
               RETURNING id, username, email, password_hash, created_at"#,
        )
        .bind(username)
        .bind(email)
        .bind(password_hash)
        .fetch_one(&self.pool)
        .await
        .map_err(map_user_db_error)?;
        Ok(row.into())
    }

    async fn find_by_username(&self, username: &str) -> Result<User, BlogError> {
        let row = sqlx::query_as::<_, UserRow>(
            r#"SELECT id, username, email, password_hash, created_at
               FROM users WHERE username = $1"#,
        )
        .bind(username)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| BlogError::Database(e.to_string()))?;
        match row {
            Some(row) => Ok(row.into()),
            None => Err(BlogError::UserNotFound),
        }
    }

    async fn find_by_email(&self, email: &str) -> Result<User, BlogError> {
        let row = sqlx::query_as::<_, UserRow>(
            r#"SELECT id, username, email, password_hash, created_at
               FROM users WHERE email = $1"#,
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| BlogError::Database(e.to_string()))?;
        match row {
            Some(row) => Ok(row.into()),
            None => Err(BlogError::UserNotFound),
        }
    }
}

fn map_user_db_error(err: sqlx::Error) -> BlogError {
    if let sqlx::Error::Database(ref db) = err {
        if db.code().as_deref() == Some("23505") {
            return BlogError::UserAlreadyExists;
        }
    }
    BlogError::Database(err.to_string())
}
