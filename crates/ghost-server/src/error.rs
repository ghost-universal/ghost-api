//! Server error types
//!
//! Types imported from ghost-schema - the single source of truth.

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};

use ghost_schema::ErrorResponse;

/// Server error type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServerError {
    /// Not implemented
    NotImplemented(String),
    /// Not found
    NotFound(String),
    /// Bad request
    BadRequest(String),
    /// Internal error
    Internal(String),
    /// Unauthorized
    Unauthorized(String),
    /// Rate limited
    RateLimited {
        retry_after: Option<u64>,
    },
}

impl ServerError {
    /// Returns the HTTP status code for this error
    pub fn status_code(&self) -> StatusCode {
        // TODO: Implement status code determination
        match self {
            ServerError::NotImplemented(_) => StatusCode::NOT_IMPLEMENTED,
            ServerError::NotFound(_) => StatusCode::NOT_FOUND,
            ServerError::BadRequest(_) => StatusCode::BAD_REQUEST,
            ServerError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ServerError::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            ServerError::RateLimited { .. } => StatusCode::TOO_MANY_REQUESTS,
        }
    }

    /// Returns the error code
    pub fn error_code(&self) -> &'static str {
        // TODO: Implement error code determination
        match self {
            ServerError::NotImplemented(_) => "NOT_IMPLEMENTED",
            ServerError::NotFound(_) => "NOT_FOUND",
            ServerError::BadRequest(_) => "BAD_REQUEST",
            ServerError::Internal(_) => "INTERNAL_ERROR",
            ServerError::Unauthorized(_) => "UNAUTHORIZED",
            ServerError::RateLimited { .. } => "RATE_LIMITED",
        }
    }

    /// Converts to an error response
    pub fn to_error_response(&self) -> ErrorResponse {
        // TODO: Implement error response conversion
        ErrorResponse::new(self.error_code(), self.message())
    }

    /// Returns the error message
    pub fn message(&self) -> String {
        // TODO: Implement message extraction
        match self {
            ServerError::NotImplemented(msg) => format!("Not implemented: {}", msg),
            ServerError::NotFound(msg) => format!("Not found: {}", msg),
            ServerError::BadRequest(msg) => msg.clone(),
            ServerError::Internal(msg) => msg.clone(),
            ServerError::Unauthorized(msg) => msg.clone(),
            ServerError::RateLimited { .. } => "Rate limit exceeded".to_string(),
        }
    }
}

impl IntoResponse for ServerError {
    fn into_response(self) -> Response {
        // TODO: Implement response conversion
        let status = self.status_code();
        let body = Json(self.to_error_response());
        (status, body).into_response()
    }
}

impl From<ghost_schema::GhostError> for ServerError {
    fn from(err: ghost_schema::GhostError) -> Self {
        // TODO: Implement GhostError conversion
        match err {
            ghost_schema::GhostError::NotImplemented(msg) => ServerError::NotImplemented(msg),
            ghost_schema::GhostError::WorkersExhausted(msg) => ServerError::NotFound(msg),
            ghost_schema::GhostError::ValidationError(msg) => ServerError::BadRequest(msg),
            ghost_schema::GhostError::AuthError(msg) => ServerError::Unauthorized(msg),
            ghost_schema::GhostError::RateLimited { retry_after, .. } => ServerError::RateLimited { retry_after },
            _ => ServerError::Internal(err.to_string()),
        }
    }
}

impl std::fmt::Display for ServerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message())
    }
}

impl std::error::Error for ServerError {}
