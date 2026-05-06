use thiserror::Error;

#[derive(Debug, Error)]
pub enum BlogError {
    #[error("user not found")]
    UserNotFound,
    #[error("user already exists")]
    UserAlreadyExists,
    #[error("invalid credentials")]
    InvalidCredentials,
    #[error("post not found")]
    PostNotFound,
    #[error("forbidden")]
    Forbidden,
    #[error("validation error: {0}")]
    Validation(String),
    #[error("database error: {0}")]
    Database(String),
    #[error("unauthorized")]
    Unauthorized,
}
