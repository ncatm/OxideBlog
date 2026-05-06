use tonic::transport::{Channel, Endpoint};

use crate::error::BlogClientError;

pub async fn connect_channel(endpoint: &str) -> Result<Channel, BlogClientError> {
    let ep = Endpoint::from_shared(endpoint.to_string())
        .map_err(|e| BlogClientError::InvalidRequest(e.to_string()))?;
    Ok(ep.connect().await?)
}
