use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, AppError>;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Block not found: {0}")]
    BlockNotFound(String),

    #[error("Proof not found for block: {0}")]
    ProofNotFound(String),

    #[error("Transaction not found: {0}")]
    TransactionNotFound(String),

    #[error("Header not found: {0}")]
    HeaderNotFound(String),

    #[error("Invalid block identifier: {0}")]
    InvalidBlockIdentifier(String),

    #[error("Invalid transaction ID: {0}")]
    InvalidTransactionId(String),

    #[error("Invalid header hash: {0}")]
    InvalidHeaderHash(String),

    #[error("Invalid query parameter: {0}")]
    InvalidQueryParameter(String),

    #[error("Store error: {0}")]
    Store(#[from] anyhow::Error),

    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Internal server error")]
    Internal,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::BlockNotFound(_) => (StatusCode::NOT_FOUND, self.to_string()),
            AppError::ProofNotFound(_) => (StatusCode::NOT_FOUND, self.to_string()),
            AppError::TransactionNotFound(_) => (StatusCode::NOT_FOUND, self.to_string()),
            AppError::HeaderNotFound(_) => (StatusCode::NOT_FOUND, self.to_string()),
            AppError::InvalidBlockIdentifier(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            AppError::InvalidTransactionId(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            AppError::InvalidHeaderHash(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            AppError::InvalidQueryParameter(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            AppError::Store(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Store error".to_string()),
            AppError::Json(_) => (StatusCode::INTERNAL_SERVER_ERROR, "JSON error".to_string()),
            AppError::Io(_) => (StatusCode::INTERNAL_SERVER_ERROR, "IO error".to_string()),
            AppError::Internal => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal server error".to_string(),
            ),
        };

        let body = Json(json!({
            "error": error_message,
            "status": status.as_u16()
        }));

        (status, body).into_response()
    }
}
