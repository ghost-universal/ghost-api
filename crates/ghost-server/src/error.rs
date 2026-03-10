//! Server error types
//!
//! Defines error types and their HTTP response conversions.
//! Types are imported from ghost-schema - the single source of truth.

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};

use ghost_schema::ErrorResponse;

/// Server error type
///
/// Represents all possible errors that can occur in the server.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServerError {
    /// Feature not yet implemented
    NotImplemented(String),
    /// Resource not found
    NotFound(String),
    /// Bad request from client
    BadRequest(String),
    /// Internal server error
    Internal(String),
    /// Unauthorized access
    Unauthorized(String),
    /// Rate limit exceeded
    RateLimited {
        /// Optional retry-after duration in seconds
        retry_after: Option<u64>,
    },
    /// Service unavailable
    ServiceUnavailable(String),
    /// Validation error
    ValidationError {
        /// Field that failed validation
        field: String,
        /// Validation message
        message: String,
    },
    /// Worker error
    WorkerError {
        /// Worker ID
        worker_id: String,
        /// Error message
        message: String,
    },
    /// Platform error
    PlatformError {
        /// Platform name
        platform: String,
        /// Error code
        code: u16,
        /// Error message
        message: String,
    },
}

impl ServerError {
    /// Returns the HTTP status code for this error
    pub fn status_code(&self) -> StatusCode {
        match self {
            ServerError::NotImplemented(_) => StatusCode::NOT_IMPLEMENTED,
            ServerError::NotFound(_) => StatusCode::NOT_FOUND,
            ServerError::BadRequest(_) => StatusCode::BAD_REQUEST,
            ServerError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ServerError::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            ServerError::RateLimited { .. } => StatusCode::TOO_MANY_REQUESTS,
            ServerError::ServiceUnavailable(_) => StatusCode::SERVICE_UNAVAILABLE,
            ServerError::ValidationError { .. } => StatusCode::BAD_REQUEST,
            ServerError::WorkerError { .. } => StatusCode::INTERNAL_SERVER_ERROR,
            ServerError::PlatformError { .. } => StatusCode::BAD_GATEWAY,
        }
    }

    /// Returns the error code string
    pub fn error_code(&self) -> &'static str {
        match self {
            ServerError::NotImplemented(_) => "NOT_IMPLEMENTED",
            ServerError::NotFound(_) => "NOT_FOUND",
            ServerError::BadRequest(_) => "BAD_REQUEST",
            ServerError::Internal(_) => "INTERNAL_ERROR",
            ServerError::Unauthorized(_) => "UNAUTHORIZED",
            ServerError::RateLimited { .. } => "RATE_LIMITED",
            ServerError::ServiceUnavailable(_) => "SERVICE_UNAVAILABLE",
            ServerError::ValidationError { .. } => "VALIDATION_ERROR",
            ServerError::WorkerError { .. } => "WORKER_ERROR",
            ServerError::PlatformError { .. } => "PLATFORM_ERROR",
        }
    }

    /// Converts to an error response
    pub fn to_error_response(&self) -> ErrorResponse {
        ErrorResponse::new(self.error_code(), self.message())
    }

    /// Returns the error message
    pub fn message(&self) -> String {
        match self {
            ServerError::NotImplemented(msg) => format!("Not implemented: {}", msg),
            ServerError::NotFound(msg) => format!("Not found: {}", msg),
            ServerError::BadRequest(msg) => msg.clone(),
            ServerError::Internal(msg) => msg.clone(),
            ServerError::Unauthorized(msg) => msg.clone(),
            ServerError::RateLimited { retry_after } => {
                match retry_after {
                    Some(secs) => format!("Rate limit exceeded. Retry after {} seconds", secs),
                    None => "Rate limit exceeded".to_string(),
                }
            }
            ServerError::ServiceUnavailable(msg) => msg.clone(),
            ServerError::ValidationError { field, message } => {
                format!("Validation error on field '{}': {}", field, message)
            }
            ServerError::WorkerError { worker_id, message } => {
                format!("Worker '{}' error: {}", worker_id, message)
            }
            ServerError::PlatformError { platform, code, message } => {
                format!("Platform '{}' error ({}): {}", platform, code, message)
            }
        }
    }
}

impl IntoResponse for ServerError {
    fn into_response(self) -> Response {
        let status = self.status_code();
        let error_response = self.to_error_response();
        let body = Json(error_response);

        // Add retry-after header for rate limited responses
        if let ServerError::RateLimited { retry_after: Some(secs) } = &self {
            let mut response = (status, body).into_response();
            response.headers_mut().insert(
                axum::http::header::RETRY_AFTER,
                axum::http::HeaderValue::from(secs),
            );
            response
        } else {
            (status, body).into_response()
        }
    }
}

