use thiserror::Error;
use tonic::Code;

#[derive(Debug, Error)]
pub enum BlogClientError {
    #[error("http error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("grpc status: {0}")]
    GrpcStatus(#[from] tonic::Status),
    #[error("grpc transport: {0}")]
    GrpcTransport(#[from] tonic::transport::Error),
    #[error("not found")]
    NotFound,
    #[error("unauthorized")]
    Unauthorized,
    #[error("invalid request: {0}")]
    InvalidRequest(String),
}

impl BlogClientError {
    pub fn from_grpc_status(status: tonic::Status) -> Self {
        match status.code() {
            Code::NotFound => Self::NotFound,
            Code::Unauthenticated => Self::Unauthorized,
            Code::PermissionDenied => Self::Unauthorized,
            _ => Self::GrpcStatus(status),
        }
    }
}
