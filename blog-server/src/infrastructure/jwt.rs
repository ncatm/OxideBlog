use chrono::{Duration, Utc};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};

use crate::domain::error::BlogError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub user_id: i64,
    pub username: String,
    pub exp: usize,
}

pub struct JwtService {
    enc: EncodingKey,
    dec: DecodingKey,
}

impl JwtService {
    pub fn new(secret: &str) -> Result<Self, BlogError> {
        if secret.len() < 32 {
            return Err(BlogError::Validation("JWT secret must be at least 32 chars".to_string()));
        }
        Ok(Self {
            enc: EncodingKey::from_secret(secret.as_bytes()),
            dec: DecodingKey::from_secret(secret.as_bytes()),
        })
    }

    pub fn generate_token(&self, user_id: i64, username: &str) -> Result<String, BlogError> {
        let claims = Claims {
            user_id,
            username: username.to_string(),
            exp: (Utc::now() + Duration::hours(24)).timestamp() as usize,
        };
        encode(&Header::default(), &claims, &self.enc).map_err(|e| BlogError::Validation(e.to_string()))
    }

    pub fn verify_token(&self, token: &str) -> Result<Claims, BlogError> {
        let validation = Validation::new(Algorithm::HS256);
        decode::<Claims>(token, &self.dec, &validation)
            .map(|d| d.claims)
            .map_err(|_| BlogError::Unauthorized)
    }
}
