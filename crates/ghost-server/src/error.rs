//! Server error types

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Server error types
#[derive(Debug, Error)]
pub enum ServerError {
    /// Feature not implemented
    #[error("Not implemented: {0}")]
    NotImplemented(String),

    /// Invalid request
    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    /// Resource not found
    #[error("Not found: {0}")]
    NotFound(String),

    /// Internal error
    #[error("Internal error: {0}")]
    Internal(String),

    /// Ghost core error
    #[error("Ghost error: {0}")]
    GhostError(#[from] ghost_schema::GhostError),
}

impl IntoResponse for ServerError {
    fn into_response(self) -> Response {
        // TODO: Implement error response generation
        let (status, message) = match &self {
            ServerError::NotImplemented(_) => (StatusCode::NOT_IMPLEMENTED, self.to_string()),
            ServerError::InvalidRequest(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            ServerError::NotFound(_) => (StatusCode::NOT_FOUND, self.to_string()),
            ServerError::Internal(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
            ServerError::GhostError(e) => {
                let status = match e {
                    ghost_schema::GhostError::RateLimited { .. } => StatusCode::TOO_MANY_REQUESTS,
                    ghost_schema::GhostError::AuthError(_) => StatusCode::UNAUTHORIZED,
                    ghost_schema::GhostError::WorkersExhausted(_) => StatusCode::SERVICE_UNAVAILABLE,
                    _ => StatusCode::INTERNAL_SERVER_ERROR,
                };
                (status, self.to_string())
            }
        };

        let body = Json(ErrorResponse {
            error: message,
            status: status.as_u16(),
        });

        (status, body).into_response()
    }
}

/// Error response body
#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    /// Error message
    pub error: String,
    /// HTTP status code
    pub status: u16,
}

impl ErrorResponse {
    /// Creates a new error response
    pub fn new(error: impl Into<String>, status: u16) -> Self {
        // TODO: Implement error response construction
        Self {
            error: error.into(),
            status,
        }
    }
}
