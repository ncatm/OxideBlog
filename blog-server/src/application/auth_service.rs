use std::sync::Arc;

use argon2::{
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
    password_hash::{SaltString, rand_core::OsRng},
};

use crate::{
    data::user_repository::UserStore,
    domain::{error::BlogError, user::User},
    infrastructure::jwt::JwtService,
};

pub struct AuthService {
    user_repo: Arc<dyn UserStore>,
    jwt: Arc<JwtService>,
}

impl AuthService {
    pub fn new(user_repo: Arc<dyn UserStore>, jwt: Arc<JwtService>) -> Self {
        Self { user_repo, jwt }
    }

    pub async fn register(&self, username: &str, email: &str, password: &str) -> Result<(String, User), BlogError> {
        if self.user_repo.find_by_username(username).await.is_ok() || self.user_repo.find_by_email(email).await.is_ok()
        {
            return Err(BlogError::UserAlreadyExists);
        }
        let hash = hash_password(password)?;
        let user = self.user_repo.create(username, email, &hash).await?;
        let token = self.jwt.generate_token(user.id, &user.username)?;
        Ok((token, user))
    }

    pub async fn login(&self, username: &str, password: &str) -> Result<(String, User), BlogError> {
        let user = self.user_repo.find_by_username(username).await?;
        verify_password(password, &user.password_hash)?;
        let token = self.jwt.generate_token(user.id, &user.username)?;
        Ok((token, user))
    }
}

fn hash_password(password: &str) -> Result<String, BlogError> {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|h| h.to_string())
        .map_err(|e| BlogError::Validation(e.to_string()))
}

fn verify_password(password: &str, hash: &str) -> Result<(), BlogError> {
    let parsed = PasswordHash::new(hash).map_err(|_| BlogError::InvalidCredentials)?;
    Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .map_err(|_| BlogError::InvalidCredentials)
}