impl From<ghost_schema::GhostError> for ServerError {
    fn from(err: ghost_schema::GhostError) -> Self {
        match err {
            ghost_schema::GhostError::NotImplemented(msg) => ServerError::NotImplemented(msg),
            ghost_schema::GhostError::WorkersExhausted(msg) => ServerError::ServiceUnavailable(msg),
            ghost_schema::GhostError::ValidationError(msg) => ServerError::BadRequest(msg),
            ghost_schema::GhostError::AuthError(msg) => ServerError::Unauthorized(msg),
            ghost_schema::GhostError::SessionExpired(msg) => ServerError::Unauthorized(msg),
            ghost_schema::GhostError::AccountSuspended { account_id, platform, reason } => {
                ServerError::Unauthorized(format!(
                    "Account {} suspended on {:?}: {}",
                    account_id,
                    platform,
                    reason.unwrap_or_else(|| "Unknown reason".to_string())
                ))
            }
            ghost_schema::GhostError::RateLimited { retry_after, .. } => {
                ServerError::RateLimited { retry_after }
            }
            ghost_schema::GhostError::PlatformError { code, message, platform } => {
                ServerError::PlatformError {
                    platform: format!("{:?}", platform),
                    code,
                    message,
                }
            }
            ghost_schema::GhostError::ScraperError { worker, message } => {
                ServerError::WorkerError {
                    worker_id: worker,
                    message,
                }
            }
            ghost_schema::GhostError::Timeout(msg) => ServerError::ServiceUnavailable(msg),
            ghost_schema::GhostError::NetworkError(msg) => ServerError::Internal(msg),
            ghost_schema::GhostError::ProxyError(msg) => ServerError::BadRequest(msg),
            ghost_schema::GhostError::AdapterError(msg) => ServerError::Internal(msg),
            ghost_schema::GhostError::ConfigError(msg) => ServerError::Internal(msg),
            ghost_schema::GhostError::BudgetExceeded(msg) => ServerError::BadRequest(msg),
            ghost_schema::GhostError::HealthCheckFailed(worker) => {
                ServerError::WorkerError {
                    worker_id: worker,
                    message: "Health check failed".to_string(),
                }
            }
            ghost_schema::GhostError::CircuitBreakerTripped(worker) => {
                ServerError::WorkerError {
                    worker_id: worker,
                    message: "Circuit breaker tripped".to_string(),
                }
            }
            ghost_schema::GhostError::WafChallenge { challenge_type, platform } => {
                ServerError::ServiceUnavailable(format!(
                    "WAF challenge ({}) detected on {:?}",
                    challenge_type, platform
                ))
            }
            ghost_schema::GhostError::IoError(msg) => ServerError::Internal(msg),
            ghost_schema::GhostError::JsonError(msg) => ServerError::BadRequest(msg),
            ghost_schema::GhostError::ParseError(msg) => ServerError::BadRequest(msg),
            ghost_schema::GhostError::Other(msg) => ServerError::Internal(msg),
        }
    }
}

impl From<std::io::Error> for ServerError {
    fn from(err: std::io::Error) -> Self {
        ServerError::Internal(err.to_string())
    }
}

impl From<serde_json::Error> for ServerError {
    fn from(err: serde_json::Error) -> Self {
        ServerError::BadRequest(format!("JSON error: {}", err))
    }
}

impl std::fmt::Display for ServerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message())
    }
}

impl std::error::Error for ServerError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_status_codes() {
        assert_eq!(ServerError::NotImplemented("test".into()).status_code(), StatusCode::NOT_IMPLEMENTED);
        assert_eq!(ServerError::NotFound("test".into()).status_code(), StatusCode::NOT_FOUND);
        assert_eq!(ServerError::BadRequest("test".into()).status_code(), StatusCode::BAD_REQUEST);
        assert_eq!(ServerError::Internal("test".into()).status_code(), StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(ServerError::Unauthorized("test".into()).status_code(), StatusCode::UNAUTHORIZED);
        assert_eq!(ServerError::RateLimited { retry_after: None }.status_code(), StatusCode::TOO_MANY_REQUESTS);
    }

    #[test]
    fn test_error_codes() {
        assert_eq!(ServerError::NotImplemented("test".into()).error_code(), "NOT_IMPLEMENTED");
        assert_eq!(ServerError::NotFound("test".into()).error_code(), "NOT_FOUND");
        assert_eq!(ServerError::BadRequest("test".into()).error_code(), "BAD_REQUEST");
    }

    #[test]
    fn test_error_messages() {
        let err = ServerError::ValidationError {
            field: "username".to_string(),
            message: "cannot be empty".to_string(),
        };
        assert!(err.message().contains("username"));
        assert!(err.message().contains("cannot be empty"));
    }

    #[test]
    fn test_from_ghost_error() {
        let ghost_err = ghost_schema::GhostError::NotImplemented("test feature".into());
        let server_err: ServerError = ghost_err.into();
        assert!(matches!(server_err, ServerError::NotImplemented(_)));
    }
}
