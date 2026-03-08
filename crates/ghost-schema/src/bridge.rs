//! Bridge types for Ghost API
//!
//! This module contains all types related to FFI bridge integration,
//! worker communication protocols, and message formats.

use serde::{Deserialize, Serialize};

use crate::{CapabilityTier, GhostError, PayloadBlob, PayloadContentType, RawContext};

// ============================================================================
// Bridge Types
// ============================================================================

/// Type of FFI bridge
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BridgeType {
    /// Python via PyO3
    PyO3,
    /// Node.js via NAPI
    Napi,
    /// Go via gRPC
    Grpc,
    /// Generic Unix Domain Socket
    Uds,
    /// In-process (native Rust)
    Native,
}

impl BridgeType {
    /// Returns whether this bridge requires external runtime
    pub fn requires_runtime(&self) -> bool {
        // TODO: Implement runtime requirement check
        matches!(
            self,
            BridgeType::PyO3 | BridgeType::Napi | BridgeType::Grpc
        )
    }

    /// Returns the runtime name
    pub fn runtime_name(&self) -> &'static str {
        // TODO: Implement runtime name
        match self {
            BridgeType::PyO3 => "python",
            BridgeType::Napi => "node",
            BridgeType::Grpc => "go",
            BridgeType::Uds => "uds",
            BridgeType::Native => "native",
        }
    }

    /// Returns the default protocol for this bridge type
    pub fn default_protocol(&self) -> WorkerProtocol {
        // TODO: Implement default protocol mapping
        match self {
            BridgeType::PyO3 => WorkerProtocol::InProcess,
            BridgeType::Napi => WorkerProtocol::InProcess,
            BridgeType::Grpc => WorkerProtocol::Grpc,
            BridgeType::Uds => WorkerProtocol::Uds,
            BridgeType::Native => WorkerProtocol::InProcess,
        }
    }
}

/// Bridge statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BridgeStats {
    /// Number of active workers
    pub active_workers: usize,
    /// Total requests handled
    pub total_requests: u64,
    /// Failed requests
    pub failed_requests: u64,
    /// Average latency in ms
    pub avg_latency_ms: u64,
    /// Memory usage in bytes
    pub memory_usage: u64,
    /// Whether the bridge is initialized
    pub is_initialized: bool,
}

impl BridgeStats {
    /// Creates new stats
    pub fn new() -> Self {
        // TODO: Implement stats construction
        Self::default()
    }

    /// Returns the success rate
    pub fn success_rate(&self) -> f64 {
        // TODO: Implement success rate calculation
        if self.total_requests == 0 {
            1.0
        } else {
            (self.total_requests - self.failed_requests) as f64 / self.total_requests as f64
        }
    }

    /// Records a request
    pub fn record(&mut self, success: bool, latency_ms: u64) {
        // TODO: Implement request recording
        self.total_requests += 1;
        if !success {
            self.failed_requests += 1;
        }
        // TODO: Update rolling average latency
        self.avg_latency_ms = latency_ms;
    }
}

/// Configuration for bridge creation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeConfig {
    /// Bridge type
    pub bridge_type: BridgeType,
    /// Maximum workers
    pub max_workers: usize,
    /// Request timeout in ms
    pub timeout_ms: u64,
    /// Memory limit in MB
    pub memory_limit_mb: u64,
    /// Path to worker script/binary
    pub worker_path: Option<String>,
    /// Protocol to use
    pub protocol: WorkerProtocol,
}

impl BridgeConfig {
    /// Creates a new bridge config
    pub fn new(bridge_type: BridgeType) -> Self {
        // TODO: Implement bridge config construction
        Self {
            bridge_type,
            max_workers: 5,
            timeout_ms: 30000,
            memory_limit_mb: 512,
            worker_path: None,
            protocol: bridge_type.default_protocol(),
        }
    }

    /// Sets the worker path
    pub fn with_worker_path(mut self, path: impl Into<String>) -> Self {
        // TODO: Implement worker path setter
        self.worker_path = Some(path.into());
        self
    }

    /// Sets the maximum workers
    pub fn with_max_workers(mut self, max: usize) -> Self {
        // TODO: Implement max workers setter
        self.max_workers = max;
        self
    }

    /// Sets the timeout
    pub fn with_timeout(mut self, timeout_ms: u64) -> Self {
        // TODO: Implement timeout setter
        self.timeout_ms = timeout_ms;
        self
    }
}

