//! Communication protocol for bridge workers

use ghost_schema::{GhostError, RawContext, PayloadBlob};

/// Protocol for worker communication
#[derive(Debug, Clone)]
pub enum WorkerProtocol {
    /// JSON over stdio
    JsonStdio,
    /// MsgPack over stdio
    MsgPackStdio,
    /// gRPC
    Grpc,
    /// Unix Domain Socket
    Uds,
    /// In-process (direct call)
    InProcess,
}

impl WorkerProtocol {
    /// Returns the serialization format
    pub fn serialization(&self) -> SerializationFormat {
        // TODO: Implement serialization format determination
        match self {
            WorkerProtocol::JsonStdio => SerializationFormat::Json,
            WorkerProtocol::MsgPackStdio => SerializationFormat::MsgPack,
            WorkerProtocol::Grpc => SerializationFormat::Protobuf,
            WorkerProtocol::Uds => SerializationFormat::MsgPack,
            WorkerProtocol::InProcess => SerializationFormat::Native,
        }
    }
}

/// Serialization format
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SerializationFormat {
    Json,
    MsgPack,
    Protobuf,
    Native,
}

/// Request message to worker
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WorkerRequest {
    /// Request ID
    pub id: String,
    /// Request context
    pub context: RawContext,
    /// Timestamp
    pub timestamp: i64,
}

impl WorkerRequest {
    /// Creates a new worker request
    pub fn new(context: RawContext) -> Self {
        // TODO: Implement request construction
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            context,
            timestamp: chrono::Utc::now().timestamp(),
        }
    }
}

/// Response message from worker
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WorkerResponse {
    /// Request ID this responds to
    pub request_id: String,
    /// Response payload
    pub payload: Option<PayloadBlob>,
    /// Error if failed
    pub error: Option<String>,
    /// Duration in ms
    pub duration_ms: u64,
}

impl WorkerResponse {
    /// Creates a successful response
    pub fn success(request_id: impl Into<String>, payload: PayloadBlob, duration_ms: u64) -> Self {
        // TODO: Implement success response construction
        Self {
            request_id: request_id.into(),
            payload: Some(payload),
            error: None,
            duration_ms,
        }
    }

    /// Creates an error response
    pub fn error(request_id: impl Into<String>, error: impl Into<String>, duration_ms: u64) -> Self {
        // TODO: Implement error response construction
        Self {
            request_id: request_id.into(),
            payload: None,
            error: Some(error.into()),
            duration_ms,
        }
    }

    /// Checks if response is successful
    pub fn is_success(&self) -> bool {
        self.error.is_none() && self.payload.is_some()
    }
}

/// Message envelope for protocol communication
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MessageEnvelope {
    /// Message type
    pub message_type: MessageType,
    /// Payload (JSON serialized)
    pub payload: String,
    /// Checksum
    pub checksum: Option<String>,
}

impl MessageEnvelope {
    /// Creates a new message envelope
    pub fn new(message_type: MessageType, payload: impl Into<String>) -> Self {
        // TODO: Implement envelope construction
        Self {
            message_type,
            payload: payload.into(),
            checksum: None,
        }
    }

    /// Serializes to bytes
    pub fn to_bytes(&self) -> Result<Vec<u8>, GhostError> {
        // TODO: Implement serialization
        serde_json::to_vec(self).map_err(|e| GhostError::ParseError(e.to_string()))
    }

    /// Deserializes from bytes
    pub fn from_bytes(data: &[u8]) -> Result<Self, GhostError> {
        // TODO: Implement deserialization
        serde_json::from_slice(data).map_err(|e| GhostError::ParseError(e.to_string()))
    }
}

/// Type of message
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MessageType {
    /// Worker registration
    Register,
    /// Worker ready
    Ready,
    /// Execute request
    Execute,
    /// Execute response
    Response,
    /// Health check
    HealthCheck,
    /// Health check response
    HealthResponse,
    /// Shutdown
    Shutdown,
    /// Error
    Error,
}

/// Worker manifest message
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WorkerManifestMessage {
    /// Worker ID
    pub worker_id: String,
    /// Worker version
    pub version: String,
    /// Capabilities
    pub capabilities: Vec<String>,
    /// Platforms
    pub platforms: Vec<String>,
    /// Worker type
    pub worker_type: String,
}

impl WorkerManifestMessage {
    /// Creates a new manifest message
    pub fn new(worker_id: impl Into<String>) -> Self {
        // TODO: Implement manifest message construction
        Self {
            worker_id: worker_id.into(),
            version: "0.1.0".to_string(),
            capabilities: Vec::new(),
            platforms: Vec::new(),
            worker_type: "unknown".to_string(),
        }
    }
}

/// Health check message
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct HealthCheckMessage {
    /// Timestamp
    pub timestamp: i64,
    /// Request ID
    pub request_id: String,
}

impl HealthCheckMessage {
    /// Creates a new health check message
    pub fn new() -> Self {
        // TODO: Implement health check message construction
        Self {
            timestamp: chrono::Utc::now().timestamp(),
            request_id: uuid::Uuid::new_v4().to_string(),
        }
    }
}

/// Health check response
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct HealthCheckResponse {
    /// Request ID being responded to
    pub request_id: String,
    /// Whether healthy
    pub healthy: bool,
    /// Current load
    pub load: f64,
    /// Memory usage
    pub memory_mb: u64,
    /// Uptime in seconds
    pub uptime_secs: u64,
}

impl HealthCheckResponse {
    /// Creates a healthy response
    pub fn healthy(request_id: impl Into<String>) -> Self {
        // TODO: Implement healthy response construction
        Self {
            request_id: request_id.into(),
            healthy: true,
            load: 0.0,
            memory_mb: 0,
            uptime_secs: 0,
        }
    }
}

// Stub modules for optional dependencies
mod uuid {
    pub struct Uuid;
    impl Uuid {
        pub fn new_v4() -> Self { Self }
        pub fn to_string(&self) -> String { "stub-uuid".to_string() }
    }
}

mod chrono {
    pub struct Utc;
    impl Utc {
        pub fn now() -> Self { Self }
        pub fn timestamp(&self) -> i64 { 0 }
    }
}
