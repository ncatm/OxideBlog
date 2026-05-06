use actix_web::{
    Error, HttpMessage,
    dev::ServiceRequest,
    error::ErrorUnauthorized,
    web::{Data, ReqData},
};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use serde::Serialize;
use std::sync::Arc;

use crate::infrastructure::jwt::JwtService;

#[derive(Debug, Clone, Serialize)]
pub struct AuthenticatedUser {
    pub user_id: i64,
    pub username: String,
}

pub async fn jwt_validator(
    req: ServiceRequest,
    credentials: BearerAuth,
) -> Result<ServiceRequest, (Error, ServiceRequest)> {
    let jwt = match req.app_data::<Data<Arc<JwtService>>>() {
        Some(j) => j,
        None => return Err((ErrorUnauthorized("jwt service not configured"), req)),
    };
    let claims = match jwt.verify_token(credentials.token()) {
        Ok(c) => c,
        Err(_) => return Err((ErrorUnauthorized("invalid token"), req)),
    };

    let req = req;
    req.extensions_mut().insert(AuthenticatedUser {
        user_id: claims.user_id,
        username: claims.username,
    });
    Ok(req)
}

pub type UserCtx = ReqData<AuthenticatedUser>;