// ============================================================================
// Protocol Types
// ============================================================================

/// Protocol for worker communication
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
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

    /// Returns whether this protocol uses stdio
    pub fn uses_stdio(&self) -> bool {
        // TODO: Implement stdio check
        matches!(
            self,
            WorkerProtocol::JsonStdio | WorkerProtocol::MsgPackStdio
        )
    }
}

/// Serialization format
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SerializationFormat {
    Json,
    MsgPack,
    Protobuf,
    Native,
}

// ============================================================================
// Message Types
// ============================================================================

/// Type of message
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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

/// Request message to worker
#[derive(Debug, Clone, Serialize, Deserialize)]
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
            id: String::new(), // TODO: Generate UUID
            context,
            timestamp: 0, // TODO: Use actual timestamp
        }
    }

    /// Serializes to JSON
    pub fn to_json(&self) -> Result<String, GhostError> {
        // TODO: Implement JSON serialization
        serde_json::to_string(self).map_err(|e| GhostError::ParseError(e.to_string()))
    }

    /// Deserializes from JSON
    pub fn from_json(json: &str) -> Result<Self, GhostError> {
        // TODO: Implement JSON deserialization
        serde_json::from_str(json).map_err(|e| GhostError::ParseError(e.to_string()))
    }
}

/// Response message from worker
#[derive(Debug, Clone, Serialize, Deserialize)]
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

    /// Serializes to JSON
    pub fn to_json(&self) -> Result<String, GhostError> {
        // TODO: Implement JSON serialization
        serde_json::to_string(self).map_err(|e| GhostError::ParseError(e.to_string()))
    }

    /// Deserializes from JSON
    pub fn from_json(json: &str) -> Result<Self, GhostError> {
        // TODO: Implement JSON deserialization
        serde_json::from_str(json).map_err(|e| GhostError::ParseError(e.to_string()))
    }
}

/// Message envelope for protocol communication
#[derive(Debug, Clone, Serialize, Deserialize)]
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

    /// Creates a register message
    pub fn register(manifest: &WorkerManifestMessage) -> Result<Self, GhostError> {
        // TODO: Implement register message creation
        let payload = serde_json::to_string(manifest)
            .map_err(|e| GhostError::ParseError(e.to_string()))?;
        Ok(Self::new(MessageType::Register, payload))
    }

    /// Creates an execute message
    pub fn execute(request: &WorkerRequest) -> Result<Self, GhostError> {
        // TODO: Implement execute message creation
        let payload = serde_json::to_string(request)
            .map_err(|e| GhostError::ParseError(e.to_string()))?;
        Ok(Self::new(MessageType::Execute, payload))
    }

    /// Creates a health check message
    pub fn health_check(check: &HealthCheckMessage) -> Result<Self, GhostError> {
        // TODO: Implement health check message creation
        let payload = serde_json::to_string(check)
            .map_err(|e| GhostError::ParseError(e.to_string()))?;
        Ok(Self::new(MessageType::HealthCheck, payload))
    }
}

/// Worker manifest message
#[derive(Debug, Clone, Serialize, Deserialize)]
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
    /// Max concurrent requests
    pub max_concurrent: u32,
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
            max_concurrent: 5,
        }
    }

    /// Adds a capability
    pub fn with_capability(mut self, capability: impl Into<String>) -> Self {
        // TODO: Implement capability addition
        self.capabilities.push(capability.into());
        self
    }

    /// Adds a platform
    pub fn with_platform(mut self, platform: impl Into<String>) -> Self {
        // TODO: Implement platform addition
        self.platforms.push(platform.into());
        self
    }
}

/// Health check message
#[derive(Debug, Clone, Serialize, Deserialize)]
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
            timestamp: 0, // TODO: Use actual timestamp
            request_id: String::new(), // TODO: Generate UUID
        }
    }
}

impl Default for HealthCheckMessage {
    fn default() -> Self {
        Self::new()
    }
}

/// Health check response
#[derive(Debug, Clone, Serialize, Deserialize)]
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

    /// Creates an unhealthy response
    pub fn unhealthy(request_id: impl Into<String>) -> Self {
        // TODO: Implement unhealthy response construction
        Self {
            request_id: request_id.into(),
            healthy: false,
            load: 0.0,
            memory_mb: 0,
            uptime_secs: 0,
        }
    }
}
